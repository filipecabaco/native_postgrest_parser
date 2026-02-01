/* tslint:disable */
/* eslint-disable */

/**
 * Result of parsing a PostgREST query, designed for TypeScript consumption.
 */
export class WasmQueryResult {
    private constructor();
    free(): void;
    [Symbol.dispose](): void;
    /**
     * Get the entire result as a JSON object
     */
    toJSON(): any;
    /**
     * Get the query parameters as a JSON string
     */
    readonly params: any;
    /**
     * Get the SQL query string
     */
    readonly query: string;
    /**
     * Get the list of tables as a JSON array
     */
    readonly tables: any;
}

/**
 * Build a WHERE clause from parsed filters.
 *
 * # Arguments
 *
 * * `filters_json` - JSON array of filter conditions
 *
 * # Returns
 *
 * Returns an object with `clause` (SQL string) and `params` (array of values).
 */
export function buildFilterClause(filters_json: any): any;

/**
 * Initialize schema cache from a database query executor.
 *
 * This function accepts a JavaScript async function that executes SQL queries
 * and returns results. The schema introspection queries will be executed via
 * this callback to populate the relationship cache.
 *
 * # Arguments
 *
 * * `query_executor` - An async JavaScript function with signature:
 *   `async (sql: string) => { rows: any[] }`
 *
 * # Example (TypeScript with PGlite)
 *
 * ```typescript
 * import { PGlite } from '@electric-sql/pglite';
 * import { initSchemaFromDb } from './pkg/postgrest_parser.js';
 *
 * const db = new PGlite();
 *
 * // Create query executor for WASM
 * const queryExecutor = async (sql: string) => {
 *   const result = await db.query(sql);
 *   return { rows: result.rows };
 * };
 *
 * // Initialize schema from database
 * await initSchemaFromDb(queryExecutor);
 * ```
 */
export function initSchemaFromDb(query_executor: Function): Promise<void>;

/**
 * Initialize WASM module (call this first from JavaScript)
 */
export function init_panic_hook(): void;

/**
 * Parse and generate SQL for a DELETE operation.
 *
 * # Arguments
 *
 * * `table` - The table name
 * * `query_string` - Query string with filters and optional returning
 * * `headers` - Optional headers as JSON string
 *
 * # Example (TypeScript)
 *
 * ```typescript
 * const result = parseDelete("users", "id=eq.123&returning=id", null);
 * console.log(result.query);   // DELETE FROM "users" WHERE ...
 * console.log(result.params);  // ["123"]
 * ```
 */
export function parseDelete(table: string, query_string: string, headers?: string | null): WasmQueryResult;

/**
 * Parse and generate SQL for an INSERT operation.
 *
 * # Arguments
 *
 * * `table` - The table name
 * * `body` - JSON body (single object or array of objects)
 * * `query_string` - Optional query string for returning, on_conflict, etc.
 * * `headers` - Optional headers as JSON string (e.g., '{"Prefer":"return=representation"}')
 *
 * # Example (TypeScript)
 *
 * ```typescript
 * const result = parseInsert("users",
 *   JSON.stringify({ name: "Alice", email: "alice@example.com" }),
 *   "on_conflict=email&returning=id,name",
 *   JSON.stringify({ Prefer: "return=representation" })
 * );
 * console.log(result.query);   // INSERT INTO "users" ...
 * console.log(result.params);  // ["Alice", "alice@example.com"]
 * ```
 */
export function parseInsert(table: string, body: string, query_string?: string | null, headers?: string | null): WasmQueryResult;

/**
 * Parse only the query string without generating SQL.
 *
 * Useful if you want to inspect the parsed structure before generating SQL.
 *
 * # Arguments
 *
 * * `query_string` - The PostgREST query string
 *
 * # Returns
 *
 * Returns the parsed parameters as a JSON object.
 */
export function parseOnly(query_string: string): any;

/**
 * Parse a PostgREST query string and convert it to SQL.
 *
 * # Arguments
 *
 * * `table` - The table name to query
 * * `query_string` - The PostgREST query string (e.g., "select=id,name&age=gte.18")
 *
 * # Returns
 *
 * Returns a `WasmQueryResult` containing the SQL query, parameters, and affected tables.
 *
 * # Example (TypeScript)
 *
 * ```typescript
 * const result = parseQueryString("users", "age=gte.18&status=eq.active");
 * console.log(result.query);   // SELECT * FROM "users" WHERE ...
 * console.log(result.params);  // ["18", "active"]
 * console.log(result.tables);  // ["users"]
 * ```
 */
export function parseQueryString(table: string, query_string: string): WasmQueryResult;

/**
 * Parse a complete HTTP request and generate appropriate SQL.
 *
 * This is the most comprehensive function - it handles all HTTP methods
 * and automatically chooses between SELECT, INSERT, UPDATE, DELETE, or RPC.
 *
 * # Arguments
 *
 * * `method` - HTTP method: "GET", "POST", "PUT", "PATCH", "DELETE"
 * * `path` - Resource path (table name or "rpc/function_name")
 * * `query_string` - URL query string
 * * `body` - Request body as JSON string (or null)
 * * `headers` - Optional headers as JSON object (for Prefer header)
 *
 * # Example (TypeScript)
 *
 * ```typescript
 * // SELECT query
 * const getResult = parseRequest("GET", "users", "age=gte.18&limit=10", null, null);
 *
 * // INSERT with upsert
 * const postResult = parseRequest("POST", "users", "on_conflict=email",
 *   JSON.stringify({ name: "Alice", email: "alice@example.com" }),
 *   JSON.stringify({ Prefer: "return=representation" })
 * );
 *
 * // RPC call
 * const rpcResult = parseRequest("POST", "rpc/my_function",
 *   "select=result",
 *   JSON.stringify({ arg1: "value" }),
 *   null
 * );
 * ```
 */
