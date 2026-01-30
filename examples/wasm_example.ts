#!/usr/bin/env -S deno run --allow-read

/**
 * PostgREST Parser WASM Example
 *
 * This example demonstrates using the PostgREST parser in TypeScript/Deno.
 *
 * Prerequisites:
 * 1. Build the WASM package: wasm-pack build --target web --features wasm
 * 2. Install Deno: https://deno.land
 *
 * Run this example:
 *   deno run --allow-read examples/wasm_example.ts
 */

import init, { parseQueryString, parseOnly } from "../pkg/postgrest_parser.js";

// Initialize WASM module
console.log("🚀 Initializing PostgREST Parser WASM...\n");
await init();

console.log("=" .repeat(80));
console.log("PostgREST Parser - WASM Example");
console.log("=" .repeat(80) + "\n");

// Example 1: Simple query
console.log("📝 Example 1: Simple SELECT with filters\n");
const example1 = parseQueryString(
  "users",
  "select=id,name,email&age=gte.18&status=eq.active"
);

console.log("Query String:", "select=id,name,email&age=gte.18&status=eq.active");
console.log("\nGenerated SQL:");
console.log(example1.query);
console.log("\nParameters:", example1.params);
console.log("Tables:", example1.tables);
console.log("\n" + "-".repeat(80) + "\n");

// Example 2: Complex query with logic operators
console.log("📝 Example 2: Complex query with AND/OR logic\n");
const example2 = parseQueryString(
  "posts",
  "and=(status.eq.published,or(featured.is.true,views.gt.1000))&order=created_at.desc&limit=10"
);

console.log("Query String:", "and=(status.eq.published,or(featured.is.true,views.gt.1000))&order=created_at.desc&limit=10");
console.log("\nGenerated SQL:");
console.log(example2.query);
console.log("\nParameters:", example2.params);
console.log("\n" + "-".repeat(80) + "\n");

// Example 3: Full-text search
console.log("📝 Example 3: Full-text search with language\n");
const example3 = parseQueryString(
  "articles",
  "content=fts(english).rust programming&order=rank.desc&limit=20"
);

console.log("Query String:", "content=fts(english).rust programming&order=rank.desc&limit=20");
console.log("\nGenerated SQL:");
console.log(example3.query);
console.log("\nParameters:", example3.params);
console.log("\n" + "-".repeat(80) + "\n");

// Example 4: JSON path operators
console.log("📝 Example 4: JSON path navigation\n");
const example4 = parseQueryString(
  "users",
  "data->profile->verified=eq.true&data->>email=like.*@company.com"
);

console.log("Query String:", "data->profile->verified=eq.true&data->>email=like.*@company.com");
console.log("\nGenerated SQL:");
console.log(example4.query);
console.log("\nParameters:", example4.params);
console.log("\n" + "-".repeat(80) + "\n");

// Example 5: Array operators
console.log("📝 Example 5: Array operations\n");
const example5 = parseQueryString(
  "posts",
  "tags=cs.{rust,elixir,go}&categories=ov.(programming,tutorial)"
);

console.log("Query String:", "tags=cs.{rust,elixir,go}&categories=ov.(programming,tutorial)");
console.log("\nGenerated SQL:");
console.log(example5.query);
console.log("\nParameters:", example5.params);
console.log("\n" + "-".repeat(80) + "\n");

// Example 6: Parse only (no SQL generation)
console.log("📝 Example 6: Parse structure without SQL generation\n");
const example6 = parseOnly("select=id,name&age=gte.18&limit=10&offset=20");

console.log("Query String:", "select=id,name&age=gte.18&limit=10&offset=20");
console.log("\nParsed structure:");
console.log(JSON.stringify(example6, null, 2));
console.log("\n" + "-".repeat(80) + "\n");

// Example 7: Realistic API query
console.log("📝 Example 7: Realistic API endpoint query\n");
const example7 = parseQueryString(
  "orders",
  [
    "select=id,total,created_at,customer(name,email)",
    "status=in.(pending,processing,shipped)",
    "created_at=gte.2024-01-01",
    "total=gt.100",
    "order=created_at.desc",
    "limit=50",
    "offset=0"
  ].join("&")
);

console.log("Query String (complex):");
console.log("  select=id,total,created_at,customer(name,email)");
console.log("  status=in.(pending,processing,shipped)");
console.log("  created_at=gte.2024-01-01");
console.log("  total=gt.100");
console.log("  order=created_at.desc");
console.log("  limit=50&offset=0");
console.log("\nGenerated SQL:");
console.log(example7.query);
console.log("\nParameters:", example7.params);
console.log("Tables:", example7.tables);
console.log("\n" + "-".repeat(80) + "\n");

// Performance test
console.log("⚡ Performance Test: Parsing 1000 queries\n");
const testQuery = "select=id,name&age=gte.18&status=eq.active&limit=10";
const iterations = 1000;

const startTime = performance.now();
for (let i = 0; i < iterations; i++) {
  parseQueryString("users", testQuery);
}
const endTime = performance.now();

const totalTime = endTime - startTime;
const avgTime = totalTime / iterations;

console.log(`Total time: ${totalTime.toFixed(2)}ms`);
console.log(`Average time: ${avgTime.toFixed(4)}ms per query`);
console.log(`Throughput: ${(iterations / (totalTime / 1000)).toFixed(0)} queries/second`);

console.log("\n" + "=".repeat(80));
console.log("✅ All examples completed successfully!");
console.log("=".repeat(80));
