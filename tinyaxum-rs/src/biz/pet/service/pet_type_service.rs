use crate::{
    app::Infrastructure,
    biz::pet::{
        model::pet_type::{
            PetType, PetTypeCreateBo, PetTypeListVo, PetTypeUpdateBo,
        },
        repository::pet_type_repo::PetTypeRepository,
    },
};

#[derive(Clone)]
pub struct PetTypeService {
    pet_type_repo: PetTypeRepository,
}

impl PetTypeService {
    pub fn new(infra: &Infrastructure) -> Self {
        Self {
            pet_type_repo: PetTypeRepository::new(infra.batis.clone()),
        }
    }

    #[tracing::instrument(skip(self))]
    pub async fn list(&self) -> Vec<PetTypeListVo> {
        match self.pet_type_repo.select_all().await {
            Ok(result) => {
                tracing::info!(count = result.len(), "查询宠物类型列表成功");
                result.into_iter().map(|pet| pet.to_list_vo()).collect()
            }
            Err(e) => {
                tracing::error!(error = %e, "查询宠物类型列表失败");
                Vec::new()
            }
        }
    }

    #[tracing::instrument(skip(self), fields(pet_type_id = id))]
    pub async fn get_pet_type_by_id(&self, id: i64) -> Option<PetType> {
        match self.pet_type_repo.select_by_id(&id).await {
            Ok(Some(pet_type)) => {
                tracing::info!(pet_type_id = id, "根据ID查询宠物类型成功");
                Some(pet_type)
            }
            Ok(None) => {
                tracing::warn!(
                    pet_type_id = id,
                    "根据ID查询宠物类型失败: 宠物类型不存在"
                );
                None
            }
            Err(e) => {
                tracing::error!(pet_type_id = id, error = %e, "根据ID查询宠物类型失败");
                None
            }
        }
    }

    #[tracing::instrument(skip(self, bo), fields(color = ?bo.color))]
    pub async fn insert_by_bo(&self, bo: PetTypeCreateBo) -> bool {
        let pet_type = PetType::from_create_bo(bo);
        match self.pet_type_repo.insert(&pet_type).await {
            Ok(_) => {
                tracing::info!(color = ?pet_type.color, "创建宠物类型成功");
                true
            }
            Err(e) => {
                tracing::error!(color = ?pet_type.color, error = %e, "创建宠物类型失败");
                false
            }
        }
    }

    #[tracing::instrument(skip(self, bo), fields(pet_type_id = ?bo.pet_type_id, color = ?bo.color))]
    pub async fn update_by_bo(&self, bo: PetTypeUpdateBo) -> bool {
        let pet_type_id = match &bo.pet_type_id {
            Some(id) => id,
            None => {
                tracing::warn!("更新宠物类型失败: 宠物类型ID为空");
                return false;
            }
        };
        let mut pet_type =
            match self.pet_type_repo.select_by_id(pet_type_id).await {
                Ok(Some(pet_type)) => pet_type,
                Ok(None) => {
                    tracing::warn!(
                        pet_type_id = pet_type_id,
                        "更新宠物类型失败: 宠物类型不存在"
                    );
                    return false;
                }
                Err(e) => {
                    tracing::error!(
                        pet_type_id = pet_type_id,
                        error = %e,
                        "更新宠物类型失败: 查询宠物类型异常"
                    );
                    return false;
                }
            };

        pet_type.from_update_bo(bo);
        match self
            .pet_type_repo
            .update_by_id(&pet_type, pet_type_id)
            .await
        {
            Ok(_) => {
                tracing::info!(pet_type_id = pet_type_id, "更新宠物类型成功");
                true
            }
            Err(e) => {
                tracing::error!(
                    pet_type_id = pet_type_id,
                    error = %e,
                    "更新宠物类型失败"
                );
                false
            }
        }
    }

    #[tracing::instrument(skip(self), fields(pet_type_id = id))]
    pub async fn delete_by_id(&self, id: i64) -> bool {
        match self.pet_type_repo.delete_by_id(&id).await {
            Ok(_) => {
                tracing::info!(pet_type_id = id, "删除宠物类型成功");
                true
            }
            Err(e) => {
                tracing::error!(pet_type_id = id, error = %e, "删除宠物类型失败");
                false
            }
        }
    }
}
