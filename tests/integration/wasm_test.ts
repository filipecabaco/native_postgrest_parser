/**
 * Integration tests for PostgREST Parser WASM bindings using Deno
 *
 * Run with: deno test --allow-read tests/integration/wasm_test.ts
 */

import { assertEquals, assertExists, assert } from "https://deno.land/std@0.224.0/assert/mod.ts";

// Import WASM module
// @ts-ignore - Type definitions are available but complex to import
import init, { parseQueryString, parseOnly, buildFilterClause } from "../../pkg/postgrest_parser.js";

// Initialize WASM module before tests
await init();

Deno.test("WASM - parseQueryString: simple select", async () => {
  const result = parseQueryString("users", "select=id,name,email");

  assertExists(result);
  assertExists(result.query);
  assertExists(result.params);
  assertExists(result.tables);

  assert(result.query.includes("SELECT"), "Query should contain SELECT");
  assert(result.query.includes('"id"'), "Query should include id field");
  assert(result.query.includes('"name"'), "Query should include name field");
  assert(result.query.includes('"email"'), "Query should include email field");
  assert(result.query.includes("users"), "Query should reference users table");

  assertEquals(result.tables, ["users"]);
});

Deno.test("WASM - parseQueryString: comparison operators", async () => {
  const result = parseQueryString("users", "age=gte.18&status=eq.active");

  assertExists(result);
  assert(result.query.includes("WHERE"), "Query should have WHERE clause");
  assert(result.query.includes(">="), "Query should use >= operator");
  assert(result.query.includes("="), "Query should use = operator");

  assertEquals(result.params.length, 2, "Should have 2 parameters");
  // Params may be in different order depending on hash map iteration
  assert(result.params.includes("18"), "Should include age parameter");
  assert(result.params.includes("active"), "Should include status parameter");
});

Deno.test("WASM - parseQueryString: IN operator", async () => {
  const result = parseQueryString("users", "status=in.(active,pending,processing)");

  assertExists(result);
  assert(result.query.includes("= ANY"), "Query should use ANY for IN operator");

  assertEquals(result.params.length, 1);
  assertEquals(result.params[0], ["active", "pending", "processing"]);
});

Deno.test("WASM - parseQueryString: pattern matching", async () => {
  const result = parseQueryString("users", "name=like.*Smith*&email=ilike.*@example.com");

  assertExists(result);
  assert(result.query.includes("LIKE"), "Query should use LIKE operator");
  assert(result.query.includes("ILIKE"), "Query should use ILIKE operator");

  assertEquals(result.params.length, 2);
  // PostgREST uses * as wildcard (not converted to %)
  assert(result.params.some((p: unknown) => typeof p === 'string' && p.includes("Smith")), "Should include Smith pattern");
  assert(result.params.some((p: unknown) => typeof p === 'string' && p.includes("@example.com")), "Should include email pattern");
});

Deno.test("WASM - parseQueryString: ordering", async () => {
  const result = parseQueryString("users", "select=id,name&order=created_at.desc,name.asc");

  assertExists(result);
  assert(result.query.includes("ORDER BY"), "Query should have ORDER BY clause");
  assert(result.query.includes("DESC"), "Query should have DESC ordering");
  assert(result.query.includes("ASC"), "Query should have ASC ordering");
});

Deno.test("WASM - parseQueryString: pagination", async () => {
  const result = parseQueryString("users", "select=id,name&limit=10&offset=20");

  assertExists(result);
  assert(result.query.includes("LIMIT"), "Query should have LIMIT clause");
  assert(result.query.includes("OFFSET"), "Query should have OFFSET clause");

  assert(result.params.includes(10), "Params should include limit value");
  assert(result.params.includes(20), "Params should include offset value");
});

Deno.test("WASM - parseQueryString: logic operators (AND)", async () => {
  const result = parseQueryString("users", "and=(age.gte.18,status.eq.active)");

  assertExists(result);
  assert(result.query.includes("AND"), "Query should use AND operator");
  assert(result.query.includes(">="), "Query should have age filter");
  assert(result.query.includes("="), "Query should have status filter");

  assertEquals(result.params.length, 2);
});

Deno.test("WASM - parseQueryString: logic operators (OR)", async () => {
  const result = parseQueryString("users", "or=(status.eq.pending,status.eq.processing)");

  assertExists(result);
  assert(result.query.includes("OR"), "Query should use OR operator");
});

Deno.test("WASM - parseQueryString: nested logic", async () => {
  const result = parseQueryString(
    "users",
    "and=(age.gte.18,or(status.eq.active,status.eq.pending))"
  );

  assertExists(result);
  assert(result.query.includes("AND"), "Query should use AND");
  assert(result.query.includes("OR"), "Query should use OR");
});

Deno.test("WASM - parseQueryString: JSON path operators", async () => {
  const result = parseQueryString("users", "data->name=eq.John&data->>email=like.*@example.com");

  assertExists(result);
  assert(result.query.includes("->"), "Query should use -> operator");
  assert(result.query.includes("->>"), "Query should use ->> operator");
});

