/// Filters for Askama templates

/// Replace certain characters within a string with others
pub fn replace<T: std::fmt::Display>(s: T, from: &str, to: &str) -> ::askama::Result<String> {
    let s = s.to_string();
    Ok(s.replace(from, to))
}
