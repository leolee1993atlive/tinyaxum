use sea_orm::{
    ActiveValue::{NotSet, Set},
    ColumnTrait, Condition, DatabaseConnection, EntityTrait, Order,
    PaginatorTrait, QueryFilter, QueryOrder,
};

use crate::{
    biz::datasource::{
        handler::data_source_handler::DataSourcePageQuery,
        model::{
            data_source::DataSource,
            data_source_entity::{
                ActiveModel, Column, Entity as BizDataSource, Model,
            },
        },
    },
    common::{model::entity::BaseEntity, repo::str_trim_or_empty},
};

impl From<&Model> for DataSource {
    fn from(m: &Model) -> Self {
        DataSource {
            data_source_id: Some(m.data_source_id),
            code: m.code.clone(),
            name: m.name.clone(),
            remark: m.remark.clone(),
            db_type: m.db_type.clone(),
            db_host: m.db_host.clone(),
            db_port: m.db_port as u16,
            db_name: m.db_name.clone(),
            db_schema: m.db_schema.clone(),
            db_username: m.db_username.clone(),
            db_password: m.db_password.clone(),
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

fn to_active(ds: &DataSource) -> ActiveModel {
    ActiveModel {
        data_source_id: match ds.data_source_id {
            Some(id) => Set(id),
            None => NotSet,
        },
        code: Set(ds.code.clone()),
        name: Set(ds.name.clone()),
        remark: Set(ds.remark.clone()),
        db_type: Set(ds.db_type.clone()),
        db_host: Set(ds.db_host.clone()),
        db_port: Set(ds.db_port as i32),
        db_name: Set(ds.db_name.clone()),
        db_schema: Set(ds.db_schema.clone()),
        db_username: Set(ds.db_username.clone()),
        db_password: Set(ds.db_password.clone()),
        version: Set(ds.base_entity.version),
        created_by: Set(ds.base_entity.created_by),
        created_date: Set(ds.base_entity.created_date),
        last_modified_by: Set(ds.base_entity.last_modified_by),
        last_modified_date: Set(ds.base_entity.last_modified_date),
        deleted_by: Set(ds.base_entity.deleted_by),
        deleted_date: Set(ds.base_entity.deleted_date),
    }
}

fn active_filter() -> Condition {
    Condition::all()
        .add(Column::DeletedBy.is_null())
        .add(Column::DeletedDate.is_null())
}

/// Falls back to `data_source_id desc`.
fn parse_order(order_by: &str) -> (Column, Order) {
    let mut parts = order_by.split_whitespace();
    let col = match parts.next().unwrap_or("data_source_id") {
        "data_source_id" => Column::DataSourceId,
        "code" => Column::Code,
        "name" => Column::Name,
        "db_type" => Column::DbType,
        "created_date" => Column::CreatedDate,
        _ => Column::DataSourceId,
    };
    let desc = !parts.next().is_some_and(|d| d.eq_ignore_ascii_case("asc"));
    if desc {
        (col, Order::Desc)
    } else {
        (col, Order::Asc)
    }
}

pub struct DataSourcePage {
    pub records: Vec<DataSource>,
    pub total: u64,
}

#[derive(Clone)]
pub struct DataSourceRepository {
    db: DatabaseConnection,
}

impl DataSourceRepository {
    pub fn new(db: DatabaseConnection) -> Self {
        Self { db }
    }

    pub async fn select_all(&self) -> Result<Vec<DataSource>, sea_orm::DbErr> {
        let models = BizDataSource::find()
            .filter(active_filter())
            .order_by_asc(Column::DataSourceId)
            .all(&self.db)
            .await?;
        Ok(models.iter().map(DataSource::from).collect())
    }

    pub async fn select_page(
        &self,
        query: &DataSourcePageQuery,
        page_no: u64,
        page_size: u64,
        order_by: &str,
    ) -> Result<DataSourcePage, sea_orm::DbErr> {
        let keyword = str_trim_or_empty(&query.keyword);
        let code = str_trim_or_empty(&query.code);
        let name = str_trim_or_empty(&query.name);
        let db_type = str_trim_or_empty(&query.db_type);

        let mut select = BizDataSource::find()
            .filter(active_filter())
            .filter(Column::DataSourceId.ne(0));

        if !keyword.is_empty() {
            select = select.filter(
                Condition::any()
                    .add(Column::Code.contains(&keyword))
                    .add(Column::Name.contains(&keyword)),
            );
        }
        if !code.is_empty() {
            select = select.filter(Column::Code.contains(&code));
        }
        if !name.is_empty() {
            select = select.filter(Column::Name.contains(&name));
        }
        if !db_type.is_empty() {
            select = select.filter(Column::DbType.eq(&db_type));
        }

        let (order_col, order_dir) = parse_order(order_by);
        select = select.order_by(order_col, order_dir);

        // Paginate: count + fetch
        let paginator = select.paginate(&self.db, page_size);
        let total = paginator.num_items().await?;
        let models = paginator.fetch_page(page_no.saturating_sub(1)).await?;

        Ok(DataSourcePage {
            records: models.iter().map(DataSource::from).collect(),
            total,
        })
    }

    pub async fn select_by_id(
        &self,
        data_source_id: &i64,
    ) -> Result<Option<DataSource>, sea_orm::DbErr> {
        let model = BizDataSource::find_by_id(*data_source_id)
            .filter(active_filter())
            .one(&self.db)
            .await?;
        Ok(model.as_ref().map(DataSource::from))
    }

    pub async fn insert(
        &self,
        data_source: &DataSource,
    ) -> Result<(), sea_orm::DbErr> {
        let active = to_active(data_source);
        BizDataSource::insert(active).exec(&self.db).await?;
        Ok(())
    }

    pub async fn update_by_id(
        &self,
        data_source: &DataSource,
        _data_source_id: &i64,
    ) -> Result<(), sea_orm::DbErr> {
        let active = to_active(data_source);
        let _ = BizDataSource::update(active)
            .validate()?
            .filter(active_filter())
            .exec(&self.db)
            .await?;
        Ok(())
    }

    pub async fn find_by_code(
        &self,
        code: &str,
    ) -> Result<Option<DataSource>, sea_orm::DbErr> {
        let model = BizDataSource::find()
            .filter(Column::Code.eq(code))
            .filter(active_filter())
            .one(&self.db)
            .await?;
        Ok(model.as_ref().map(DataSource::from))
    }

    pub async fn find_by_code_excluding(
        &self,
        code: &str,
        exclude_id: &i64,
    ) -> Result<Option<DataSource>, sea_orm::DbErr> {
        let model = BizDataSource::find()
            .filter(Column::Code.eq(code))
            .filter(Column::DataSourceId.ne(*exclude_id))
            .filter(active_filter())
            .one(&self.db)
            .await?;
        Ok(model.as_ref().map(DataSource::from))
    }

    pub async fn delete_by_id(
        &self,
        data_source_id: &i64,
    ) -> Result<(), sea_orm::DbErr> {
        BizDataSource::delete_by_id(*data_source_id)
            .exec(&self.db)
            .await?;
        Ok(())
    }
}
