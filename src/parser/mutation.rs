use crate::ast::{ConflictAction, DeleteParams, InsertParams, OnConflict, UpdateParams};
use crate::error::{Error, ParseError};
use crate::parser::{
    parse_json_body, parse_order, parse_select, validate_insert_body, validate_update_body,
};
use std::collections::HashMap;

/// Parses INSERT operation parameters from query string and body
///
/// # Arguments
///
/// * `query_string` - Query parameters (e.g., "returning=id&on_conflict=email")
/// * `body` - JSON body with values to insert
///
/// # Examples
///
/// ```
/// use postgrest_parser::parser::parse_insert_params;
///
/// let body = r#"{"name": "Alice", "age": 30}"#;
/// let params = parse_insert_params("returning=id,created_at", body).unwrap();
/// assert!(params.returning.is_some());
/// ```
pub fn parse_insert_params(query_string: &str, body: &str) -> Result<InsertParams, Error> {
    // Parse and validate body
    let json_value = parse_json_body(body)?;
    let values = validate_insert_body(json_value)?;

    let mut params = InsertParams::new(values);

    // Parse query parameters
    let query_params = parse_query_params(query_string);

    // Parse returning clause (PostgREST uses 'select' parameter for mutations)
    if let Some(select_str) = query_params.get("select") {
        let returning = parse_select(select_str)?;
        params = params.with_returning(returning);
    } else if let Some(returning_str) = query_params.get("returning") {
        // Also support 'returning' for backwards compatibility
        let returning = parse_select(returning_str)?;
        params = params.with_returning(returning);
    }

    // Parse columns specification
    if let Some(columns_str) = query_params.get("columns") {
        let columns: Vec<String> = columns_str
            .split(',')
            .map(|s| s.trim().to_string())
            .collect();
        if !columns.is_empty() && !columns.iter().any(|c| c.is_empty()) {
            params = params.with_columns(columns);
        }
    }

    // Parse on_conflict specification
    if let Some(on_conflict_str) = query_params.get("on_conflict") {
        let on_conflict = parse_on_conflict(on_conflict_str)?;
        params = params.with_on_conflict(on_conflict);
    }

    Ok(params)
}

/// Parses UPDATE operation parameters from query string and body
///
/// # Arguments
///
/// * `query_string` - Query parameters with filters (e.g., "id=eq.123&returning=*")
/// * `body` - JSON body with values to update
///
/// # Examples
///
/// ```
/// use postgrest_parser::parser::parse_update_params;
///
/// let body = r#"{"status": "active"}"#;
/// let params = parse_update_params("id=eq.123", body).unwrap();
/// assert!(params.has_filters());
/// ```
pub fn parse_update_params(query_string: &str, body: &str) -> Result<UpdateParams, Error> {
    // Parse and validate body
    let json_value = parse_json_body(body)?;
    let set_values = validate_update_body(json_value)?;

    let mut params = UpdateParams::new(set_values);

    // Parse query parameters
    let query_params = parse_query_params(query_string);

    // Parse filters (anything that's not a reserved key)
    let filters = crate::parse_params_from_pairs(
        query_params
            .iter()
            .filter(|(k, _)| !is_reserved_key(k))
            .map(|(k, v)| (k.clone(), v.clone()))
            .collect(),
    )?;

    params = params.with_filters(filters.filters);

    // Parse order
    if let Some(order_str) = query_params.get("order") {
        let order = parse_order(order_str)?;
        params = params.with_order(order);
    }

    // Parse limit
    if let Some(limit_str) = query_params.get("limit") {
        let limit = limit_str.parse::<u64>().map_err(|_| {
            Error::Parse(ParseError::InvalidLimit(format!(
                "Invalid limit value: {}",
                limit_str
            )))
        })?;
        params = params.with_limit(limit);
    }

    // Parse returning clause (PostgREST uses 'select' parameter for mutations)
    if let Some(select_str) = query_params.get("select") {
        let returning = parse_select(select_str)?;
        params = params.with_returning(returning);
    } else if let Some(returning_str) = query_params.get("returning") {
        // Also support 'returning' for backwards compatibility
        let returning = parse_select(returning_str)?;
        params = params.with_returning(returning);
    }

    Ok(params)
}

