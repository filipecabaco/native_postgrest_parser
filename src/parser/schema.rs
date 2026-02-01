use crate::ast::ResolvedTable;
use crate::error::{Error, ParseError};
use std::collections::HashMap;

/// Resolves the schema and table name from various sources with priority:
/// 1. Explicit schema in table name (e.g., "auth.users")
/// 2. Header-based (Accept-Profile for GET, Content-Profile for POST/PATCH/DELETE)
/// 3. Default "public" schema
///
/// # Arguments
///
/// * `table` - Table name, optionally schema-qualified (e.g., "users" or "auth.users")
/// * `method` - HTTP method (GET, POST, PATCH, DELETE)
/// * `headers` - Optional headers map containing profile headers
///
/// # Examples
///
/// ```
/// use postgrest_parser::parser::resolve_schema;
/// use std::collections::HashMap;
///
/// // Explicit schema in table name
/// let table = resolve_schema("auth.users", "GET", None).unwrap();
/// assert_eq!(table.schema, "auth");
/// assert_eq!(table.name, "users");
///
/// // Header-based schema
/// let mut headers = HashMap::new();
/// headers.insert("Accept-Profile".to_string(), "myschema".to_string());
/// let table = resolve_schema("users", "GET", Some(&headers)).unwrap();
/// assert_eq!(table.schema, "myschema");
///
/// // Default public schema
/// let table = resolve_schema("users", "POST", None).unwrap();
/// assert_eq!(table.schema, "public");
/// ```
pub fn resolve_schema(
    table: &str,
    method: &str,
    headers: Option<&HashMap<String, String>>,
) -> Result<ResolvedTable, Error> {
    if table.is_empty() {
        return Err(Error::Parse(ParseError::InvalidTableName(
            "Table name cannot be empty".to_string(),
        )));
    }

    // Try to parse schema from table name first (e.g., "schema.table")
    if let Some((schema, name)) = parse_qualified_table(table)? {
        validate_identifier(&schema)?;
        validate_identifier(&name)?;
        return Ok(ResolvedTable::new(schema, name));
    }

    // Table name is not schema-qualified, use headers or default
    validate_identifier(table)?;

    let schema = get_profile_header(method, headers).unwrap_or_else(|| "public".to_string());
    validate_identifier(&schema)?;

    Ok(ResolvedTable::new(schema, table))
}

/// Parses a potentially schema-qualified table name.
///
/// Returns (schema, table) if qualified, None if not qualified.
///
/// # Examples
///
/// ```
/// use postgrest_parser::parser::parse_qualified_table;
///
/// let result = parse_qualified_table("auth.users").unwrap();
/// assert_eq!(result, Some(("auth".to_string(), "users".to_string())));
///
/// let result = parse_qualified_table("users").unwrap();
/// assert_eq!(result, None);
/// ```
pub fn parse_qualified_table(table: &str) -> Result<Option<(String, String)>, Error> {
    let parts: Vec<&str> = table.split('.').collect();

    match parts.len() {
        1 => Ok(None),
        2 => {
            let schema = parts[0].trim();
            let name = parts[1].trim();

            if schema.is_empty() || name.is_empty() {
                return Err(Error::Parse(ParseError::InvalidTableName(format!(
                    "Invalid qualified table name: '{}'",
                    table
                ))));
            }

            Ok(Some((schema.to_string(), name.to_string())))
        }
        _ => Err(Error::Parse(ParseError::InvalidTableName(format!(
            "Invalid table name format: '{}'. Expected 'table' or 'schema.table'",
            table
        )))),
    }
}

