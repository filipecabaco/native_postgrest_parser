# PostgREST Parser - Known Limitations

## ⚠️ Critical: Resource Embedding (Relations) Not Fully Implemented

### The Problem

Our parser **parses** relation syntax like `users(id,name,posts(title))` but **does NOT generate valid SQL** for it.

### What We Generate (INVALID)

```rust
// Query: select=id,name,posts(title,content)
let result = query_string_to_sql("users", "select=id,name,posts(title,content)").unwrap();
```

Generates:
```sql
SELECT "id", "name",
  (SELECT json_agg(row_to_json(posts."title", posts."content")) AS posts FROM posts)
FROM "users"
```

**This SQL is broken** because:
- ❌ No JOIN condition between `users` and `posts`
- ❌ Returns ALL posts for EVERY user (Cartesian product)
- ❌ No foreign key relationship resolution

### What PostgREST Actually Does

PostgREST requires **database schema introspection** to:

1. **Inspect `information_schema`** to find foreign key relationships
2. **Resolve relationship direction** (one-to-many, many-to-one, many-to-many)
3. **Generate proper JOINs** with foreign key conditions

Real PostgREST generates:
```sql
SELECT "id", "name",
  COALESCE(
    (SELECT json_agg(row_to_json(posts_1))
     FROM (
       SELECT "title", "content"
       FROM "posts"
       WHERE "posts"."user_id" = "users"."id"  -- Foreign key condition!
     ) posts_1
    ), '[]'
  ) AS "posts"
FROM "users"
```