/// Parses DELETE operation parameters from query string
///
/// # Arguments
///
/// * `query_string` - Query parameters with filters (e.g., "id=eq.123&returning=*")
///
/// # Examples
///
/// ```
/// use postgrest_parser::parser::parse_delete_params;
///
/// let params = parse_delete_params("status=eq.deleted").unwrap();
/// assert!(params.has_filters());
/// ```
pub fn parse_delete_params(query_string: &str) -> Result<DeleteParams, Error> {
    let mut params = DeleteParams::new();

    // Parse query parameters
    let query_params = parse_query_params(query_string);

    // Parse filters
    let filters = crate::parse_params_from_pairs(
        query_params
            .iter()
            .filter(|(k, _)| !is_reserved_key(k))
            .map(|(k, v)| (k.clone(), v.clone()))
            .collect(),
    )?;

    params = params.with_filters(filters.filters);

    // Parse order
    if let Some(order_str) = query_params.get("order") {
        let order = parse_order(order_str)?;
        params = params.with_order(order);
    }

    // Parse limit
    if let Some(limit_str) = query_params.get("limit") {
        let limit = limit_str.parse::<u64>().map_err(|_| {
            Error::Parse(ParseError::InvalidLimit(format!(
                "Invalid limit value: {}",
                limit_str
            )))
        })?;
        params = params.with_limit(limit);
    }

    // Parse returning clause (PostgREST uses 'select' parameter for mutations)
    if let Some(select_str) = query_params.get("select") {
        let returning = parse_select(select_str)?;
        params = params.with_returning(returning);
    } else if let Some(returning_str) = query_params.get("returning") {
        // Also support 'returning' for backwards compatibility
        let returning = parse_select(returning_str)?;
        params = params.with_returning(returning);
    }

    Ok(params)
}

fn parse_query_params(query_string: &str) -> HashMap<String, String> {
    query_string
        .split('&')
        .filter_map(|pair| {
            let parts: Vec<&str> = pair.splitn(2, '=').collect();
            if parts.len() == 2 {
                Some((parts[0].to_string(), parts[1].to_string()))
            } else {
                None
            }
        })
        .collect()
}

fn is_reserved_key(key: &str) -> bool {
    matches!(
        key,
        "select" | "order" | "limit" | "offset" | "on_conflict" | "columns" | "returning"
    )
}

