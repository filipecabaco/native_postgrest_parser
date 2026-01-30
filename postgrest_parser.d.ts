/**
 * TypeScript definitions for postgrest-parser WASM bindings
 *
 * @module postgrest-parser
 */

/**
 * Query result containing SQL, parameters, and affected tables
 */
export interface QueryResult {
  /**
   * The generated PostgreSQL SELECT query with parameter placeholders ($1, $2, etc.)
   */
  readonly query: string;

  /**
   * Array of parameter values corresponding to placeholders in the query
   */
  readonly params: any[];

  /**
   * List of table names referenced in the query
   */
  readonly tables: string[];

  /**
   * Convert the result to a plain JSON object
   */
  toJSON(): QueryResult;
}

/**
 * Filter clause result containing WHERE clause and parameters
 */
export interface FilterClauseResult {
  /**
   * The WHERE clause SQL fragment (without the "WHERE" keyword)
   */
  clause: string;

  /**
   * Parameter values referenced in the clause
   */
  params: any[];
}

/**
 * Parse a PostgREST query string and convert it to SQL.
 *
 * @param table - The table name to query
 * @param queryString - The PostgREST query string (e.g., "select=id,name&age=gte.18")
 * @returns Query result with SQL, params, and tables
 * @throws Error if parsing or SQL generation fails
 *
 * @example
 * ```typescript
 * const result = parseQueryString("users", "age=gte.18&status=eq.active&limit=10");
 * console.log(result.query);   // SELECT * FROM "users" WHERE "age" >= $1 AND "status" = $2 LIMIT $3
 * console.log(result.params);  // ["18", "active", 10]
 * console.log(result.tables);  // ["users"]
 * ```
 */
export function parseQueryString(table: string, queryString: string): QueryResult;

/**
 * Parse a query string without generating SQL.
 *
 * Useful for inspecting the parsed structure or validating queries.
 *
 * @param queryString - The PostgREST query string
 * @returns Parsed parameters as a JSON object
 * @throws Error if parsing fails
 *
 * @example
 * ```typescript
 * const parsed = parseOnly("age=gte.18&order=name.asc");
 * console.log(parsed.filters);  // Array of filter conditions
 * console.log(parsed.order);    // Array of order terms
 * ```
 */
export function parseOnly(queryString: string): any;

/**
 * Build a WHERE clause from filter conditions.
 *
 * @param filters - JSON array of filter conditions
 * @returns Object with clause (SQL string) and params (array of values)
 * @throws Error if building the clause fails
 *
 * @example
 * ```typescript
 * const filters = [{
 *   Filter: {
 *     field: { name: "age", json_path: [], cast: null },
 *     operator: "Gte",
 *     value: { Single: "18" },
 *     quantifier: null,
 *     language: null,
 *     negated: false
 *   }
 * }];
 *
 * const result = buildFilterClause(filters);
 * console.log(result.clause);  // "age" >= $1
 * console.log(result.params);  // ["18"]
 * ```
 */
export function buildFilterClause(filters: any[]): FilterClauseResult;

/**
 * Initialize the WASM module. Must be called before using any functions.
 *
 * @example
 * ```typescript
 * import init, { parseQueryString } from './postgrest_parser.js';
 *
 * await init();
 * const result = parseQueryString("users", "id=eq.1");
 * ```
 */
export default function init(input?: RequestInfo | URL): Promise<void>;
