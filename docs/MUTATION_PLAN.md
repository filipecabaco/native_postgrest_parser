# Implementation Plan: PostgREST Mutation Support (INSERT, UPDATE, DELETE)

## Overview

Add INSERT, UPDATE, and DELETE mutation support to the `native_postgrest_parser` crate with a unified `parse()` API and PostgREST schema support. Currently only SELECT queries are supported. This plan follows existing architectural patterns and reuses 100% of the filter infrastructure.

**Key Features:**
- Unified `parse(method, table, query_string, body, headers)` API for all operations
- Schema resolution via PostgREST headers (`Accept-Profile`, `Content-Profile`)
- Support for explicit schema in table names (`schema.table`)
- Safe UPDATE/DELETE with mandatory filter enforcement
- Full WASM/JavaScript support with header handling

**Estimated Effort:** 5-6 weeks (assuming part-time work)
**New Code:** ~2,500-3,000 lines
**Test Coverage:** 270+ new tests (Rust + WASM)

## Architecture Decision: Unified Parse API with Schema Support

```rust
use std::collections::HashMap;

pub enum Operation {
    Select(ParsedParams),
    Insert(InsertParams),
    Update(UpdateParams),
    Delete(DeleteParams),
}

// Single unified entry point for all operations
pub fn parse(
    method: &str,           // "GET", "POST", "PATCH", "DELETE"
    table: &str,            // "users" or "myschema.users"
    query_string: &str,
    body: Option<&str>,
    headers: Option<&HashMap<String, String>>, // For schema detection
) -> Result<Operation, Error>

pub fn to_sql(table: &str, operation: &Operation) -> Result<QueryResult, Error>
```

**Rationale:**
- Single API surface - no distinction between "queries" and "mutations"
- Mirrors HTTP semantics directly (method determines operation)
- Schema support via PostgREST headers (`Accept-Profile`, `Content-Profile`)
- Better type safety, easier WASM bindings, future-proof

### Schema Resolution

Schema is resolved in priority order:

1. **Explicit in table name** (highest priority): `"myschema.users"` → use `myschema`
2. **Header-based** (PostgREST standard):
   - `GET` → `Accept-Profile` header
   - `POST/PATCH/DELETE` → `Content-Profile` header
3. **Default**: `"public"`

```rust
struct ResolvedTable {
    schema: String,  // "public", "auth", "myschema", etc.
    name: String,    // "users", "posts", etc.
}

fn resolve_schema(
    table: &str,
    method: &str,
    headers: Option<&HashMap<String, String>>
) -> ResolvedTable
```

### Public API Surface

**Core Functions (src/lib.rs):**
```rust
// Unified parse function - handles all HTTP methods
pub fn parse(
    method: &str,
    table: &str,
    query_string: &str,
    body: Option<&str>,
    headers: Option<&HashMap<String, String>>,
) -> Result<Operation, Error>

// SQL generation for any operation
pub fn to_sql(table: &str, operation: &Operation) -> Result<QueryResult, Error>
```

**Operation Routing:**
- `GET` → `Operation::Select(ParsedParams)`
- `POST` → `Operation::Insert(InsertParams)`
- `PATCH` → `Operation::Update(UpdateParams)`
- `DELETE` → `Operation::Delete(DeleteParams)`

