use serde::{Deserialize, Serialize};
use validator::Validate;

use crate::{
    common::model::entity::BaseEntity,
    middleware::extractors::CurrentUser,
    util::comm_util::{to_string, to_vec},
};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SqlSnippet {
    pub sql_snippet_id: Option<i64>,
    pub sql_snippet_name: String,
    pub sql_snippet_content: String,
    pub sql_snippet_desc: Option<String>,
    pub sql_snippet_tag: Option<String>,
    #[serde(flatten)]
    pub base_entity: BaseEntity,
}

impl SqlSnippet {
    pub fn from_create_bo(bo: SqlSnippetCreateBo, user: &CurrentUser) -> Self {
        SqlSnippet {
            sql_snippet_id: None,
            sql_snippet_name: bo.sql_snippet_name,
            sql_snippet_content: bo.sql_snippet_content,
            sql_snippet_desc: bo.sql_snippet_desc,
            sql_snippet_tag: to_string(bo.sql_snippet_tag),
            base_entity: BaseEntity::new(user.get_user_id()),
        }
    }

    #[allow(clippy::wrong_self_convention)]
    pub fn from_update_bo(
        &mut self,
        bo: SqlSnippetUpdateBo,
        user: &CurrentUser,
    ) {
        self.sql_snippet_id = bo.sql_snippet_id;
        self.sql_snippet_name = bo.sql_snippet_name;
        self.sql_snippet_content = bo.sql_snippet_content;
        self.sql_snippet_desc = bo.sql_snippet_desc;
        self.sql_snippet_tag = to_string(bo.sql_snippet_tag);
        self.base_entity.update(user.get_user_id());
    }

    pub fn to_list_vo(&self) -> SqlSnippetListVo {
        SqlSnippetListVo {
            sql_snippet_id: self.sql_snippet_id.unwrap_or_default(),
            sql_snippet_name: self.sql_snippet_name.clone(),
            sql_snippet_content: self.sql_snippet_content.clone(),
            sql_snippet_desc: self.sql_snippet_desc.clone(),
            sql_snippet_tag: to_vec(self.sql_snippet_tag.clone()),
            base_entity: self.base_entity.clone(),
        }
    }

    pub fn to_detail_vo(&self) -> SqlSnippetDetailVo {
        SqlSnippetDetailVo {
            sql_snippet_id: self.sql_snippet_id.unwrap_or_default(),
            sql_snippet_name: self.sql_snippet_name.clone(),
            sql_snippet_content: self.sql_snippet_content.clone(),
            sql_snippet_desc: self.sql_snippet_desc.clone(),
            sql_snippet_tag: to_vec(self.sql_snippet_tag.clone()),
            base_entity: self.base_entity.clone(),
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, Validate)]
#[serde(rename_all = "camelCase")]
pub struct SqlSnippetCreateBo {
    #[validate(length(min = 1, message = "sql_snippet_name cannot be empty"))]
    pub sql_snippet_name: String,
    #[validate(length(
        min = 1,
        message = "sql_snippet_content cannot be empty"
    ))]
    pub sql_snippet_content: String,
    pub sql_snippet_desc: Option<String>,
    #[serde(default)]
    pub sql_snippet_tag: Vec<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize, Validate)]
#[serde(rename_all = "camelCase")]
pub struct SqlSnippetUpdateBo {
    #[validate(required)]
    pub sql_snippet_id: Option<i64>,
    #[validate(length(min = 1, message = "sql_snippet_name cannot be empty"))]
    pub sql_snippet_name: String,
    #[validate(length(
        min = 1,
        message = "sql_snippet_content cannot be empty"
    ))]
    pub sql_snippet_content: String,
    pub sql_snippet_desc: Option<String>,
    #[serde(default)]
    pub sql_snippet_tag: Vec<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize, Validate)]
#[serde(rename_all = "camelCase")]
pub struct SqlSnippetListVo {
    pub sql_snippet_id: i64,
    pub sql_snippet_name: String,
    pub sql_snippet_content: String,
    pub sql_snippet_desc: Option<String>,
    pub sql_snippet_tag: Vec<String>,
    #[serde(flatten)]
    pub base_entity: BaseEntity,
}

#[derive(Clone, Debug, Serialize, Deserialize, Validate)]
#[serde(rename_all = "camelCase")]
pub struct SqlSnippetDetailVo {
    pub sql_snippet_id: i64,
    pub sql_snippet_name: String,
    pub sql_snippet_content: String,
    pub sql_snippet_desc: Option<String>,
    pub sql_snippet_tag: Vec<String>,
    #[serde(flatten)]
    pub base_entity: BaseEntity,
}
