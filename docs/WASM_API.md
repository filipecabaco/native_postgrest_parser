# PostgREST Parser - WASM API Reference

Complete TypeScript/JavaScript API for the PostgREST parser WASM module.

## Installation

```bash
# Build WASM package
wasm-pack build --target web --features wasm

# Use in your project
import init, {
  parseQueryString,
  parseInsert,
  parseUpdate,
  parseDelete,
  parseRpc,
  parseRequest,
  parseOnly,
  buildFilterClause
} from './pkg/postgrest_parser.js';

// Initialize WASM module
await init();
```

## Quick Reference - HTTP Method Routing

The `parseRequest()` function automatically routes HTTP methods to SQL operations:

```typescript
// HTTP Method    → SQL Operation  → Example Query String
parseRequest("GET",    path, qs) // → SELECT      select=id,name&age=gte.18
parseRequest("POST",   path, qs) // → INSERT      returning=id,created_at
parseRequest("PUT",    path, qs) // → UPSERT      id=eq.123&returning=*
parseRequest("PATCH",  path, qs) // → UPDATE      id=eq.123&status=eq.active
parseRequest("DELETE", path, qs) // → DELETE      id=eq.123&returning=id
```

**Special Routing:**
- POST to `rpc/function_name` → RPC function call
- PUT with query filters → INSERT with auto ON CONFLICT
- All methods support optional `headers` for `Prefer` options

## Core Functions

The WASM API provides two layers of functions:

### 🎯 Primary Layer: HTTP Request Routing

**`parseRequest(method, path, queryString, body?, headers?)`** - Main entry point that routes HTTP methods to appropriate operations.

This is the **recommended starting point** - it mirrors PostgREST's HTTP API and automatically handles method-to-operation mapping.

### 🔧 Secondary Layer: Operation-Specific Functions

These provide direct access to specific operations when you know the exact operation type:

- **`parseQueryString(table, queryString)`** - Direct SELECT (equivalent to GET)
- **`parseInsert(table, body, queryString?, headers?)`** - Direct INSERT (equivalent to POST)
- **`parseUpdate(table, body, queryString, headers?)`** - Direct UPDATE (equivalent to PATCH)
- **`parseDelete(table, queryString, headers?)`** - Direct DELETE
- **`parseRpc(functionName, body?, queryString?, headers?)`** - Direct RPC call

**Architecture:**

```
HTTP Request (method + path + query + body + headers)
           ↓
    parseRequest() ← Main routing wrapper
           ↓
    ┌──────┴──────────────────────────────┐
    ↓           ↓           ↓          ↓          ↓
 GET        POST        PUT       PATCH     DELETE
    ↓           ↓           ↓          ↓          ↓
SELECT     INSERT      UPSERT    UPDATE    DELETE
    ↓           ↓           ↓          ↓          ↓
parseQuery  parseInsert  parseInsert  parseUpdate  parseDelete
    └───────────┴───────────┴──────────┴──────────┘
                         ↓
                  SQL Generation
                         ↓
                  WasmQueryResult
                  { query, params, tables }
```

**Special Cases:**
- POST to `rpc/*` → `parseRpc()` (function calls)
- PUT with query filters → INSERT with auto-generated ON CONFLICT

---

### parseQueryString(table, queryString)

Parse a PostgREST query string and generate SQL for SELECT operations.

**Parameters:**
- `table: string` - Table name (can be schema-qualified: "schema.table")
- `queryString: string` - PostgREST query string

**Returns:** `WasmQueryResult`

**Example:**
```typescript
const result = parseQueryString("users",
  "select=id,name,email&age=gte.18&status=eq.active&order=created_at.desc&limit=10"
);

console.log(result.query);   // SELECT "id", "name", "email" FROM "users" WHERE ...
console.log(result.params);  // ["18", "active", 10]
console.log(result.tables);  // ["users"]
```

### parseInsert(table, body, queryString?, headers?)

Generate SQL for INSERT operations.

**Parameters:**
- `table: string` - Table name
- `body: string` - JSON body (single object or array for bulk insert)
- `queryString?: string` - Optional query string for RETURNING, on_conflict, etc.
- `headers?: string` - Optional headers as JSON (e.g., `{"Prefer":"return=representation"}`)

**Returns:** `WasmQueryResult`

**Examples:**