See [PostgREST source](https://github.com/PostgREST/postgrest/blob/main/src/PostgREST/SchemaCache.hs) for schema introspection implementation.

## Why This Library Doesn't Do Schema Introspection

This is a **parser library** that:
- ✅ Parses PostgREST query strings
- ✅ Generates parameterized SQL for filters, ordering, pagination
- ✅ Handles mutations (INSERT/UPDATE/DELETE) safely
- ✅ Supports RPC function calls
- ❌ Does NOT connect to databases
- ❌ Does NOT inspect schemas
- ❌ Does NOT resolve foreign key relationships

### Design Decision

This library is designed for scenarios where:
1. You want to parse PostgREST syntax **without** database access
2. You'll handle relation resolution in your own application layer
3. You need a lightweight parser for validation/transformation

## What Works vs What Doesn't

### ✅ Fully Working Features

#### All Filter Operators (22+)
```rust
query_string_to_sql("users", "age=gte.18&status=in.(active,pending)")
// ✅ Valid SQL with proper WHERE clause
```

#### Mutations with Safety
```rust
parse("PATCH", "users", "id=eq.123", Some(r#"{"status":"active"}"#), None)
// ✅ Generates: UPDATE "users" SET "status" = $1 WHERE "id" = $2
```

#### RPC Function Calls
```rust
parse("POST", "rpc/calculate_total", "", Some(r#"{"order_id":123}"#), None)
// ✅ Generates: SELECT * FROM "public"."calculate_total"("order_id" := $1)
```

#### Complex Logic
```rust
query_string_to_sql("posts", "and=(status.eq.published,or(featured.is.true,views.gt.1000))")
// ✅ Valid SQL with nested AND/OR conditions
```

### ⚠️ Partially Working (Syntax Only)

#### Resource Embedding
```rust
parse_query_string("select=id,name,posts(title)")
// ✅ Parses successfully
// ⚠️ AST is correct
// ❌ Generated SQL is invalid (missing JOIN conditions)
```

**Status**: Syntax parsing works, SQL generation is incomplete.

**Workaround**: Parse to AST only, then handle relations in your app:
```rust
use postgrest_parser::parse_query_string;

let params = parse_query_string("select=id,name,posts(title)").unwrap();
// Inspect params.select to find relations
// Build your own JOINs using your schema knowledge
```

### ❌ Not Supported

#### Schema-Based Validation
- No validation that tables/columns exist
- No type checking against database schema
- No constraint validation

#### Automatic JOIN Generation
- No foreign key discovery
- No relationship resolution
- No LATERAL join generation

#### View Expansion
- No view materialization
- No recursive CTEs for relations

## Comparison with PostgREST

| Feature | PostgREST | This Library |
|---------|-----------|--------------|
| Filter operators (22+) | ✅ | ✅ |
| Logic operators (AND/OR/NOT) | ✅ | ✅ |
| JSON operators | ✅ | ✅ |
| Full-text search | ✅ | ✅ |
| Ordering & pagination | ✅ | ✅ |
| INSERT/UPDATE/DELETE | ✅ | ✅ |
| ON CONFLICT (upsert) | ✅ | ✅ |
| RPC function calls | ✅ | ✅ |
| Prefer headers | ✅ | ✅ (parsed) |
| **Resource embedding** | ✅ | ⚠️ Syntax only |
| **Schema introspection** | ✅ Required | ❌ Not implemented |
| **Foreign key resolution** | ✅ | ❌ |
| **View expansion** | ✅ | ❌ |
| **Computed columns** | ✅ | ❌ |

## Recommended Use Cases

### ✅ Good Fit

1. **Query validation** - Validate PostgREST queries before sending to API
2. **Query transformation** - Parse, modify, and regenerate queries
3. **Static analysis** - Analyze query patterns without database
4. **Client libraries** - Parse PostgREST responses
5. **Simple CRUD** - Single-table queries work perfectly
6. **Serverless/Edge** - No database connection needed

### ❌ Not a Good Fit

1. **Full PostgREST replacement** - Use actual PostgREST
2. **Automatic relation resolution** - Requires schema access
3. **Complex multi-table JOINs** - Need foreign key metadata
4. **Schema validation** - No database introspection

## Future Enhancements

To support relations, this library would need:

1. **Schema Provider Interface**
   ```rust
   trait SchemaProvider {
       fn get_foreign_keys(&self, table: &str) -> Vec<ForeignKey>;
       fn get_relationships(&self, table: &str) -> Vec<Relationship>;
   }
   ```

2. **Optional Schema Introspection**
   - Connect to database
   - Cache `information_schema` data
   - Resolve foreign keys at build time

3. **JOIN Generation**
   - Detect relation types (1:N, N:1, N:M)
   - Generate LATERAL joins
   - Handle junction tables

This would require:
- Database connection capability
- Schema caching
- Complex JOIN logic
- Breaking change to API (schema parameter required)

## Workarounds

### Option 1: Use PostgREST Directly

For production use with relations, use [PostgREST](https://postgrest.org/) itself.

### Option 2: Custom Relation Handling

```rust
use postgrest_parser::{parse_query_string, SelectItem, ItemType};

let params = parse_query_string("select=id,name,posts(title)").unwrap();

for item in params.select.unwrap() {
    match item.item_type {
        ItemType::Relation => {
            // Handle relation yourself:
            // 1. Detect foreign key from your schema
            // 2. Generate proper JOIN
            // 3. Build subquery with WHERE condition
            println!("Need to resolve relation: {}", item.name);
        }
        ItemType::Field => {
            // Regular field - works fine
        }
        _ => {}
    }
}
```

### Option 3: Schema-First Approach

```rust
// Define your schema statically
struct Schema {
    users_to_posts: ForeignKey {
        from: "users.id",
        to: "posts.user_id"
    }
}

// Use schema to build correct SQL
fn build_with_schema(query: &str, schema: &Schema) -> String {
    // Your custom logic here
}
```

## Conclusion

This library provides **90% of PostgREST functionality** for single-table operations and mutations. For multi-table relations, you need either:

1. PostgREST itself (recommended for production)
2. Custom relation handling with your schema knowledge
3. Contribute schema introspection to this library 🙏

The parser correctly handles all PostgREST syntax - it just doesn't have the schema metadata to generate valid multi-table SQL.
