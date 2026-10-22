use serde::Serialize;

use crate::common::vo::sql_run::SqlRunResultVo;

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DashboardRunVo {
    pub result: SqlRunResultVo,
    pub cached: bool,
}