```typescript
// Simple INSERT
const insert = parseInsert("users",
  JSON.stringify({ name: "Alice", email: "alice@example.com" }),
  "returning=id,name",
  null
);

// Bulk INSERT
const bulkInsert = parseInsert("products",
  JSON.stringify([
    { name: "Laptop", price: 999.99 },
    { name: "Mouse", price: 29.99 }
  ]),
  "returning=id",
  null
);

// UPSERT (INSERT with conflict resolution)
const upsert = parseInsert("users",
  JSON.stringify({ email: "alice@example.com", name: "Alice Updated" }),
  "on_conflict=email&returning=id,email",
  JSON.stringify({ Prefer: "return=representation" })
);

// UPSERT with selective column updates
const upsertSelective = parseInsert("products",
  JSON.stringify({ sku: "LAP-001", name: "Laptop", price: 1299, stock: 50 }),
  "on_conflict=sku,update_columns=price,stock&returning=*",
  null
);
```

### parseUpdate(table, body, queryString, headers?)

Generate SQL for UPDATE operations.

**Parameters:**
- `table: string` - Table name
- `body: string` - JSON object with fields to update
- `queryString: string` - Query string with filters (required for safety)
- `headers?: string` - Optional headers as JSON

**Returns:** `WasmQueryResult`

**Examples:**

```typescript
// Simple UPDATE
const update = parseUpdate("users",
  JSON.stringify({ status: "active" }),
  "id=eq.123&returning=id,status",
  null
);

// UPDATE with multiple filters
const updateMultiple = parseUpdate("products",
  JSON.stringify({ price: 899.99, on_sale: true }),
  "category=eq.electronics&price=gt.1000&returning=*",
  null
);

// UPDATE with complex logic
const updateComplex = parseUpdate("orders",
  JSON.stringify({ status: "cancelled" }),
  "or=(status.eq.pending,status.eq.processing)&created_at=lt.2024-01-01",
  JSON.stringify({ Prefer: "return=minimal" })
);
```

### parseDelete(table, queryString, headers?)

Generate SQL for DELETE operations.

**Parameters:**
- `table: string` - Table name
- `queryString: string` - Query string with filters (required for safety)
- `headers?: string` - Optional headers as JSON

**Returns:** `WasmQueryResult`

**Examples:**

```typescript
// Simple DELETE
const del = parseDelete("users", "id=eq.456", null);

// DELETE with RETURNING
const delReturning = parseDelete("sessions",
  "expires_at=lt.2024-01-01&returning=id,user_id",
  null
);

// DELETE with LIMIT and ORDER BY
const delLimited = parseDelete("logs",
  "level=eq.debug&created_at=lt.2024-01-01&limit=1000&order=created_at.asc",
  JSON.stringify({ Prefer: "count=exact" })
);
```

### parseRpc(functionName, body?, queryString?, headers?)

Generate SQL for RPC (stored procedure/function) calls.

**Parameters:**
- `functionName: string` - Function name (can be schema-qualified: "schema.function")
- `body?: string` - JSON object with function arguments (null for no args)
- `queryString?: string` - Optional query string for filtering/ordering results
- `headers?: string` - Optional headers as JSON

**Returns:** `WasmQueryResult`

**Examples:**

```typescript
// RPC with arguments
const rpc = parseRpc("calculate_order_total",
  JSON.stringify({ order_id: 123, tax_rate: 0.08 }),
  "select=total,tax&limit=1",
  null
);

// RPC with result filtering
const rpcFiltered = parseRpc("search_products",
  JSON.stringify({ search_term: "laptop" }),
  "select=id,name,price&price=lte.1500&order=price.asc&limit=10",
  null
);

// RPC without arguments
const rpcNoArgs = parseRpc("get_current_stats",
  null,
  "select=total_users,total_orders",
  null
);

// Schema-qualified RPC
const rpcQualified = parseRpc("analytics.generate_report",
  JSON.stringify({ start_date: "2024-01-01", end_date: "2024-01-31" }),
  null,
  null
);
```

### parseRequest(method, path, queryString, body?, headers?)

**🌟 PRIMARY ENTRY POINT - Main wrapper for all operations**

Parse a complete HTTP request and automatically route to the appropriate SQL operation.

This is the **recommended function** for most use cases - it acts as a smart routing layer that interprets HTTP methods according to PostgREST conventions and delegates to the appropriate operation.

