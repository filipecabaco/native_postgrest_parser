/**
 * Type-safe TypeScript wrapper for PostgREST Parser
 *
 * This module provides a type-safe, idiomatic TypeScript API on top of
 * the auto-generated WASM bindings, improving developer experience.
 */
import type { HttpMethod, QueryResult, RequestHeaders, SelectOptions, InsertOptions, UpdateOptions, DeleteOptions, RpcOptions } from "./types.js";
export { default as init, initSchemaFromDb } from "./postgrest_parser.js";
/**
 * Type-safe PostgREST Parser client
 *
 * Provides strongly-typed methods for generating PostgREST-compatible SQL queries.
 *
 * @example
 * ```typescript
 * const client = new PostgRESTParser();
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
export declare class PostgRESTParser {
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
     *
     * @example
     * ```typescript
     * const result = client.parseRequest("GET", "users", "age=gte.18", null, null);
     * const rows = await db.query(result.query, result.params);
     * ```
     */
    parseRequest(method: HttpMethod, path: string, queryString: string, body?: Record<string, unknown> | Record<string, unknown>[] | null, headers?: RequestHeaders | null): QueryResult;
    /**
     * Generate a SELECT query
     *
     * @param table - Table name to query
     * @param options - Query options (filters, ordering, pagination)
     * @returns Query result with SQL, params, and tables
     *
     * @example
     * ```typescript
     * const result = client.select("users", {
     *   filters: { "age": "gte.18", "status": "eq.active" },
     *   order: ["created_at.desc"],
     *   limit: 10,
     *   offset: 0
     * });
     * ```
     */
    select(table: string, options?: SelectOptions): QueryResult;
    /**
     * Generate an INSERT query
     *
     * @param table - Table name
     * @param data - Data to insert (single object or array of objects)
     * @param options - Insert options (returning, onConflict, prefer)
     * @returns Query result with SQL, params, and tables
     *
     * @example
     * ```typescript
     * const result = client.insert("users", {
     *   name: "Alice",
     *   email: "alice@example.com"
     * }, {
     *   returning: "*",
     *   prefer: { return: "representation" }
     * });
     * ```
     */
    insert(table: string, data: Record<string, unknown> | Record<string, unknown>[], options?: InsertOptions): QueryResult;
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
     *
     * @example
     * ```typescript
     * const result = client.upsert("users", {
     *   email: "alice@example.com",
     *   name: "Alice Updated"
     * }, ["email"], {
     *   returning: "*"
     * });
     * ```
     */
    upsert(table: string, data: Record<string, unknown>, conflictColumns: string[], options?: InsertOptions): QueryResult;
    /**
     * Generate an UPDATE query
     *
     * @param table - Table name
     * @param data - Data to update
     * @param filters - Filter conditions to match rows
     * @param options - Update options (returning, prefer)
     * @returns Query result with SQL, params, and tables
     *
     * @example
     * ```typescript
     * const result = client.update("users", {
     *   status: "active"
     * }, {
     *   "id": "eq.123"
     * }, {
     *   returning: "id,status"
     * });
     * ```
     */
    update(table: string, data: Record<string, unknown>, filters: Record<string, string>, options?: UpdateOptions): QueryResult;
    /**
     * Generate a DELETE query
     *
     * @param table - Table name
     * @param filters - Filter conditions to match rows to delete
     * @param options - Delete options (returning, prefer)
     * @returns Query result with SQL, params, and tables
     *
     * @example
     * ```typescript
     * const result = client.delete("users", {
     *   "status": "eq.inactive",
     *   "last_login": "lt.2023-01-01"
     * }, {
     *   returning: "id"
     * });
     * ```
     */
    delete(table: string, filters: Record<string, string>, options?: DeleteOptions): QueryResult;
    /**
     * Generate an RPC (stored procedure/function) call
     *
     * @param functionName - Function name (can include schema)
     * @param args - Function arguments as object
     * @param options - RPC options (select, filters, ordering)
     * @returns Query result with SQL, params, and tables
     *
     * @example
     * ```typescript
     * const result = client.rpc("calculate_total", {
     *   order_id: 123,
     *   tax_rate: 0.08
     * }, {
     *   select: ["total", "tax"],
     *   limit: 1
     * });
     * ```
     */
    rpc(functionName: string, args?: Record<string, unknown>, options?: RpcOptions): QueryResult;
    /**
     * Parse a query string without generating SQL
     *
     * Useful for inspecting the parsed structure before generating SQL.
     *
     * @param queryString - PostgREST query string
     * @returns Parsed query parameters as object
     *
     * @example
     * ```typescript
     * const parsed = client.parseOnly("age=gte.18&status=eq.active&order=created_at.desc");
     * console.log(parsed); // { filters: [...], order: [...] }
     * ```
     */
    parseOnly(queryString: string): unknown;
    /**
     * Build a WHERE clause from filter conditions
     *
     * @param filters - Filter conditions as object
     * @returns Object with clause (SQL string) and params (array of values)
     *
     * @example
     * ```typescript
     * const filters = [
     *   { column: "age", operator: "gte", value: "18" },
     *   { column: "status", operator: "eq", value: "active" }
     * ];
     * const result = client.buildFilterClause(filters);
     * console.log(result.clause); // "age >= $1 AND status = $2"
     * console.log(result.params);  // ["18", "active"]
     * ```
     */
    buildFilterClause(filters: unknown): unknown;
}
/**
 * Create a new PostgREST Parser client instance
 *
 * @returns New PostgRESTParser instance
 *
 * @example
 * ```typescript
 * import { createClient } from './pkg/client.js';
 *
 * const client = createClient();
 * const result = client.select("users", { limit: 10 });
 * ```
 */
export declare function createClient(): PostgRESTParser;
export type { HttpMethod, QueryResult, RequestHeaders, SelectOptions, InsertOptions, UpdateOptions, DeleteOptions, RpcOptions, PreferOptions, PostgRESTParserError, } from "./types.js";
