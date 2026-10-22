pub fn to_string(vec: Vec<String>) -> Option<String> {
    if vec.is_empty() {
        None
    } else {
        Some(vec.join(","))
    }
}

pub fn to_vec(option_string: Option<String>) -> Vec<String> {
    match option_string {
        None => Vec::new(),
        Some(s) => s
            .split(',')
            .map(|piece| piece.trim().to_string()) // 去掉前后空格
            .filter(|piece| !piece.is_empty()) // 跳过空片段
            .collect(),
    }
}