**Example Usage:**
```rust
use std::collections::HashMap;
use native_postgrest_parser::{parse, to_sql};

// SELECT with Accept-Profile header
let mut headers = HashMap::new();
headers.insert("Accept-Profile".to_string(), "public".to_string());
let op = parse("GET", "users", "id=eq.123", None, Some(&headers))?;
let result = to_sql("users", &op)?;
// → SELECT * FROM "public"."users" WHERE "id" = $1

// INSERT with Content-Profile header
let mut headers = HashMap::new();
headers.insert("Content-Profile".to_string(), "auth".to_string());
let body = r#"{"email": "user@example.com", "name": "Alice"}"#;
let op = parse("POST", "users", "returning=id", Some(body), Some(&headers))?;
let result = to_sql("users", &op)?;
// → INSERT INTO "auth"."users" ("email", "name") VALUES ($1, $2) RETURNING "id"

// UPDATE with explicit schema in table name (overrides header)
let body = r#"{"status": "active"}"#;
let op = parse("PATCH", "myschema.users", "id=eq.123", Some(body), None)?;
let result = to_sql("myschema.users", &op)?;
// → UPDATE "myschema"."users" SET "status" = $1 WHERE "id" = $2

// DELETE with Content-Profile header
let mut headers = HashMap::new();
headers.insert("Content-Profile".to_string(), "analytics".to_string());
let op = parse("DELETE", "events", "created_at=lt.2023-01-01", None, Some(&headers))?;
let result = to_sql("events", &op)?;
// → DELETE FROM "analytics"."events" WHERE "created_at" < $1
```

## Key Insights from PostgREST

- **Schema handling:** Uses `Accept-Profile` (reads) and `Content-Profile` (writes) headers, defaults to `public`
- **INSERT (POST):** Body is JSON object/array, supports `on_conflict`, `columns`, `returning` params
- **UPDATE (PATCH):** Body is JSON object with new values, **requires filters** (safety), supports `limit`+`order`, `returning`
- **DELETE:** No body, **requires filters** (safety), supports `limit`+`order`, `returning`
- **Shared:** All operations use same filter syntax (22+ operators), JSON paths, type casting

## Implementation Phases

### Phase 1: Foundation (Week 1)

Create AST types, schema resolution, and error infrastructure.

**Files to Create:**
- `src/ast/mutation.rs` (~350 lines)
  - `InsertParams`, `UpdateParams`, `DeleteParams`
  - `InsertValues` enum (Single/Bulk)
  - `OnConflict`, `ConflictAction`
  - `ResolvedTable` struct (schema + name)
  - Builder methods (immutable pattern like `ParsedParams`)
  - 70 unit tests (including schema tests)

- `src/parser/schema.rs` (~150 lines)
  - `resolve_schema(table, method, headers) -> ResolvedTable`
  - `parse_qualified_table(table) -> (Option<schema>, name)`
  - `get_profile_header(method, headers) -> Option<String>`
  - 25 unit tests covering:
    - Explicit schema in table name: `"auth.users"`
    - Header-based resolution: `Accept-Profile`, `Content-Profile`
    - Default to `public` schema
    - Invalid schema names
    - Case sensitivity

**Files to Modify:**
- `src/ast/mod.rs` - Add `pub mod mutation;`
- `src/parser/mod.rs` - Add `pub mod schema;`
- `src/error/parse.rs` - Add mutation errors:
  - `InvalidJsonBody(String)`
  - `InvalidInsertBody(String)`
  - `EmptyUpdateBody`
  - `InvalidUpdateBody(String)`
  - `InvalidOnConflict(String)`
  - `UnsupportedMethod(String)`
  - `InvalidSchema(String)`
  - `InvalidTableName(String)`
- `src/error/sql.rs` - Add safety errors:
  - `UnsafeUpdate` - no WHERE clause
  - `UnsafeDelete` - no WHERE clause
  - `LimitWithoutOrder` - determinism issue
  - `NoInsertValues`
  - `NoUpdateSet`
- `src/parser/filter.rs` - Update `reserved_key` to include `"on_conflict"`, `"columns"`, `"returning"`

**Verification:**
```bash
cargo test --lib ast::mutation
cargo test --lib parser::schema
cargo test --lib error::
```

**Example Schema Resolution:**
```rust
// Explicit schema
resolve_schema("auth.users", "GET", None)
// → ResolvedTable { schema: "auth", name: "users" }

// Header-based (GET)
let mut headers = HashMap::new();
headers.insert("Accept-Profile".to_string(), "myschema".to_string());
resolve_schema("users", "GET", Some(&headers))
// → ResolvedTable { schema: "myschema", name: "users" }

// Default
resolve_schema("users", "POST", None)
// → ResolvedTable { schema: "public", name: "users" }
```

