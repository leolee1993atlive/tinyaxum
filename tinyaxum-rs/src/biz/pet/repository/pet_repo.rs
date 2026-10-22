use sea_orm::{
    ActiveModelTrait, ColumnTrait, DatabaseConnection, EntityTrait,
    QueryFilter,
};

use crate::biz::pet::model::{pet::Pet, pet_entity};

#[derive(Clone)]
pub struct PetRepository {
    db: DatabaseConnection,
}

impl PetRepository {
    pub fn new(db: DatabaseConnection) -> Self {
        Self { db }
    }

    pub async fn select_all(
        &self,
    ) -> Result<Vec<Pet>, sea_orm::DbErr> {
        let models = pet_entity::Entity::find()
            .filter(pet_entity::Column::DeletedBy.is_null())
            .filter(pet_entity::Column::DeletedDate.is_null())
            .all(&self.db)
            .await?;
        Ok(models.into_iter().map(|m| m.into()).collect())
    }

    pub async fn select_by_id(
        &self,
        pet_id: &i64,
    ) -> Result<Option<Pet>, sea_orm::DbErr> {
        let model = pet_entity::Entity::find_by_id(*pet_id)
            .filter(pet_entity::Column::DeletedBy.is_null())
            .filter(pet_entity::Column::DeletedDate.is_null())
            .one(&self.db)
            .await?;
        Ok(model.map(|m| m.into()))
    }

    pub async fn insert(
        &self,
        pet: &Pet,
    ) -> Result<(), sea_orm::DbErr> {
        let am: pet_entity::ActiveModel = pet.into();
        am.insert(&self.db).await?;
        Ok(())
    }

    pub async fn update_by_id(
        &self,
        pet: &Pet,
        pet_id: &i64,
    ) -> Result<(), sea_orm::DbErr> {
        let am: pet_entity::ActiveModel = pet.into();
        pet_entity::Entity::update(am)
            .filter(pet_entity::Column::DeletedBy.is_null())
            .filter(pet_entity::Column::DeletedDate.is_null())
            .exec(&self.db)
            .await?;
        Ok(())
    }

    pub async fn delete_by_id(
        &self,
        pet_id: &i64,
    ) -> Result<(), sea_orm::DbErr> {
        pet_entity::Entity::delete_by_id(*pet_id)
            .filter(pet_entity::Column::DeletedBy.is_null())
            .filter(pet_entity::Column::DeletedDate.is_null())
            .exec(&self.db)
            .await?;
        Ok(())
    }
}
