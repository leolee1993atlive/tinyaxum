use sea_orm::entity::prelude::*;

use crate::biz::pet::model::pet::Pet;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq)]
#[sea_orm(table_name = "pet")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub pet_id: i64,
    pub name: String,
    pub birth_date: Option<String>,
    pub pet_type_id: Option<i64>,
    pub owner_id: Option<i64>,
    pub version: i64,
    pub created_by: i64,
    pub created_date: DateTimeUtc,
    pub last_modified_by: i64,
    pub last_modified_date: DateTimeUtc,
    pub deleted_by: Option<i64>,
    pub deleted_date: Option<DateTimeUtc>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}

impl From<Model> for Pet {
    fn from(m: Model) -> Self {
        Pet {
            pet_id: Some(m.pet_id),
            name: m.name,
            birth_date: m.birth_date,
            pet_type_id: m.pet_type_id,
            owner_id: m.owner_id,
            base_entity: crate::common::model::entity::BaseEntity {
                version: m.version,
                created_by: m.created_by,
                created_date: m.created_date,
                last_modified_by: m.last_modified_by,
                last_modified_date: m.last_modified_date,
                deleted_by: m.deleted_by,
                deleted_date: m.deleted_date,
            },
        }
    }
}

impl From<&Pet> for ActiveModel {
    fn from(pet: &Pet) -> Self {
        use sea_orm::Set;
        ActiveModel {
            pet_id: match pet.pet_id {
                Some(id) => Set(id),
                None => NotSet,
            },
            name: Set(pet.name.clone()),
            birth_date: Set(pet.birth_date.clone()),
            pet_type_id: Set(pet.pet_type_id),
            owner_id: Set(pet.owner_id),
            version: Set(pet.base_entity.version),
            created_by: Set(pet.base_entity.created_by),
            created_date: Set(pet.base_entity.created_date),
            last_modified_by: Set(pet.base_entity.last_modified_by),
            last_modified_date: Set(pet.base_entity.last_modified_date),
            deleted_by: Set(pet.base_entity.deleted_by),
            deleted_date: Set(pet.base_entity.deleted_date),
        }
    }
}
