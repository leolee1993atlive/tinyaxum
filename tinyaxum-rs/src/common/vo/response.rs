use axum::{
    Json,
    http::StatusCode,
    response::{IntoResponse, Response},
};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize)]
#[serde(untagged)]
pub enum R<T> {
    NoDataSuccess(ApiResponse<()>),
    Success(ApiResponse<T>),
    Error(ApiResponse<()>),
}

impl<T> IntoResponse for R<T>
where
    T: Serialize,
{
    fn into_response(self) -> Response {
        match self {
            R::NoDataSuccess(r) => r.into_response(),
            R::Success(r) => r.into_response(),
            R::Error(r) => r.into_response(),
        }
    }
}

impl<T> R<T> {
    pub fn ok() -> Self {
        R::NoDataSuccess(ApiResponse::<()>::ok())
    }

    pub fn ok_with_data(data: T) -> Self {
        R::Success(ApiResponse::ok_with_data(data))
    }

    pub fn ok_with_data_and_message(data: T, message: String) -> Self {
        R::Success(ApiResponse::ok_with_data_and_message(data, message))
    }

    pub fn error_with_code_and_message(code: u16, message: String) -> Self {
        R::Error(ApiResponse::<()>::error_with_code_and_message(
            code, message,
        ))
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ApiResponse<T> {
    pub code: u16,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<T>,
}

impl<T> ApiResponse<T> {
    pub fn ok() -> ApiResponse<()> {
        ApiResponse {
            code: 200,
            message: "成功".to_string(),
            data: None,
        }
    }

    pub fn ok_with_data(data: T) -> ApiResponse<T> {
        ApiResponse {
            code: 200,
            message: "成功".to_string(),
            data: Some(data),
        }
    }

    pub fn ok_with_data_and_message(
        data: T,
        message: String,
    ) -> ApiResponse<T> {
        ApiResponse {
            code: 200,
            message,
            data: Some(data),
        }
    }

    pub fn error_with_code_and_message(
        code: u16,
        message: String,
    ) -> ApiResponse<()> {
        ApiResponse {
            code,
            message,
            data: None,
        }
    }
}

impl<T> IntoResponse for ApiResponse<T>
where
    T: Serialize,
{
    fn into_response(self) -> Response {
        let status_code = StatusCode::from_u16(self.code)
            .unwrap_or(StatusCode::INTERNAL_SERVER_ERROR);
        (status_code, Json(self)).into_response()
    }
}