fn parse_on_conflict(spec: &str) -> Result<OnConflict, Error> {
    // Format: "column1,column2" or "column1,column2.action"
    // where action can be "do_nothing" or "do_update" (default: do_nothing)

    let parts: Vec<&str> = spec.split('.').collect();

    let (columns_str, action) = match parts.len() {
        1 => (parts[0], ConflictAction::DoNothing), // Default to DO NOTHING
        2 => {
            let action = match parts[1].to_lowercase().as_str() {
                "do_nothing" => ConflictAction::DoNothing,
                "do_update" => ConflictAction::DoUpdate,
                _ => {
                    return Err(Error::Parse(ParseError::InvalidOnConflict(format!(
                        "Invalid conflict action: '{}'. Expected 'do_nothing' or 'do_update'",
                        parts[1]
                    ))))
                }
            };
            (parts[0], action)
        }
        _ => {
            return Err(Error::Parse(ParseError::InvalidOnConflict(format!(
                "Invalid on_conflict format: '{}'",
                spec
            ))))
        }
    };

    let columns: Vec<String> = columns_str
        .split(',')
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .collect();

    if columns.is_empty() {
        return Err(Error::Parse(ParseError::InvalidOnConflict(
            "on_conflict must specify at least one column".to_string(),
        )));
    }

    Ok(OnConflict {
        columns,
        action,
        where_clause: None,
        update_columns: None,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_insert_params_simple() {
        let body = r#"{"name": "Alice", "age": 30}"#;
        let result = parse_insert_params("", body);
        assert!(result.is_ok());
        let params = result.unwrap();
        assert!(params.returning.is_none());
        assert!(params.on_conflict.is_none());
    }

    #[test]
    fn test_parse_insert_params_with_returning() {
        let body = r#"{"name": "Alice"}"#;
        let result = parse_insert_params("returning=id,created_at", body);
        assert!(result.is_ok());
        let params = result.unwrap();
        assert!(params.returning.is_some());
        assert_eq!(params.returning.unwrap().len(), 2);
    }

    #[test]
    fn test_parse_insert_params_with_on_conflict() {
        let body = r#"{"email": "alice@example.com"}"#;
        let result = parse_insert_params("on_conflict=email", body);
        assert!(result.is_ok());
        let params = result.unwrap();
        assert!(params.on_conflict.is_some());
        let conflict = params.on_conflict.unwrap();
        assert_eq!(conflict.columns, vec!["email"]);
        assert_eq!(conflict.action, ConflictAction::DoNothing);
    }

    #[test]
    fn test_parse_insert_params_with_columns() {
        let body = r#"{"name": "Alice", "age": 30, "extra": "ignored"}"#;
        let result = parse_insert_params("columns=name,age", body);
        assert!(result.is_ok());
        let params = result.unwrap();
        assert!(params.columns.is_some());
        assert_eq!(params.columns.unwrap(), vec!["name", "age"]);
    }

    #[test]
    fn test_parse_update_params_simple() {
        let body = r#"{"status": "active"}"#;
        let result = parse_update_params("id=eq.123", body);
        assert!(result.is_ok());
        let params = result.unwrap();
        assert!(params.has_filters());
        assert_eq!(params.set_values.len(), 1);
    }

    #[test]
    fn test_parse_update_params_with_limit() {
        let body = r#"{"status": "active"}"#;
        let result = parse_update_params("status=eq.pending&limit=10", body);
        assert!(result.is_ok());
        let params = result.unwrap();
        assert_eq!(params.limit, Some(10));
    }

    #[test]
    fn test_parse_update_params_with_order() {
        let body = r#"{"status": "active"}"#;
        let result = parse_update_params("status=eq.pending&order=created_at.desc", body);
        assert!(result.is_ok());
        let params = result.unwrap();
        assert_eq!(params.order.len(), 1);
    }

    #[test]
    fn test_parse_delete_params_simple() {
        let result = parse_delete_params("id=eq.123");
        assert!(result.is_ok());
        let params = result.unwrap();
        assert!(params.has_filters());
    }

    #[test]
    fn test_parse_delete_params_with_returning() {
        let result = parse_delete_params("status=eq.deleted&returning=*");
        assert!(result.is_ok());
        let params = result.unwrap();
        assert!(params.returning.is_some());
    }

    #[test]
    fn test_parse_on_conflict_do_nothing() {
        let result = parse_on_conflict("email");
        assert!(result.is_ok());
        let conflict = result.unwrap();
        assert_eq!(conflict.columns, vec!["email"]);
        assert_eq!(conflict.action, ConflictAction::DoNothing);
    }

    #[test]
    fn test_parse_on_conflict_do_update() {
        let result = parse_on_conflict("email.do_update");
        assert!(result.is_ok());
        let conflict = result.unwrap();
        assert_eq!(conflict.columns, vec!["email"]);
        assert_eq!(conflict.action, ConflictAction::DoUpdate);
    }

    #[test]
    fn test_parse_on_conflict_multiple_columns() {
        let result = parse_on_conflict("email,username.do_nothing");
        assert!(result.is_ok());
        let conflict = result.unwrap();
        assert_eq!(conflict.columns, vec!["email", "username"]);
        assert_eq!(conflict.action, ConflictAction::DoNothing);
    }

    #[test]
    fn test_parse_on_conflict_invalid_action() {
        let result = parse_on_conflict("email.invalid_action");
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_on_conflict_empty_columns() {
        let result = parse_on_conflict("");
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_query_params() {
        let params = parse_query_params("id=eq.123&status=eq.active&limit=10");
        assert_eq!(params.len(), 3);
        assert_eq!(params.get("id"), Some(&"eq.123".to_string()));
        assert_eq!(params.get("limit"), Some(&"10".to_string()));
    }

    #[test]
    fn test_is_reserved_key() {
        assert!(is_reserved_key("select"));
        assert!(is_reserved_key("order"));
        assert!(is_reserved_key("limit"));
        assert!(is_reserved_key("returning"));
        assert!(is_reserved_key("on_conflict"));
        assert!(!is_reserved_key("id"));
        assert!(!is_reserved_key("status"));
    }
}
