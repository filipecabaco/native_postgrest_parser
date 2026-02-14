import { readFile } from "node:fs/promises";
import { createRequire } from "node:module";

const require = createRequire(import.meta.url);
const pkgPath = require.resolve("postgrest-parser");
const pkg = await import("postgrest-parser");

const {
  default: init,
  parseQueryString,
  parseOnly,
  buildFilterClause,
  parseInsert,
  parseUpdate,
  parseDelete,
  parseRpc,
  parseRequest,
} = pkg;

let failures = 0;

function assert(condition, message) {
  if (!condition) {
    console.error(`FAIL: ${message}`);
    failures++;
  }
}

function assertIncludes(str, substr, message) {
  assert(
    typeof str === "string" && str.includes(substr),
    `${message} — expected "${substr}" in "${str}"`
  );
}

// Initialize WASM
const wasmPath = pkgPath.replace("postgrest_parser.js", "postgrest_parser_bg.wasm");
const wasmBytes = await readFile(wasmPath);
await init({ module_or_path: wasmBytes });

console.log("--- parseQueryString ---");
{
  const r = parseQueryString("users", "select=id,name&age=gte.18&limit=10");
  assert(r.query, "query should exist");
  assert(r.params, "params should exist");
  assert(r.tables, "tables should exist");
  assertIncludes(r.query, "SELECT", "should have SELECT");
  assertIncludes(r.query, "WHERE", "should have WHERE");
  assertIncludes(r.query, "LIMIT", "should have LIMIT");
  assert(r.tables.includes("users"), "tables should include users");
  console.log("  basic select + filter + limit: OK");
}

{
  const r = parseQueryString("users", "and=(age.gte.18,or(role.eq.admin,role.eq.user))");
  assertIncludes(r.query, "AND", "should have AND");
  assertIncludes(r.query, "OR", "should have OR");
  console.log("  nested logic operators: OK");
}

{
  const r = parseQueryString("users", "");
  assertIncludes(r.query, "SELECT", "empty query should produce SELECT");
  console.log("  empty query string: OK");
}

console.log("\n--- parseOnly ---");
{
  const r = parseOnly("age=gte.18&status=eq.active&limit=10");
  assert(r.filters && r.filters.length === 2, "should have 2 filters");
  assert(r.limit === 10, "should have limit 10");
  console.log("  parse without SQL: OK");
}

console.log("\n--- buildFilterClause ---");
{
  const parsed = parseOnly("age=gte.18&status=eq.active");
  const r = buildFilterClause(parsed.filters);
  assert(r.clause, "clause should exist");
  assert(r.params, "params should exist");
  console.log("  filter clause from parsed filters: OK");
}

console.log("\n--- parseInsert ---");
{
  const r = parseInsert("users", JSON.stringify({ name: "Alice", age: 30 }));
  assertIncludes(r.query, "INSERT", "should have INSERT");
  console.log("  simple insert: OK");
}

console.log("\n--- parseUpdate ---");
{
  const r = parseUpdate("users", JSON.stringify({ name: "Bob" }), "id=eq.1");
  assertIncludes(r.query, "UPDATE", "should have UPDATE");
  assertIncludes(r.query, "WHERE", "should have WHERE");
  console.log("  simple update: OK");
}

console.log("\n--- parseDelete ---");
{
  const r = parseDelete("users", "id=eq.1");
  assertIncludes(r.query, "DELETE", "should have DELETE");
  assertIncludes(r.query, "WHERE", "should have WHERE");
  console.log("  simple delete: OK");
}

console.log("\n--- parseRpc ---");
{
  const r = parseRpc("get_user", JSON.stringify({ user_id: 1 }));
  assertIncludes(r.query, "get_user", "should reference function");
  console.log("  rpc call: OK");
}

console.log("\n--- parseRequest (HTTP routing) ---");
{
  const r = parseRequest("GET", "users", "select=id,name&limit=5");
  assertIncludes(r.query, "SELECT", "GET should produce SELECT");
  console.log("  GET -> SELECT: OK");
}
{
  const r = parseRequest(
    "POST",
    "users",
    "",
    JSON.stringify({ name: "Alice" })
  );
  assertIncludes(r.query, "INSERT", "POST should produce INSERT");
  console.log("  POST -> INSERT: OK");
}
{
  const r = parseRequest("DELETE", "users", "id=eq.1");
  assertIncludes(r.query, "DELETE", "DELETE should produce DELETE");
  console.log("  DELETE -> DELETE: OK");
}

console.log("\n--- toJSON serialization ---");
{
  const r = parseQueryString("users", "age=gte.18");
  const json = r.toJSON();
  assert(json.query, "toJSON should have query");
  assert(json.params, "toJSON should have params");
  assert(json.tables, "toJSON should have tables");
  console.log("  toJSON: OK");
}

if (failures > 0) {
  console.error(`\n${failures} test(s) failed`);
  process.exit(1);
} else {
  console.log("\nAll e2e tests passed");
}
