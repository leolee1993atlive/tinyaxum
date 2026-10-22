use sea_orm::entity::prelude::*;

use crate::biz::pet::model::pet_type::PetType;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq)]
#[sea_orm(table_name = "pet_type")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub pet_type_id: i64,
    pub color: String,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}

impl From<Model> for PetType {
    fn from(m: Model) -> Self {
        PetType {
            pet_type_id: Some(m.pet_type_id),
            color: m.color,
        }
    }
}

impl From<&PetType> for ActiveModel {
    fn from(pt: &PetType) -> Self {
        use sea_orm::Set;
        ActiveModel {
            pet_type_id: match pt.pet_type_id {
                Some(id) => Set(id),
                None => NotSet,
            },
            color: Set(pt.color.clone()),
        }
    }
}
