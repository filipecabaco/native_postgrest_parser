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

use serde::{Deserialize, Serialize};
use wasm_bindgen::prelude::*;

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
pub fn parse_query_string_wasm(
    table: &str,
    query_string: &str,
) -> Result<WasmQueryResult, JsValue> {
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

/// Parse and generate SQL for an INSERT operation.
///
/// # Arguments
///
/// * `table` - The table name
/// * `body` - JSON body (single object or array of objects)
/// * `query_string` - Optional query string for returning, on_conflict, etc.
/// * `headers` - Optional headers as JSON string (e.g., '{"Prefer":"return=representation"}')
///
/// # Example (TypeScript)
///
/// ```typescript
/// const result = parseInsert("users",
///   JSON.stringify({ name: "Alice", email: "alice@example.com" }),
///   "on_conflict=email&returning=id,name",
///   JSON.stringify({ Prefer: "return=representation" })
/// );
/// console.log(result.query);   // INSERT INTO "users" ...
/// console.log(result.params);  // ["Alice", "alice@example.com"]
/// ```
#[wasm_bindgen(js_name = parseInsert)]
pub fn parse_insert_wasm(
    table: &str,
    body: &str,
    query_string: Option<String>,
    headers: Option<String>,
) -> Result<WasmQueryResult, JsValue> {
    parse_request_wasm(
        "POST",
        table,
        &query_string.unwrap_or_default(),
        Some(body.to_string()),
        headers,
    )
}

/// Parse and generate SQL for an UPDATE operation.
///
/// # Arguments
///
/// * `table` - The table name
/// * `body` - JSON object with fields to update
/// * `query_string` - Query string with filters and optional returning
/// * `headers` - Optional headers as JSON string
///
/// # Example (TypeScript)
///
/// ```typescript
/// const result = parseUpdate("users",
///   JSON.stringify({ status: "active" }),
///   "id=eq.123&returning=id,status",
///   null
/// );
/// console.log(result.query);   // UPDATE "users" SET ...
/// console.log(result.params);  // ["active", "123"]
/// ```
#[wasm_bindgen(js_name = parseUpdate)]
pub fn parse_update_wasm(
    table: &str,
    body: &str,
    query_string: &str,
    headers: Option<String>,
) -> Result<WasmQueryResult, JsValue> {
    parse_request_wasm(
        "PATCH",
        table,
        query_string,
        Some(body.to_string()),
        headers,
    )
}

/// Parse and generate SQL for a DELETE operation.
///
/// # Arguments
///
/// * `table` - The table name
/// * `query_string` - Query string with filters and optional returning
/// * `headers` - Optional headers as JSON string
///
/// # Example (TypeScript)
///
/// ```typescript
/// const result = parseDelete("users", "id=eq.123&returning=id", null);
/// console.log(result.query);   // DELETE FROM "users" WHERE ...
/// console.log(result.params);  // ["123"]
/// ```
#[wasm_bindgen(js_name = parseDelete)]
pub fn parse_delete_wasm(
    table: &str,
    query_string: &str,
    headers: Option<String>,
) -> Result<WasmQueryResult, JsValue> {
    parse_request_wasm("DELETE", table, query_string, None, headers)
}

/// Parse and generate SQL for an RPC (stored procedure/function) call.
///
/// # Arguments
///
/// * `function_name` - The function name (can include schema: "schema.function")
/// * `body` - JSON object with function arguments (or null for no args)
/// * `query_string` - Optional query string for filtering/ordering results
/// * `headers` - Optional headers as JSON string
///
/// # Example (TypeScript)
///
/// ```typescript
/// const result = parseRpc("calculate_total",
///   JSON.stringify({ order_id: 123, tax_rate: 0.08 }),
///   "select=total,tax&limit=1",
///   null
/// );
/// console.log(result.query);   // SELECT * FROM calculate_total(...)
/// console.log(result.params);  // [123, 0.08]
/// ```
#[wasm_bindgen(js_name = parseRpc)]
pub fn parse_rpc_wasm(
    function_name: &str,
    body: Option<String>,
    query_string: Option<String>,
    headers: Option<String>,
) -> Result<WasmQueryResult, JsValue> {
    let path = format!("rpc/{}", function_name);
    parse_request_wasm(
        "POST",
        &path,
        &query_string.unwrap_or_default(),
        body,
        headers,
    )
}

/// Parse a complete HTTP request and generate appropriate SQL.
///
/// This is the most comprehensive function - it handles all HTTP methods
/// and automatically chooses between SELECT, INSERT, UPDATE, DELETE, or RPC.
///
/// # Arguments
///
/// * `method` - HTTP method: "GET", "POST", "PUT", "PATCH", "DELETE"
/// * `path` - Resource path (table name or "rpc/function_name")
/// * `query_string` - URL query string
/// * `body` - Request body as JSON string (or null)
/// * `headers` - Optional headers as JSON object (for Prefer header)
///
/// # Example (TypeScript)
///
/// ```typescript
/// // SELECT query
/// const getResult = parseRequest("GET", "users", "age=gte.18&limit=10", null, null);
///
/// // INSERT with upsert
/// const postResult = parseRequest("POST", "users", "on_conflict=email",
///   JSON.stringify({ name: "Alice", email: "alice@example.com" }),
///   JSON.stringify({ Prefer: "return=representation" })
/// );
///
/// // RPC call
/// const rpcResult = parseRequest("POST", "rpc/my_function",
///   "select=result",
///   JSON.stringify({ arg1: "value" }),
///   null
/// );
/// ```
#[wasm_bindgen(js_name = parseRequest)]
pub fn parse_request_wasm(
    method: &str,
    path: &str,
    query_string: &str,
    body: Option<String>,
    headers: Option<String>,
) -> Result<WasmQueryResult, JsValue> {
    // Parse headers if provided
    let headers_map: Option<std::collections::HashMap<String, String>> = if let Some(h) = headers {
        serde_json::from_str(&h).ok()
    } else {
        None
    };

    let operation = crate::parse(
        method,
        path,
        query_string,
        body.as_deref(),
        headers_map.as_ref(),
    )
    .map_err(|e| JsValue::from_str(&format!("Parse error: {}", e)))?;

    let result = crate::operation_to_sql(path, &operation)
        .map_err(|e| JsValue::from_str(&format!("SQL generation error: {}", e)))?;

    Ok(WasmQueryResult {
        query: result.query,
        params: result.params,
        tables: result.tables,
    })
}

/// Initialize schema cache from a database query executor.
///
/// This function accepts a JavaScript async function that executes SQL queries
/// and returns results. The schema introspection queries will be executed via
/// this callback to populate the relationship cache.
///
/// # Arguments
///
/// * `query_executor` - An async JavaScript function with signature:
///   `async (sql: string) => { rows: any[] }`
///
/// # Example (TypeScript with PGlite)
///
/// ```typescript
/// import { PGlite } from '@electric-sql/pglite';
/// import { initSchemaFromDb } from './pkg/postgrest_parser.js';
///
/// const db = new PGlite();
///
/// // Create query executor for WASM
/// const queryExecutor = async (sql: string) => {
///   const result = await db.query(sql);
///   return { rows: result.rows };
/// };
///
/// // Initialize schema from database
/// await initSchemaFromDb(queryExecutor);
/// ```
#[wasm_bindgen(js_name = initSchemaFromDb)]
pub async fn init_schema_from_db(query_executor: js_sys::Function) -> Result<(), JsValue> {
    // Query for foreign keys from pg_catalog
    let fk_query = r#"
        SELECT
            con.conname AS constraint_name,
            sn.nspname AS from_schema,
            sc.relname AS from_table,
            sa.attname AS from_column,
            tn.nspname AS to_schema,
            tc.relname AS to_table,
            ta.attname AS to_column
        FROM pg_constraint con
        JOIN pg_class sc ON sc.oid = con.conrelid
        JOIN pg_namespace sn ON sn.oid = sc.relnamespace
        JOIN pg_class tc ON tc.oid = con.confrelid
        JOIN pg_namespace tn ON tn.oid = tc.relnamespace
        JOIN pg_attribute sa ON sa.attrelid = sc.oid AND sa.attnum = con.conkey[1]
        JOIN pg_attribute ta ON ta.attrelid = tc.oid AND ta.attnum = con.confkey[1]
        WHERE con.contype = 'f'
          AND sn.nspname NOT IN ('pg_catalog', 'information_schema')
          AND array_length(con.conkey, 1) = 1
        ORDER BY sn.nspname, sc.relname, con.conname
    "#;

    // Call the JavaScript query executor
    let this = JsValue::null();
    let sql_arg = JsValue::from_str(fk_query);
    let promise = query_executor
        .call1(&this, &sql_arg)
        .map_err(|e| JsValue::from_str(&format!("Query executor call failed: {:?}", e)))?;

    // Await the promise
    let js_future = wasm_bindgen_futures::JsFuture::from(js_sys::Promise::from(promise));
    let result = js_future
        .await
        .map_err(|e| JsValue::from_str(&format!("Query execution failed: {:?}", e)))?;

    // Parse the result - expect { rows: [...] }
    let result_obj = js_sys::Object::from(result);
    let rows_value = js_sys::Reflect::get(&result_obj, &JsValue::from_str("rows"))
        .map_err(|e| JsValue::from_str(&format!("Result missing 'rows' property: {:?}", e)))?;

    // Deserialize rows to ForeignKey structs
    let foreign_keys: Vec<crate::schema_cache::ForeignKey> =
        serde_wasm_bindgen::from_value(rows_value)
            .map_err(|e| JsValue::from_str(&format!("Failed to parse foreign keys: {}", e)))?;

    // TODO: Build SchemaCache and store globally for use in SQL generation
    // For now, just log success
    web_sys::console::log_1(&JsValue::from_str(&format!(
        "Schema loaded: {} foreign keys",
        foreign_keys.len()
    )));

    Ok(())
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

    #[wasm_bindgen_test]
    fn test_parse_insert_wasm() {
        let body = r#"{"name":"Alice","email":"alice@example.com"}"#;
        let result =
            parse_insert_wasm("users", body, Some("returning=id".to_string()), None).unwrap();
        assert!(result.query.contains("INSERT"));
        assert!(result.query.contains("users"));
    }

    #[wasm_bindgen_test]
    fn test_parse_update_wasm() {
        let body = r#"{"status":"active"}"#;
        let result = parse_update_wasm("users", body, "id=eq.123", None).unwrap();
        assert!(result.query.contains("UPDATE"));
        assert!(result.query.contains("users"));
    }

    #[wasm_bindgen_test]
    fn test_parse_delete_wasm() {
        let result = parse_delete_wasm("users", "id=eq.123", None).unwrap();
        assert!(result.query.contains("DELETE"));
        assert!(result.query.contains("users"));
    }

    #[wasm_bindgen_test]
    fn test_parse_rpc_wasm() {
        let body = r#"{"arg1":"value1"}"#;
        let result = parse_rpc_wasm("my_function", Some(body.to_string()), None, None).unwrap();
        assert!(result.query.contains("my_function"));
    }

    #[wasm_bindgen_test]
    fn test_parse_request_get() {
        let result = parse_request_wasm("GET", "users", "age=gte.18", None, None).unwrap();
        assert!(result.query.contains("SELECT"));
    }

    #[wasm_bindgen_test]
    fn test_parse_request_post() {
        let body = r#"{"name":"Alice"}"#;
        let result = parse_request_wasm("POST", "users", "", Some(body.to_string()), None).unwrap();
        assert!(result.query.contains("INSERT"));
    }
}
