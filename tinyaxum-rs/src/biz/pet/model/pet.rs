use serde::{Deserialize, Serialize};
use validator::Validate;

use crate::{common::model::entity::BaseEntity, middleware::extractors::CurrentUser};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Pet {
    pub pet_id: Option<i64>,
    pub name: String,
    pub birth_date: Option<String>,
    pub pet_type_id: Option<i64>,
    pub owner_id: Option<i64>,
    #[serde(flatten)]
    pub base_entity: BaseEntity,
}

impl Pet {
    pub fn from_create_bo(bo: PetCreateBo, user: &CurrentUser) -> Self {
        Self {
            pet_id: None,
            name: bo.name,
            birth_date: bo.birth_date,
            pet_type_id: bo.pet_type_id,
            owner_id: bo.owner_id,
            base_entity: BaseEntity::new(user.get_user_id()),
        }
    }

    pub fn from_update_bo(self: &mut Self, bo: PetUpdateBo, user: &CurrentUser) {
        self.pet_id = bo.pet_id;
        self.name = bo.name;
        self.birth_date = bo.birth_date;
        self.pet_type_id = bo.pet_type_id;
        self.owner_id = bo.owner_id;
        self.base_entity.update(user.get_user_id());
    }

    pub fn to_list_vo(&self) -> PetListVo {
        PetListVo {
            pet_id: self.pet_id.unwrap_or_default(),
            name: self.name.clone(),
            birth_date: self.birth_date.clone(),
            pet_type_id: self.pet_type_id,
            owner_id: self.owner_id,
            base_entity: self.base_entity.clone(),
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, Validate)]
#[serde(rename_all = "camelCase")]
pub struct PetCreateBo {
    #[validate(length(min = 1, message = "name cannot be empty"))]
    pub name: String,
    pub birth_date: Option<String>,
    pub pet_type_id: Option<i64>,
    pub owner_id: Option<i64>,
}

#[derive(Clone, Debug, Serialize, Deserialize, Validate)]
#[serde(rename_all = "camelCase")]
pub struct PetUpdateBo {
    #[validate(required)]
    pub pet_id: Option<i64>,
    #[validate(length(min = 1, message = "name cannot be empty"))]
    pub name: String,
    pub birth_date: Option<String>,
    pub pet_type_id: Option<i64>,
    pub owner_id: Option<i64>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PetListVo {
    pub pet_id: i64,
    pub name: String,
    pub birth_date: Option<String>,
    pub pet_type_id: Option<i64>,
    pub owner_id: Option<i64>,
    #[serde(flatten)]
    pub base_entity: BaseEntity,
}
