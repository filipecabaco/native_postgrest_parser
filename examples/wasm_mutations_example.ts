#!/usr/bin/env -S deno run --allow-read

/**
 * PostgREST Parser WASM - Mutations & RPC Example
 *
 * This example demonstrates all mutation operations (INSERT, UPDATE, DELETE),
 * RPC (stored procedure/function) calls, and HTTP method routing.
 *
 * Covers 21 comprehensive examples:
 * - INSERT (single, bulk, UPSERT with ON CONFLICT)
 * - UPDATE (simple, complex filters, OR logic)
 * - DELETE (simple, with RETURNING, with LIMIT)
 * - RPC (with args, without args, with filtering, schema-qualified)
 * - HTTP method routing (GET, POST, PUT, PATCH, DELETE)
 * - PUT auto-UPSERT with ON CONFLICT from query filters
 *
 * Prerequisites:
 * 1. Build the WASM package: wasm-pack build --target web --features wasm
 * 2. Install Deno: https://deno.land
 *
 * Run this example:
 *   deno run --allow-read examples/wasm_mutations_example.ts
 */

import init, {
  parseInsert,
  parseUpdate,
  parseDelete,
  parseRpc,
  parseRequest,
} from "../pkg/postgrest_parser.js";

// Initialize WASM module
console.log("🚀 Initializing PostgREST Parser WASM...\n");
await init();

console.log("=".repeat(80));
console.log("PostgREST Parser - Mutations & RPC Examples");
console.log("=".repeat(80) + "\n");

// ============================================================================
// INSERT OPERATIONS
// ============================================================================

console.log("📝 INSERT Operations\n");
console.log("-".repeat(80) + "\n");

// Example 1: Simple INSERT
console.log("Example 1: Simple INSERT with single row\n");
const insert1 = parseInsert(
  "users",
  JSON.stringify({ name: "Alice Johnson", email: "alice@example.com", age: 28 }),
  null
);

console.log("Body:", { name: "Alice Johnson", email: "alice@example.com", age: 28 });
console.log("\nGenerated SQL:");
console.log(insert1.query);
console.log("\nParameters:", insert1.params);
console.log("\n" + "-".repeat(80) + "\n");

// Example 2: INSERT with RETURNING
console.log("Example 2: INSERT with RETURNING clause\n");
const insert2 = parseInsert(
  "users",
  JSON.stringify({ name: "Bob Smith", email: "bob@example.com" }),
  "returning=id,name,created_at"
);

console.log("Body:", { name: "Bob Smith", email: "bob@example.com" });
console.log("Query string: returning=id,name,created_at");
console.log("\nGenerated SQL:");
console.log(insert2.query);
console.log("\nParameters:", insert2.params);
console.log("\n" + "-".repeat(80) + "\n");

// Example 3: Bulk INSERT
console.log("Example 3: Bulk INSERT with multiple rows\n");
const bulkInsert = parseInsert(
  "products",
  JSON.stringify([
    { name: "Laptop", price: 999.99, category: "electronics" },
    { name: "Mouse", price: 29.99, category: "electronics" },
    { name: "Desk", price: 299.99, category: "furniture" },
  ]),
  "returning=id,name"
);

console.log("Body: [3 products...]");
console.log("Query string: returning=id,name");
console.log("\nGenerated SQL:");
console.log(bulkInsert.query);
console.log("\nParameters:", bulkInsert.params);
console.log("\n" + "-".repeat(80) + "\n");

// Example 4: UPSERT (INSERT with ON CONFLICT)
console.log("Example 4: UPSERT - INSERT with conflict resolution\n");
const upsert = parseInsert(
  "users",
  JSON.stringify({ email: "alice@example.com", name: "Alice Updated", status: "active" }),
  "on_conflict=email&returning=id,name,email"
);

console.log("Body:", { email: "alice@example.com", name: "Alice Updated" });
console.log("Query string: on_conflict=email&returning=id,name,email");
console.log("\nGenerated SQL:");
console.log(upsert.query);
console.log("\nParameters:", upsert.params);
console.log("\n" + "-".repeat(80) + "\n");

