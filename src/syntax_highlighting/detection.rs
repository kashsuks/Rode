pub fn detect_language(filename: &str) -> Option<String> {
    let extension = std::path::Path::new(filename)
        .extension()
        .and_then(|e| e.to_str())?;

    match extension {
        "rs" => Some("rust".to_string()),
        "js" | "mjs" | "cjs" | "jsx" => Some("javascript".to_string()),
        "ts" | "tsx" => Some("typescript".to_string()),
        "py" => Some("python".to_string()),
        _ => None,
    }
}
