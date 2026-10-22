use sea_orm::{
    ActiveModelTrait, ColumnTrait, DatabaseConnection, EntityTrait,
    QueryFilter,
};

use crate::biz::pet::model::{pet_type::PetType, pet_type_entity};

#[derive(Clone)]
pub struct PetTypeRepository {
    db: DatabaseConnection,
}

impl PetTypeRepository {
    pub fn new(db: DatabaseConnection) -> Self {
        Self { db }
    }

    pub async fn select_all(
        &self,
    ) -> Result<Vec<PetType>, sea_orm::DbErr> {
        let models = pet_type_entity::Entity::find()
            .filter(pet_type_entity::Column::DeletedBy.is_null())
            .filter(pet_type_entity::Column::DeletedDate.is_null())
            .all(&self.db)
            .await?;
        Ok(models.into_iter().map(|m| m.into()).collect())
    }

    pub async fn select_by_id(
        &self,
        pet_type_id: &i64,
    ) -> Result<Option<PetType>, sea_orm::DbErr> {
        let model =
            pet_type_entity::Entity::find_by_id(*pet_type_id)
                .filter(pet_type_entity::Column::DeletedBy.is_null())
                .filter(pet_type_entity::Column::DeletedDate.is_null())
                .one(&self.db)
                .await?;
        Ok(model.map(|m| m.into()))
    }

    pub async fn insert(
        &self,
        pet_type: &PetType,
    ) -> Result<(), sea_orm::DbErr> {
        let am: pet_type_entity::ActiveModel = pet_type.into();
        am.insert(&self.db).await?;
        Ok(())
    }

    pub async fn update_by_id(
        &self,
        pet_type: &PetType,
        pet_type_id: &i64,
    ) -> Result<(), sea_orm::DbErr> {
        let am: pet_type_entity::ActiveModel = pet_type.into();
        pet_type_entity::Entity::update(am)
            .filter(pet_type_entity::Column::DeletedBy.is_null())
            .filter(pet_type_entity::Column::DeletedDate.is_null())
            .exec(&self.db)
            .await?;
        Ok(())
    }

    pub async fn delete_by_id(
        &self,
        pet_type_id: &i64,
    ) -> Result<(), sea_orm::DbErr> {
        pet_type_entity::Entity::delete_by_id(*pet_type_id)
            .exec(&self.db)
            .await?;
        Ok(())
    }
}