/// Gets the appropriate profile header based on HTTP method.
///
/// - GET → Accept-Profile
/// - POST/PATCH/DELETE → Content-Profile
///
/// # Examples
///
/// ```
/// use postgrest_parser::parser::get_profile_header;
/// use std::collections::HashMap;
///
/// let mut headers = HashMap::new();
/// headers.insert("Accept-Profile".to_string(), "myschema".to_string());
/// let schema = get_profile_header("GET", Some(&headers));
/// assert_eq!(schema, Some("myschema".to_string()));
///
/// let mut headers = HashMap::new();
/// headers.insert("Content-Profile".to_string(), "auth".to_string());
/// let schema = get_profile_header("POST", Some(&headers));
/// assert_eq!(schema, Some("auth".to_string()));
/// ```
pub fn get_profile_header(
    method: &str,
    headers: Option<&HashMap<String, String>>,
) -> Option<String> {
    let headers = headers?;

    let header_name = match method.to_uppercase().as_str() {
        "GET" => "Accept-Profile",
        "POST" | "PATCH" | "DELETE" => "Content-Profile",
        _ => return None,
    };

    // Check both exact case and case-insensitive
    headers
        .get(header_name)
        .or_else(|| {
            let lowercase = header_name.to_lowercase();
            headers
                .iter()
                .find(|(k, _)| k.to_lowercase() == lowercase)
                .map(|(_, v)| v)
        })
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
}

