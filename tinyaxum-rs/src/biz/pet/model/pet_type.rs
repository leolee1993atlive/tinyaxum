use serde::{Deserialize, Serialize};
use validator::Validate;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PetType {
    pub pet_type_id: Option<i64>,
    pub color: String,
}

impl PetType {
    pub fn from_create_bo(bo: PetTypeCreateBo) -> Self {
        Self {
            pet_type_id: None,
            color: bo.color,
        }
    }

    pub fn from_update_bo(self: &mut Self, bo: PetTypeUpdateBo) {
        self.pet_type_id = bo.pet_type_id;
        self.color = bo.color;
    }

    pub fn to_list_vo(&self) -> PetTypeListVo {
        PetTypeListVo {
            pet_type_id: self.pet_type_id.unwrap_or_default(),
            color: self.color.clone(),
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, Validate)]
#[serde(rename_all = "camelCase")]
pub struct PetTypeCreateBo {
    #[validate(length(min = 1, message = "color cannot be empty"))]
    pub color: String,
}

#[derive(Clone, Debug, Serialize, Deserialize, Validate)]
#[serde(rename_all = "camelCase")]
pub struct PetTypeUpdateBo {
    #[validate(required)]
    pub pet_type_id: Option<i64>,
    #[validate(length(min = 1, message = "color cannot be empty"))]
    pub color: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PetTypeListVo {
    pub pet_type_id: i64,
    pub color: String,
}
