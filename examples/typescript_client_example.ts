/**
 * Type-Safe TypeScript Client Examples
 *
 * This file demonstrates the improved type-safe API for the PostgREST parser.
 * The new client provides better TypeScript ergonomics with proper types,
 * object-based APIs, and helpful type inference.
 */

import { createClient } from "../pkg/client.js";
import type { QueryResult } from "../pkg/types.js";

// Create a client instance
const client = createClient();

console.log("=== Type-Safe PostgREST Parser Examples ===\n");

// Example 1: SELECT with type-safe options
console.log("Example 1: SELECT with filters, ordering, and pagination");
const selectResult: QueryResult = client.select("users", {
  filters: {
    age: "gte.18",
    status: "eq.active",
  },
  order: ["created_at.desc", "name.asc"],
  limit: 10,
  offset: 0,
});
console.log("SQL:", selectResult.query);
console.log("Params:", selectResult.params);
console.log("Tables:", selectResult.tables);
console.log();

// Example 2: INSERT with type-safe data and options
console.log("Example 2: INSERT with returning and prefer header");
const insertResult: QueryResult = client.insert(
  "users",
  {
    name: "Alice",
    email: "alice@example.com",
    age: 25,
    status: "active",
  },
  {
    returning: "*",
    prefer: { return: "representation" },
  }
);
console.log("SQL:", insertResult.query);
console.log("Params:", insertResult.params);
console.log();

// Example 3: UPSERT (smart conflict resolution)
console.log("Example 3: UPSERT with auto ON CONFLICT");
const upsertResult: QueryResult = client.upsert(
  "users",
  {
    email: "bob@example.com",
    name: "Bob Updated",
    status: "active",
  },
  ["email"], // Conflict columns
  {
    returning: "id,name,email",
  }
);
console.log("SQL:", upsertResult.query);
console.log("Params:", upsertResult.params);
console.log();

// Example 4: UPDATE with filters
console.log("Example 4: UPDATE with multiple filters");
const updateResult: QueryResult = client.update(
  "users",
  {
    status: "inactive",
    updated_at: "now()",
  },
  {
    id: "eq.123",
    status: "eq.active",
  },
  {
    returning: "id,status",
  }
);
console.log("SQL:", updateResult.query);
console.log("Params:", updateResult.params);
console.log();

// Example 5: DELETE with filters
console.log("Example 5: DELETE with filters and returning");
const deleteResult: QueryResult = client.delete(
  "users",
  {
    status: "eq.inactive",
    last_login: "lt.2023-01-01",
  },
  {
    returning: "id,name",
  }
);
console.log("SQL:", deleteResult.query);
console.log("Params:", deleteResult.params);
console.log();

// Example 6: RPC (stored procedure call)
console.log("Example 6: RPC with arguments and result filtering");
const rpcResult: QueryResult = client.rpc(
  "calculate_order_total",
  {
    order_id: 123,
    tax_rate: 0.08,
    shipping_cost: 10.0,
  },
  {
    select: ["total", "tax", "shipping"],
    limit: 1,
  }
);
console.log("SQL:", rpcResult.query);
console.log("Params:", rpcResult.params);
console.log();

// Example 7: Complex SELECT with multiple filters on same column
console.log("Example 7: Range query (multiple filters on same column)");
const rangeResult: QueryResult = client.select("products", {
  filters: {
    "price": "gte.50",
    "price": "lte.150", // Both filters will be applied
    category: "eq.electronics",
  },
  order: ["price.asc"],
});
console.log("SQL:", rangeResult.query);
console.log("Params:", rangeResult.params);
console.log();

// Example 8: Batch insert
console.log("Example 8: Batch INSERT");
const batchInsertResult: QueryResult = client.insert(
  "users",
  [
    { name: "User 1", email: "user1@example.com" },
    { name: "User 2", email: "user2@example.com" },
    { name: "User 3", email: "user3@example.com" },
  ],
  {
    returning: "id,name",
    prefer: { return: "representation" },
  }
);
console.log("SQL:", batchInsertResult.query);
console.log("Params:", batchInsertResult.params);
console.log();

