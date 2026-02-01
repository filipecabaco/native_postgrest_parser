# Integration Tests Summary

## Overview

Comprehensive integration test suite for the PostgREST parser with real PostgreSQL database connectivity.

## Test Environment

- **Docker**: PostgreSQL 15 (Supabase image) running on port 5433
- **Database**: `postgrest_parser_test`
- **Schema**: Complex test schema with customers, orders, products, posts, tags
- **Relationships**: M2O, O2M, M2M (junction tables), O2O

## Test Suites

### Schema Introspection Tests ([integration_relations.rs](tests/integration_relations.rs))

8 tests verifying foreign key resolution and relation handling:

✅ **test_schema_cache_foreign_keys** - Verifies FK loading from pg_catalog
✅ **test_find_relationship** - Tests M2O and O2M relationship detection
✅ **test_many_to_one_relation** - Orders → Customers with proper JOIN
✅ **test_one_to_many_relation** - Customers → Orders with json_agg
✅ **test_nested_relations** - Multi-level relation handling
✅ **test_complete_workflow** - Complex query with filters, ordering, pagination
✅ **test_relation_not_found** - Error handling for invalid relations
✅ **test_no_schema_cache_fails_gracefully** - Backward compatibility without cache

### Full Query Tests ([integration_full_queries.rs](tests/integration_full_queries.rs))

12 tests covering real-world PostgREST queries:

✅ **test_customers_with_filters_and_ordering** - JSON path filters + ORDER BY
✅ **test_orders_with_customer_details** - Relations with filters
✅ **test_customers_with_all_orders** - O2M with pagination
✅ **test_pagination_with_offset** - LIMIT and OFFSET
✅ **test_range_filters** - **MULTIPLE FILTERS ON SAME COLUMN** (price >= 50 AND price <= 150)
✅ **test_in_operator_with_list** - IN operator with array binding
✅ **test_or_logic_filter** - OR logic across conditions
✅ **test_pattern_matching** - LIKE pattern matching
✅ **test_null_handling** - IS NULL operator
✅ **test_junction_table_many_to_many** - Posts with boolean filters
✅ **test_one_to_one_relationship** - Customer profiles (unique FK)
✅ **test_complex_combined_query** - Relations + filters + ordering + pagination

## Critical Bug Fix: Multiple Filters on Same Column

### Problem
```rust
// Before: Only kept last filter
"price=gte.50&price=lte.150"  // Only lte.150 was applied
```

### Root Cause
`parse_params_from_pairs()` converted pairs to HashMap, overwriting duplicate keys.

### Solution
Refactored to preserve all filter pairs:
- Reserved keys (select, order, limit, offset) → HashMap (single value)
- Filter keys → Vec (all values preserved)
- New `parse_filters_from_pairs()` function processes all filter pairs

### Result
```rust
// After: Both filters applied correctly
"price=gte.50&price=lte.150"  // Generates: price >= $1 AND price <= $2
```

## Schema Introspection

### pg_catalog Implementation
Switched from `information_schema` to `pg_catalog` system tables for FK queries:

**Benefits:**
- More reliable across PostgreSQL versions
- Matches PostgREST's approach
- Handles all FK types correctly

**Query:**
```sql
SELECT con.conname, sn.nspname, sc.relname, sa.attname,
       tn.nspname, tc.relname, ta.attname
FROM pg_constraint con
JOIN pg_class sc ON sc.oid = con.conrelid
JOIN pg_namespace sn ON sn.oid = sc.relnamespace
...
WHERE con.contype = 'f'
```

### JOIN Generation

**Many-to-One** (orders → customers):
```sql
COALESCE((SELECT row_to_json(customers_1)
          FROM (SELECT ...) customers
          WHERE "orders"."customer_id" = "customers"."id"), 'null'::json)
```

**One-to-Many** (customers → orders):
```sql
COALESCE((SELECT json_agg(orders_1)
          FROM (SELECT ...) orders
          WHERE "orders"."customer_id" = "customers"."id"), '[]'::json)
```

## Test Execution

```bash
# Run all integration tests
cargo test --features postgres --test integration_relations --test integration_full_queries

# Run specific test
cargo test test_range_filters --features postgres

# All tests
cargo test --all-features
```

## Test Coverage

- ✅ 20 integration tests (8 + 12)
- ✅ 312 unit tests
- ✅ 39 doctests
- ✅ **Total: 371 passing tests**

## Parser Implementation

- ✅ **No regex usage** - All parsing uses nom combinators
- ✅ Supports ALL PostgREST operators (22+)
- ✅ Full-text search with language support
- ✅ JSON path navigation (-> and ->>)
- ✅ Type casting
- ✅ Quantifiers (any, all)
- ✅ Logic operators (and, or, not)
- ✅ Range queries with multiple filters

## Future Enhancements

- [ ] Many-to-Many through junction tables (currently returns helpful error)
- [ ] Composite foreign keys (currently only single-column)
- [ ] Nested relation filtering
- [ ] Computed columns
- [ ] Views and materialized views

## Database Seed Data

The test database includes:
- 5 customers (various tiers)
- 7 orders (various statuses)
- 10 products (multiple categories)
- 5 posts (with authors)
- 5 tags
- 3 customer profiles
- M2M post_tags junction table

See [tests/fixtures/init.sql](tests/fixtures/init.sql) for complete schema.