---

### Phase 2: INSERT Support (Week 2)

Full INSERT functionality with RETURNING and ON CONFLICT.

**Files to Create:**
- `src/parser/body.rs` (~200 lines)
  - `parse_json_body(body: &str) -> Result<Value, ParseError>`
  - `validate_insert_body(value) -> Result<InsertValues, ParseError>`
  - `validate_update_body(value) -> Result<Map, ParseError>`
  - 40 unit tests

- `src/parser/mutation.rs` (~150 lines for INSERT)
  - `parse_insert_params(query_string, body) -> Result<InsertParams, Error>`
  - Parse `on_conflict=col1,col2`
  - Parse `columns=col1,col2,col3`
  - Parse `returning=col1,col2`
  - 30 unit tests

- `src/sql/mutation.rs` (~250 lines for INSERT)
  - `build_insert(&mut self, resolved_table, params) -> Result<QueryResult, SqlError>`
  - `build_values_clause(&mut self, values) -> Result<(), SqlError>`
  - `build_on_conflict_clause(&mut self, on_conflict) -> Result<(), SqlError>`
  - `build_returning_clause(&mut self, items) -> Result<(), SqlError>` (reuse from SELECT)
  - Schema-qualified table names in SQL
  - 45 unit tests (including schema tests)

- `tests/mutation/insert_tests.rs` (~220 lines)
  - 23 integration tests covering:
    - Single row insert
    - Bulk insert (2, 10, 100 rows)
    - ON CONFLICT DO NOTHING
    - ON CONFLICT DO UPDATE
    - RETURNING clause
    - JSON path fields
    - Type casting
    - Schema handling (explicit, header, default)

**Files to Modify:**
- `src/lib.rs` - Add unified `parse()` and `to_sql()` entry points
- `src/parser/mod.rs` - Export body parsing functions and schema module
- `src/sql/mod.rs` - Export mutation module

**Verification:**
```bash
cargo test mutation::insert
cargo test --test insert_tests
```

**Example Output:**
```sql
-- Single row (default public schema)
INSERT INTO "public"."users" ("name", "age") VALUES ($1, $2)

-- Bulk
INSERT INTO "public"."users" ("name", "age") VALUES ($1, $2), ($3, $4)

-- ON CONFLICT
INSERT INTO "public"."users" ("email", "name") VALUES ($1, $2)
ON CONFLICT ("email") DO UPDATE SET "name" = EXCLUDED."name"

-- Custom schema (from header or table name)
INSERT INTO "auth"."users" ("email", "provider") VALUES ($1, $2)
```

---

### Phase 3: UPDATE Support (Week 3)

Safe UPDATE with mandatory filter enforcement.

**Files to Update:**
- `src/parser/mutation.rs` (+150 lines)
  - `parse_update_params(query_string, body) -> Result<UpdateParams, Error>`
  - Reuse filter parsing from existing code
  - Parse `limit`, `order`, `returning`
  - 25 unit tests

- `src/sql/mutation.rs` (+200 lines)
  - `build_update(&mut self, resolved_table, params) -> Result<QueryResult, SqlError>`
  - `build_set_clause(&mut self, set_values) -> Result<(), SqlError>`
  - `validate_update_safety(&self, params) -> Result<(), SqlError>`
  - Reuse `build_where_clause`, `build_order_clause`, `build_limit_offset`
  - Schema-qualified table names in SQL
  - 40 unit tests (including schema tests)

- `tests/mutation/update_tests.rs` (~250 lines)
  - 28 integration tests covering:
    - UPDATE with simple filters
    - UPDATE with complex logic (AND/OR)
    - UPDATE with limit+order
    - UPDATE with JSON path fields
    - UPDATE with custom schema (header and table name)
    - **Safety:** UPDATE without filters (should error)
    - **Safety:** LIMIT without ORDER (should error)
    - RETURNING clause

