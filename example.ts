/**
 * Example usage of postgrest-parser from TypeScript/JavaScript
 *
 * This demonstrates how to use the WASM bindings to parse PostgREST queries
 * and generate PostgreSQL SQL.
 */

import init, { parseQueryString, parseOnly, buildFilterClause } from 'postgrest-parser';

async function main() {
  // Initialize the WASM module (required once at startup)
  await init();

  console.log('=== PostgREST Parser Examples ===\n');

  // Example 1: Simple query with filters
  console.log('Example 1: Simple Filtering');
  const result1 = parseQueryString("users", "age=gte.18&status=eq.active");
  console.log('Query:', result1.query);
  console.log('Params:', result1.params);
  console.log('Tables:', result1.tables);
  console.log();

  // Example 2: Complex query with select, filters, order, and pagination
  console.log('Example 2: Complex Query');
  const result2 = parseQueryString(
    "users",
    "select=id,name,email&age=gte.18&status=in.(active,pending)&order=created_at.desc&limit=10&offset=20"
  );
  console.log('Query:', result2.query);
  console.log('Params:', result2.params);
  console.log('Tables:', result2.tables);
  console.log();

  // Example 3: JSON path navigation
  console.log('Example 3: JSON Path Navigation');
  const result3 = parseQueryString(
    "users",
    "data->name=eq.John&data->>email=like.*@example.com"
  );
  console.log('Query:', result3.query);
  console.log('Params:', result3.params);
  console.log();

  // Example 4: Full-text search
  console.log('Example 4: Full-Text Search');
  const result4 = parseQueryString(
    "articles",
    "content=fts(english).search term&published=is.true"
  );
  console.log('Query:', result4.query);
  console.log('Params:', result4.params);
  console.log();

  // Example 5: Logic operators (AND/OR)
  console.log('Example 5: Logic Operators');
  const result5 = parseQueryString(
    "users",
    "and=(age.gte.18,status.eq.active,or(role.eq.admin,role.eq.moderator))"
  );
  console.log('Query:', result5.query);
  console.log('Params:', result5.params);
  console.log();

  // Example 6: Array operators
  console.log('Example 6: Array Operators');
  const result6 = parseQueryString(
    "posts",
    "tags=cs.{rust}&tags=ov.(elixir,typescript)"
  );
  console.log('Query:', result6.query);
  console.log('Params:', result6.params);
  console.log();

  // Example 7: Type casting
  console.log('Example 7: Type Casting');
  const result7 = parseQueryString(
    "products",
    "price::numeric=gt.100&data->stock::int=gte.10"
  );
  console.log('Query:', result7.query);
  console.log('Params:', result7.params);
  console.log();

  // Example 8: Quantifiers
  console.log('Example 8: Quantifiers');
  const result8 = parseQueryString(
    "posts",
    "tags=eq(any).{rust,elixir,go}"
  );
  console.log('Query:', result8.query);
  console.log('Params:', result8.params);
  console.log();

  // Example 9: Parse only (without SQL generation)
  console.log('Example 9: Parse Only');
  const parsed = parseOnly("age=gte.18&status=eq.active&order=name.asc");
  console.log('Parsed structure:', JSON.stringify(parsed, null, 2));
  console.log();

  // Example 10: Select with nested relations
  console.log('Example 10: Nested Relations');
  const result10 = parseQueryString(
    "users",
    "select=id,name,orders(id,total,items(name,price))"
  );
  console.log('Query:', result10.query);
  console.log('Tables:', result10.tables);
  console.log();

  // Example 11: Negation
  console.log('Example 11: Negation');
  const result11 = parseQueryString(
    "users",
    "status=not.eq.deleted&age=not.lt.18"
  );
  console.log('Query:', result11.query);
  console.log('Params:', result11.params);
  console.log();

  // Example 12: IS operators
  console.log('Example 12: IS Operators');
  const result12 = parseQueryString(
    "users",
    "deleted_at=is.null&active=is.true"
  );
  console.log('Query:', result12.query);
  console.log('Params:', result12.params);
  console.log();

  // Example 13: Use in Node.js with pg library
  console.log('Example 13: Integration with node-postgres');
  const sqlResult = parseQueryString("users", "age=gte.18&status=eq.active&limit=10");
  console.log('Ready for pg.query():');
  console.log(`  client.query('${sqlResult.query}', ${JSON.stringify(sqlResult.params)})`);
  console.log();

  // Example 14: Error handling
  console.log('Example 14: Error Handling');
  try {
    parseQueryString("users", "invalid_operator=xyz.123");
  } catch (error) {
    console.log('Caught error:', error);
  }
}

// Run examples
main().catch(console.error);
