/**
 * Type-safe TypeScript definitions for PostgREST Parser
 *
 * This file provides strongly-typed interfaces that replace the `any` types
 * in the auto-generated wasm bindings, improving TypeScript developer experience.
 */
/**
 * Supported HTTP methods for PostgREST operations
 */
export type HttpMethod = "GET" | "POST" | "PUT" | "PATCH" | "DELETE";
/**
 * SQL parameter value types
 */
export type SqlValue = string | number | boolean | null;
/**
 * Array of SQL parameter values (can include arrays for IN operator)
 */
export type SqlParam = SqlValue | string[];
/**
 * Query result containing SQL query, parameters, and affected tables
 */
export interface QueryResult {
    /** The generated SQL query string */
    query: string;
    /** Parameterized values for the query ($1, $2, etc.) */
    params: SqlParam[];
    /** List of tables referenced in the query */
    tables: string[];
}
/**
 * PostgREST filter operators
 */
export type FilterOperator = "eq" | "neq" | "gt" | "gte" | "lt" | "lte" | "like" | "ilike" | "match" | "imatch" | "in" | "is" | "fts" | "plfts" | "phfts" | "wfts" | "cs" | "cd" | "ov" | "sl" | "sr" | "nxr" | "nxl" | "adj";
/**
 * Single filter condition
 */
export interface Filter {
    column: string;
    operator: FilterOperator;
    value: string | string[];
    negate?: boolean;
}
/**
 * Logic tree for complex filters (AND/OR/NOT)
 */
export interface LogicCondition {
    operator: "and" | "or" | "not";
    conditions: (Filter | LogicCondition)[];
}
/**
 * Order direction
 */
export type OrderDirection = "asc" | "desc";
/**
 * Nulls position in ordering
 */
export type NullsPosition = "first" | "last";
/**
 * Single order clause
 */
export interface OrderBy {
    column: string;
    direction: OrderDirection;
    nulls?: NullsPosition;
}
/**
 * Parsed query parameters
 */
export interface ParsedQuery {
    select?: string[];
    filters?: (Filter | LogicCondition)[];
    order?: OrderBy[];
    limit?: number;
    offset?: number;
    on_conflict?: string[];
    returning?: string[];
}
/**
 * PostgREST Prefer header options
 */
export interface PreferOptions {
    /** Return representation: "minimal" | "representation" | "headers-only" */
    return?: "minimal" | "representation" | "headers-only";
    /** Resolution for conflicts: "merge-duplicates" | "ignore-duplicates" */
    resolution?: "merge-duplicates" | "ignore-duplicates";
    /** Missing columns behavior: "default" | "null" */
    missing?: "default" | "null";
    /** Count algorithm: "exact" | "planned" | "estimated" */
    count?: "exact" | "planned" | "estimated";
}
/**
 * Request headers (type-safe alternative to JSON string)
 */
export interface RequestHeaders {
    Prefer?: string;
    [key: string]: string | undefined;
}
/**
 * Query options for SELECT operations
 */
export interface SelectOptions {
    /** Columns to select (comma-separated or array) */
    select?: string | string[];
    /** Filter conditions */
    filters?: Record<string, string>;
    /** Order by clauses */
    order?: string | string[];
    /** Limit number of results */
    limit?: number;
    /** Offset for pagination */
    offset?: number;
    /** Count total rows */
    count?: "exact" | "planned" | "estimated";
}
/**
 * Insert options
 */
export interface InsertOptions {
    /** Columns to return (default: "*") */
    returning?: string | string[];
    /** Columns to use for conflict resolution */
    onConflict?: string | string[];
    /** Prefer header options */
    prefer?: PreferOptions;
}
/**
 * Update options
 */
export interface UpdateOptions {
    /** Columns to return */
    returning?: string | string[];
    /** Prefer header options */
    prefer?: PreferOptions;
}
/**
 * Delete options
 */
export interface DeleteOptions {
    /** Columns to return */
    returning?: string | string[];
    /** Prefer header options */
    prefer?: PreferOptions;
}
/**
 * RPC function options
 */
export interface RpcOptions {
    /** Columns to select from function result */
    select?: string | string[];
    /** Filter function results */
    filters?: Record<string, string>;
    /** Order function results */
    order?: string | string[];
    /** Limit function results */
    limit?: number;
    /** Offset for pagination */
    offset?: number;
}
/**
 * Error thrown by the parser
 */
export declare class PostgRESTParserError extends Error {
    readonly kind: "parse" | "sql_generation" | "invalid_input";
    constructor(message: string, kind: "parse" | "sql_generation" | "invalid_input");
}
/**
 * Filter clause result
 */
export interface FilterClause {
    /** WHERE clause SQL string */
    clause: string;
    /** Parameter values for the clause */
    params: SqlParam[];
}