**Files to Modify:**
- `src/lib.rs` - Update unified `parse()` to handle PATCH
- `src/sql/mod.rs` - Add UPDATE to `to_sql()` routing
- `src/sql/builder.rs` - Make `build_order_clause` and `build_limit_offset` public (for reuse)

**Verification:**
```bash
cargo test mutation::update
cargo test --test update_tests

# Safety tests should FAIL appropriately
cargo test update_without_filters_errors
cargo test limit_without_order_errors
```

**Example Output:**
```sql
-- Safe UPDATE (default public schema)
UPDATE "public"."users" SET "status" = $1, "updated_at" = $2
WHERE "id" = $3 AND "age" > $4

-- With LIMIT+ORDER
UPDATE "public"."tasks" SET "status" = $1
WHERE "created_at" < $2
ORDER BY "created_at" ASC
LIMIT $3

-- Custom schema (from Content-Profile header)
UPDATE "auth"."users" SET "last_login" = $1
WHERE "id" = $2
```

---

### Phase 4: DELETE Support (Week 4)

Safe DELETE with mandatory filter enforcement.

**Files to Update:**
- `src/parser/mutation.rs` (+100 lines)
  - `parse_delete_params(query_string) -> Result<DeleteParams, Error>`
  - Reuse filter parsing
  - 20 unit tests

- `src/sql/mutation.rs` (+150 lines)
  - `build_delete(&mut self, resolved_table, params) -> Result<QueryResult, SqlError>`
  - `validate_delete_safety(&self, params) -> Result<(), SqlError>`
  - Schema-qualified table names in SQL
  - 30 unit tests (including schema tests)

- `tests/mutation/delete_tests.rs` (~200 lines)
  - 23 integration tests (including schema scenarios)
- `tests/mutation/safety_tests.rs` (~150 lines)
  - 10 dedicated safety tests for UPDATE/DELETE

**Files to Modify:**
- `src/lib.rs` - Update unified `parse()` to handle DELETE
- `src/sql/mod.rs` - Add DELETE to `to_sql()` routing

**Verification:**
```bash
cargo test mutation::delete
cargo test --test delete_tests
cargo test --test safety_tests
```

**Example Output:**
```sql
-- Safe DELETE (default public schema)
DELETE FROM "public"."comments"
WHERE ("status" = $1 OR ("created_at" < $2 AND "verified" IS FALSE))

-- With LIMIT+ORDER
DELETE FROM "public"."logs"
WHERE "created_at" < $1
ORDER BY "created_at" ASC
LIMIT $2

-- Custom schema (from Content-Profile header or table name)
DELETE FROM "analytics"."events"
WHERE "created_at" < $1
```

---

### Phase 5: WASM Bindings (Week 5)

TypeScript/JavaScript support with unified parse API and schema support.

**Files to Modify:**
- `src/wasm.rs` (+200 lines)
  - `WasmOperation` enum (mirrors Rust `Operation`)
  - `parse_wasm(method, table, query_string, body?, headers?)` - unified entry point
  - `to_sql_wasm(table, operation)` - SQL generation
  - Convert Rust `HashMap<String, String>` to JS `Record<string, string>`
  - Convenience functions (optional):
    - `parse_select_wasm(table, query_string, headers?)`
    - `parse_insert_wasm(table, query_string, body, headers?)`
    - `parse_update_wasm(table, query_string, body, headers?)`
    - `parse_delete_wasm(table, query_string, headers?)`

- `tests/integration/wasm_test.ts` (+350 lines)
  - 35 WASM integration tests:
    - Unified `parse()` API for all operations
    - Schema resolution (explicit, headers, default)
    - INSERT single/bulk/on_conflict with schemas
    - UPDATE with/without filters with schemas
    - DELETE with/without filters with schemas
    - Error handling
    - Performance benchmarks

**Verification:**
```bash
wasm-pack build --target web --features wasm
deno test --allow-read tests/integration/wasm_test.ts
```

