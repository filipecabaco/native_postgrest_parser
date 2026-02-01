use super::{LogicCondition, OrderTerm, SelectItem};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;

/// Parameters for calling a PostgreSQL function (RPC - Remote Procedure Call).
///
/// PostgREST allows calling database functions via `POST /rpc/function_name` with named arguments.
/// Function results can be filtered, ordered, and paginated like regular queries.
///
/// # SQL Generation
///
/// Generates queries like:
/// ```sql
/// SELECT * FROM "schema"."function_name"(arg1 := $1, arg2 := $2)
/// WHERE filter_column = $3
/// ORDER BY order_column
/// LIMIT $4 OFFSET $5
/// ```
///
/// # Examples
///
/// ```
/// use postgrest_parser::{RpcParams, parse_filter, parse_order, LogicCondition};
/// use std::collections::HashMap;
/// use serde_json::json;
///
/// // Simple function call
/// let mut args = HashMap::new();
/// args.insert("user_id".to_string(), json!(123));
///
/// let params = RpcParams::new("get_user_profile", args);
/// assert_eq!(params.function_name, "get_user_profile");
///
/// // Function call with filtering and pagination
/// let mut args = HashMap::new();
/// args.insert("department".to_string(), json!("engineering"));
///
/// let filter = parse_filter("active", "eq.true").unwrap();
/// let order = parse_order("salary.desc").unwrap();
///
/// let params = RpcParams::new("list_employees", args)
///     .with_filters(vec![LogicCondition::Filter(filter)])
///     .with_order(order)
///     .with_limit(20)
///     .with_offset(40);
///
/// assert_eq!(params.limit, Some(20));
/// assert_eq!(params.offset, Some(40));
/// ```
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RpcParams {
    /// The name of the function to call
    pub function_name: String,
    /// Named arguments to pass to the function
    pub args: HashMap<String, Value>,
    /// Optional filters to apply to function results
    pub filters: Vec<LogicCondition>,
    /// Optional ordering for function results
    pub order: Vec<OrderTerm>,
    /// Optional result limit
    pub limit: Option<u64>,
    /// Optional result offset (for pagination)
    pub offset: Option<u64>,
    /// Optional columns to select from function results
    pub returning: Option<Vec<SelectItem>>,
}

impl RpcParams {
    /// Creates new RPC parameters with function name and arguments.
    pub fn new(function_name: impl Into<String>, args: HashMap<String, Value>) -> Self {
        Self {
            function_name: function_name.into(),
            args,
            filters: Vec::new(),
            order: Vec::new(),
            limit: None,
            offset: None,
            returning: None,
        }
    }

    /// Adds filters to apply to function results.
    pub fn with_filters(mut self, filters: Vec<LogicCondition>) -> Self {
        self.filters = filters;
        self
    }

    /// Adds ordering to function results.
    pub fn with_order(mut self, order: Vec<OrderTerm>) -> Self {
        self.order = order;
        self
    }

    /// Sets the maximum number of results to return.
    pub fn with_limit(mut self, limit: u64) -> Self {
        self.limit = Some(limit);
        self
    }

    /// Sets the offset for pagination.
    pub fn with_offset(mut self, offset: u64) -> Self {
        self.offset = Some(offset);
        self
    }

    /// Specifies which columns to select from function results.
    pub fn with_returning(mut self, returning: Vec<SelectItem>) -> Self {
        self.returning = Some(returning);
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rpc_params_new() {
        let mut args = HashMap::new();
        args.insert("name".to_string(), Value::String("test".to_string()));

        let params = RpcParams::new("my_function", args.clone());

        assert_eq!(params.function_name, "my_function");
        assert_eq!(params.args, args);
        assert!(params.filters.is_empty());
        assert!(params.order.is_empty());
        assert_eq!(params.limit, None);
        assert_eq!(params.offset, None);
        assert_eq!(params.returning, None);
    }

    #[test]
    fn test_rpc_params_builder() {
        let mut args = HashMap::new();
        args.insert("user_id".to_string(), Value::Number(123.into()));

        let params = RpcParams::new("get_user_posts", args)
            .with_limit(10)
            .with_offset(20);

        assert_eq!(params.function_name, "get_user_posts");
        assert_eq!(params.limit, Some(10));
        assert_eq!(params.offset, Some(20));
    }

    #[test]
    fn test_rpc_params_empty_args() {
        let params = RpcParams::new("health_check", HashMap::new());

        assert_eq!(params.function_name, "health_check");
        assert!(params.args.is_empty());
    }
}
