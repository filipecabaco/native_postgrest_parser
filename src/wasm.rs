//! WASM bindings for TypeScript/JavaScript usage.
//!
//! This module provides a TypeScript-friendly interface to the PostgREST parser.
//!
//! ## Trust Model
//!
//! - **Schema IDs** are opaque keys into an in-process cache. Callers are
//!   responsible for ensuring that schema IDs are not attacker-controlled.
//!   If a caller passes the wrong schema ID, the parser will silently use
//!   that schema's foreign key relationships — there is no authorization check.
//!
//! - **`query_executor`** callbacks passed to [`init_schema_from_db`] are fully
//!   trusted. The callback receives a hardcoded SQL introspection query and the
//!   result is used to populate the FK cache. A malicious callback could inject
//!   fabricated FK data, causing the parser to generate JOINs to arbitrary tables.
//!   Only pass query executors that run against the correct tenant database.
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
use std::cell::RefCell;
use std::collections::HashMap;
use std::sync::Arc;
use wasm_bindgen::prelude::*;

use crate::schema_cache::{ForeignKey, SchemaCache};

#[cfg(feature = "wasm")]
use console_error_panic_hook;

// Per-thread schema cache store, keyed by schema_id.
// Each tenant/schema gets its own SchemaCache entry.
//
// Uses `thread_local! + RefCell` instead of `Mutex` because WASM is
// single-threaded — avoids unnecessary atomic synchronization overhead
// on every parse call.
thread_local! {
    static SCHEMA_STORE: RefCell<HashMap<String, Arc<SchemaCache>>> =
        RefCell::new(HashMap::new());
}

/// Default schema key used when no schema_id is provided
const DEFAULT_SCHEMA_KEY: &str = "default";

