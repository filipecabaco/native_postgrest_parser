//! # PostgREST Query Parser
//!
//! A high-performance Rust library for parsing PostgREST query strings into structured SQL queries.
//!
//! ## Features
//!
//! - **Complete PostgREST API Support**: All 22+ filter operators (eq, neq, gt, gte, lt, lte, like, ilike, match, imatch, in, is, fts, plfts, phfts, wfts, cs, cd, ov, sl, sr, nxl, nxr, adj)
//! - **Logic Operators**: AND, OR, NOT with arbitrary nesting
//! - **JSON Path Navigation**: `->` and `->>` operators for JSONB fields
//! - **Type Casting**: Cast fields with `::type` syntax
//! - **Full-Text Search**: Multiple FTS operators with language support
//! - **Quantifiers**: `any` and `all` quantifiers for array comparisons
//! - **Array/Range Operators**: PostgreSQL array and range type support
//! - **Ordering**: Multi-column ordering with nulls handling
//! - **Pagination**: `limit` and `offset` support
//! - **SQL Generation**: Convert parsed queries to parameterized PostgreSQL SQL
//!
//! ## Quick Start
//!
//! ```rust
//! use postgrest_parser::{parse_query_string, query_string_to_sql};
//!
//! // Parse a PostgREST query string
//! let query = "select=id,name,email&age=gte.18&status=in.(active,pending)&order=created_at.desc&limit=10";
//! let params = parse_query_string(query).unwrap();
//!
//! assert!(params.has_select());
//! assert!(params.has_filters());
//! assert_eq!(params.limit, Some(10));
//!
//! // Convert to SQL
//! let result = query_string_to_sql("users", query).unwrap();
//! println!("SQL: {}", result.query);
//! println!("Params: {:?}", result.params);
//! ```
//!
//! ## Filter Operators
//!
//! ### Comparison Operators
//! - `eq` - Equal to
//! - `neq` - Not equal to
//! - `gt` - Greater than
//! - `gte` - Greater than or equal to
//! - `lt` - Less than
//! - `lte` - Less than or equal to
//!
//! ### Pattern Matching
//! - `like` - SQL LIKE pattern matching
//! - `ilike` - Case-insensitive LIKE
//! - `match` - POSIX regex match
//! - `imatch` - Case-insensitive regex match
//!
//! ### Array Operators
//! - `in` - Value in list
//! - `cs` - Contains (array/range)
//! - `cd` - Contained in (array/range)
//! - `ov` - Overlaps (array)
//!
//! ### Full-Text Search
//! - `fts` - Full-text search using plainto_tsquery
//! - `plfts` - Plain full-text search (alias for fts)
//! - `phfts` - Phrase full-text search using phraseto_tsquery
//! - `wfts` - Websearch full-text search using websearch_to_tsquery
//!
//! ### Range Operators
//! - `sl` - Strictly left of
//! - `sr` - Strictly right of
//! - `nxl` - Does not extend to right of
//! - `nxr` - Does not extend to left of
//! - `adj` - Adjacent to
//!
//! ### Special Operators
//! - `is` - IS NULL, IS TRUE, IS FALSE, etc.
//!
//! ## Examples
//!
//! ### Simple Filtering
//! ```rust
//! use postgrest_parser::parse_query_string;
//!
//! let query = "age=gte.18&status=eq.active";
//! let params = parse_query_string(query).unwrap();
//! assert_eq!(params.filters.len(), 2);
//! ```
//!
//! ### Logic Operators
//! ```rust
//! use postgrest_parser::parse_query_string;
//!
//! let query = "and=(age.gte.18,status.eq.active)";
//! let params = parse_query_string(query).unwrap();
//! assert!(params.has_filters());
//! ```
//!
//! ### JSON Path Navigation
//! ```rust
//! use postgrest_parser::parse_query_string;
//!
//! let query = "data->name=eq.John&data->>email=like.*@example.com";
//! let params = parse_query_string(query).unwrap();
//! assert_eq!(params.filters.len(), 2);
//! ```
//!
//! ### Full-Text Search
//! ```rust
//! use postgrest_parser::parse_query_string;
//!
//! let query = "content=fts(english).search term";
//! let params = parse_query_string(query).unwrap();
//! ```
//!
//! ### Quantifiers
//! ```rust
//! use postgrest_parser::parse_query_string;
//!
//! let query = "tags=eq(any).{rust,elixir}";
//! let params = parse_query_string(query).unwrap();
//! ```

use serde::Serialize;

