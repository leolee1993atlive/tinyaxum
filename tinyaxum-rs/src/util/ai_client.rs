use std::time::Duration;

use futures_util::StreamExt;
use serde::{Deserialize, Serialize};

use crate::error::error::{AppError, AppResult};

#[derive(Clone, Debug, Serialize)]
pub struct ChatMessage {
    pub role: String,
    pub content: String,
}

#[derive(Clone, Debug, Default)]
pub struct TokenUsage {
    pub prompt_tokens: i32,
    pub completion_tokens: i32,
}

#[derive(Clone, Debug, Default)]
pub struct ChatChunk {
    pub delta_content: String,
    pub usage: Option<TokenUsage>,
    pub done: bool,
}

#[derive(Clone, Debug)]
#[allow(dead_code)]
pub struct OnceResponse {
    pub content: String,
    pub usage: Option<TokenUsage>,
}

#[derive(Serialize)]
struct OpenAiRequest<'a> {
    model: &'a str,
    messages: &'a [ChatMessage],
    temperature: f32,
    stream: bool,
}

#[derive(Deserialize)]
struct OpenAiNonStreamResp {
    choices: Vec<OpenAiChoice>,
    #[serde(default)]
    usage: Option<OpenAiUsage>,
}

#[derive(Deserialize)]
struct OpenAiChoice {
    message: OpenAiChoiceMsg,
}

#[derive(Deserialize)]
struct OpenAiChoiceMsg {
    #[serde(default)]
    content: Option<String>,
}

#[derive(Deserialize)]
struct OpenAiUsage {
    #[serde(default)]
    prompt_tokens: i32,
    #[serde(default)]
    completion_tokens: i32,
}

fn join_url(base_url: &str, path: &str) -> String {
    let trimmed = base_url.trim_end_matches('/');
    format!("{}{}", trimmed, path)
}

static HTTP_CLIENT: std::sync::LazyLock<reqwest::Client> =
    std::sync::LazyLock::new(|| {
        reqwest::Client::builder()
            .timeout(Duration::from_secs(120))
            .build()
            .expect("Failed to build reqwest Client")
    });

pub async fn chat_completion_once(
    base_url: &str,
    api_key: &str,
    model: &str,
    messages: Vec<ChatMessage>,
    temperature: f32,
) -> AppResult<OnceResponse> {
    let url = join_url(base_url, "/chat/completions");
    let body = OpenAiRequest {
        model,
        messages: &messages,
        temperature,
        stream: false,
    };

    let client = HTTP_CLIENT.clone();

    let resp = client
        .post(&url)
        .bearer_auth(api_key)
        .json(&body)
        .send()
        .await
        .map_err(|e| AppError::Internal(format!("调用上游失败: {e}")))?;

    let status = resp.status();
    let text = resp
        .text()
        .await
        .map_err(|e| AppError::Internal(format!("读取上游响应失败: {e}")))?;

    if !status.is_success() {
        return Err(AppError::BadRequest(format!(
            "UPSTREAM_ERROR: {} {}",
            status.as_u16(),
            text
        )));
    }

    let parsed: OpenAiNonStreamResp =
        serde_json::from_str(&text).map_err(|e| {
            AppError::Internal(format!("UPSTREAM_INVALID: {e}; body={text}"))
        })?;

    let content = parsed
        .choices
        .first()
        .and_then(|c| c.message.content.clone())
        .unwrap_or_default();
    let usage = parsed.usage.map(|u| TokenUsage {
        prompt_tokens: u.prompt_tokens,
        completion_tokens: u.completion_tokens,
    });

    Ok(OnceResponse { content, usage })
}

#[derive(Deserialize)]
struct OpenAiStreamChunk {
    #[serde(default)]
    choices: Vec<OpenAiStreamChoice>,
    #[serde(default)]
    usage: Option<OpenAiUsage>,
}

#[derive(Deserialize)]
struct OpenAiStreamChoice {
    #[serde(default)]
    delta: OpenAiDelta,
}

#[derive(Deserialize, Default)]
struct OpenAiDelta {
    #[serde(default)]
    content: Option<String>,
}

