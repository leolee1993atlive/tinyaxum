use serde::{Deserialize, Serialize};
use serde_json::Value;
use validator::Validate;

use crate::error::error::{AppError, AppResult};

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DashboardConfigVo {
    pub config: Value,
}

#[derive(Clone, Debug, Serialize, Deserialize, Validate)]
#[serde(rename_all = "camelCase")]
pub struct DashboardConfigSaveBo {
    pub config: Value,
}

fn default_dashboard_config_json() -> Value {
    serde_json::json!({
        "version": 1,
        "tabs": {
            "overview": { "items": [] },
            "analytics": { "items": [] },
            "charts": { "items": [] }
        }
    })
}

pub fn default_dashboard_config() -> Value {
    default_dashboard_config_json()
}

/// 校验看板配置 JSON（只约束版本与最小结构）。
pub fn validate_dashboard_config_json(v: &Value) -> AppResult<()> {
    if !v.is_object() {
        return Err(AppError::BadRequest(
            "config 必须为 JSON 对象".to_string(),
        ));
    }
    let Some(ver) = v.get("version") else {
        return Err(AppError::BadRequest("config.version 缺失".to_string()));
    };
    if ver.as_u64() != Some(1) {
        return Err(AppError::BadRequest(
            "config.version 仅支持 1".to_string(),
        ));
    }
    let Some(tabs) = v.get("tabs") else {
        return Err(AppError::BadRequest("config.tabs 缺失".to_string()));
    };
    let Some(tabs_obj) = tabs.as_object() else {
        return Err(AppError::BadRequest("config.tabs 必须为对象".to_string()));
    };
    for key in ["overview", "analytics", "charts"] {
        let Some(tab) = tabs_obj.get(key) else {
            return Err(AppError::BadRequest(format!(
                "config.tabs.{key} 缺失"
            )));
        };
        let Some(tab_obj) = tab.as_object() else {
            return Err(AppError::BadRequest(format!(
                "config.tabs.{key} 必须为对象"
            )));
        };
        let Some(items_val) = tab_obj.get("items") else {
            continue;
        };
        let Some(arr) = items_val.as_array() else {
            return Err(AppError::BadRequest(format!(
                "config.tabs.{key}.items 必须为数组"
            )));
        };
        for (i, it) in arr.iter().enumerate() {
            let Some(obj) = it.as_object() else {
                return Err(AppError::BadRequest(format!(
                    "config.tabs.{key}.items[{i}] 必须为对象"
                )));
            };
            let ok = obj
                .get("sqlQueryId")
                .and_then(|x| x.as_i64())
                .is_some_and(|n| n > 0);
            if !ok {
                return Err(AppError::BadRequest(format!(
                    "config.tabs.{key}.items[{i}].sqlQueryId 无效"
                )));
            }
            if let Some(defaults_val) = obj.get("defaults") {
                let Some(defaults_obj) = defaults_val.as_object() else {
                    return Err(AppError::BadRequest(format!(
                        "config.tabs.{key}.items[{i}].defaults 必须为对象"
                    )));
                };
                for (dk, dv) in defaults_obj {
                    if dk.is_empty() {
                        return Err(AppError::BadRequest(format!(
                            "config.tabs.{key}.items[{i}].defaults 的键不能为空"
                        )));
                    }
                    if !dv.is_string() {
                        return Err(AppError::BadRequest(format!(
                            "config.tabs.{key}.items[{i}].defaults.{dk} 必须为字符串"
                        )));
                    }
                }
            }
        }
    }
    Ok(())
}