pub mod ast;
pub mod error;
pub mod parser;
pub mod sql;

#[cfg(feature = "wasm")]
pub mod wasm;

pub use ast::{
    Cardinality, Column, Direction, Field, Filter, FilterOperator, FilterValue, ItemHint,
    ItemType, JsonOp, Junction, LogicCondition, LogicOperator, LogicTree, Nulls, OrderTerm,
    ParsedParams, Quantifier, Relationship, SelectItem, Table,
};
pub use error::{Error, ParseError, SqlError};
pub use parser::{
    field, identifier, json_path, json_path_segment, logic_key, parse_filter, parse_logic,
    parse_order, parse_order_term, parse_select, reserved_key, type_cast,
};
pub use sql::{QueryBuilder, QueryResult};

/// Parses a PostgREST query string into structured parameters.
///
/// # Arguments
///
/// * `query_string` - A query string in PostgREST format (e.g., "select=id,name&age=gte.18")
///
/// # Returns
///
/// Returns `Ok(ParsedParams)` containing parsed select, filters, order, limit, and offset,
/// or an `Err(Error)` if parsing fails.
///
/// # Examples
///
/// ```
/// use postgrest_parser::parse_query_string;
///
/// let query = "select=id,name&age=gte.18&order=created_at.desc&limit=10";
/// let params = parse_query_string(query).unwrap();
///
/// assert!(params.has_select());
/// assert!(params.has_filters());
/// assert_eq!(params.limit, Some(10));
/// ```
pub fn parse_query_string(query_string: &str) -> Result<ParsedParams, Error> {
    let pairs: Vec<(String, String)> = query_string
        .split('&')
        .filter_map(|pair| {
            let parts: Vec<&str> = pair.splitn(2, '=').collect();
            if parts.len() == 2 {
                Some((parts[0].to_string(), parts[1].to_string()))
            } else {
                None
            }
        })
        .collect();

    parse_params_from_pairs(pairs)
}

/// Parses query parameters from a HashMap into structured parameters.
///
/// This is useful when you already have parsed URL parameters (e.g., from a web framework).
///
/// # Arguments
///
/// * `params` - A HashMap containing query parameter key-value pairs
///
/// # Examples
///
/// ```
/// use postgrest_parser::parse_params;
/// use std::collections::HashMap;
///
/// let mut params = HashMap::new();
/// params.insert("select".to_string(), "id,name".to_string());
/// params.insert("age".to_string(), "gte.18".to_string());
///
/// let parsed = parse_params(&params).unwrap();
/// assert!(parsed.has_select());
/// assert!(parsed.has_filters());
/// ```
pub fn parse_params(
    params: &std::collections::HashMap<String, String>,
) -> Result<ParsedParams, Error> {
    let select_str = params.get("select").map(|s| s.to_string());
    let order_str = params.get("order").map(|s| s.to_string());
    let filters = parse_filters_from_map(params)?;
    let limit = params.get("limit").and_then(|s| s.parse::<u64>().ok());
    let offset = params.get("offset").and_then(|s| s.parse::<u64>().ok());

    let mut parsed = ParsedParams::new().with_filters(filters);

    if let Some(select_str) = select_str {
        parsed = parsed.with_select(parse_select(&select_str)?);
    }

    if let Some(order_str) = order_str {
        parsed = parsed.with_order(parse_order(&order_str)?);
    }

    if let Some(lim) = limit {
        parsed = parsed.with_limit(lim);
    }

    if let Some(off) = offset {
        parsed = parsed.with_offset(off);
    }

    Ok(parsed)
}

pub fn parse_params_from_pairs(pairs: Vec<(String, String)>) -> Result<ParsedParams, Error> {
    let mut params_map = std::collections::HashMap::new();
    for (key, value) in pairs {
        params_map.insert(key, value);
    }
    parse_params(&params_map)
}

/// Converts parsed parameters into a parameterized PostgreSQL SELECT query.
///
/// # Arguments
///
/// * `table` - The table name to query
/// * `params` - Parsed parameters containing select, filters, order, limit, and offset
///
/// # Returns
///
/// Returns a `QueryResult` containing the SQL query string and parameter values.
///
/// # Examples
///
/// ```
/// use postgrest_parser::{parse_query_string, to_sql};
///
/// let params = parse_query_string("age=gte.18&order=name.asc&limit=10").unwrap();
/// let result = to_sql("users", &params).unwrap();
///
/// assert!(result.query.contains("SELECT"));
/// assert!(result.query.contains("WHERE"));
/// assert!(result.query.contains("ORDER BY"));
/// assert!(result.query.contains("LIMIT"));
/// ```
pub fn to_sql(table: &str, params: &ParsedParams) -> Result<QueryResult, Error> {
    if table.is_empty() {
        return Err(Error::Sql(SqlError::EmptyTableName));
    }

    let mut builder = QueryBuilder::new();
    builder.build_select(table, params).map_err(Error::Sql)
}

