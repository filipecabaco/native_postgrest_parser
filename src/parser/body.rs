use crate::ast::InsertValues;
use crate::error::{Error, ParseError};
use serde_json::Value;
use std::collections::HashMap;

/// Parses a JSON body string into a serde_json::Value
///
/// # Arguments
///
/// * `body` - JSON string representing the request body
///
/// # Examples
///
/// ```
/// use postgrest_parser::parser::parse_json_body;
///
/// let body = r#"{"name": "Alice", "age": 30}"#;
/// let value = parse_json_body(body).unwrap();
/// assert!(value.is_object());
/// ```
pub fn parse_json_body(body: &str) -> Result<Value, Error> {
    serde_json::from_str(body).map_err(|e| {
        Error::Parse(ParseError::InvalidJsonBody(format!(
            "Failed to parse JSON body: {}",
            e
        )))
    })
}

/// Validates and converts a JSON value into InsertValues
///
/// Supports both single object and array of objects.
///
/// # Arguments
///
/// * `value` - Parsed JSON value to validate
///
/// # Examples
///
/// ```
/// use postgrest_parser::parser::{parse_json_body, validate_insert_body};
///
/// // Single row
/// let body = r#"{"name": "Alice", "age": 30}"#;
/// let value = parse_json_body(body).unwrap();
/// let insert_values = validate_insert_body(value).unwrap();
///
/// // Multiple rows
/// let body = r#"[{"name": "Alice"}, {"name": "Bob"}]"#;
/// let value = parse_json_body(body).unwrap();
/// let insert_values = validate_insert_body(value).unwrap();
/// ```
pub fn validate_insert_body(value: Value) -> Result<InsertValues, Error> {
    match value {
        Value::Object(map) => {
            if map.is_empty() {
                return Err(Error::Parse(ParseError::InvalidInsertBody(
                    "Insert body cannot be an empty object".to_string(),
                )));
            }
            let mut hash_map = HashMap::new();
            for (k, v) in map {
                hash_map.insert(k, v);
            }
            Ok(InsertValues::Single(hash_map))
        }
        Value::Array(arr) => {
            if arr.is_empty() {
                return Err(Error::Parse(ParseError::InvalidInsertBody(
                    "Insert body cannot be an empty array".to_string(),
                )));
            }

            let mut rows = Vec::new();
            for (idx, item) in arr.into_iter().enumerate() {
                match item {
                    Value::Object(map) => {
                        if map.is_empty() {
                            return Err(Error::Parse(ParseError::InvalidInsertBody(format!(
                                "Row {} is an empty object",
                                idx
                            ))));
                        }
                        let mut hash_map = HashMap::new();
                        for (k, v) in map {
                            hash_map.insert(k, v);
                        }
                        rows.push(hash_map);
                    }
                    _ => {
                        return Err(Error::Parse(ParseError::InvalidInsertBody(format!(
                            "Row {} must be an object, got {:?}",
                            idx, item
                        ))));
                    }
                }
            }

            Ok(InsertValues::Bulk(rows))
        }
        _ => Err(Error::Parse(ParseError::InvalidInsertBody(
            "Insert body must be an object or array of objects".to_string(),
        ))),
    }
}

