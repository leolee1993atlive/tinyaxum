use crate::{
    app::Infrastructure,
    common::vo::page::{PageResult, normalize_paging, sanitize_sort},
    error::error::{AppError, AppResult},
    middleware::extractors::CurrentUser,
    sys::user::{
        handler::user_handler::UserPageQuery,
        model::user::{User, UserCreateBo, UserListVo, UserUpdateBo},
        repository::user_repo::UserRepository,
    },
    util::crypto_util::hash_password,
};

#[derive(Clone)]
pub struct UserService {
    user_repo: UserRepository,
}

impl UserService {
    pub fn new(infra: &Infrastructure) -> Self {
        Self {
            user_repo: UserRepository::new(infra.db.clone()),
        }
    }

    #[tracing::instrument(skip(self))]
    pub async fn list(&self) -> AppResult<Vec<UserListVo>> {
        let result = self.user_repo.select_all().await?;
        tracing::info!(count = result.len(), "查询用户列表成功");
        Ok(result.into_iter().map(|user| user.to_list_vo()).collect())
    }

    #[tracing::instrument(skip(self), fields(page = query.page.page_no, size = query.page.page_size))]
    pub async fn page(
        &self,
        query: UserPageQuery,
    ) -> AppResult<PageResult<UserListVo>> {
        let (current, size, _offset) = normalize_paging(&query.page);
        let allowed = [
            ("userId", "user_id"),
            ("username", "username"),
            ("nickName", "nick_name"),
            ("createdDate", "created_date"),
        ];
        let mut order_by =
            sanitize_sort(&query.page.sort, &query.page.order, &allowed);
        if order_by.is_none() {
            order_by = Some("user_id desc".to_string());
        }

        let order_str = order_by.as_deref().unwrap_or("user_id desc");
        let page_result = self
            .user_repo
            .select_page(&query, current as u64, size as u64, order_str)
            .await?;
        if page_result.total <= 0 {
            return Ok(PageResult::empty(current, size));
        }
        let pages = (page_result.total + size - 1) / size;
        tracing::info!(
            current,
            size,
            total = page_result.total,
            pages,
            "用户分页查询成功"
        );
        Ok(PageResult {
            records: page_result
                .records
                .into_iter()
                .map(|u| u.to_list_vo())
                .collect(),
            total: page_result.total,
            pages,
            current,
            size,
        })
    }

    #[tracing::instrument(skip(self), fields(user_id = id))]
    pub async fn get_user_by_id(&self, id: i64) -> AppResult<User> {
        let user =
            self.user_repo.select_by_id(&id).await?.ok_or_else(|| {
                AppError::NotFound(format!("用户不存在 (id={})", id))
            })?;
        tracing::info!(user_id = id, "根据ID查询用户成功");
        Ok(user)
    }

    #[tracing::instrument(skip(self), fields(username = username))]
    pub async fn get_user_by_username(
        &self,
        username: &str,
    ) -> AppResult<User> {
        let user = self
            .user_repo
            .select_by_username(username)
            .await?
            .ok_or_else(|| {
                AppError::NotFound(format!(
                    "用户不存在 (username={})",
                    username
                ))
            })?;
        tracing::info!(username = username, "根据用户名查询用户成功");
        Ok(user)
    }

    #[tracing::instrument(skip(self, bo, current_user), fields(operator = %current_user.username, new_username = ?bo.username))]
    pub async fn insert_by_bo(
        &self,
        bo: UserCreateBo,
        current_user: &CurrentUser,
    ) -> AppResult<()> {
        let user = User::from_create_bo(bo, current_user).map_err(|e| {
            AppError::Internal(format!("创建用户失败: 构建用户实体失败: {}", e))
        })?;
        self.user_repo.insert(&user).await?;
        tracing::info!(
            username = ?user.username,
            operator = %current_user.username,
            "创建用户成功"
        );
        Ok(())
    }

    #[tracing::instrument(skip(self, bo, current_user), fields(operator = %current_user.username, user_id = ?bo.user_id))]
    pub async fn update_by_bo(
        &self,
        bo: UserUpdateBo,
        current_user: &CurrentUser,
    ) -> AppResult<()> {
        let user_id = bo.user_id.ok_or_else(|| {
            AppError::BadRequest("更新用户失败: 用户ID为空".to_string())
        })?;
        let mut user = self
            .user_repo
            .select_by_id(&user_id)
            .await?
            .ok_or_else(|| {
                AppError::NotFound(format!(
                    "更新用户失败: 用户不存在 (id={})",
                    user_id
                ))
            })?;

        user.from_update_bo(bo, current_user);
        self.user_repo.update_by_id(&user, &user_id).await?;
        tracing::info!(
            user_id = user_id,
            operator = %current_user.username,
            "更新用户成功"
        );
        Ok(())
    }

    #[tracing::instrument(skip(self, current_user), fields(operator = %current_user.username, user_id = id))]
    pub async fn delete_by_id(
        &self,
        id: i64,
        current_user: &CurrentUser,
    ) -> AppResult<()> {
        let mut user =
            self.user_repo.select_by_id(&id).await?.ok_or_else(|| {
                AppError::NotFound(format!(
                    "软删除用户失败: 用户不存在 (id={})",
                    id
                ))
            })?;

        user.base_entity.delete(current_user.get_user_id());

        self.user_repo.update_by_id(&user, &id).await?;
        tracing::info!(
            user_id = id,
            operator = %current_user.username,
            "软删除用户成功"
        );
        Ok(())
    }

    #[tracing::instrument(skip(self), fields(user_id = id))]
    pub async fn true_delete_by_id(&self, id: i64) -> AppResult<()> {
        self.user_repo.delete_by_id(&id).await?;
        tracing::info!(user_id = id, "彻底删除用户成功");
        Ok(())
    }

    #[tracing::instrument(skip(self, password), fields(user_id = id))]
    pub async fn update_password(
        &self,
        id: i64,
        password: String,
    ) -> AppResult<()> {
        let mut user =
            self.user_repo.select_by_id(&id).await?.ok_or_else(|| {
                AppError::NotFound(format!(
                    "更新密码失败: 用户不存在 (id={})",
                    id
                ))
            })?;

        user.password = hash_password(password.as_str()).map_err(|e| {
            AppError::Internal(format!("更新密码失败: 密码哈希失败: {}", e))
        })?;

        self.user_repo.update_by_id(&user, &id).await?;
        tracing::info!(user_id = id, "更新密码成功");
        Ok(())
    }
}
