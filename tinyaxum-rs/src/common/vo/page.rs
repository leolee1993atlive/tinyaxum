use serde::{Deserialize, Deserializer};
use validator::Validate;

const DEFAULT_PAGE_SIZE: i64 = 10;
const MAX_PAGE_SIZE: i64 = 200;

/// Query / x-www-form-urlencoded scalars arrive as strings; Axum's form
/// deserializer does not coerce `"1"` into `i64` by default.
pub fn deserialize_opt_i64_from_str<'de, D>(
    deserializer: D,
) -> Result<Option<i64>, D::Error>
where
    D: Deserializer<'de>,
{
    let opt = Option::<String>::deserialize(deserializer)?;
    match opt {
        None => Ok(None),
        Some(s) if s.trim().is_empty() => Ok(None),
        Some(s) => s
            .trim()
            .parse::<i64>()
            .map(Some)
            .map_err(serde::de::Error::custom),
    }
}

#[derive(Debug, Clone, Default, Deserialize, Validate)]
#[serde(rename_all = "camelCase")]
pub struct PageQuery {
    #[serde(default, deserialize_with = "deserialize_opt_i64_from_str")]
    #[validate(range(min = 1, max = 1000))]
    pub page_no: Option<i64>,
    #[serde(default, deserialize_with = "deserialize_opt_i64_from_str")]
    #[validate(range(min = 1, max = 200))]
    pub page_size: Option<i64>,
    pub sort: Option<String>,
    pub order: Option<String>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PageResult<T> {
    pub records: Vec<T>,
    pub total: i64,
    pub pages: i64,
    pub current: i64,
    pub size: i64,
}

impl<T> PageResult<T> {
    pub fn empty(current: i64, size: i64) -> Self {
        Self {
            records: Vec::new(),
            total: 0,
            pages: 0,
            current,
            size,
        }
    }
}

pub fn normalize_paging(q: &PageQuery) -> (i64, i64, i64) {
    let mut current = q.page_no.unwrap_or(1);
    let mut size = q.page_size.unwrap_or(DEFAULT_PAGE_SIZE);

    if current < 1 {
        current = 1;
    }
    if size < 1 {
        size = DEFAULT_PAGE_SIZE;
    }
    if size > MAX_PAGE_SIZE {
        size = MAX_PAGE_SIZE;
    }

    let offset = (current - 1) * size;
    (current, size, offset)
}

pub fn sanitize_sort(
    sort: &Option<String>,
    order: &Option<String>,
    allowed: &[(&str, &str)],
) -> Option<String> {
    let sort = sort.as_deref()?.trim();
    if sort.is_empty() {
        return None;
    }

    let column = allowed
        .iter()
        .find_map(|(key, col)| (*key == sort).then_some(*col))?;

    let dir = match order.as_deref().map(|s| s.to_ascii_lowercase()) {
        Some(ref s) if s == "asc" => "asc",
        Some(ref s) if s == "desc" => "desc",
        _ => "desc",
    };

    Some(format!("{column} {dir}"))
}