pub fn parse_sse_data_line(data: &str) -> AppResult<Option<ChatChunk>> {
    let trimmed = data.trim();
    if trimmed.is_empty() {
        return Ok(None);
    }
    if trimmed == "[DONE]" {
        return Ok(Some(ChatChunk {
            done: true,
            ..Default::default()
        }));
    }
    let parsed: OpenAiStreamChunk =
        serde_json::from_str(trimmed).map_err(|e| {
            AppError::Internal(format!("UPSTREAM_INVALID: {e}; data={trimmed}"))
        })?;
    let delta_content = parsed
        .choices
        .first()
        .and_then(|c| c.delta.content.clone())
        .unwrap_or_default();
    let usage = parsed.usage.map(|u| TokenUsage {
        prompt_tokens: u.prompt_tokens,
        completion_tokens: u.completion_tokens,
    });
    Ok(Some(ChatChunk {
        delta_content,
        usage,
        done: false,
    }))
}

pub fn chat_completion_stream(
    base_url: String,
    api_key: String,
    model: String,
    messages: Vec<ChatMessage>,
    temperature: f32,
) -> impl futures_util::Stream<Item = AppResult<ChatChunk>> {
    async_stream::stream! {
        let url = join_url(&base_url, "/chat/completions");
        let body = OpenAiRequest {
            model: &model,
            messages: &messages,
            temperature,
            stream: true,
        };

        let client = HTTP_CLIENT.clone();

        let resp = match client
            .post(&url)
            .bearer_auth(&api_key)
            .json(&body)
            .send()
            .await
        {
            Ok(r) => r,
            Err(e) => {
                yield Err(AppError::Internal(format!("调用上游失败: {e}")));
                return;
            }
        };

        let status = resp.status();
        if !status.is_success() {
            let text = resp.text().await.unwrap_or_default();
            yield Err(AppError::BadRequest(format!(
                "UPSTREAM_ERROR: {} {}",
                status.as_u16(),
                text
            )));
            return;
        }

        let mut byte_stream = resp.bytes_stream();
        let mut buf = String::new();

        while let Some(item) = byte_stream.next().await {
            match item {
                Ok(bytes) => {
                    match std::str::from_utf8(&bytes) {
                        Ok(s) => buf.push_str(s),
                        Err(e) => {
                            yield Err(AppError::Internal(format!(
                                "UPSTREAM_INVALID: 非 UTF-8 字节: {e}"
                            )));
                            return;
                        }
                    }

                    while let Some(pos) = buf.find("\n\n") {
                        let event_block = buf[..pos].to_string();
                        buf = buf[pos + 2..].to_string();

                        for line in event_block.lines() {
                            if let Some(rest) = line.strip_prefix("data:") {
                                let data = rest.trim_start();
                                match parse_sse_data_line(data) {
                                    Ok(Some(chunk)) => {
                                        let done = chunk.done;
                                        yield Ok(chunk);
                                        if done {
                                            return;
                                        }
                                    }
                                    Ok(None) => {}
                                    Err(e) => {
                                        yield Err(e);
                                        return;
                                    }
                                }
                            }
                        }
                    }
                }
                Err(e) => {
                    yield Err(AppError::Internal(format!(
                        "UPSTREAM_TIMEOUT_OR_BROKEN: {e}"
                    )));
                    return;
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_sse_chunk_basic() {
        let r =
            parse_sse_data_line(r#"{"choices":[{"delta":{"content":"SEL"}}]}"#)
                .unwrap()
                .unwrap();
        assert_eq!(r.delta_content, "SEL");
        assert!(!r.done);
        assert!(r.usage.is_none());
    }

    #[test]
    fn parse_sse_chunk_done_signal() {
        let r = parse_sse_data_line("[DONE]").unwrap().unwrap();
        assert!(r.done);
    }

    #[test]
    fn parse_sse_chunk_with_usage() {
        let r = parse_sse_data_line(
            r#"{"choices":[],"usage":{"prompt_tokens":12,"completion_tokens":34}}"#,
        )
        .unwrap()
        .unwrap();
        assert_eq!(r.delta_content, "");
        let u = r.usage.unwrap();
        assert_eq!(u.prompt_tokens, 12);
        assert_eq!(u.completion_tokens, 34);
    }

    #[test]
    fn parse_sse_chunk_empty_string_returns_none() {
        let r = parse_sse_data_line("").unwrap();
        assert!(r.is_none());
    }

    #[test]
    fn parse_sse_chunk_malformed_returns_err() {
        let r = parse_sse_data_line(r#"{not json"#);
        assert!(r.is_err());
    }
}
