# PostgREST Parser for Rust

[![Crates.io](https://img.shields.io/crates/v/postgrest-parser.svg)](https://crates.io/crates/postgrest-parser)
[![Documentation](https://docs.rs/postgrest-parser/badge.svg)](https://docs.rs/postgrest-parser)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

A high-performance Rust implementation of the PostgREST URL-to-SQL parser, supporting both native and WASM targets.

## Features

- ✅ **Complete PostgREST API**: All 22+ filter operators fully implemented
- ✅ **Logic Operators**: AND, OR, NOT with arbitrary nesting depth
- ✅ **Select Parsing**: Fields, relations, spreads, aliases, JSON paths, type casting
- ✅ **Full-Text Search**: Multiple FTS operators with language support
- ✅ **Array/Range Operations**: PostgreSQL array and range type support
- ✅ **Quantifiers**: `any` and `all` for array comparisons
- ✅ **Order Parsing**: Multi-column ordering with nulls handling
- ✅ **Parameterized SQL**: Safe SQL generation with $1, $2, etc. placeholders
- ✅ **Zero Regex**: Uses nom parser combinators for better performance
- ✅ **Type Safe**: Comprehensive error handling with thiserror
- ✅ **WASM Support**: Full TypeScript/JavaScript bindings for browser and Deno (optional feature)
- ✅ **171 Tests**: Comprehensive test coverage (148 Rust + 23 WASM integration tests)

## Installation

### Rust

Add to your `Cargo.toml`:

```toml
[dependencies]
postgrest-parser = "0.1.0"
```

### TypeScript/JavaScript (WASM)

The parser is available as a WebAssembly module for use in browsers and JavaScript runtimes.

#### Building the WASM Package

```bash
# Install wasm-pack if you haven't already
cargo install wasm-pack

# Build for web (browsers, Deno, etc.)
wasm-pack build --target web --features wasm

# Or for Node.js
wasm-pack build --target nodejs --features wasm

# Development build (faster, larger)
wasm-pack build --dev --target web --features wasm
```

#### Installation in Your Project

After building, the WASM package is available in the `pkg/` directory:

```bash
# Copy to your project
cp -r pkg/ /path/to/your/project/postgrest-parser/

# Or publish to npm (requires package.json configuration)
cd pkg && npm publish
```

#### Usage Examples

##### Basic Usage (Browser/Deno)

```typescript
import init, { parseQueryString } from './postgrest_parser.js';

// Initialize WASM module (call once)
await init();

// Parse a PostgREST query string
const result = parseQueryString(
  "users",
  "age=gte.18&status=in.(active,pending)&order=created_at.desc&limit=10"
);

console.log('SQL:', result.query);
// SELECT * FROM "users" WHERE "age" >= $1 AND "status" = ANY($2) ORDER BY "created_at" DESC LIMIT $3

console.log('Params:', result.params);
// ["18", ["active", "pending"], 10]

console.log('Tables:', result.tables);
// ["users"]
```

##### Parse Only (Without SQL Generation)

```typescript
import init, { parseOnly } from './postgrest_parser.js';

await init();

// Parse query structure without generating SQL
const parsed = parseOnly("select=id,name&age=gte.18&limit=10");

console.log(parsed);
// {
//   select: [{ field: "id" }, { field: "name" }],
//   filters: [...],
//   limit: 10,
//   offset: null,
//   order: []
// }
```

##### Using with Deno

```typescript
// run_parser.ts
import init, { parseQueryString } from "./pkg/postgrest_parser.js";

await init();

const result = parseQueryString(
  "posts",
  "select=id,title,author(name)&status=eq.published&created_at=gte.2024-01-01"
);

console.log("Generated SQL:", result.query);
console.log("Parameters:", result.params);
```

Run with:
```bash
deno run --allow-read run_parser.ts
```

##### Real-World Example: API Endpoint

```typescript
// api/posts.ts
import init, { parseQueryString } from '../postgrest_parser.js';

// Initialize once at startup
await init();

export async function handlePostsRequest(request: Request) {
  const url = new URL(request.url);
  const queryString = url.search.slice(1); // Remove leading '?'

  try {
    const result = parseQueryString("posts", queryString);

    // Execute query with your database client
    const rows = await db.query(result.query, result.params);

    return new Response(JSON.stringify(rows), {
      headers: { 'Content-Type': 'application/json' }
    });
  } catch (error) {
    return new Response(
      JSON.stringify({ error: error.message }),
      { status: 400 }
    );
  }
}
```

##### JSON Serialization

```typescript
const result = parseQueryString("users", "age=gte.18");

// Convert to plain JSON object
const json = result.toJSON();
console.log(JSON.stringify(json, null, 2));
// {
//   "query": "SELECT * FROM \"users\" WHERE \"age\" >= $1",
//   "params": ["18"],
//   "tables": ["users"]
// }
```

#### WASM API Reference

##### `parseQueryString(table: string, queryString: string): WasmQueryResult`

Parses a PostgREST query string and generates SQL.

- **Parameters:**
  - `table` - Table name to query
  - `queryString` - PostgREST query string (e.g., `"select=id,name&age=gte.18"`)
- **Returns:** `WasmQueryResult` with:
  - `query` - Generated PostgreSQL SELECT statement
  - `params` - Array of parameter values
  - `tables` - Array of table names referenced
- **Throws:** Error if parsing or SQL generation fails

##### `parseOnly(queryString: string): ParsedParams`

Parses query string without generating SQL.

- **Parameters:**
  - `queryString` - PostgREST query string
- **Returns:** Parsed parameters object
- **Throws:** Error if parsing fails

##### `WasmQueryResult`

Properties:
- `query: string` - The SQL query
- `params: any[]` - Query parameters
- `tables: string[]` - Referenced tables
- `toJSON(): object` - Serialize to JSON

#### Running Tests

We provide comprehensive integration tests using Deno:

```bash
# Install Deno
curl -fsSL https://deno.land/install.sh | sh

# Build WASM
wasm-pack build --target web --features wasm

# Run integration tests
deno test --allow-read tests/integration/wasm_test.ts

# Or use the Deno task
deno task test:wasm
```

See [tests/integration/README.md](tests/integration/README.md) for detailed test documentation.

#### Performance

WASM performance benchmarks (from integration tests):

- **Average parse time:** ~0.01ms per query
- **100 queries:** ~1ms total
- **Throughput:** ~100,000 queries/second in browser

The WASM build maintains near-native performance while running in JavaScript environments.

#### Browser Compatibility

Tested and working in:
- ✅ Chrome 90+
- ✅ Firefox 89+
- ✅ Safari 15+
- ✅ Edge 90+
- ✅ Deno 1.x+
- ✅ Node.js 16+ (with `--target nodejs`)

#### Troubleshooting

**Module not found:**
```typescript
// Make sure to use the correct path to pkg/
import init from './pkg/postgrest_parser.js';  // ✅
import init from './postgrest_parser.js';      // ❌
```

**WASM initialization:**
```typescript
// Always call init() before using other functions
await init();  // ✅
parseQueryString(...);  // ✅

parseQueryString(...);  // ❌ Will fail - init() not called
```

**Type errors in TypeScript:**
```typescript
// Use generated .d.ts files
import init, { parseQueryString } from './pkg/postgrest_parser.js';
// Type definitions are in ./pkg/postgrest_parser.d.ts
```

## Usage

### Rust: Basic Query Parsing

```rust
use postgrest_parser::*;

// Parse a query string
let params = parse_query_string("select=id,name&id=eq.1&order=id.desc&limit=10")?;
assert!(params.has_select());
assert!(params.has_filters());

// Generate SQL
let result = to_sql("users", &params)?;
println!("Query: {}", result.query);
println!("Params: {:?}", result.params);
// Output:
// Query: SELECT "id", "name" FROM "users" WHERE "id" = $1 ORDER BY "id" DESC LIMIT $2
// Params: [String("1"), Number(10)]
```

### Filter Operators

#### Comparison Operators

```rust
// Equality
let params = parse_query_string("id=eq.1")?;                  // WHERE "id" = $1
let params = parse_query_string("status=neq.deleted")?;       // WHERE "status" <> $1

// Comparison
let params = parse_query_string("age=gt.18")?;                // WHERE "age" > $1
let params = parse_query_string("age=gte.18")?;               // WHERE "age" >= $1
let params = parse_query_string("age=lt.65")?;                // WHERE "age" < $1
let params = parse_query_string("age=lte.65")?;               // WHERE "age" <= $1

// Negation works with all operators
let params = parse_query_string("age=not.gt.18")?;            // WHERE "age" <= $1
```

#### Pattern Matching

```rust
// SQL LIKE operators
let params = parse_query_string("name=like.*Smith%")?;        // WHERE "name" LIKE $1
let params = parse_query_string("name=ilike.*smith%")?;       // WHERE "name" ILIKE $1 (case-insensitive)

// POSIX regex
let params = parse_query_string("name=match.^John")?;         // WHERE "name" ~ $1
let params = parse_query_string("name=imatch.^john")?;        // WHERE "name" ~* $1 (case-insensitive)
```

#### List and Array Operators

```rust
// IN operator
let params = parse_query_string("status=in.(active,pending)")?;  // WHERE "status" = ANY($1)

// Array contains
let params = parse_query_string("tags=cs.{rust}")?;              // WHERE "tags" @> $1
let params = parse_query_string("tags=cd.{rust,elixir}")?;       // WHERE "tags" <@ $1

// Array overlap
let params = parse_query_string("tags=ov.(rust,elixir)")?;       // WHERE "tags" && $1
```

#### Full-Text Search

```rust
// Basic FTS (uses plainto_tsquery)
let params = parse_query_string("content=fts.search term")?;
// WHERE to_tsvector('english', "content") @@ plainto_tsquery('english', $1)

// With custom language
let params = parse_query_string("content=fts(french).terme")?;
// WHERE to_tsvector('french', "content") @@ plainto_tsquery('french', $1)

// Phrase search
let params = parse_query_string("content=phfts.exact phrase")?;
// WHERE to_tsvector('english', "content") @@ phraseto_tsquery('english', $1)

// Websearch (most lenient)
let params = parse_query_string("content=wfts.search query")?;
// WHERE to_tsvector('english', "content") @@ websearch_to_tsquery('english', $1)
```

#### Range Operators (PostgreSQL ranges)

```rust
let params = parse_query_string("range=sl.[1,10)")?;             // WHERE "range" << $1 (strictly left)
let params = parse_query_string("range=sr.[1,10)")?;             // WHERE "range" >> $1 (strictly right)
let params = parse_query_string("range=nxl.[1,10)")?;            // WHERE "range" &< $1
let params = parse_query_string("range=nxr.[1,10)")?;            // WHERE "range" &> $1
let params = parse_query_string("range=adj.[1,10)")?;            // WHERE "range" -|- $1 (adjacent)
```

#### Special Operators

```rust
// IS operator
let params = parse_query_string("deleted_at=is.null")?;          // WHERE "deleted_at" IS NULL
let params = parse_query_string("deleted_at=is.not_null")?;      // WHERE "deleted_at" IS NOT NULL
let params = parse_query_string("active=is.true")?;              // WHERE "active" IS TRUE
let params = parse_query_string("active=is.false")?;             // WHERE "active" IS FALSE
```

#### Quantifiers

```rust
// ANY quantifier
let params = parse_query_string("tags=eq(any).{rust,elixir}")?;  // WHERE "tags" = ANY($1)

// ALL quantifier
let params = parse_query_string("tags=eq(all).{rust}")?;         // WHERE "tags" = ALL($1)
```

#### JSON Path Navigation

```rust
// Arrow operator (returns JSON)
let params = parse_query_string("data->name=eq.test")?;
// WHERE "data"->'name' = $1

// Double arrow operator (returns text)
let params = parse_query_string("data->>email=like.*@example.com")?;
// WHERE "data"->>'email' LIKE $1

// Nested paths
let params = parse_query_string("data->user->name=eq.John")?;
// WHERE "data"->'user'->'name' = $1
```

#### Type Casting

```rust
let params = parse_query_string("price::numeric=gt.100")?;
// WHERE "price"::numeric > $1

let params = parse_query_string("data->age::int=gte.18")?;
// WHERE ("data"->'age')::int >= $1
```

### Logic Trees

```rust
// AND conditions
let params = parse_query_string("and=(id.eq.1,name.eq.john)")?;

// OR conditions
let params = parse_query_string("or=(status.eq.pending,status.eq.processing)")?;

// Nested logic
let params = parse_query_string("and=(id.eq.1,or(status.eq.active,status.eq.pending))")?;

// Negated logic
let params = parse_query_string("not.and=(id.eq.1,name.eq.john)")?;
```

### Select with Relations

```rust
let params = parse_query_string("select=id,client(id,name),posts(title)")?;
assert!(params.select.is_some());
```

### Ordering

```rust
// Single column
let params = parse_query_string("order=id.desc")?;

// Multiple columns
let params = parse_query_string("order=id.desc,name.asc")?;

// With nulls handling
let params = parse_query_string("order=id.desc.nullslast")?;
```

## Development

### Building

```bash
# Native
cargo build --release

# WASM
cargo build --release --target wasm32-unknown-unknown
wasm-pack build --target web
```

### Testing

```bash
# Run all tests
cargo test

# Run specific test
cargo test test_parse_query_string

# Run with output
cargo test -- --nocapture
```

### Benchmarks

```bash
# Run all benchmarks
cargo bench

# Run specific benchmark group
cargo bench simple_parsing
cargo bench realistic_workloads

# See BENCHMARKS.md for detailed results and analysis
```

#### Latest Benchmark Results

Benchmarked on Darwin 24.6.0 (macOS) with release optimizations.

**Simple Parsing Performance:**

| Operation | Time (median) | Throughput |
|-----------|---------------|------------|
| `select=id,name,email` | 1.14 µs | 880K ops/s |
| `age=gte.18` | 781 ns | 1.28M ops/s |
| `order=created_at.desc` | 1.27 µs | 790K ops/s |
| `limit=10&offset=20` | 520 ns | 1.92M ops/s |

**Realistic Workload Performance:**

| Workload | Time (median) | Throughput | Description |
|----------|---------------|------------|-------------|
| User Search | 7.19 µs | 139K ops/s | SELECT + 2 filters + ORDER + LIMIT |
| Paginated List | 6.84 µs | 146K ops/s | SELECT + relation + filter + ORDER + pagination |
| Filtered Report | 9.92 µs | 101K ops/s | SELECT + relation + 4 filters + ORDER |
| Complex Search | 10.66 µs | 94K ops/s | SELECT + FTS + array ops + 3 filters + ORDER |
| Dashboard Aggregation | 7.84 µs | 128K ops/s | Complex logic tree + date range + ORDER |

**Operator Performance:**

| Operator Category | Example | Time (median) |
|-------------------|---------|---------------|
| Comparison (`eq`, `gte`) | `id=eq.1` | ~750-783 ns |
| Pattern Match | `name=like.*Smith*` | ~783-803 ns |
| List Operations | `status=in.(a,b,c)` | ~1.07 µs |
| Full-Text Search | `content=fts.term` | ~941 ns |
| FTS with Language | `content=fts(french).terme` | ~974 ns |
| Array Operations | `tags=cs.{rust,elixir}` | ~787-1.06 µs |
| Range Operations | `range=sl.[1,10)` | ~1.01 µs |
| JSON Path | `data->name=eq.test` | ~934-1.13 µs |
| Type Casting | `price::numeric=gt.100` | ~1.10 µs |
| Quantifiers | `tags=eq(any).{a,b}` | ~907-1.04 µs |

**SQL Generation (End-to-End):**

| Query Type | Time (median) | Throughput |
|------------|---------------|------------|
| Simple SELECT | 2.21 µs | 452K ops/s |
| With Filters | 3.03 µs | 330K ops/s |
| With ORDER | 3.75 µs | 267K ops/s |
| Complex Query | 7.57 µs | 132K ops/s |

**Query Scaling:**

- 1 filter: 1.97 µs
- 3 filters: 4.09 µs (2.1x)
- 5 filters: 6.79 µs (3.4x)
- 10 filters: 13.40 µs (6.8x)

**Performance vs Reference Implementation:**
- Simple queries: **1.3-6x faster**
- Complex queries: **1.3x faster**
- All operations under 15 µs

See [BENCHMARKS.md](BENCHMARKS.md) for complete performance analysis.

## Complete Operator Reference

| Operator | PostgREST | SQL | Example |
|----------|-----------|-----|---------|
| `eq` | Equal | `=` | `id=eq.1` |
| `neq` | Not equal | `<>` | `status=neq.deleted` |
| `gt` | Greater than | `>` | `age=gt.18` |
| `gte` | Greater than or equal | `>=` | `age=gte.18` |
| `lt` | Less than | `<` | `age=lt.65` |
| `lte` | Less than or equal | `<=` | `age=lte.65` |
| `like` | LIKE pattern | `LIKE` | `name=like.*Smith*` |
| `ilike` | Case-insensitive LIKE | `ILIKE` | `name=ilike.*smith*` |
| `match` | POSIX regex | `~` | `name=match.^John` |
| `imatch` | Case-insensitive regex | `~*` | `name=imatch.^john` |
| `in` | In list | `= ANY($1)` | `status=in.(active,pending)` |
| `is` | IS check | `IS` | `deleted=is.null` |
| `fts` | Full-text search | `@@` | `content=fts.search` |
| `plfts` | Plain FTS | `@@` | `content=plfts.search` |
| `phfts` | Phrase FTS | `@@` | `content=phfts.exact phrase` |
| `wfts` | Websearch FTS | `@@` | `content=wfts.query` |
| `cs` | Contains | `@>` | `tags=cs.{rust}` |
| `cd` | Contained in | `<@` | `tags=cd.{rust,elixir}` |
| `ov` | Overlaps | `&&` | `tags=ov.(rust,elixir)` |
| `sl` | Strictly left | `<<` | `range=sl.[1,10)` |
| `sr` | Strictly right | `>>` | `range=sr.[1,10)` |
| `nxl` | Not extends right | `&<` | `range=nxl.[1,10)` |
| `nxr` | Not extends left | `&>` | `range=nxr.[1,10)` |
| `adj` | Adjacent | `-|-` | `range=adj.[1,10)` |

## Performance

- **Zero-copy parsing** where possible with nom combinators
- **No regex usage** - all parsing done with efficient pattern matching
- **148 passing tests** with comprehensive coverage
- **Optimized for Rust** - leverages Rust's zero-cost abstractions

## Architecture

- **AST** ([src/ast/](src/ast/)): Typed intermediate representation of parsed queries
- **Parser** ([src/parser/](src/parser/)): nom-based combinator parsers (no regex)
  - `common.rs` - Shared parsing utilities (identifiers, lists, JSON paths)
  - `filter.rs` - Filter/operator parsing
  - `logic.rs` - Logic tree parsing (AND/OR/NOT)
  - `order.rs` - ORDER BY clause parsing
  - `select.rs` - SELECT clause parsing with relations
- **SQL Builder** ([src/sql/](src/sql/)): Parameterized PostgreSQL SQL generation
- **Error Handling** ([src/error/](src/error/)): Typed errors using thiserror

## Development

### Building

```bash
# Native
cargo build --release

# With all features
cargo build --release --features full

# WASM (if you need browser support)
cargo build --release --target wasm32-unknown-unknown --features wasm
wasm-pack build --target web
```

### Testing

```bash
# Run all Rust tests (148 tests)
cargo test

# Run specific test
cargo test test_parse_query_string

# Run with output
cargo test -- --nocapture

# Check code quality
cargo clippy -- -D warnings

# Run WASM integration tests (23 tests)
# Requires Deno: https://deno.land
wasm-pack build --target web --features wasm
deno test --allow-read tests/integration/wasm_test.ts

# Or use Deno task
deno task test:wasm
```

### Code Quality

```bash
# Format code
cargo fmt

# Run linter
cargo clippy

# Check for security vulnerabilities
cargo audit
```

## Roadmap

- [x] Complete PostgREST filter operator support (22+ operators)
- [x] Logic trees with arbitrary nesting
- [x] Full-text search with language support
- [x] Array and range operators
- [x] Quantifiers (any/all)
- [x] Comprehensive test coverage (148 tests)
- [x] WASM bindings for TypeScript/JavaScript with Deno integration tests
- [x] Benchmark suite comparing to reference implementation
- [ ] Count aggregation support
- [ ] `on_conflict` parameter support
- [ ] Relation column filtering

## Contributing

Contributions are welcome! Areas of interest:

- Additional test cases and edge cases
- Performance optimizations
- WASM/JavaScript bindings
- Documentation improvements
- Bug reports and fixes

## License

MIT

## Acknowledgments

- Inspired by the [PostgREST](https://postgrest.org/) project
- Parser built with [nom](https://github.com/rust-bakery/nom)
- Reference implementation: [postgrest_parser (Elixir)](https://github.com/supabase/postgrest_parser)