**Example Usage:**
```typescript
import init, { parse, toSql } from "./pkg/postgrest_parser.js";

await init();

// SELECT (default public schema)
const selectOp = parse("GET", "users", "id=eq.123");
const { sql, params } = toSql("users", selectOp);
// → SELECT * FROM "public"."users" WHERE "id" = $1

// INSERT with Content-Profile header
const insertOp = parse(
  "POST",
  "users",
  "returning=id,created_at",
  JSON.stringify({ name: "Alice", age: 30 }),
  { "Content-Profile": "auth" }
);
// → INSERT INTO "auth"."users" ...

// UPDATE with explicit schema in table name
const updateOp = parse(
  "PATCH",
  "myschema.users",
  "id=eq.123",
  JSON.stringify({ status: "active" })
);
// → UPDATE "myschema"."users" SET "status" = $1 WHERE "id" = $2

// DELETE with Accept-Profile header (reads)
const deleteOp = parse(
  "DELETE",
  "users",
  "status=eq.deleted&created_at=lt.2020-01-01",
  undefined,
  { "Content-Profile": "analytics" }
);
// → DELETE FROM "analytics"."users" WHERE ...

// Convenience functions (optional, for clearer code)
import { parseSelect, parseInsert, parseUpdate, parseDelete } from "./pkg/postgrest_parser.js";

const result = parseSelect("users", "id=eq.123", { "Accept-Profile": "public" });
```

---

### Phase 6: Documentation & Polish (Week 6)

Production-ready release.

**Documentation Updates:**
- `README.md` - Add mutation examples and API reference
- `BENCHMARKS.md` - Add mutation performance benchmarks
- `CHANGELOG.md` - Document v0.2.0 release with mutations
- Create `docs/MUTATIONS.md` - Comprehensive mutation guide

**Additional Testing:**
- Add property-based tests (proptest) for mutations
- Add edge case tests (empty values, special characters, etc.)
- Target: 95%+ code coverage for mutation code

**Performance:**
- Add mutation benchmarks to `benches/`
- Expected: INSERT ~50-100μs, UPDATE/DELETE ~80-120μs

**Verification:**
```bash
cargo test --all
cargo bench
cargo doc --open
```

## Critical Files Reference

**Pattern Templates (Read First):**
1. [src/ast/params.rs](../src/ast/params.rs) - Immutable builder pattern
2. [src/sql/builder.rs](../src/sql/builder.rs) - SQL generation pattern
3. [src/parser/filter.rs](../src/parser/filter.rs) - nom parser pattern

**Reuse Directly:**
- Filter parsing (22+ operators) - `src/parser/filter.rs`
- WHERE clause generation - `src/sql/builder.rs:build_where_clause()`
- Field handling (JSON paths, type casting) - `src/parser/common.rs`
- Error types infrastructure - `src/error/mod.rs`

## Safety Mechanisms

**Compile-Time:**
- Type system enforces proper params per operation
- Builder pattern requires necessary fields

**Runtime:**
```rust
// UPDATE/DELETE validation
fn validate_update_safety(params: &UpdateParams) -> Result<(), SqlError> {
    if params.filters.is_empty() {
        return Err(SqlError::UnsafeUpdate);
    }
    if params.limit.is_some() && params.order.is_empty() {
        return Err(SqlError::LimitWithoutOrder);
    }
    Ok(())
}
```

**Default:** Hard errors for unsafe operations. Future: Add `allow_unsafe` flag for advanced users.

## Testing Strategy

**Total New Tests:** 320+

**Unit Tests (195):**
- AST: 70 tests (including schema types)
- Schema resolution: 25 tests
- Parser: 75 tests
- Body validation: 40 tests
- SQL builder: 115 tests (including schema tests)

**Integration Tests (92):**
- INSERT: 23 tests (including schema scenarios)
- UPDATE: 28 tests (including schema scenarios)
- DELETE: 23 tests (including schema scenarios)
- Safety: 10 tests
- Schema integration: 8 tests (cross-cutting schema tests)

**WASM Tests (35):**
- Unified parse API
- Schema resolution from headers
- All operation types with schemas
- Error handling
- Performance benchmarks

