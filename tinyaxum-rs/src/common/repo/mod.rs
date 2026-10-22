/// Trim optional string, returning empty string if None or
/// whitespace-only.
pub fn str_trim_or_empty(s: &Option<String>) -> String {
    s.as_ref()
        .map(|v| v.trim())
        .filter(|v| !v.is_empty())
        .unwrap_or_default()
        .to_owned()
}