**Parameters:**
- `method: string` - HTTP method: "GET", "POST", "PUT", "PATCH", "DELETE"
- `path: string` - Resource path (table name or "rpc/function_name")
- `queryString: string` - URL query string
- `body?: string` - Request body as JSON
- `headers?: string` - Headers as JSON object

**Returns:** `WasmQueryResult`

**HTTP Method Routing:**

| HTTP Method | Operation | Body Required | Query Filters | Notes |
|-------------|-----------|---------------|---------------|-------|
| **GET** | SELECT | No | Optional | Read data with filters, ordering, pagination |
| **POST** | INSERT | Yes | No* | Create new rows (or RPC if path starts with `rpc/`) |
| **PUT** | UPSERT | Yes | Optional** | Insert or update (auto ON CONFLICT from query filters) |
| **PATCH** | UPDATE | Yes | Required*** | Update existing rows matching filters |
| **DELETE** | DELETE | No | Required*** | Delete rows matching filters |

\* POST supports `on_conflict` in query string for explicit upsert
\*\* PUT auto-generates ON CONFLICT from query string filters (e.g., `id=eq.123` becomes conflict target)
\*\*\* Required for safety - prevents accidental bulk updates/deletes

**Examples:**

```typescript
// GET request → SELECT
const get = parseRequest("GET", "users",
  "age=gte.18&status=eq.active&order=name.asc&limit=10",
  null,
  null
);

// POST request → INSERT
const post = parseRequest("POST", "users",
  "on_conflict=email&returning=id,name",
  JSON.stringify({ name: "Alice", email: "alice@example.com" }),
  JSON.stringify({ Prefer: "return=representation" })
);

// PUT request → UPSERT (automatic ON CONFLICT)
// Query filters (id=eq.123) become conflict target
const put = parseRequest("PUT", "users",
  "id=eq.123&returning=*",
  JSON.stringify({ id: 123, name: "Alice Updated", status: "active" }),
  null
);
// Generates: INSERT ... ON CONFLICT (id) DO UPDATE SET ...

// PATCH request → UPDATE
const patch = parseRequest("PATCH", "users",
  "id=eq.123&returning=*",
  JSON.stringify({ status: "verified" }),
  null
);

// DELETE request → DELETE
const del = parseRequest("DELETE", "sessions",
  "expires_at=lt.2024-01-01&returning=count",
  null,
  JSON.stringify({ Prefer: "count=exact" })
);

// POST to rpc/* → RPC function call
const rpc = parseRequest("POST", "rpc/authenticate_user",
  "select=token,expires_at",
  JSON.stringify({ email: "alice@example.com", password: "secret" }),
  null
);
```

**When to use parseRequest vs specialized functions:**

✅ **Use parseRequest when:**
- You have an HTTP method and want automatic routing
- Building an HTTP proxy or middleware layer
- You need PostgREST-compatible behavior
- Working with dynamic requests from a web framework

✅ **Use specialized functions (parseInsert, parseUpdate, etc.) when:**
- You know the exact operation type at compile time
- Building type-safe APIs with explicit operations
- You want clearer intent in your code
- Working with non-HTTP contexts

### parseOnly(queryString)

Parse query string structure without generating SQL.

Useful for inspecting parsed parameters before SQL generation.

**Parameters:**
- `queryString: string` - PostgREST query string

**Returns:** Parsed parameters as JSON

**Example:**
```typescript
const parsed = parseOnly("select=id,name&age=gte.18&limit=10");
console.log(JSON.stringify(parsed, null, 2));
// {
//   "select": [...],
//   "filters": [...],
//   "order": [],
//   "limit": 10,
//   "offset": null
// }
```

### buildFilterClause(filtersJson)

Build a WHERE clause from filter conditions.

**Parameters:**
- `filtersJson: any` - JSON array of filter conditions

**Returns:** Object with `clause` (SQL string) and `params` (array)

**Example:**
```typescript
const filters = [
  { type: "Filter", field: "age", operator: "gte", value: "18" },
  { type: "Filter", field: "status", operator: "eq", value: "active" }
];

const result = buildFilterClause(filters);
console.log(result.clause);  // WHERE "age" >= $1 AND "status" = $2
console.log(result.params);  // ["18", "active"]
```

## Practical Use Cases

### Use Case 1: HTTP Proxy/Middleware Layer

