use crate::ast::RpcParams;
use crate::error::{Error, ParseError};
use crate::parser::{parse_json_body, parse_order, parse_select};
use serde_json::Value;
use std::collections::HashMap;

/// Parses RPC parameters from query string and body
///
/// # Arguments
///
/// * `function_name` - Name of the function to call
/// * `query_string` - Query parameters (e.g., "limit=10&order=created_at.desc")
/// * `body` - Optional JSON body with function arguments
///
/// # Examples
///
/// ```
/// use postgrest_parser::parser::parse_rpc_params;
///
/// let body = r#"{"user_id": 123, "status": "active"}"#;
/// let params = parse_rpc_params("get_user_posts", "", Some(body)).unwrap();
/// assert_eq!(params.function_name, "get_user_posts");
/// ```
pub fn parse_rpc_params(
    function_name: &str,
    query_string: &str,
    body: Option<&str>,
) -> Result<RpcParams, Error> {
    // Parse arguments from body if present
    let args = if let Some(body_str) = body {
        let json_value = parse_json_body(body_str)?;
        validate_rpc_args(json_value)?
    } else {
        HashMap::new()
    };

    let mut params = RpcParams::new(function_name, args);

    // Parse query parameters
    let query_params = parse_query_params(query_string);

    // Parse filters - all non-reserved keys become filters
    let filter_pairs: Vec<(String, String)> = query_params
        .iter()
        .filter(|(k, _)| !is_reserved_key(k))
        .map(|(k, v)| (k.clone(), v.clone()))
        .collect();

    if !filter_pairs.is_empty() {
        let parsed_params = crate::parse_params_from_pairs(filter_pairs)?;
        params = params.with_filters(parsed_params.filters);
    }

    // Parse order clause
    if let Some(order_str) = query_params.get("order") {
        let order = parse_order(order_str)?;
        params = params.with_order(order);
    }

    // Parse limit
    if let Some(limit_str) = query_params.get("limit") {
        let limit = limit_str
            .parse::<u64>()
            .map_err(|_| Error::Parse(ParseError::InvalidLimit(limit_str.to_string())))?;
        params = params.with_limit(limit);
    }

    // Parse offset
    if let Some(offset_str) = query_params.get("offset") {
        let offset = offset_str
            .parse::<u64>()
            .map_err(|_| Error::Parse(ParseError::InvalidOffset(offset_str.to_string())))?;
        params = params.with_offset(offset);
    }

    // Parse returning (select) clause
    if let Some(select_str) = query_params.get("select") {
        let returning = parse_select(select_str)?;
        params = params.with_returning(returning);
    }

    Ok(params)
}

fn validate_rpc_args(value: Value) -> Result<HashMap<String, Value>, Error> {
    match value {
        Value::Object(map) => {
            let mut hash_map = HashMap::new();
            for (k, v) in map {
                hash_map.insert(k, v);
            }
            Ok(hash_map)
        }
        _ => Err(Error::Parse(ParseError::InvalidJsonBody(
            "RPC arguments must be a JSON object".to_string(),
        ))),
    }
}

fn parse_query_params(query_string: &str) -> HashMap<String, String> {
    if query_string.is_empty() {
        return HashMap::new();
    }

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
    matches!(key, "select" | "order" | "limit" | "offset")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_rpc_params_with_body() {
        let body = r#"{"user_id": 123, "status": "active"}"#;
        let params = parse_rpc_params("get_user_posts", "", Some(body)).unwrap();

        assert_eq!(params.function_name, "get_user_posts");
        assert_eq!(params.args.len(), 2);
        assert_eq!(params.args.get("user_id"), Some(&Value::Number(123.into())));
    }

    #[test]
    fn test_parse_rpc_params_no_body() {
        let params = parse_rpc_params("health_check", "", None).unwrap();

        assert_eq!(params.function_name, "health_check");
        assert!(params.args.is_empty());
    }

    #[test]
    fn test_parse_rpc_params_with_limit_offset() {
        let params = parse_rpc_params("list_users", "limit=10&offset=20", None).unwrap();

        assert_eq!(params.limit, Some(10));
        assert_eq!(params.offset, Some(20));
    }

    #[test]
    fn test_parse_rpc_params_with_order() {
        let params = parse_rpc_params("list_users", "order=created_at.desc", None).unwrap();

        assert_eq!(params.order.len(), 1);
    }

    #[test]
    fn test_parse_rpc_params_with_filters() {
        let params = parse_rpc_params("list_users", "age=gt.18&status=eq.active", None).unwrap();

        assert_eq!(params.filters.len(), 2);
    }

    #[test]
    fn test_parse_rpc_params_with_select() {
        let params = parse_rpc_params("get_posts", "select=id,title,author", None).unwrap();

        assert!(params.returning.is_some());
        assert_eq!(params.returning.unwrap().len(), 3);
    }

    #[test]
    fn test_parse_rpc_invalid_body() {
        let body = r#"["not", "an", "object"]"#;
        let result = parse_rpc_params("test_func", "", Some(body));

        assert!(result.is_err());
    }

    #[test]
    fn test_parse_rpc_complex_scenario() {
        let body = r#"{"department_id": 5}"#;
        let query = "age=gte.25&salary=lt.100000&order=name.asc&limit=50&select=id,name,salary";
        let params = parse_rpc_params("find_employees", query, Some(body)).unwrap();

        assert_eq!(params.function_name, "find_employees");
        assert_eq!(params.args.len(), 1);
        assert_eq!(params.filters.len(), 2);
        assert_eq!(params.order.len(), 1);
        assert_eq!(params.limit, Some(50));
        assert!(params.returning.is_some());
    }
}
