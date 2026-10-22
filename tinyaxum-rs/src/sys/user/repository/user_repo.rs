use sea_orm::{
    ActiveValue::{NotSet, Set},
    ColumnTrait, Condition, DatabaseConnection, EntityTrait, Order,
    PaginatorTrait, QueryFilter, QueryOrder,
    entity::prelude::*,
};

use crate::{
    common::{model::entity::BaseEntity, repo::str_trim_or_empty},
    sys::user::model::user::User,
};

/// SeaORM entity for the `sys_user` table.
#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq)]
#[sea_orm(table_name = "sys_user")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub user_id: i64,
    pub username: String,
    pub password: String,
    pub nick_name: String,
    pub email: Option<String>,
    pub active: String,
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

impl From<&Model> for User {
    fn from(m: &Model) -> Self {
        User {
            user_id: Some(m.user_id),
            username: m.username.clone(),
            password: m.password.clone(),
            nick_name: m.nick_name.clone(),
            email: m.email.clone(),
            active: m.active.clone(),
            base_entity: BaseEntity {
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

/// Convert a domain `User` into a SeaORM `ActiveModel` for
/// insert/update operations.
fn user_to_active_model(user: &User) -> ActiveModel {
    ActiveModel {
        user_id: match user.user_id {
            Some(id) => Set(id),
            None => NotSet,
        },
        username: Set(user.username.clone()),
        password: Set(user.password.clone()),
        nick_name: Set(user.nick_name.clone()),
        email: Set(user.email.clone()),
        active: Set(user.active.clone()),
        version: Set(user.base_entity.version),
        created_by: Set(user.base_entity.created_by),
        created_date: Set(user.base_entity.created_date),
        last_modified_by: Set(user.base_entity.last_modified_by),
        last_modified_date: Set(user.base_entity.last_modified_date),
        deleted_by: Set(user.base_entity.deleted_by),
        deleted_date: Set(user.base_entity.deleted_date),
    }
}

/// Soft-delete filter: both `deleted_by` AND `deleted_date` must
/// be NULL for a row to be considered active.
fn soft_delete_filter() -> Condition {
    Condition::all()
        .add(Column::DeletedBy.is_null())
        .add(Column::DeletedDate.is_null())
}

/// User data-access repository backed by SeaORM.
#[derive(Clone)]
pub struct UserRepository {
    db: DatabaseConnection,
}

impl UserRepository {
    pub fn new(db: DatabaseConnection) -> Self {
        Self { db }
    }

    /// List all users, filtering out soft-deleted rows for consistency
    /// with other repositories.
    pub async fn select_all(&self) -> Result<Vec<User>, sea_orm::DbErr> {
        let models = Entity::find()
            .filter(soft_delete_filter())
            .all(&self.db)
            .await?;
        Ok(models.iter().map(User::from).collect())
    }

    /// Look up a single user by primary key, respecting soft-delete
    /// (`deleted_by IS NULL AND deleted_date IS NULL`).
    pub async fn select_by_id(
        &self,
        user_id: &i64,
    ) -> Result<Option<User>, sea_orm::DbErr> {
        let model = Entity::find_by_id(*user_id)
            .filter(soft_delete_filter())
            .one(&self.db)
            .await?;
        Ok(model.as_ref().map(User::from))
    }

    /// Look up a single user by username, respecting soft-delete.
    pub async fn select_by_username(
        &self,
        username: &str,
    ) -> Result<Option<User>, sea_orm::DbErr> {
        let model = Entity::find()
            .filter(Column::Username.eq(username))
            .filter(soft_delete_filter())
            .one(&self.db)
            .await?;
        Ok(model.as_ref().map(User::from))
    }

    /// Insert a new user row.
    pub async fn insert(&self, user: &User) -> Result<(), sea_orm::DbErr> {
        let active = user_to_active_model(user);
        Entity::insert(active).exec(&self.db).await?;
        Ok(())
    }

    /// Update a user identified by `user_id`, only when the row
    /// has **not** been soft-deleted.
    pub async fn update_by_id(
        &self,
        user: &User,
        user_id: &i64,
    ) -> Result<(), sea_orm::DbErr> {
        let active = user_to_active_model(user);
        let _ = Entity::find_by_id(*user_id)
            .filter(soft_delete_filter())
            .one(&self.db)
            .await?
            .ok_or(sea_orm::DbErr::RecordNotFound(
                "User not found".to_string(),
            ))?;
        active.update(&self.db).await?;
        Ok(())
    }

    /// Hard-delete a user identified by `user_id`, only when the
    /// row has **not** been soft-deleted.
    pub async fn delete_by_id(
        &self,
        user_id: &i64,
    ) -> Result<(), sea_orm::DbErr> {
        Entity::delete_by_id(*user_id)
            .filter(soft_delete_filter())
            .exec(&self.db)
            .await?;
        Ok(())
    }

    /// Replaces the legacy rbatis `htmlsql_select_page!` macro.
    /// Dynamic filter logic from `src/xml/user_repo.html`:
    ///   - keyword  → username OR nick_name LIKE
    ///   - username → username LIKE
    ///   - nick_name → nick_name LIKE
    ///   - active   → exact match
    pub async fn select_page(
        &self,
        query: &crate::sys::user::handler::user_handler::UserPageQuery,
        page_no: u64,
        page_size: u64,
        order_by: &str,
    ) -> Result<crate::common::vo::page::PageResult<User>, sea_orm::DbErr> {
        let keyword = str_trim_or_empty(&query.keyword);
        let username = str_trim_or_empty(&query.username);
        let nick_name = str_trim_or_empty(&query.nick_name);
        let active = str_trim_or_empty(&query.active);

        let mut select = Entity::find().filter(soft_delete_filter());

        if !username.is_empty() {
            select = select.filter(Column::Username.contains(&username));
        }
        if !nick_name.is_empty() {
            select = select.filter(Column::NickName.contains(&nick_name));
        }
        if !active.is_empty() {
            select = select.filter(Column::Active.eq(&active));
        }
        if !keyword.is_empty() {
            select = select.filter(
                Condition::any()
                    .add(Column::Username.contains(&keyword))
                    .add(Column::NickName.contains(&keyword)),
            );
        }

        let (order_col, order_dir) = parse_order(order_by);
        select = select.order_by(order_col, order_dir);

        let paginator = select.paginate(&self.db, page_size);
        let total = paginator.num_items().await?;
        let models = paginator.fetch_page(page_no.saturating_sub(1)).await?;

        let records: Vec<User> = models.iter().map(User::from).collect();
        let pages = if page_size == 0 {
            0
        } else {
            total.div_ceil(page_size)
        };

        Ok(crate::common::vo::page::PageResult {
            records,
            total: total as i64,
            pages: pages as i64,
            current: page_no as i64,
            size: page_size as i64,
        })
    }
}

fn parse_order(order_by: &str) -> (Column, Order) {
    let mut parts = order_by.split_whitespace();
    let col = match parts.next().unwrap_or("user_id") {
        "user_id" => Column::UserId,
        "username" => Column::Username,
        "nick_name" => Column::NickName,
        "created_date" => Column::CreatedDate,
        _ => Column::UserId,
    };
    let desc = !parts.next().is_some_and(|d| d.eq_ignore_ascii_case("asc"));
    if desc {
        (col, Order::Desc)
    } else {
        (col, Order::Asc)
    }
}