Use `parseRequest()` to create a lightweight proxy that translates HTTP requests to SQL:

```typescript
import init, { parseRequest } from './pkg/postgrest_parser.js';
await init();

// Express.js middleware example
app.use('/api/:table', async (req, res) => {
  try {
    const result = parseRequest(
      req.method,                    // GET, POST, PUT, PATCH, DELETE
      req.params.table,              // Table name from URL
      req.query.toString(),          // Query string parameters
      req.body ? JSON.stringify(req.body) : null,
      JSON.stringify(req.headers)    // Pass through headers
    );

    // Execute against your database
    const rows = await db.query(result.query, result.params);
    res.json(rows);
  } catch (error) {
    res.status(400).json({ error: error.message });
  }
});

// Now your API automatically supports PostgREST syntax:
// GET  /api/users?age=gte.18&select=id,name
// POST /api/users + body { "name": "Alice" }
// PUT  /api/users?id=eq.123 + body { "id": 123, "name": "Alice" }
// PATCH /api/users?id=eq.123 + body { "status": "active" }
// DELETE /api/users?id=eq.123
```

### Use Case 2: Edge Function/Serverless

Parse requests at the edge without a full PostgREST server:

```typescript
// Cloudflare Worker / Deno Deploy / Supabase Edge Function
import init, { parseRequest } from './pkg/postgrest_parser.js';

await init();

Deno.serve(async (req) => {
  const url = new URL(req.url);
  const path = url.pathname.slice(1); // Remove leading /
  const query = url.search.slice(1);  // Remove leading ?

  let body = null;
  if (req.method !== 'GET' && req.method !== 'DELETE') {
    body = await req.text();
  }

  const headers = Object.fromEntries(req.headers);

  const result = parseRequest(
    req.method,
    path,
    query,
    body,
    JSON.stringify(headers)
  );

  // Execute with Postgres client
  const data = await postgres.query(result.query, result.params);

  return new Response(JSON.stringify(data), {
    headers: { 'Content-Type': 'application/json' }
  });
});
```

### Use Case 3: Query Builder UI

Build a visual query builder that generates SQL:

```typescript
import { parseRequest } from './pkg/postgrest_parser.js';

function buildQuery(state) {
  const queryString = new URLSearchParams({
    select: state.selectedColumns.join(','),
    ...state.filters,
    order: state.orderBy,
    limit: state.limit
  }).toString();

  const result = parseRequest(
    'GET',
    state.tableName,
    queryString,
    null,
    null
  );

  // Show SQL to user
  document.getElementById('sql-preview').textContent = result.query;
  document.getElementById('params-preview').textContent =
    JSON.stringify(result.params, null, 2);
}
```

### Use Case 4: Multi-Tenant SaaS with RLS

Combine with Row Level Security for multi-tenant apps:

```typescript
async function handleTenantRequest(tenantId, req) {
  const result = parseRequest(
    req.method,
    req.path,
    req.queryString,
    req.body,
    JSON.stringify(req.headers)
  );

  // Add tenant_id to query automatically
  const tenantedQuery = result.query.replace(
    'WHERE',
    `WHERE "tenant_id" = $${result.params.length + 1} AND`
  );

  const tenantedParams = [...result.params, tenantId];

  return await db.query(tenantedQuery, tenantedParams);
}
```

### Use Case 5: GraphQL to PostgREST Bridge

Convert GraphQL queries to PostgREST format:

```typescript
function graphqlToPostgrest(graphqlQuery) {
  // Extract fields, filters, etc. from GraphQL query
  const { table, fields, where, orderBy, limit } = parseGraphQL(graphqlQuery);

  // Build PostgREST query string
  const queryParts = [];
  if (fields) queryParts.push(`select=${fields.join(',')}`);
  if (where) queryParts.push(...convertFilters(where));
  if (orderBy) queryParts.push(`order=${orderBy}`);
  if (limit) queryParts.push(`limit=${limit}`);

  const queryString = queryParts.join('&');

  // Convert to SQL
  return parseRequest('GET', table, queryString, null, null);
}
```

## Return Types

### WasmQueryResult

All SQL generation functions return a `WasmQueryResult` object:

```typescript
interface WasmQueryResult {
  query: string;           // Generated SQL query
  params: any[];           // Parameter values for $1, $2, etc.
  tables: string[];        // List of tables referenced in the query
}
```