/// Look up a schema cache by schema_id, returning None if not found.
fn get_schema_cache(schema_id: Option<&str>) -> Option<Arc<SchemaCache>> {
    let key = schema_id.unwrap_or(DEFAULT_SCHEMA_KEY);
    SCHEMA_STORE.with(|store| store.borrow().get(key).cloned())
}

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
/// * `schema_id` - Optional schema cache key for per-tenant schema resolution
///
/// # Returns
///
/// Returns a `WasmQueryResult` containing the SQL query, parameters, and affected tables.
#[wasm_bindgen(js_name = parseQueryString)]
pub fn parse_query_string_wasm(
    table: &str,
    query_string: &str,
    schema_id: Option<String>,
) -> Result<WasmQueryResult, JsValue> {
    // Delegate to parseRequest for consistent schema cache handling
    parse_request_wasm("GET", table, query_string, None, None, schema_id)
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
/// * `schema_id` - Optional schema cache key for per-tenant schema resolution
#[wasm_bindgen(js_name = parseInsert)]
pub fn parse_insert_wasm(
    table: &str,
    body: &str,
    query_string: Option<String>,
    headers: Option<String>,
    schema_id: Option<String>,
) -> Result<WasmQueryResult, JsValue> {
    parse_request_wasm(
        "POST",
        table,
        &query_string.unwrap_or_default(),
        Some(body.to_string()),
        headers,
        schema_id,
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
/// * `schema_id` - Optional schema cache key for per-tenant schema resolution
#[wasm_bindgen(js_name = parseUpdate)]
pub fn parse_update_wasm(
    table: &str,
    body: &str,
    query_string: &str,
    headers: Option<String>,
    schema_id: Option<String>,
) -> Result<WasmQueryResult, JsValue> {
    parse_request_wasm(
        "PATCH",
        table,
        query_string,
        Some(body.to_string()),
        headers,
        schema_id,
    )
}

/// Parse and generate SQL for a DELETE operation.
///
/// # Arguments
///
/// * `table` - The table name
/// * `query_string` - Query string with filters and optional returning
/// * `headers` - Optional headers as JSON string
/// * `schema_id` - Optional schema cache key for per-tenant schema resolution
#[wasm_bindgen(js_name = parseDelete)]
pub fn parse_delete_wasm(
    table: &str,
    query_string: &str,
    headers: Option<String>,
    schema_id: Option<String>,
) -> Result<WasmQueryResult, JsValue> {
    parse_request_wasm("DELETE", table, query_string, None, headers, schema_id)
}

/// Parse and generate SQL for an RPC (stored procedure/function) call.
///
/// # Arguments
///
/// * `function_name` - The function name (can include schema: "schema.function")
/// * `body` - JSON object with function arguments (or null for no args)
/// * `query_string` - Optional query string for filtering/ordering results
/// * `headers` - Optional headers as JSON string
/// * `schema_id` - Optional schema cache key for per-tenant schema resolution
#[wasm_bindgen(js_name = parseRpc)]
pub fn parse_rpc_wasm(
    function_name: &str,
    body: Option<String>,
    query_string: Option<String>,
    headers: Option<String>,
    schema_id: Option<String>,
) -> Result<WasmQueryResult, JsValue> {
    let path = format!("rpc/{}", function_name);
    parse_request_wasm(
        "POST",
        &path,
        &query_string.unwrap_or_default(),
        body,
        headers,
        schema_id,
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
/// * `schema_id` - Optional schema cache key for per-tenant schema resolution.
///   Must match a key previously passed to [`init_schema_from_db`]. If not
///   provided, falls back to the "default" cache entry (if one exists).
#[wasm_bindgen(js_name = parseRequest)]
pub fn parse_request_wasm(
    method: &str,
    path: &str,
    query_string: &str,
    body: Option<String>,
    headers: Option<String>,
    schema_id: Option<String>,
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

    let cache = get_schema_cache(schema_id.as_deref());
    let result = crate::operation_to_sql_with_cache(path, &operation, cache)
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
/// # Trust
///
/// The `query_executor` is **fully trusted**. It receives a hardcoded SQL
/// introspection query and its result is used verbatim to build the FK cache.
/// Only pass executors that connect to the correct tenant database.
///
/// # Arguments
///
/// * `schema_id` - A unique key to store this schema under (e.g., tenant ID).
///   If empty, uses "default". Callers must ensure schema IDs are not
///   attacker-controlled — passing another tenant's ID would overwrite their
///   cached schema.
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
/// // Initialize schema for a specific tenant
/// await initSchemaFromDb("tenant-123", queryExecutor);
/// ```
#[wasm_bindgen(js_name = initSchemaFromDb)]
pub async fn init_schema_from_db(
    schema_id: &str,
    query_executor: js_sys::Function,
) -> Result<(), JsValue> {
    let key = if schema_id.is_empty() {
        DEFAULT_SCHEMA_KEY.to_string()
    } else {
        schema_id.to_string()
    };

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
    let result = wasm_bindgen_futures::JsFuture::from(js_sys::Promise::from(promise))
        .await
        .map_err(|e| JsValue::from_str(&format!("Query execution failed: {:?}", e)))?;

    // Extract rows from the result
    let rows = js_sys::Reflect::get(&result, &JsValue::from_str("rows"))
        .map_err(|e| JsValue::from_str(&format!("Failed to get rows from result: {:?}", e)))?;

    let rows_array = js_sys::Array::from(&rows);
    let mut foreign_keys = Vec::new();
    let required_fields = [
        "constraint_name",
        "from_schema",
        "from_table",
        "from_column",
        "to_schema",
        "to_table",
        "to_column",
    ];

    for i in 0..rows_array.length() {
        let row = rows_array.get(i);

        // Extract and validate all required fields
        let mut fields = Vec::with_capacity(required_fields.len());
        let mut valid = true;

        for field_name in &required_fields {
            match js_sys::Reflect::get(&row, &JsValue::from_str(field_name))
                .ok()
                .and_then(|v| v.as_string())
                .filter(|s| !s.is_empty())
            {
                Some(value) => fields.push(value),
                None => {
                    valid = false;
                    break;
                }
            }
        }

        if !valid {
            continue; // Skip rows with missing or empty fields
        }

        foreign_keys.push(ForeignKey {
            constraint_name: fields[0].clone(),
            from_schema: fields[1].clone(),
            from_table: fields[2].clone(),
            from_column: fields[3].clone(),
            to_schema: fields[4].clone(),
            to_table: fields[5].clone(),
            to_column: fields[6].clone(),
        });
    }

    let cache = SchemaCache::from_foreign_keys(foreign_keys);

    // Store in thread-local map
    SCHEMA_STORE.with(|store| {
        store.borrow_mut().insert(key, Arc::new(cache));
    });

    Ok(())
}

/// Clear a schema cache entry, freeing its memory.
///
/// Call this when a tenant is paused or evicted to prevent memory leaks.
/// If the schema ID does not exist, this is a no-op.
///
/// # Arguments
///
/// * `schema_id` - The schema key to remove. If empty, removes "default".
///
/// # Example (TypeScript)
///
/// ```typescript
/// import { clearSchema } from './pkg/postgrest_parser.js';
///
/// // Clear a tenant's schema when they are paused/evicted
/// clearSchema("tenant-123");
/// ```
#[wasm_bindgen(js_name = clearSchema)]
pub fn clear_schema(schema_id: &str) {
    let key = if schema_id.is_empty() {
        DEFAULT_SCHEMA_KEY
    } else {
        schema_id
    };

    SCHEMA_STORE.with(|store| {
        store.borrow_mut().remove(key);
    });
}

/// Clear all schema cache entries, freeing all cached memory.
///
/// Useful as a safety net during shutdown or when all tenants are being evicted.
///
/// # Example (TypeScript)
///
/// ```typescript
/// import { clearAllSchemas } from './pkg/postgrest_parser.js';
///
/// // Clear everything on shutdown
/// clearAllSchemas();
/// ```
#[wasm_bindgen(js_name = clearAllSchemas)]
pub fn clear_all_schemas() {
    SCHEMA_STORE.with(|store| {
        store.borrow_mut().clear();
    });
}

#[cfg(test)]
mod tests {
    use super::*;
    use wasm_bindgen_test::*;

    #[wasm_bindgen_test]
    fn test_parse_query_string_wasm() {
        let result = parse_query_string_wasm("users", "age=gte.18&status=eq.active", None).unwrap();
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
            parse_insert_wasm("users", body, Some("returning=id".to_string()), None, None)
                .unwrap();
        assert!(result.query.contains("INSERT"));
        assert!(result.query.contains("users"));
    }

    #[wasm_bindgen_test]
    fn test_parse_update_wasm() {
        let body = r#"{"status":"active"}"#;
        let result = parse_update_wasm("users", body, "id=eq.123", None, None).unwrap();
        assert!(result.query.contains("UPDATE"));
        assert!(result.query.contains("users"));
    }

    #[wasm_bindgen_test]
    fn test_parse_delete_wasm() {
        let result = parse_delete_wasm("users", "id=eq.123", None, None).unwrap();
        assert!(result.query.contains("DELETE"));
        assert!(result.query.contains("users"));
    }

    #[wasm_bindgen_test]
    fn test_parse_rpc_wasm() {
        let body = r#"{"arg1":"value1"}"#;
        let result =
            parse_rpc_wasm("my_function", Some(body.to_string()), None, None, None).unwrap();
        assert!(result.query.contains("my_function"));
    }

    #[wasm_bindgen_test]
    fn test_parse_request_get() {
        let result = parse_request_wasm("GET", "users", "age=gte.18", None, None, None).unwrap();
        assert!(result.query.contains("SELECT"));
    }

    #[wasm_bindgen_test]
    fn test_parse_request_post() {
        let body = r#"{"name":"Alice"}"#;
        let result =
            parse_request_wasm("POST", "users", "", Some(body.to_string()), None, None).unwrap();
        assert!(result.query.contains("INSERT"));
    }

    #[wasm_bindgen_test]
    fn test_clear_schema() {
        // Store a schema
        SCHEMA_STORE.with(|store| {
            store
                .borrow_mut()
                .insert("test-tenant".to_string(), Arc::new(SchemaCache::new()));
        });

        // Verify it exists
        assert!(get_schema_cache(Some("test-tenant")).is_some());

        // Clear it
        clear_schema("test-tenant");

        // Verify it's gone
        assert!(get_schema_cache(Some("test-tenant")).is_none());
    }

    #[wasm_bindgen_test]
    fn test_clear_all_schemas() {
        // Store multiple schemas
        SCHEMA_STORE.with(|store| {
            let mut s = store.borrow_mut();
            s.insert("tenant-a".to_string(), Arc::new(SchemaCache::new()));
            s.insert("tenant-b".to_string(), Arc::new(SchemaCache::new()));
        });

        assert!(get_schema_cache(Some("tenant-a")).is_some());
        assert!(get_schema_cache(Some("tenant-b")).is_some());

        clear_all_schemas();

        assert!(get_schema_cache(Some("tenant-a")).is_none());
        assert!(get_schema_cache(Some("tenant-b")).is_none());
    }
}
