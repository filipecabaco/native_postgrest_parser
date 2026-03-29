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
 * * `schema_id` - A unique key to store this schema under (e.g., tenant ID).
 *   If empty, uses "default".
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
 * // Initialize schema for a specific tenant
 * await initSchemaFromDb("tenant-123", queryExecutor);
 * ```
 */
export function initSchemaFromDb(schema_id: string, query_executor: Function): Promise<void>;

/**
 * Clear a schema cache entry, freeing its memory.
 *
 * # Arguments
 *
 * * `schema_id` - The schema key to remove. If empty, removes "default".
 */
export function clearSchema(schema_id: string): void;

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
 * * `schema_id` - Optional schema cache key for per-tenant schema resolution
 */
export function parseDelete(table: string, query_string: string, headers?: string | null, schema_id?: string | null): WasmQueryResult;

/**
 * Parse and generate SQL for an INSERT operation.
 *
 * # Arguments
 *
 * * `table` - The table name
 * * `body` - JSON body (single object or array of objects)
 * * `query_string` - Optional query string for returning, on_conflict, etc.
 * * `headers` - Optional headers as JSON string
 * * `schema_id` - Optional schema cache key for per-tenant schema resolution
 */
export function parseInsert(table: string, body: string, query_string?: string | null, headers?: string | null, schema_id?: string | null): WasmQueryResult;

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
 * * `schema_id` - Optional schema cache key for per-tenant schema resolution
 *
 * # Returns
 *
 * Returns a `WasmQueryResult` containing the SQL query, parameters, and affected tables.
 */
export function parseQueryString(table: string, query_string: string, schema_id?: string | null): WasmQueryResult;

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
 * * `schema_id` - Optional schema cache key for per-tenant schema resolution
 */
export function parseRequest(method: string, path: string, query_string: string, body?: string | null, headers?: string | null, schema_id?: string | null): WasmQueryResult;

/**
 * Parse and generate SQL for an RPC (stored procedure/function) call.
 *
 * # Arguments
 *
 * * `function_name` - The function name (can include schema: "schema.function")
 * * `body` - JSON object with function arguments (or null for no args)
 * * `query_string` - Optional query string for filtering/ordering results
 * * `headers` - Optional headers as JSON string
 * * `schema_id` - Optional schema cache key for per-tenant schema resolution
 */
export function parseRpc(function_name: string, body?: string | null, query_string?: string | null, headers?: string | null, schema_id?: string | null): WasmQueryResult;

/**
 * Parse and generate SQL for an UPDATE operation.
 *
 * # Arguments
 *
 * * `table` - The table name
 * * `body` - JSON object with fields to update
 * * `query_string` - Query string with filters and optional returning
 * * `headers` - Optional headers as JSON string
 * * `schema_id` - Optional schema cache key for per-tenant schema resolution
 */
export function parseUpdate(table: string, body: string, query_string: string, headers?: string | null, schema_id?: string | null): WasmQueryResult;

export type InitInput = RequestInfo | URL | Response | BufferSource | WebAssembly.Module;

export interface InitOutput {
    readonly memory: WebAssembly.Memory;
    readonly __wbg_wasmqueryresult_free: (a: number, b: number) => void;
    readonly buildFilterClause: (a: number, b: number) => void;
    readonly initSchemaFromDb: (a: number, b: number, c: number) => number;
    readonly clearSchema: (a: number, b: number) => void;
    readonly parseDelete: (a: number, b: number, c: number, d: number, e: number, f: number, g: number, h: number, i: number) => void;
    readonly parseInsert: (a: number, b: number, c: number, d: number, e: number, f: number, g: number, h: number, i: number, j: number, k: number) => void;
    readonly parseOnly: (a: number, b: number, c: number) => void;
    readonly parseQueryString: (a: number, b: number, c: number, d: number, e: number, f: number, g: number) => void;
    readonly parseRequest: (a: number, b: number, c: number, d: number, e: number, f: number, g: number, h: number, i: number, j: number, k: number, l: number, m: number) => void;
    readonly parseRpc: (a: number, b: number, c: number, d: number, e: number, f: number, g: number, h: number, i: number, j: number, k: number) => void;
    readonly parseUpdate: (a: number, b: number, c: number, d: number, e: number, f: number, g: number, h: number, i: number, j: number, k: number) => void;
    readonly wasmqueryresult_params: (a: number) => number;
    readonly wasmqueryresult_query: (a: number, b: number) => void;
    readonly wasmqueryresult_tables: (a: number) => number;
    readonly wasmqueryresult_toJSON: (a: number) => number;
    readonly init_panic_hook: () => void;
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