/// Validates and converts a JSON value into a HashMap for UPDATE operations
///
/// # Arguments
///
/// * `value` - Parsed JSON value to validate
///
/// # Examples
///
/// ```
/// use postgrest_parser::parser::{parse_json_body, validate_update_body};
///
/// let body = r#"{"status": "active", "updated_at": "2024-01-01"}"#;
/// let value = parse_json_body(body).unwrap();
/// let update_values = validate_update_body(value).unwrap();
/// assert_eq!(update_values.len(), 2);
/// ```
pub fn validate_update_body(value: Value) -> Result<HashMap<String, Value>, Error> {
    match value {
        Value::Object(map) => {
            if map.is_empty() {
                return Err(Error::Parse(ParseError::EmptyUpdateBody));
            }
            let mut hash_map = HashMap::new();
            for (k, v) in map {
                hash_map.insert(k, v);
            }
            Ok(hash_map)
        }
        _ => Err(Error::Parse(ParseError::InvalidUpdateBody(
            "Update body must be an object".to_string(),
        ))),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_json_body_valid_object() {
        let body = r#"{"name": "Alice", "age": 30}"#;
        let result = parse_json_body(body);
        assert!(result.is_ok());
        let value = result.unwrap();
        assert!(value.is_object());
    }

    #[test]
    fn test_parse_json_body_valid_array() {
        let body = r#"[{"name": "Alice"}, {"name": "Bob"}]"#;
        let result = parse_json_body(body);
        assert!(result.is_ok());
        let value = result.unwrap();
        assert!(value.is_array());
    }

    #[test]
    fn test_parse_json_body_invalid_json() {
        let body = r#"{"name": "Alice""#; // Missing closing brace
        let result = parse_json_body(body);
        assert!(result.is_err());
    }

    #[test]
    fn test_validate_insert_body_single_object() {
        let value = serde_json::json!({"name": "Alice", "age": 30});
        let result = validate_insert_body(value);
        assert!(result.is_ok());
        let insert_values = result.unwrap();
        assert_eq!(insert_values.len(), 1);
    }

    #[test]
    fn test_validate_insert_body_empty_object() {
        let value = serde_json::json!({});
        let result = validate_insert_body(value);
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            Error::Parse(ParseError::InvalidInsertBody(_))
        ));
    }

    #[test]
    fn test_validate_insert_body_array() {
        let value = serde_json::json!([
            {"name": "Alice", "age": 30},
            {"name": "Bob", "age": 25}
        ]);
        let result = validate_insert_body(value);
        assert!(result.is_ok());
        let insert_values = result.unwrap();
        assert_eq!(insert_values.len(), 2);
    }

    #[test]
    fn test_validate_insert_body_empty_array() {
        let value = serde_json::json!([]);
        let result = validate_insert_body(value);
        assert!(result.is_err());
    }

    #[test]
    fn test_validate_insert_body_array_with_empty_object() {
        let value = serde_json::json!([{"name": "Alice"}, {}]);
        let result = validate_insert_body(value);
        assert!(result.is_err());
    }

    #[test]
    fn test_validate_insert_body_array_with_non_object() {
        let value = serde_json::json!([{"name": "Alice"}, "invalid"]);
        let result = validate_insert_body(value);
        assert!(result.is_err());
    }

    #[test]
    fn test_validate_insert_body_invalid_type() {
        let value = serde_json::json!("string");
        let result = validate_insert_body(value);
        assert!(result.is_err());

        let value = serde_json::json!(123);
        let result = validate_insert_body(value);
        assert!(result.is_err());

        let value = serde_json::json!(true);
        let result = validate_insert_body(value);
        assert!(result.is_err());
    }

    #[test]
    fn test_validate_update_body_valid() {
        let value = serde_json::json!({"status": "active", "updated_at": "2024-01-01"});
        let result = validate_update_body(value);
        assert!(result.is_ok());
        let update_values = result.unwrap();
        assert_eq!(update_values.len(), 2);
        assert!(update_values.contains_key("status"));
        assert!(update_values.contains_key("updated_at"));
    }

    #[test]
    fn test_validate_update_body_empty_object() {
        let value = serde_json::json!({});
        let result = validate_update_body(value);
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            Error::Parse(ParseError::EmptyUpdateBody)
        ));
    }

    #[test]
    fn test_validate_update_body_invalid_type() {
        let value = serde_json::json!([{"status": "active"}]);
        let result = validate_update_body(value);
        assert!(result.is_err());

        let value = serde_json::json!("string");
        let result = validate_update_body(value);
        assert!(result.is_err());
    }

    #[test]
    fn test_validate_update_body_nested_values() {
        let value = serde_json::json!({
            "user": {
                "name": "Alice",
                "age": 30
            },
            "metadata": ["tag1", "tag2"]
        });
        let result = validate_update_body(value);
        assert!(result.is_ok());
        let update_values = result.unwrap();
        assert_eq!(update_values.len(), 2);
    }

    #[test]
    fn test_bulk_insert_consistency() {
        let value = serde_json::json!([
            {"name": "Alice", "age": 30},
            {"name": "Bob", "age": 25},
            {"name": "Charlie", "age": 35}
        ]);
        let result = validate_insert_body(value);
        assert!(result.is_ok());
        let insert_values = result.unwrap();

        match insert_values {
            InsertValues::Bulk(rows) => {
                assert_eq!(rows.len(), 3);
                for row in &rows {
                    assert!(row.contains_key("name"));
                    assert!(row.contains_key("age"));
                }
            }
            _ => panic!("Expected Bulk insert"),
        }
    }
}