/// Validates that an identifier (schema or table name) contains only valid characters.
///
/// Valid identifiers:
/// - Start with a letter (a-z, A-Z) or underscore
/// - Contain only letters, digits, underscores
/// - Are not empty
fn validate_identifier(identifier: &str) -> Result<(), Error> {
    if identifier.is_empty() {
        return Err(Error::Parse(ParseError::InvalidTableName(
            "Identifier cannot be empty".to_string(),
        )));
    }

    let first_char = identifier.chars().next().unwrap();
    if !first_char.is_alphabetic() && first_char != '_' {
        return Err(Error::Parse(ParseError::InvalidSchema(format!(
            "Identifier '{}' must start with a letter or underscore",
            identifier
        ))));
    }

    for ch in identifier.chars() {
        if !ch.is_alphanumeric() && ch != '_' {
            return Err(Error::Parse(ParseError::InvalidSchema(format!(
                "Identifier '{}' contains invalid character '{}'",
                identifier, ch
            ))));
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_resolve_schema_explicit_in_table_name() {
        let table = resolve_schema("auth.users", "GET", None).unwrap();
        assert_eq!(table.schema, "auth");
        assert_eq!(table.name, "users");
    }

    #[test]
    fn test_resolve_schema_with_accept_profile_header() {
        let mut headers = HashMap::new();
        headers.insert("Accept-Profile".to_string(), "myschema".to_string());
        let table = resolve_schema("users", "GET", Some(&headers)).unwrap();
        assert_eq!(table.schema, "myschema");
        assert_eq!(table.name, "users");
    }

    #[test]
    fn test_resolve_schema_with_content_profile_header() {
        let mut headers = HashMap::new();
        headers.insert("Content-Profile".to_string(), "auth".to_string());
        let table = resolve_schema("users", "POST", Some(&headers)).unwrap();
        assert_eq!(table.schema, "auth");
        assert_eq!(table.name, "users");
    }

    #[test]
    fn test_resolve_schema_default_public() {
        let table = resolve_schema("users", "POST", None).unwrap();
        assert_eq!(table.schema, "public");
        assert_eq!(table.name, "users");
    }

    #[test]
    fn test_resolve_schema_explicit_overrides_header() {
        let mut headers = HashMap::new();
        headers.insert("Accept-Profile".to_string(), "other".to_string());
        let table = resolve_schema("auth.users", "GET", Some(&headers)).unwrap();
        assert_eq!(table.schema, "auth"); // Explicit wins
        assert_eq!(table.name, "users");
    }

    #[test]
    fn test_resolve_schema_empty_table_name() {
        let result = resolve_schema("", "GET", None);
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_qualified_table_with_schema() {
        let result = parse_qualified_table("auth.users").unwrap();
        assert_eq!(result, Some(("auth".to_string(), "users".to_string())));
    }

    #[test]
    fn test_parse_qualified_table_without_schema() {
        let result = parse_qualified_table("users").unwrap();
        assert_eq!(result, None);
    }

    #[test]
    fn test_parse_qualified_table_multiple_dots() {
        let result = parse_qualified_table("schema.table.extra");
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_qualified_table_empty_parts() {
        let result = parse_qualified_table(".users");
        assert!(result.is_err());

        let result = parse_qualified_table("schema.");
        assert!(result.is_err());
    }

    #[test]
    fn test_get_profile_header_accept_profile() {
        let mut headers = HashMap::new();
        headers.insert("Accept-Profile".to_string(), "myschema".to_string());
        let schema = get_profile_header("GET", Some(&headers));
        assert_eq!(schema, Some("myschema".to_string()));
    }

    #[test]
    fn test_get_profile_header_content_profile_post() {
        let mut headers = HashMap::new();
        headers.insert("Content-Profile".to_string(), "auth".to_string());
        let schema = get_profile_header("POST", Some(&headers));
        assert_eq!(schema, Some("auth".to_string()));
    }

    #[test]
    fn test_get_profile_header_content_profile_patch() {
        let mut headers = HashMap::new();
        headers.insert("Content-Profile".to_string(), "auth".to_string());
        let schema = get_profile_header("PATCH", Some(&headers));
        assert_eq!(schema, Some("auth".to_string()));
    }

    #[test]
    fn test_get_profile_header_content_profile_delete() {
        let mut headers = HashMap::new();
        headers.insert("Content-Profile".to_string(), "auth".to_string());
        let schema = get_profile_header("DELETE", Some(&headers));
        assert_eq!(schema, Some("auth".to_string()));
    }

    #[test]
    fn test_get_profile_header_no_headers() {
        let schema = get_profile_header("GET", None);
        assert_eq!(schema, None);
    }

    #[test]
    fn test_get_profile_header_empty_value() {
        let mut headers = HashMap::new();
        headers.insert("Accept-Profile".to_string(), "".to_string());
        let schema = get_profile_header("GET", Some(&headers));
        assert_eq!(schema, None);
    }

    #[test]
    fn test_get_profile_header_whitespace_trimmed() {
        let mut headers = HashMap::new();
        headers.insert("Accept-Profile".to_string(), "  myschema  ".to_string());
        let schema = get_profile_header("GET", Some(&headers));
        assert_eq!(schema, Some("myschema".to_string()));
    }

    #[test]
    fn test_get_profile_header_case_insensitive() {
        let mut headers = HashMap::new();
        headers.insert("accept-profile".to_string(), "myschema".to_string());
        let schema = get_profile_header("GET", Some(&headers));
        assert_eq!(schema, Some("myschema".to_string()));
    }

    #[test]
    fn test_validate_identifier_valid() {
        assert!(validate_identifier("users").is_ok());
        assert!(validate_identifier("_users").is_ok());
        assert!(validate_identifier("users_table").is_ok());
        assert!(validate_identifier("users123").is_ok());
        assert!(validate_identifier("MyTable").is_ok());
    }

    #[test]
    fn test_validate_identifier_invalid_start() {
        assert!(validate_identifier("123users").is_err());
        assert!(validate_identifier("-users").is_err());
    }

    #[test]
    fn test_validate_identifier_invalid_chars() {
        assert!(validate_identifier("users-table").is_err());
        assert!(validate_identifier("users.table").is_err());
        assert!(validate_identifier("users@table").is_err());
    }

    #[test]
    fn test_validate_identifier_empty() {
        assert!(validate_identifier("").is_err());
    }

    #[test]
    fn test_resolved_table_qualified_name() {
        let table = ResolvedTable::new("auth", "users");
        assert_eq!(table.qualified_name(), "\"auth\".\"users\"");
    }
}