**Properties:**
- `query` - The parameterized SQL query with $1, $2, ... placeholders
- `params` - Array of parameter values (strings, numbers, arrays, etc.)
- `tables` - Array of table names referenced in the query

## HTTP Method Details

### PUT Method - Smart UPSERT

The PUT method implements PostgREST's UPSERT behavior with intelligent ON CONFLICT generation.

**How PUT works:**

1. **Automatic ON CONFLICT**: PUT extracts column names from query string filters to create conflict targets
2. **Smart Routing**: If filters present → generates ON CONFLICT, if no filters → regular INSERT
3. **PostgREST Compatible**: Matches PostgREST's PUT behavior exactly

**Examples:**

```typescript
// PUT with query filter → Auto-generates ON CONFLICT
const result = parseRequest(
  "PUT",
  "users",
  "email=eq.alice@example.com&returning=*",
  JSON.stringify({
    email: "alice@example.com",
    name: "Alice",
    age: 30
  }),
  null
);

// Generates SQL:
// INSERT INTO "users" ("email", "name", "age")
// VALUES ($1, $2, $3)
// ON CONFLICT ("email") DO UPDATE
// SET "email" = EXCLUDED."email",
//     "name" = EXCLUDED."name",
//     "age" = EXCLUDED."age"
// RETURNING *

// Multiple conflict columns
const result2 = parseRequest(
  "PUT",
  "inventory",
  "warehouse_id=eq.123&product_id=eq.456",
  JSON.stringify({
    warehouse_id: 123,
    product_id: 456,
    quantity: 100
  }),
  null
);
// ON CONFLICT ("warehouse_id", "product_id") DO UPDATE ...

// PUT without filters → Regular INSERT (no conflict handling)
const result3 = parseRequest(
  "PUT",
  "logs",
  "returning=id",
  JSON.stringify({ message: "New log entry" }),
  null
);
// INSERT INTO "logs" ... (no ON CONFLICT)
```

**PUT vs POST for UPSERT:**

| Method | ON CONFLICT Behavior | Use When |
|--------|---------------------|----------|
| **POST** | Explicit via `on_conflict` query param | You want control over conflict resolution |
| **PUT** | Auto-generated from query filters | You want RESTful semantics (PUT = idempotent) |

```typescript
// POST - Explicit ON CONFLICT
parseRequest(
  "POST",
  "users",
  "on_conflict=email,update_columns=name,age&returning=*",
  JSON.stringify({ email: "alice@example.com", name: "Alice", age: 30 }),
  null
);

// PUT - Auto ON CONFLICT from filters
parseRequest(
  "PUT",
  "users",
  "email=eq.alice@example.com&returning=*",
  JSON.stringify({ email: "alice@example.com", name: "Alice", age: 30 }),
  null
);
```

**Best Practices:**
- ✅ Use PUT when building RESTful APIs (PUT is idempotent by definition)
- ✅ Use POST with `on_conflict` when you need selective column updates
- ✅ Always include the conflict columns in the body to match query filters
- ⚠️ PUT without filters acts as regular INSERT (no upsert)

### PATCH vs PUT

| Aspect | PATCH | PUT |
|--------|-------|-----|
| **Semantics** | Partial update | Full replace (or create if not exists) |
| **Operation** | UPDATE | INSERT with ON CONFLICT |
| **Filters** | Required (WHERE clause) | Optional (conflict target) |
| **Body** | Only fields to update | Complete resource representation |
| **Idempotent** | No* | Yes |

\* PATCH with same filters is idempotent, but semantically it's a partial update

```typescript
// PATCH - Update specific fields
parseRequest(
  "PATCH",
  "users",
  "id=eq.123",
  JSON.stringify({ status: "active" }), // Only updating status
  null
);
// UPDATE "users" SET "status" = $1 WHERE "id" = $2

// PUT - Replace/create entire resource
parseRequest(
  "PUT",
  "users",
  "id=eq.123",
  JSON.stringify({
    id: 123,
    name: "Alice",
    email: "alice@example.com",
    status: "active"
  }), // Full resource
  null
);
// INSERT ... ON CONFLICT (id) DO UPDATE SET ...
```

## Supported Features

### Filter Operators

All PostgREST operators are supported:

**Comparison:**
- `eq`, `neq`, `gt`, `gte`, `lt`, `lte`

