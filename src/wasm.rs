//! WASM bindings for TypeScript/JavaScript usage.
//!
//! This module provides a TypeScript-friendly interface to the PostgREST parser.
//!
//! ## Usage from TypeScript
//!
//! ```typescript
//! import init, { parseQueryString, QueryResult } from './pkg/postgrest_parser.js';
//!
//! await init();
//!
//! const result = parseQueryString(
//!   "users",
//!   "select=id,name,email&age=gte.18&status=in.(active,pending)&order=created_at.desc&limit=10"
//! );
//!
//! console.log('SQL:', result.query);
//! console.log('Params:', result.params);
//! console.log('Tables:', result.tables);
//! ```

use wasm_bindgen::prelude::*;
use serde::{Serialize, Deserialize};

#[cfg(feature = "wasm")]
use console_error_panic_hook;

/// Initialize WASM module (call this first from JavaScript)
#[cfg(feature = "wasm")]
#[wasm_bindgen(start)]
pub fn init_panic_hook() {
    console_error_panic_hook::set_once();
}

/// Result of parsing a PostgREST query, designed for TypeScript consumption.
#[wasm_bindgen]
#[derive(Serialize, Deserialize)]
pub struct WasmQueryResult {
    /// The generated PostgreSQL SELECT query
    query: String,
    /// Query parameters as JSON values ($1, $2, etc.)
    params: Vec<serde_json::Value>,
    /// List of tables referenced in the query
    tables: Vec<String>,
}

#[wasm_bindgen]
impl WasmQueryResult {
    /// Get the SQL query string
    #[wasm_bindgen(getter)]
    pub fn query(&self) -> String {
        self.query.clone()
    }

    /// Get the query parameters as a JSON string
    #[wasm_bindgen(getter)]
    pub fn params(&self) -> JsValue {
        serde_wasm_bindgen::to_value(&self.params).unwrap_or(JsValue::NULL)
    }

    /// Get the list of tables as a JSON array
    #[wasm_bindgen(getter)]
    pub fn tables(&self) -> JsValue {
        serde_wasm_bindgen::to_value(&self.tables).unwrap_or(JsValue::NULL)
    }

    /// Get the entire result as a JSON object
    #[wasm_bindgen(js_name = toJSON)]
    pub fn to_json(&self) -> JsValue {
        serde_wasm_bindgen::to_value(self).unwrap_or(JsValue::NULL)
    }
}

/// Parse a PostgREST query string and convert it to SQL.
///
/// # Arguments
///
/// * `table` - The table name to query
/// * `query_string` - The PostgREST query string (e.g., "select=id,name&age=gte.18")
///
/// # Returns
///
/// Returns a `WasmQueryResult` containing the SQL query, parameters, and affected tables.
///
/// # Example (TypeScript)
///
/// ```typescript
/// const result = parseQueryString("users", "age=gte.18&status=eq.active");
/// console.log(result.query);   // SELECT * FROM "users" WHERE ...
/// console.log(result.params);  // ["18", "active"]
/// console.log(result.tables);  // ["users"]
/// ```
#[wasm_bindgen(js_name = parseQueryString)]
pub fn parse_query_string_wasm(table: &str, query_string: &str) -> Result<WasmQueryResult, JsValue> {
    let params = crate::parse_query_string(query_string)
        .map_err(|e| JsValue::from_str(&format!("Parse error: {}", e)))?;

    let result = crate::to_sql(table, &params)
        .map_err(|e| JsValue::from_str(&format!("SQL generation error: {}", e)))?;

    Ok(WasmQueryResult {
        query: result.query,
        params: result.params,
        tables: result.tables,
    })
}

/// Parse only the query string without generating SQL.
///
/// Useful if you want to inspect the parsed structure before generating SQL.
///
/// # Arguments
///
/// * `query_string` - The PostgREST query string
///
/// # Returns
///
/// Returns the parsed parameters as a JSON object.
#[wasm_bindgen(js_name = parseOnly)]
pub fn parse_only_wasm(query_string: &str) -> Result<JsValue, JsValue> {
    let params = crate::parse_query_string(query_string)
        .map_err(|e| JsValue::from_str(&format!("Parse error: {}", e)))?;

    serde_wasm_bindgen::to_value(&params)
        .map_err(|e| JsValue::from_str(&format!("Serialization error: {}", e)))
}

/// Build a WHERE clause from parsed filters.
///
/// # Arguments
///
/// * `filters_json` - JSON array of filter conditions
///
/// # Returns
///
/// Returns an object with `clause` (SQL string) and `params` (array of values).
#[wasm_bindgen(js_name = buildFilterClause)]
pub fn build_filter_clause_wasm(filters_json: JsValue) -> Result<JsValue, JsValue> {
    let filters: Vec<crate::LogicCondition> = serde_wasm_bindgen::from_value(filters_json)
        .map_err(|e| JsValue::from_str(&format!("Invalid filters JSON: {}", e)))?;

    let result = crate::build_filter_clause(&filters)
        .map_err(|e| JsValue::from_str(&format!("Filter clause error: {}", e)))?;

    serde_wasm_bindgen::to_value(&result)
        .map_err(|e| JsValue::from_str(&format!("Serialization error: {}", e)))
}

#[cfg(test)]
mod tests {
    use super::*;
    use wasm_bindgen_test::*;

    #[wasm_bindgen_test]
    fn test_parse_query_string_wasm() {
        let result = parse_query_string_wasm("users", "age=gte.18&status=eq.active").unwrap();
        assert!(result.query.contains("SELECT"));
        assert!(result.query.contains("users"));
    }

    #[wasm_bindgen_test]
    fn test_parse_only_wasm() {
        let result = parse_only_wasm("age=gte.18").unwrap();
        assert!(!result.is_null());
    }
}