// Example 5: UPSERT with selective update
console.log("Example 5: UPSERT with selective column updates\n");
const upsertSelective = parseInsert(
  "products",
  JSON.stringify({ sku: "LAP-001", name: "Laptop Pro", price: 1299.99, stock: 50 }),
  "on_conflict=sku,update_columns=price,stock&returning=id,sku,price,stock"
);

console.log("Body: { sku, name, price, stock }");
console.log("Query string: on_conflict=sku,update_columns=price,stock");
console.log("\nGenerated SQL:");
console.log(upsertSelective.query);
console.log("\nParameters:", upsertSelective.params);
console.log("\n" + "-".repeat(80) + "\n");

// ============================================================================
// UPDATE OPERATIONS
// ============================================================================

console.log("📝 UPDATE Operations\n");
console.log("-".repeat(80) + "\n");

// Example 6: Simple UPDATE
console.log("Example 6: Simple UPDATE with filter\n");
const update1 = parseUpdate(
  "users",
  JSON.stringify({ status: "inactive" }),
  "id=eq.123"
);

console.log("Body:", { status: "inactive" });
console.log("Query string: id=eq.123");
console.log("\nGenerated SQL:");
console.log(update1.query);
console.log("\nParameters:", update1.params);
console.log("\n" + "-".repeat(80) + "\n");

// Example 7: UPDATE with multiple filters
console.log("Example 7: UPDATE with multiple filters and RETURNING\n");
const update2 = parseUpdate(
  "products",
  JSON.stringify({ price: 899.99, stock: 0 }),
  "category=eq.electronics&price=gt.1000&returning=id,name,price"
);

console.log("Body:", { price: 899.99, stock: 0 });
console.log("Query string: category=eq.electronics&price=gt.1000&returning=id,name,price");
console.log("\nGenerated SQL:");
console.log(update2.query);
console.log("\nParameters:", update2.params);
console.log("\n" + "-".repeat(80) + "\n");

// Example 8: UPDATE with complex logic
console.log("Example 8: UPDATE with OR logic\n");
const update3 = parseUpdate(
  "orders",
  JSON.stringify({ status: "cancelled", cancelled_at: "2024-01-15T10:30:00Z" }),
  "or=(status.eq.pending,status.eq.processing)&created_at=lt.2024-01-01&returning=id,status"
);

console.log("Body:", { status: "cancelled", cancelled_at: "..." });
console.log("Query string: or=(status.eq.pending,status.eq.processing)&created_at=lt.2024-01-01");
console.log("\nGenerated SQL:");
console.log(update3.query);
console.log("\nParameters:", update3.params);
console.log("\n" + "-".repeat(80) + "\n");

// ============================================================================
// DELETE OPERATIONS
// ============================================================================

console.log("📝 DELETE Operations\n");
console.log("-".repeat(80) + "\n");

// Example 9: Simple DELETE
console.log("Example 9: Simple DELETE with filter\n");
const delete1 = parseDelete("users", "id=eq.456");

console.log("Query string: id=eq.456");
console.log("\nGenerated SQL:");
console.log(delete1.query);
console.log("\nParameters:", delete1.params);
console.log("\n" + "-".repeat(80) + "\n");

// Example 10: DELETE with RETURNING
console.log("Example 10: DELETE with RETURNING deleted rows\n");
const delete2 = parseDelete(
  "temporary_sessions",
  "expires_at=lt.2024-01-01&returning=id,user_id,expires_at"
);

console.log("Query string: expires_at=lt.2024-01-01&returning=id,user_id,expires_at");
console.log("\nGenerated SQL:");
console.log(delete2.query);
console.log("\nParameters:", delete2.params);
console.log("\n" + "-".repeat(80) + "\n");

// Example 11: DELETE with complex filter
console.log("Example 11: DELETE with multiple conditions\n");
const delete3 = parseDelete(
  "logs",
  "and=(level.eq.debug,created_at.lt.2024-01-01)&limit=1000&order=created_at.asc"
);

console.log("Query string: and=(level.eq.debug,created_at.lt.2024-01-01)&limit=1000");
console.log("\nGenerated SQL:");
console.log(delete3.query);
console.log("\nParameters:", delete3.params);
console.log("\n" + "-".repeat(80) + "\n");

// ============================================================================
// RPC (Stored Procedures / Functions)
// ============================================================================

console.log("📝 RPC Operations (Stored Procedures)\n");
console.log("-".repeat(80) + "\n");