**Pattern Matching:**
- `like`, `ilike` - SQL LIKE (case-sensitive/insensitive)
- `match`, `imatch` - POSIX regex

**Array & Range:**
- `in` - Value in list
- `cs`, `cd` - Contains, Contained by
- `ov` - Overlaps
- `sl`, `sr`, `nxl`, `nxr`, `adj` - Range operators

**Full-Text Search:**
- `fts` - to_tsvector / plainto_tsquery
- `plfts` - plainto_tsquery
- `phfts` - phraseto_tsquery
- `wfts` - websearch_to_tsquery

**Special:**
- `is` - IS NULL, IS TRUE, IS FALSE
- `not` - Negation prefix

### Logic Operators

- `and=(filter1,filter2)` - AND logic
- `or=(filter1,filter2)` - OR logic
- `not.operator` - Negation

### Quantifiers

- `eq(any).{val1,val2}` - = ANY(array)
- `gt(all).{val1,val2}` - > ALL(array)

### JSON Operations

- `data->field` - JSON field access (returns JSON)
- `data->>field` - JSON text extraction (returns text)
- `data->field1->field2` - Deep navigation

### Type Casting

- `price::numeric=gt.100` - Cast to type before comparison

### Ordering

- `order=col1.asc,col2.desc.nullslast`
- Null handling: `nullsfirst`, `nullslast`

### Pagination

- `limit=10` - LIMIT clause
- `offset=20` - OFFSET clause

### RETURNING Clause

All mutations support RETURNING:
- `returning=id,name,created_at` - Return specific columns
- `returning=*` - Return all columns

### ON CONFLICT (Upsert)

- `on_conflict=email` - Simple conflict target
- `on_conflict=email,update_columns=name,status` - Selective update

## Performance

The parser is optimized for high throughput:

- **SELECT**: ~1000-2000 queries/second
- **INSERT**: ~1000-1500 operations/second
- **UPDATE/DELETE**: ~1000-1500 operations/second
- **RPC**: ~1500-2000 calls/second

All parsing uses nom combinators (no regex) for maximum performance.

## Security

### SQL Injection Prevention

All values are parameterized - never concatenated into SQL:

```typescript
// ✅ Safe - uses parameters
const result = parseQueryString("users", "name=eq.'; DROP TABLE users; --");
// Generates: WHERE "name" = $1
// Params: ["'; DROP TABLE users; --"]

// The malicious input becomes a harmless string parameter
```

### Required Filters for Mutations

UPDATE and DELETE require filters for safety:

```typescript
// ❌ Error - UPDATE without filter
parseUpdate("users", JSON.stringify({ status: "deleted" }), "");

// ✅ Safe - filter required
parseUpdate("users", JSON.stringify({ status: "deleted" }), "id=eq.123");
```

## Examples

See comprehensive examples in:
- [examples/wasm_example.ts](examples/wasm_example.ts) - SELECT queries (20 examples)
- [examples/wasm_mutations_example.ts](examples/wasm_mutations_example.ts) - Mutations & RPC (20 examples)

Run examples:
```bash
# SELECT examples
deno run --allow-read examples/wasm_example.ts

# Mutation examples
deno run --allow-read examples/wasm_mutations_example.ts
```

## Error Handling

All functions return `Result<WasmQueryResult, JsValue>`. Errors are thrown as JavaScript exceptions:

```typescript
try {
  const result = parseInsert("users", "invalid json", null, null);
} catch (error) {
  console.error("Parse error:", error);
  // Parse error: Invalid JSON in body
}
```

## Type Definitions

For TypeScript projects, you can add type definitions:

```typescript
interface WasmQueryResult {
  query: string;
  params: any[];
  tables: string[];
}

declare function parseQueryString(table: string, queryString: string): WasmQueryResult;
declare function parseInsert(table: string, body: string, queryString?: string, headers?: string): WasmQueryResult;
declare function parseUpdate(table: string, body: string, queryString: string, headers?: string): WasmQueryResult;
declare function parseDelete(table: string, queryString: string, headers?: string): WasmQueryResult;
declare function parseRpc(functionName: string, body?: string, queryString?: string, headers?: string): WasmQueryResult;
declare function parseRequest(method: string, path: string, queryString: string, body?: string, headers?: string): WasmQueryResult;
declare function parseOnly(queryString: string): any;
declare function buildFilterClause(filtersJson: any): { clause: string; params: any[] };
```
