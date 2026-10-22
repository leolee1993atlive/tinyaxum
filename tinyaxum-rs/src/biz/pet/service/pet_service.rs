use crate::{
    app::Infrastructure,
    biz::pet::{
        model::pet::{Pet, PetCreateBo, PetListVo, PetUpdateBo},
        repository::pet_repo::PetRepository,
    },
    error::error::{AppError, AppResult},
    sys::user::model::user::User,
    middleware::extractors::CurrentUser,
};

#[derive(Clone)]
pub struct PetService {
    pet_repo: PetRepository,
}

impl PetService {
    pub fn new(infra: &Infrastructure) -> Self {
        Self {
            pet_repo: PetRepository::new(infra.batis.clone()),
        }
    }

    #[tracing::instrument(skip(self))]
    pub async fn list(&self) -> AppResult<Vec<PetListVo>> {
        let result = self.pet_repo.select_all().await?;
        tracing::info!(count = result.len(), "查询宠物列表成功");
        Ok(result.into_iter().map(|pet| pet.to_list_vo()).collect())
    }

    #[tracing::instrument(skip(self), fields(pet_id = id))]
    pub async fn get_pet_by_id(&self, id: i64) -> AppResult<Pet> {
        let pet = self
            .pet_repo
            .select_by_id(&id)
            .await?
            .ok_or_else(|| AppError::NotFound(format!("宠物不存在 (id={})", id)))?;
        tracing::info!(pet_id = id, "根据ID查询宠物成功");
        Ok(pet)
    }

    #[tracing::instrument(skip(self, bo, current_user), fields(pet_name = ?bo.name, owner_id = ?bo.owner_id))]
    pub async fn insert_by_bo(
        &self,
        bo: PetCreateBo,
        current_user: &CurrentUser,
    ) -> AppResult<()> {
        let pet = Pet::from_create_bo(bo, current_user);
        self.pet_repo.insert(&pet).await?;
        tracing::info!(
            pet_name = ?pet.name,
            owner_id = ?pet.owner_id,
            operator = %current_user.username,
            "创建宠物成功"
        );
        Ok(())
    }

    #[tracing::instrument(skip(self, bo, current_user), fields(pet_id = ?bo.pet_id, owner_id = ?bo.owner_id))]
    pub async fn update_by_bo(
        &self,
        bo: PetUpdateBo,
        current_user: &CurrentUser,
    ) -> AppResult<()> {
        let pet_id = bo.pet_id.ok_or_else(|| {
            AppError::BadRequest("更新宠物失败: 宠物ID为空".to_string())
        })?;
        let mut pet = self
            .pet_repo
            .select_by_id(pet_id)
            .await?
            .ok_or_else(|| {
                AppError::NotFound(format!(
                    "更新宠物失败: 宠物不存在 (id={})",
                    pet_id
                ))
            })?;

        pet.from_update_bo(bo, current_user);
        self.pet_repo.update_by_id(&pet, pet_id).await?;
        tracing::info!(
            pet_id = pet_id,
            operator = %current_user.username,
            "更新宠物成功"
        );
        Ok(())
    }

    #[tracing::instrument(skip(self, current_user), fields(pet_id = id))]
    pub async fn delete_by_id(&self, id: i64, current_user: &CurrentUser) -> AppResult<()> {
        let mut pet = self
            .pet_repo
            .select_by_id(&id)
            .await?
            .ok_or_else(|| {
                AppError::NotFound(format!(
                    "软删除宠物失败: 宠物不存在 (id={})",
                    id
                ))
            })?;

        pet.base_entity.delete(current_user.get_user_id());

        self.pet_repo.update_by_id(&pet, &id).await?;
        tracing::info!(
            pet_id = id,
            operator = %current_user.username,
            "软删除宠物成功"
        );
        Ok(())
    }

    #[tracing::instrument(skip(self), fields(pet_id = id))]
    pub async fn true_delete_by_id(&self, id: i64) -> AppResult<()> {
        self.pet_repo.delete_by_id(&id).await?;
        tracing::info!(pet_id = id, "彻底删除宠物成功");
        Ok(())
    }
}