export function parseRequest(method: string, path: string, query_string: string, body?: string | null, headers?: string | null): WasmQueryResult;

/**
 * Parse and generate SQL for an RPC (stored procedure/function) call.
 *
 * # Arguments
 *
 * * `function_name` - The function name (can include schema: "schema.function")
 * * `body` - JSON object with function arguments (or null for no args)
 * * `query_string` - Optional query string for filtering/ordering results
 * * `headers` - Optional headers as JSON string
 *
 * # Example (TypeScript)
 *
 * ```typescript
 * const result = parseRpc("calculate_total",
 *   JSON.stringify({ order_id: 123, tax_rate: 0.08 }),
 *   "select=total,tax&limit=1",
 *   null
 * );
 * console.log(result.query);   // SELECT * FROM calculate_total(...)
 * console.log(result.params);  // [123, 0.08]
 * ```
 */
export function parseRpc(function_name: string, body?: string | null, query_string?: string | null, headers?: string | null): WasmQueryResult;

/**
 * Parse and generate SQL for an UPDATE operation.
 *
 * # Arguments
 *
 * * `table` - The table name
 * * `body` - JSON object with fields to update
 * * `query_string` - Query string with filters and optional returning
 * * `headers` - Optional headers as JSON string
 *
 * # Example (TypeScript)
 *
 * ```typescript
 * const result = parseUpdate("users",
 *   JSON.stringify({ status: "active" }),
 *   "id=eq.123&returning=id,status",
 *   null
 * );
 * console.log(result.query);   // UPDATE "users" SET ...
 * console.log(result.params);  // ["active", "123"]
 * ```
 */
export function parseUpdate(table: string, body: string, query_string: string, headers?: string | null): WasmQueryResult;

export type InitInput = RequestInfo | URL | Response | BufferSource | WebAssembly.Module;

export interface InitOutput {
    readonly memory: WebAssembly.Memory;
    readonly __wbg_wasmqueryresult_free: (a: number, b: number) => void;
    readonly buildFilterClause: (a: number, b: number) => void;
    readonly initSchemaFromDb: (a: number) => number;
    readonly parseDelete: (a: number, b: number, c: number, d: number, e: number, f: number, g: number) => void;
    readonly parseInsert: (a: number, b: number, c: number, d: number, e: number, f: number, g: number, h: number, i: number) => void;
    readonly parseOnly: (a: number, b: number, c: number) => void;
    readonly parseQueryString: (a: number, b: number, c: number, d: number, e: number) => void;
    readonly parseRequest: (a: number, b: number, c: number, d: number, e: number, f: number, g: number, h: number, i: number, j: number, k: number) => void;
    readonly parseRpc: (a: number, b: number, c: number, d: number, e: number, f: number, g: number, h: number, i: number) => void;
    readonly parseUpdate: (a: number, b: number, c: number, d: number, e: number, f: number, g: number, h: number, i: number) => void;
    readonly wasmqueryresult_params: (a: number) => number;
    readonly wasmqueryresult_query: (a: number, b: number) => void;
    readonly wasmqueryresult_tables: (a: number) => number;
    readonly wasmqueryresult_toJSON: (a: number) => number;
    readonly init_panic_hook: () => void;
    readonly __wasm_bindgen_func_elem_366: (a: number, b: number) => void;
    readonly __wasm_bindgen_func_elem_442: (a: number, b: number, c: number, d: number) => void;
    readonly __wasm_bindgen_func_elem_367: (a: number, b: number, c: number) => void;
    readonly __wbindgen_export: (a: number, b: number) => number;
    readonly __wbindgen_export2: (a: number, b: number, c: number, d: number) => number;
    readonly __wbindgen_export3: (a: number) => void;
    readonly __wbindgen_export4: (a: number, b: number, c: number) => void;
    readonly __wbindgen_add_to_stack_pointer: (a: number) => number;
    readonly __wbindgen_start: () => void;
}

export type SyncInitInput = BufferSource | WebAssembly.Module;

/**
 * Instantiates the given `module`, which can either be bytes or
 * a precompiled `WebAssembly.Module`.
 *
 * @param {{ module: SyncInitInput }} module - Passing `SyncInitInput` directly is deprecated.
 *
 * @returns {InitOutput}
 */
export function initSync(module: { module: SyncInitInput } | SyncInitInput): InitOutput;

/**
 * If `module_or_path` is {RequestInfo} or {URL}, makes a request and
 * for everything else, calls `WebAssembly.instantiate` directly.
 *
 * @param {{ module_or_path: InitInput | Promise<InitInput> }} module_or_path - Passing `InitInput` directly is deprecated.
 *
 * @returns {Promise<InitOutput>}
 */
export default function __wbg_init (module_or_path?: { module_or_path: InitInput | Promise<InitInput> } | InitInput | Promise<InitInput>): Promise<InitOutput>;
