/**
 * Type-safe TypeScript wrapper for PostgREST Parser
 *
 * This module provides a type-safe, idiomatic TypeScript API on top of
 * the auto-generated WASM bindings, improving developer experience.
 */
import { parseRequest as wasmParseRequest, parseInsert as wasmParseInsert, parseUpdate as wasmParseUpdate, parseDelete as wasmParseDelete, parseRpc as wasmParseRpc, parseOnly as wasmParseOnly, buildFilterClause as wasmBuildFilterClause, } from "./postgrest_parser.js";
// Re-export WASM initialization functions
export { default as init, initSchemaFromDb, clearSchema } from "./postgrest_parser.js";
/**
 * Convert WASM result to typed QueryResult
 */
function toQueryResult(wasmResult) {
    return {
        query: wasmResult.query,
        params: wasmResult.params,
        tables: wasmResult.tables,
    };
}
/**
 * Convert headers object to JSON string
 */
function headersToJson(headers) {
    return headers ? JSON.stringify(headers) : undefined;
}
/**
 * Convert PreferOptions to Prefer header value
 */
function preferToHeader(prefer) {
    if (!prefer)
        return undefined;
    const parts = [];
    if (prefer.return)
        parts.push(`return=${prefer.return}`);
    if (prefer.resolution)
        parts.push(`resolution=${prefer.resolution}`);
    if (prefer.missing)
        parts.push(`missing=${prefer.missing}`);
    if (prefer.count)
        parts.push(`count=${prefer.count}`);
    return parts.length > 0 ? parts.join(",") : undefined;
}
/**
 * Build query string from filters and options
 */
function buildQueryString(filters, options) {
    const parts = [];
    // Add filters
    if (filters) {
        for (const [key, value] of Object.entries(filters)) {
            parts.push(`${key}=${value}`);
        }
    }
    // Add select
    if (options?.select) {
        const select = Array.isArray(options.select)
            ? options.select.join(",")
            : options.select;
        parts.push(`select=${select}`);
    }
    // Add order
    if (options?.order) {
        const order = Array.isArray(options.order)
            ? options.order.join(",")
            : options.order;
        parts.push(`order=${order}`);
    }
    // Add limit
    if (options?.limit !== undefined) {
        parts.push(`limit=${options.limit}`);
    }
    // Add offset
    if (options?.offset !== undefined) {
        parts.push(`offset=${options.offset}`);
    }
    // Add on_conflict
    if (options?.onConflict) {
        const onConflict = Array.isArray(options.onConflict)
            ? options.onConflict.join(",")
            : options.onConflict;
        parts.push(`on_conflict=${onConflict}`);
    }
    // Add returning
    if (options?.returning) {
        const returning = Array.isArray(options.returning)
            ? options.returning.join(",")
            : options.returning;
        parts.push(`returning=${returning}`);
    }
    return parts.join("&");
}
/**
 * Type-safe PostgREST Parser client
 *
 * Provides strongly-typed methods for generating PostgREST-compatible SQL queries.
 * Optionally bound to a schema ID for per-tenant schema resolution.
 *
 * @example
 * ```typescript
 * // Default client (no schema binding)
 * const client = new PostgRESTParser();
 *
 * // Per-tenant client with schema binding
 * const tenantClient = new PostgRESTParser("tenant-123");
 *
 * // SELECT query
 * const getUsers = client.select("users", {
 *   filters: { "age": "gte.18", "status": "eq.active" },
 *   order: ["created_at.desc"],
 *   limit: 10
 * });
 *
 * // INSERT query
 * const createUser = client.insert("users", {
 *   name: "Alice",
 *   email: "alice@example.com"
 * }, {
 *   returning: "*",
 *   prefer: { return: "representation" }
 * });
 *
 * // Execute with your database client
 * const rows = await db.query(getUsers.query, getUsers.params);
 * ```
 */
export class PostgRESTParser {
    /**
     * Optional schema ID for per-tenant schema resolution.
     * When set, all parse calls will use this schema's cached
     * foreign key information for relation resolution.
     */
    schemaId;

    /**
     * Create a new PostgRESTParser instance.
     *
     * @param schemaId - Optional schema cache key for per-tenant schema resolution.
     *   Must match a key previously passed to `initSchemaFromDb()`.
     */
    constructor(schemaId) {
        this.schemaId = schemaId;
    }