// Example 12: RPC with arguments
console.log("Example 12: RPC function call with arguments\n");
const rpc1 = parseRpc(
  "calculate_order_total",
  JSON.stringify({ order_id: 123, tax_rate: 0.08, discount: 10.00 }),
  null
);

console.log("Function: calculate_order_total");
console.log("Arguments:", { order_id: 123, tax_rate: 0.08, discount: 10.00 });
console.log("\nGenerated SQL:");
console.log(rpc1.query);
console.log("\nParameters:", rpc1.params);
console.log("\n" + "-".repeat(80) + "\n");

// Example 13: RPC with result filtering
console.log("Example 13: RPC with result filtering and ordering\n");
const rpc2 = parseRpc(
  "search_products",
  JSON.stringify({ search_term: "laptop", category: "electronics" }),
  "select=id,name,price&price=lte.1500&order=price.asc&limit=10"
);

console.log("Function: search_products");
console.log("Arguments:", { search_term: "laptop", category: "electronics" });
console.log("Query string: select=id,name,price&price=lte.1500&order=price.asc&limit=10");
console.log("\nGenerated SQL:");
console.log(rpc2.query);
console.log("\nParameters:", rpc2.params);
console.log("\n" + "-".repeat(80) + "\n");

// Example 14: RPC without arguments
console.log("Example 14: RPC function without arguments\n");
const rpc3 = parseRpc("get_current_stats", null, "select=total_users,total_orders");

console.log("Function: get_current_stats");
console.log("Arguments: none");
console.log("Query string: select=total_users,total_orders");
console.log("\nGenerated SQL:");
console.log(rpc3.query);
console.log("\n" + "-".repeat(80) + "\n");

// Example 15: Schema-qualified RPC
console.log("Example 15: Schema-qualified RPC function\n");
const rpc4 = parseRpc(
  "analytics.generate_report",
  JSON.stringify({ start_date: "2024-01-01", end_date: "2024-01-31", report_type: "sales" }),
  "returning=report_id,status"
);

console.log("Function: analytics.generate_report");
console.log("Arguments:", { start_date: "2024-01-01", end_date: "2024-01-31", report_type: "sales" });
console.log("\nGenerated SQL:");
console.log(rpc4.query);
console.log("\nParameters:", rpc4.params);
console.log("\n" + "-".repeat(80) + "\n");

// ============================================================================
// COMPLETE HTTP REQUEST PARSING
// ============================================================================

console.log("📝 Complete HTTP Request Parsing\n");
console.log("-".repeat(80) + "\n");

// Example 16: GET request (SELECT)
console.log("Example 16: HTTP GET - Auto-detects SELECT\n");
const httpGet = parseRequest(
  "GET",
  "users",
  "age=gte.18&status=eq.active&order=created_at.desc&limit=10",
  null,
  null
);

console.log("Method: GET");
console.log("Path: users");
console.log("Query: age=gte.18&status=eq.active&order=created_at.desc&limit=10");
console.log("\nGenerated SQL:");
console.log(httpGet.query);
console.log("\nParameters:", httpGet.params);
console.log("\n" + "-".repeat(80) + "\n");

// Example 17: POST request (INSERT)
console.log("Example 17: HTTP POST - Auto-detects INSERT\n");
const httpPost = parseRequest(
  "POST",
  "users",
  "returning=id,name,email",
  JSON.stringify({ name: "Charlie", email: "charlie@example.com" }),
  JSON.stringify({ Prefer: "return=representation" })
);

console.log("Method: POST");
console.log("Path: users");
console.log("Body:", { name: "Charlie", email: "charlie@example.com" });
console.log("Headers: Prefer: return=representation");
console.log("\nGenerated SQL:");
console.log(httpPost.query);
console.log("\nParameters:", httpPost.params);
console.log("\n" + "-".repeat(80) + "\n");

// Example 18: PUT request (UPSERT with auto ON CONFLICT)
console.log("Example 18: HTTP PUT - Auto-detects UPSERT with ON CONFLICT\n");
const httpPut = parseRequest(
  "PUT",
  "users",
  "email=eq.charlie@example.com&returning=id,name,email",
  JSON.stringify({
    email: "charlie@example.com",
    name: "Charlie Updated",
    status: "active"
  }),
  null
);