/// Parses a PostgREST query string and converts it directly to SQL.
///
/// This is a convenience function that combines `parse_query_string` and `to_sql`.
///
/// # Arguments
///
/// * `table` - The table name to query
/// * `query_string` - A PostgREST query string
///
/// # Returns
///
/// Returns a `QueryResult` containing the SQL query and parameters.
///
/// # Examples
///
/// ```
/// use postgrest_parser::query_string_to_sql;
///
/// let result = query_string_to_sql(
///     "users",
///     "select=id,name,email&age=gte.18&status=eq.active"
/// ).unwrap();
///
/// assert!(result.query.contains("SELECT"));
/// assert!(result.query.contains("\"id\""));
/// assert_eq!(result.tables, vec!["users"]);
/// ```
pub fn query_string_to_sql(table: &str, query_string: &str) -> Result<QueryResult, Error> {
    let params = parse_query_string(query_string)?;
    to_sql(table, &params)
}

/// Builds a WHERE clause from filter conditions without the full query.
///
/// Useful when you need just the filter clause portion of a query.
///
/// # Arguments
///
/// * `filters` - A slice of logic conditions (filters)
///
/// # Returns
///
/// Returns a `FilterClauseResult` containing the WHERE clause and parameters.
///
/// # Examples
///
/// ```
/// use postgrest_parser::{build_filter_clause, Filter, Field, FilterOperator, FilterValue, LogicCondition};
///
/// let filter = LogicCondition::Filter(Filter::new(
///     Field::new("age"),
///     FilterOperator::Gte,
///     FilterValue::Single("18".to_string()),
/// ));
///
/// let result = build_filter_clause(&[filter]).unwrap();
/// assert!(result.clause.contains("\"age\""));
/// assert!(result.clause.contains(">="));
/// ```
pub fn build_filter_clause(filters: &[LogicCondition]) -> Result<FilterClauseResult, Error> {
    let mut builder = QueryBuilder::new();
    builder.build_where_clause(filters).map_err(Error::Sql)?;

    Ok(FilterClauseResult {
        clause: builder.sql.clone(),
        params: builder.params.clone(),
    })
}

fn parse_filters_from_map(
    params: &std::collections::HashMap<String, String>,
) -> Result<Vec<LogicCondition>, Error> {
    let mut filters = Vec::new();

    for (key, value) in params {
        if parser::filter::reserved_key(key) {
            continue;
        }

        if parser::logic::logic_key(key) {
            let tree = parse_logic(key, value)?;
            filters.push(LogicCondition::Logic(tree));
        } else {
            let filter = parse_filter(key, value)?;
            filters.push(LogicCondition::Filter(filter));
        }
    }

    Ok(filters)
}