    /**
     * Parse a complete HTTP request and generate appropriate SQL
     *
     * This is the universal routing method that handles all HTTP methods.
     *
     * @param method - HTTP method: "GET", "POST", "PUT", "PATCH", "DELETE"
     * @param path - Resource path (table name or "rpc/function_name")
     * @param queryString - URL query string
     * @param body - Request body (object or null)
     * @param headers - Request headers (object or null)
     * @returns Query result with SQL, params, and tables
     */
    parseRequest(method, path, queryString, body, headers) {
        const bodyJson = body ? JSON.stringify(body) : undefined;
        const headersJson = headers ? headersToJson(headers) : undefined;
        const result = wasmParseRequest(method, path, queryString, bodyJson, headersJson, this.schemaId);
        return toQueryResult(result);
    }
    /**
     * Generate a SELECT query
     *
     * @param table - Table name to query
     * @param options - Query options (filters, ordering, pagination)
     * @returns Query result with SQL, params, and tables
     */
    select(table, options = {}) {
        const queryString = buildQueryString(options.filters, options);
        const headers = options.count
            ? { Prefer: `count=${options.count}` }
            : undefined;
        const result = wasmParseRequest("GET", table, queryString, undefined, headersToJson(headers), this.schemaId);
        return toQueryResult(result);
    }
    /**
     * Generate an INSERT query
     *
     * @param table - Table name
     * @param data - Data to insert (single object or array of objects)
     * @param options - Insert options (returning, onConflict, prefer)
     * @returns Query result with SQL, params, and tables
     */
    insert(table, data, options = {}) {
        const queryString = buildQueryString(undefined, {
            onConflict: options.onConflict,
            returning: options.returning,
        });
        const preferHeader = preferToHeader(options.prefer);
        const headers = preferHeader
            ? { Prefer: preferHeader }
            : undefined;
        const result = wasmParseInsert(table, JSON.stringify(data), queryString || undefined, headersToJson(headers), this.schemaId);
        return toQueryResult(result);
    }
    /**
     * Generate an UPSERT query (INSERT with ON CONFLICT)
     *
     * PUT method auto-generates ON CONFLICT from filter columns.
     *
     * @param table - Table name
     * @param data - Data to upsert
     * @param conflictColumns - Columns to use for conflict detection
     * @param options - Upsert options (returning, prefer)
     * @returns Query result with SQL, params, and tables
     */
    upsert(table, data, conflictColumns, options = {}) {
        // Build filters from conflict columns for PUT auto-conflict
        const filters = {};
        for (const col of conflictColumns) {
            if (col in data) {
                filters[col] = `eq.${data[col]}`;
            }
        }
        const queryString = buildQueryString(filters, {
            returning: options.returning,
        });
        const preferHeader = preferToHeader(options.prefer);
        const headers = preferHeader
            ? { Prefer: preferHeader }
            : undefined;
        const result = wasmParseRequest("PUT", table, queryString, JSON.stringify(data), headersToJson(headers), this.schemaId);
        return toQueryResult(result);
    }
    /**
     * Generate an UPDATE query
     *
     * @param table - Table name
     * @param data - Data to update
     * @param filters - Filter conditions to match rows
     * @param options - Update options (returning, prefer)
     * @returns Query result with SQL, params, and tables
     */
    update(table, data, filters, options = {}) {
        const queryString = buildQueryString(filters, {
            returning: options.returning,
        });
        const preferHeader = preferToHeader(options.prefer);
        const headers = preferHeader
            ? { Prefer: preferHeader }
            : undefined;
        const result = wasmParseUpdate(table, JSON.stringify(data), queryString, headersToJson(headers), this.schemaId);
        return toQueryResult(result);
    }
    /**
     * Generate a DELETE query
     *
     * @param table - Table name
     * @param filters - Filter conditions to match rows to delete
     * @param options - Delete options (returning, prefer)
     * @returns Query result with SQL, params, and tables
     */
    delete(table, filters, options = {}) {
        const queryString = buildQueryString(filters, {
            returning: options.returning,
        });
        const preferHeader = preferToHeader(options.prefer);
        const headers = preferHeader
            ? { Prefer: preferHeader }
            : undefined;
        const result = wasmParseDelete(table, queryString, headersToJson(headers), this.schemaId);
        return toQueryResult(result);
    }
    /**
     * Generate an RPC (stored procedure/function) call
     *
     * @param functionName - Function name (can include schema)
     * @param args - Function arguments as object
     * @param options - RPC options (select, filters, ordering)
     * @returns Query result with SQL, params, and tables
     */
    rpc(functionName, args = {}, options = {}) {
        const queryString = buildQueryString(options.filters, options);
        const result = wasmParseRpc(functionName, JSON.stringify(args), queryString || undefined, undefined, this.schemaId);
        return toQueryResult(result);
    }
    /**
     * Parse a query string without generating SQL
     *
     * Useful for inspecting the parsed structure before generating SQL.
     *
     * @param queryString - PostgREST query string
     * @returns Parsed query parameters as object
     */
    parseOnly(queryString) {
        return wasmParseOnly(queryString);
    }
    /**
     * Build a WHERE clause from filter conditions
     *
     * @param filters - Filter conditions as object
     * @returns Object with clause (SQL string) and params (array of values)
     */
    buildFilterClause(filters) {
        return wasmBuildFilterClause(filters);
    }
}
/**
 * Create a new PostgREST Parser client instance
 *
 * @param schemaId - Optional schema cache key for per-tenant schema resolution.
 *   Must match a key previously passed to `initSchemaFromDb()`.
 * @returns New PostgRESTParser instance
 *
 * @example
 * ```typescript
 * import { createClient } from './pkg/client.js';
 *
 * // Default client
 * const client = createClient();
 *
 * // Per-tenant client
 * const tenantClient = createClient("tenant-123");
 *
 * const result = client.select("users", { limit: 10 });
 * ```
 */
export function createClient(schemaId) {
    return new PostgRESTParser(schemaId);
}