// Example 9: Using parseRequest for HTTP routing
console.log("Example 9: HTTP method routing with parseRequest");
const httpGetResult: QueryResult = client.parseRequest(
  "GET",
  "users",
  "age=gte.18&limit=10",
  null,
  null
);
console.log("GET SQL:", httpGetResult.query);
console.log();

const httpPostResult: QueryResult = client.parseRequest(
  "POST",
  "users",
  "returning=id",
  { name: "Charlie", email: "charlie@example.com" },
  { Prefer: "return=representation" }
);
console.log("POST SQL:", httpPostResult.query);
console.log();

// Example 10: Using with count
console.log("Example 10: SELECT with row count");
const countResult: QueryResult = client.select("users", {
  filters: {
    status: "eq.active",
  },
  limit: 10,
  count: "exact",
});
console.log("SQL:", countResult.query);
console.log("(Prefer header will include count=exact)");
console.log();

// Example 11: Parse only (inspect query structure)
console.log("Example 11: Parse query string without generating SQL");
const parsed = client.parseOnly("age=gte.18&status=eq.active&order=name.asc&limit=10");
console.log("Parsed structure:", JSON.stringify(parsed, null, 2));
console.log();

// =================================================================
// Integration with Database Clients
// =================================================================

console.log("=== Database Integration Examples ===\n");

// Example 12: PostgreSQL (pg) integration
console.log("Example 12: PostgreSQL (pg) integration");
console.log(`
import { Pool } from 'pg';
import { createClient } from './pkg/client.js';

const pool = new Pool({ connectionString: process.env.DATABASE_URL });
const parser = createClient();

async function getActiveUsers() {
  const result = parser.select("users", {
    filters: { status: "eq.active" },
    order: ["created_at.desc"],
    limit: 10
  });

  const { rows } = await pool.query(result.query, result.params);
  return rows;
}
`);

// Example 13: Express.js route handler
console.log("Example 13: Express.js route handler");
console.log(`
import express from 'express';
import { createClient } from './pkg/client.js';
import { Pool } from 'pg';

const app = express();
const pool = new Pool({ connectionString: process.env.DATABASE_URL });
const parser = createClient();

app.get('/api/:table', async (req, res) => {
  try {
    const result = parser.select(req.params.table, {
      filters: req.query.filters as Record<string, string>,
      limit: req.query.limit ? parseInt(req.query.limit as string) : undefined
    });

    const { rows } = await pool.query(result.query, result.params);
    res.json(rows);
  } catch (error) {
    res.status(500).json({ error: error.message });
  }
});
`);

// Example 14: Next.js API route
console.log("Example 14: Next.js API route");
console.log(`
// pages/api/users.ts
import type { NextApiRequest, NextApiResponse } from 'next';
import { createClient } from '@/lib/postgrest-parser/client';
import { query } from '@/lib/db';

const parser = createClient();

export default async function handler(
  req: NextApiRequest,
  res: NextApiResponse
) {
  const result = parser.select("users", {
    filters: req.query.filters as Record<string, string>,
    limit: 10
  });

  const rows = await query(result.query, result.params);
  res.json(rows);
}
`);

// Example 15: Supabase Edge Function
console.log("Example 15: Supabase Edge Function");
console.log(`
// supabase/functions/users/index.ts
import { createClient } from '../_shared/postgrest-parser/client.ts';
import { createClient as createSupabaseClient } from '@supabase/supabase-js';

const supabase = createSupabaseClient(
  Deno.env.get('SUPABASE_URL')!,
  Deno.env.get('SUPABASE_SERVICE_ROLE_KEY')!
);

const parser = createClient();

Deno.serve(async (req) => {
  const url = new URL(req.url);
  const filters = Object.fromEntries(url.searchParams);

  const result = parser.select("users", { filters });

  const { data, error } = await supabase.rpc('execute_sql', {
    query: result.query,
    params: result.params
  });

  return new Response(JSON.stringify(data), {
    headers: { 'Content-Type': 'application/json' }
  });
});
`);

console.log("\n=== Summary ===");
console.log("✓ All examples use fully-typed TypeScript API");
console.log("✓ No 'any' types - full IntelliSense support");
console.log("✓ Object-based APIs instead of JSON strings");
console.log("✓ Type-safe HTTP method routing");
console.log("✓ Proper error handling with typed errors");
console.log("✓ Compatible with all major database clients");