console.log("Method: PUT");
console.log("Path: users");
console.log("Query: email=eq.charlie@example.com&returning=id,name,email");
console.log("Body:", { email: "charlie@example.com", name: "Charlie Updated", status: "active" });
console.log("\nGenerated SQL:");
console.log(httpPut.query);
console.log("\nParameters:", httpPut.params);
console.log("\nNote: PUT auto-generates ON CONFLICT from query filters (email=eq.charlie@example.com)");
console.log("This makes PUT idempotent - safe to retry without duplicates");
console.log("\n" + "-".repeat(80) + "\n");

// Example 19: PATCH request (UPDATE)
console.log("Example 19: HTTP PATCH - Auto-detects UPDATE\n");
const httpPatch = parseRequest(
  "PATCH",
  "users",
  "id=eq.123&returning=id,status,updated_at",
  JSON.stringify({ status: "verified" }),
  null
);

console.log("Method: PATCH");
console.log("Path: users");
console.log("Query: id=eq.123&returning=id,status,updated_at");
console.log("Body:", { status: "verified" });
console.log("\nGenerated SQL:");
console.log(httpPatch.query);
console.log("\nParameters:", httpPatch.params);
console.log("\n" + "-".repeat(80) + "\n");

// Example 20: DELETE request
console.log("Example 20: HTTP DELETE - Auto-detects DELETE\n");
const httpDelete = parseRequest(
  "DELETE",
  "sessions",
  "expires_at=lt.2024-01-01&returning=count",
  null,
  JSON.stringify({ Prefer: "count=exact" })
);

console.log("Method: DELETE");
console.log("Path: sessions");
console.log("Query: expires_at=lt.2024-01-01&returning=count");
console.log("Headers: Prefer: count=exact");
console.log("\nGenerated SQL:");
console.log(httpDelete.query);
console.log("\nParameters:", httpDelete.params);
console.log("\n" + "-".repeat(80) + "\n");

// Example 21: RPC via POST
console.log("Example 21: HTTP POST to RPC endpoint\n");
const httpRpc = parseRequest(
  "POST",
  "rpc/authenticate_user",
  "select=token,user_id,expires_at",
  JSON.stringify({ email: "alice@example.com", password: "secret123" }),
  null
);

console.log("Method: POST");
console.log("Path: rpc/authenticate_user");
console.log("Body:", { email: "alice@example.com", password: "[REDACTED]" });
console.log("\nGenerated SQL:");
console.log(httpRpc.query);
console.log("\nParameters:", httpRpc.params);
console.log("\n" + "-".repeat(80) + "\n");

// Performance test for mutations
console.log("⚡ Performance Test: 1000 INSERT operations\n");
const testBody = JSON.stringify({ name: "Test User", email: "test@example.com" });
const iterations = 1000;

const startTime = performance.now();
for (let i = 0; i < iterations; i++) {
  parseInsert("users", testBody, null);
}
const endTime = performance.now();

const totalTime = endTime - startTime;
const avgTime = totalTime / iterations;

console.log(`Total time: ${totalTime.toFixed(2)}ms`);
console.log(`Average time: ${avgTime.toFixed(4)}ms per INSERT`);
console.log(`Throughput: ${(iterations / (totalTime / 1000)).toFixed(0)} operations/second`);

console.log("\n" + "=".repeat(80));
console.log("✅ All 21 mutation examples completed successfully!");
console.log("=".repeat(80));
console.log("\n💡 Key Features Demonstrated:");
console.log("  • INSERT with single and bulk rows");
console.log("  • UPSERT with ON CONFLICT and selective updates");
console.log("  • UPDATE with complex filters and RETURNING");
console.log("  • DELETE with LIMIT and ORDER BY");
console.log("  • RPC calls with arguments and result filtering");
console.log("  • HTTP method routing: GET→SELECT, POST→INSERT, PUT→UPSERT, PATCH→UPDATE, DELETE→DELETE");
console.log("  • PUT auto-generates ON CONFLICT from query filters (idempotent operations)");
console.log("  • All operations support RETURNING clause");
console.log("  • Full parameter binding for SQL injection prevention");
console.log("  • parseRequest() as universal routing wrapper for all operations");
console.log("=".repeat(80));
