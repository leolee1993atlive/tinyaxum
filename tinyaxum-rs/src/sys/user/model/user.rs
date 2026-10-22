use serde::{Deserialize, Serialize};
use validator::Validate;

use crate::{
    common::model::entity::{BaseEntity, BaseVo},
    error::error::AppResult,
    middleware::extractors::CurrentUser,
    util::crypto_util::hash_password,
};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct User {
    pub user_id: Option<i64>,
    pub username: String,
    pub password: String,
    pub nick_name: String,
    pub email: Option<String>,
    pub active: String,
    #[serde(flatten)]
    pub base_entity: BaseEntity,
}

impl User {
    pub fn from_create_bo(
        bo: UserCreateBo,
        user: &CurrentUser,
    ) -> AppResult<Self> {
        Ok(Self {
            user_id: None,
            username: bo.username,
            password: hash_password(&bo.password)?,
            nick_name: bo.nick_name,
            email: bo.email,
            active: bo.active,
            base_entity: BaseEntity::new(user.get_user_id()),
        })
    }

    #[allow(clippy::wrong_self_convention)]
    pub fn from_update_bo(&mut self, bo: UserUpdateBo, user: &CurrentUser) {
        self.user_id = bo.user_id;
        self.username = bo.username;
        self.nick_name = bo.nick_name;
        self.email = bo.email;
        self.active = bo.active;
        self.base_entity.update(user.get_user_id());
    }

    pub fn to_list_vo(&self) -> UserListVo {
        UserListVo {
            user_id: self.user_id.unwrap_or_default(),
            username: self.username.clone(),
            nick_name: self.nick_name.clone(),
            email: self.email.clone(),
            active: self.active.clone(),
            base_entity: BaseVo::new(self.base_entity.clone()),
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, Validate)]
#[serde(rename_all = "camelCase")]
pub struct UserCreateBo {
    #[validate(length(min = 1, message = "username cannot be empty"))]
    pub username: String,
    #[validate(length(
        min = 6,
        max = 50,
        message = "密码长度至少6位且不超过50位"
    ))]
    pub password: String,
    pub nick_name: String,
    pub email: Option<String>,
    pub active: String,
}

#[derive(Clone, Debug, Serialize, Deserialize, Validate)]
#[serde(rename_all = "camelCase")]
pub struct UserUpdateBo {
    #[validate(required)]
    pub user_id: Option<i64>,
    #[validate(length(min = 1, message = "username cannot be empty"))]
    pub username: String,
    pub nick_name: String,
    pub email: Option<String>,
    pub active: String,
}

#[derive(Clone, Debug, Serialize, Deserialize, Validate)]
#[serde(rename_all = "camelCase")]
pub struct UserUpdatePasswordBo {
    #[validate(required)]
    pub user_id: Option<i64>,
    #[validate(length(
        min = 6,
        max = 50,
        message = "密码长度至少6位且不超过50位"
    ))]
    pub password: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UserListVo {
    pub user_id: i64,
    pub username: String,
    pub nick_name: String,
    pub email: Option<String>,
    pub active: String,
    #[serde(flatten)]
    pub base_entity: BaseVo,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UserInfo {
    pub user_id: i64,
    pub username: String,
}