## Real-World Test Cases

```sql
-- User signup with upsert (auth schema)
INSERT INTO "auth"."users" ("email", "password_hash", "created_at")
VALUES ($1, $2, $3)
ON CONFLICT ("email") DO UPDATE SET "password_hash" = EXCLUDED."password_hash"

-- Bulk status update (public schema)
UPDATE "public"."tasks" SET "status" = $1, "updated_at" = $2
WHERE "created_at" < $3 AND "status" = $4
ORDER BY "created_at" ASC
LIMIT $5

-- Soft delete with complex logic (public schema)
DELETE FROM "public"."comments"
WHERE ("status" = $1 OR ("created_at" < $2 AND "verified" IS FALSE))

-- Cross-schema query (explicit schema in table name)
SELECT * FROM "analytics"."events"
WHERE "user_id" = $1 AND "created_at" > $2
```

## Success Criteria

✅ All 320+ tests pass
✅ Unified `parse()` API handles all operations (GET/POST/PATCH/DELETE)
✅ Schema support via PostgREST headers (Accept-Profile, Content-Profile)
✅ Schema resolution works (explicit, headers, default to public)
✅ WASM integration works in browser and Deno with headers support
✅ Safety validations prevent unsafe UPDATE/DELETE
✅ Performance matches SELECT (~50-120μs per operation)
✅ Documentation complete with examples
✅ Backward compatible (no breaking changes to existing API)
✅ Code coverage >95% for new code

## Migration Impact

**Breaking Changes:** None (new functionality only)

**New Dependencies:** None (uses existing `serde_json`)

**Version:** Bump to v0.2.0 (minor version, new features)

## Rollout Plan

1. **Week 1-4:** Internal development (phases 1-4)
2. **Week 5:** WASM support
3. **Week 6:** Documentation, testing, polish
4. **Week 7:** Beta release for community testing
5. **Week 8:** Stable v0.2.0 release

## Post-Implementation

**Future Enhancements (v0.3.0+):**
- `allow_unsafe` flag for UPDATE/DELETE without filters
- UPSERT as first-class operation
- Batch mutation support (multi-table)
- Stored procedure support
- RPC function calls

## Summary of Key Architectural Decisions

### 1. Unified Parse API
Instead of separate `parse_query_string()` for SELECT and `parse_mutation()` for INSERT/UPDATE/DELETE, we use a single unified entry point:

```rust
pub fn parse(method: &str, table: &str, query_string: &str, body: Option<&str>, headers: Option<&HashMap<String, String>>) -> Result<Operation, Error>
```

**Benefits:**
- Single, intuitive API surface
- HTTP method determines operation type (GET/POST/PATCH/DELETE)
- Easier to use and document
- Natural fit for REST APIs and WASM bindings

### 2. Schema Support via Headers
Full PostgREST-compatible schema resolution:

1. **Explicit schema in table name** (highest priority): `"auth.users"`
2. **Header-based**:
   - `Accept-Profile` for reads (GET)
   - `Content-Profile` for writes (POST/PATCH/DELETE)
3. **Default**: `"public"` schema

**Benefits:**
- PostgREST compatibility
- Multi-tenancy support
- Explicit schema control when needed
- Follows PostgreSQL best practices

### 3. Schema-Qualified SQL Output
All generated SQL uses schema-qualified table names:

```sql
SELECT * FROM "public"."users" WHERE ...
INSERT INTO "auth"."users" (...) VALUES ...
UPDATE "myschema"."users" SET ...
DELETE FROM "analytics"."events" WHERE ...
```

**Benefits:**
- Explicit and unambiguous
- No reliance on `search_path`
- Better security (prevents schema hijacking)
- Easier to debug and audit

### 4. Single SQL Generation Function
Unified `to_sql()` function instead of separate `select_to_sql()`, `insert_to_sql()`, etc.:

```rust
pub fn to_sql(table: &str, operation: &Operation) -> Result<QueryResult, Error>
```

Internal routing based on `Operation` enum variant.

---

**End of Plan**
