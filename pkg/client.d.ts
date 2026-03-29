/**
 * Type-safe TypeScript wrapper for PostgREST Parser
 *
 * This module provides a type-safe, idiomatic TypeScript API on top of
 * the auto-generated WASM bindings, improving developer experience.
 */
import type { HttpMethod, QueryResult, RequestHeaders, SelectOptions, InsertOptions, UpdateOptions, DeleteOptions, RpcOptions } from "./types.js";
export { default as init, initSchemaFromDb, clearSchema, clearAllSchemas } from "./postgrest_parser.js";
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
 * // Execute with your database client
 * const rows = await db.query(getUsers.query, getUsers.params);
 * ```
 */
export declare class PostgRESTParser {
    /**
     * Optional schema ID for per-tenant schema resolution.
     */
    readonly schemaId?: string;

    /**
     * Create a new PostgRESTParser instance.
     *
     * @param schemaId - Optional schema cache key for per-tenant schema resolution.
     *   Must match a key previously passed to `initSchemaFromDb()`.
     */
    constructor(schemaId?: string);

    /**
     * Parse a complete HTTP request and generate appropriate SQL
     */
    parseRequest(method: HttpMethod, path: string, queryString: string, body?: Record<string, unknown> | Record<string, unknown>[] | null, headers?: RequestHeaders | null): QueryResult;

    /**
     * Generate a SELECT query
     */
    select(table: string, options?: SelectOptions): QueryResult;

    /**
     * Generate an INSERT query
     */
    insert(table: string, data: Record<string, unknown> | Record<string, unknown>[], options?: InsertOptions): QueryResult;

    /**
     * Generate an UPSERT query (INSERT with ON CONFLICT)
     */
    upsert(table: string, data: Record<string, unknown>, conflictColumns: string[], options?: InsertOptions): QueryResult;

    /**
     * Generate an UPDATE query
     */
    update(table: string, data: Record<string, unknown>, filters: Record<string, string>, options?: UpdateOptions): QueryResult;

    /**
     * Generate a DELETE query
     */
    delete(table: string, filters: Record<string, string>, options?: DeleteOptions): QueryResult;

    /**
     * Generate an RPC (stored procedure/function) call
     */
    rpc(functionName: string, args?: Record<string, unknown>, options?: RpcOptions): QueryResult;

    /**
     * Parse a query string without generating SQL
     */
    parseOnly(queryString: string): unknown;

    /**
     * Build a WHERE clause from filter conditions
     */
    buildFilterClause(filters: unknown): unknown;
}
/**
 * Create a new PostgREST Parser client instance
 *
 * @param schemaId - Optional schema cache key for per-tenant schema resolution.
 * @returns New PostgRESTParser instance
 */
export declare function createClient(schemaId?: string): PostgRESTParser;
export type { HttpMethod, QueryResult, RequestHeaders, SelectOptions, InsertOptions, UpdateOptions, DeleteOptions, RpcOptions, PreferOptions, PostgRESTParserError, } from "./types.js";