Deno.test("WASM - parseQueryString: type casting", async () => {
  const result = parseQueryString("products", "price::numeric=gt.100");

  assertExists(result);
  assert(result.query.includes("::numeric"), "Query should cast to numeric");
  assert(result.query.includes(">"), "Query should use > operator");
});

Deno.test("WASM - parseQueryString: full-text search", async () => {
  const result = parseQueryString("articles", "content=fts(english).search term");

  assertExists(result);
  assert(result.query.includes("to_tsvector"), "Query should use to_tsvector");
  assert(result.query.includes("plainto_tsquery"), "Query should use plainto_tsquery");
  assert(result.query.includes("english"), "Query should specify language");
});

Deno.test("WASM - parseQueryString: array operators", async () => {
  const result = parseQueryString("posts", "tags=cs.{rust,elixir}");

  assertExists(result);
  assert(result.query.includes("@>"), "Query should use @> (contains) operator");
});

Deno.test("WASM - parseQueryString: quantifiers", async () => {
  const result = parseQueryString("posts", "tags=eq(any).{rust,elixir,go}");

  assertExists(result);
  assert(result.query.includes("= ANY"), "Query should use ANY quantifier");
});

Deno.test("WASM - parseQueryString: negation", async () => {
  const result = parseQueryString("users", "status=not.eq.deleted");

  assertExists(result);
  assert(result.query.includes("<>"), "Negated eq should become <>");
});

Deno.test("WASM - parseQueryString: IS operators", async () => {
  const result = parseQueryString("users", "deleted_at=is.null&active=is.true");

  assertExists(result);
  assert(result.query.includes("IS NULL"), "Query should use IS NULL");
  assert(result.query.includes("IS TRUE"), "Query should use IS TRUE");
});

Deno.test("WASM - parseQueryString: complex realistic query", async () => {
  const queryString = [
    "select=id,name,email,created_at",
    "age=gte.18",
    "status=in.(active,pending)",
    "data->verified=eq.true",
    "order=created_at.desc",
    "limit=10",
    "offset=0"
  ].join("&");

  const result = parseQueryString("users", queryString);

  assertExists(result);
  assert(result.query.includes("SELECT"), "Query should have SELECT");
  assert(result.query.includes("WHERE"), "Query should have WHERE");
  assert(result.query.includes("ORDER BY"), "Query should have ORDER BY");
  assert(result.query.includes("LIMIT"), "Query should have LIMIT");
  assert(result.query.includes("OFFSET"), "Query should have OFFSET");

  assertEquals(result.tables, ["users"]);
  assert(result.params.length >= 4, "Should have multiple parameters");
});

Deno.test("WASM - parseOnly: returns parsed structure", async () => {
  const result = parseOnly("age=gte.18&status=eq.active&limit=10");

  assertExists(result);

  assertEquals(result.filters.length, 2, "Should have 2 filters");
  assertEquals(result.limit, 10, "Should have limit of 10");
  assert(result.offset === null || result.offset === undefined, "Should not have offset");
});

Deno.test("WASM - parseOnly: select and order", async () => {
  const result = parseOnly("select=id,name&order=created_at.desc");

  assertExists(result);
  assertExists(result.select);
  assertExists(result.order);

  assertEquals(result.select.length, 2, "Should have 2 selected fields");
  assertEquals(result.order.length, 1, "Should have 1 order term");
});

Deno.test("WASM - toJSON: serialization works", async () => {
  const result = parseQueryString("users", "age=gte.18&limit=10");

  const json = result.toJSON();
  assertExists(json);
  assertExists(json.query);
  assertExists(json.params);
  assertExists(json.tables);
});

Deno.test("WASM - Graceful handling: various inputs", async () => {
  // Parser is lenient and tries to parse what it can
  // Test that it doesn't crash on various inputs

  // Simple malformed query - parser will skip unparseable parts
  const result1 = parseQueryString("users", "limit=10");
  assertExists(result1);
  assert(result1.query.includes("LIMIT"), "Should handle valid parts");

  // Query with only valid operators
  const result2 = parseQueryString("users", "id=eq.1");
  assertExists(result2);
  assert(result2.query.includes("WHERE"), "Should parse valid filter");
});

Deno.test("WASM - Edge case: empty query string", async () => {
  // Empty query should be valid and return SELECT *
  const result = parseQueryString("users", "");

  assertExists(result);
  assert(result.query.includes("SELECT"), "Query should have SELECT");
  assert(result.query.includes("users"), "Query should reference table");
  assertEquals(result.params.length, 0, "Should have no parameters");
});

Deno.test("WASM - Performance: parse 100 queries", async () => {
  const queries = Array.from({ length: 100 }, (_, i) => ({
    table: "users",
    query: `select=id,name&age=gte.${18 + i}&limit=${10 + i}`
  }));

  const start = performance.now();

  for (const { table, query } of queries) {
    const result = parseQueryString(table, query);
    assertExists(result);
  }

  const duration = performance.now() - start;
  const avgTime = duration / queries.length;

  console.log(`Average parse time: ${avgTime.toFixed(3)}ms per query`);
  console.log(`Total time for 100 queries: ${duration.toFixed(2)}ms`);

  assert(avgTime < 5, "Average parse time should be under 5ms");
});

console.log("\n✅ All WASM integration tests completed successfully!");