/// Result of building a filter clause.
///
/// Contains the SQL WHERE clause fragment and associated parameter values.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct FilterClauseResult {
    /// The WHERE clause SQL fragment (without the "WHERE" keyword)
    pub clause: String,
    /// Parameter values referenced in the clause
    pub params: Vec<serde_json::Value>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_query_string_empty() {
        let result = parse_query_string("");
        assert!(result.is_ok());
        let params = result.unwrap();
        assert!(params.is_empty());
    }

    #[test]
    fn test_parse_query_string_simple() {
        let result = parse_query_string("select=id,name&id=eq.1");
        assert!(result.is_ok());
        let params = result.unwrap();
        assert!(params.has_select());
        assert!(params.has_filters());
    }

    #[test]
    fn test_parse_query_string_with_order() {
        let result = parse_query_string("select=id&order=id.desc");
        assert!(result.is_ok());
        let params = result.unwrap();
        assert!(params.has_select());
        assert!(!params.order.is_empty());
    }

    #[test]
    fn test_parse_query_string_with_limit() {
        let result = parse_query_string("select=id&limit=10");
        assert!(result.is_ok());
        let params = result.unwrap();
        assert_eq!(params.limit, Some(10));
    }

    #[test]
    fn test_to_sql_simple() {
        let params = ParsedParams::new()
            .with_select(vec![SelectItem::field("id"), SelectItem::field("name")]);

        let result = to_sql("users", &params);
        assert!(result.is_ok());
        let query = result.unwrap();
        assert!(query.query.contains("SELECT"));
        assert!(query.query.contains("users"));
    }

    #[test]
    fn test_query_string_to_sql() {
        let result = query_string_to_sql("users", "select=id,name");
        assert!(result.is_ok());
        let query = result.unwrap();
        assert!(query.query.contains("SELECT"));
        assert!(query.query.contains("users"));
        assert_eq!(query.tables, vec!["users"]);
    }

    #[test]
    fn test_build_filter_clause() {
        let filter = LogicCondition::Filter(Filter::new(
            Field::new("id"),
            FilterOperator::Eq,
            FilterValue::Single("1".to_string()),
        ));

        let result = build_filter_clause(&[filter]);
        assert!(result.is_ok());
        let clause = result.unwrap();
        assert!(clause.clause.contains("\"id\""));
        assert!(clause.clause.contains("="));
    }

    #[test]
    fn test_complex_query_with_multiple_filters() {
        let query_str = "select=id,name,email&age=gte.18&status=in.(active,pending)&order=created_at.desc&limit=10";
        let result = parse_query_string(query_str);
        assert!(result.is_ok());
        let params = result.unwrap();

        assert!(params.has_select());
        assert!(params.has_filters());
        assert_eq!(params.filters.len(), 2);
        assert_eq!(params.order.len(), 1);
        assert_eq!(params.limit, Some(10));
    }

    #[test]
    fn test_query_with_logic_operators() {
        let query_str = "and=(age.gte.18,status.eq.active)";
        let result = parse_query_string(query_str);
        assert!(result.is_ok());
        let params = result.unwrap();
        assert!(params.has_filters());
    }

    #[test]
    fn test_query_with_json_path() {
        let query_str = "data->name=eq.John&data->age=gt.25";
        let result = parse_query_string(query_str);
        assert!(result.is_ok());
        let params = result.unwrap();
        assert_eq!(params.filters.len(), 2);
    }

    #[test]
    fn test_query_with_type_cast() {
        let query_str = "price::numeric=gt.100";
        let result = parse_query_string(query_str);
        assert!(result.is_ok());
        let params = result.unwrap();
        assert_eq!(params.filters.len(), 1);
    }

    #[test]
    fn test_query_to_sql_with_comparison_operators() {
        let query_str = "age=gte.18&price=lte.100";
        let result = query_string_to_sql("users", query_str);
        assert!(result.is_ok());
        let query = result.unwrap();
        assert!(query.query.contains(">="));
        assert!(query.query.contains("<="));
        assert_eq!(query.params.len(), 2);
    }

    #[test]
    fn test_query_to_sql_with_fts() {
        let query_str = "content=fts(english).search term";
        let result = query_string_to_sql("articles", query_str);
        assert!(result.is_ok());
        let query = result.unwrap();
        assert!(query.query.contains("to_tsvector"));
        assert!(query.query.contains("plainto_tsquery"));
        assert!(query.query.contains("english"));
    }

    #[test]
    fn test_query_to_sql_with_array_operators() {
        let query_str = "tags=cs.{rust}";
        let result = query_string_to_sql("posts", query_str);
        assert!(result.is_ok());
        let query = result.unwrap();
        assert!(query.query.contains("@>"));
    }

    #[test]
    fn test_query_to_sql_with_negation() {
        let query_str = "status=not.eq.deleted";
        let result = query_string_to_sql("users", query_str);
        assert!(result.is_ok());
        let query = result.unwrap();
        assert!(query.query.contains("<>"));
    }

    #[test]
    fn test_complex_nested_query() {
        let query_str = "select=id,name,orders(id,total)&status=eq.active&age=gte.18&order=created_at.desc&limit=10&offset=20";
        let result = parse_query_string(query_str);
        assert!(result.is_ok());
        let params = result.unwrap();

        assert!(params.has_select());
        assert_eq!(params.filters.len(), 2);
        assert_eq!(params.order.len(), 1);
        assert_eq!(params.limit, Some(10));
        assert_eq!(params.offset, Some(20));
    }

    #[test]
    fn test_query_with_quantifiers() {
        let query_str = "tags=eq(any).{rust,elixir,go}";
        let result = query_string_to_sql("posts", query_str);
        assert!(result.is_ok());
        let query = result.unwrap();
        assert!(query.query.contains("= ANY"));
    }
}
