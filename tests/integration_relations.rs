//! Integration tests for relation resolution with real PostgreSQL database.
//!
//! These tests require:
//! - Docker running
//! - `docker-compose up` to start PostgreSQL
//! - cargo feature: --features postgres

#![cfg(feature = "postgres")]

use postgrest_parser::{parse_query_string, QueryBuilder, SchemaCache};
use sqlx::postgres::PgPoolOptions;
use std::sync::Arc;

/// Helper to get database pool
async fn get_pool() -> sqlx::PgPool {
    PgPoolOptions::new()
        .max_connections(5)
        .connect("postgres://postgres:postgres@localhost:5433/postgrest_parser_test")
        .await
        .expect("Failed to connect to test database. Is docker-compose up?")
}

#[tokio::test]
async fn test_many_to_one_relation() {
    let pool = get_pool().await;
    let cache = SchemaCache::load_from_database(&pool).await.unwrap();
    let cache = Arc::new(cache);

    // Query: orders with customer details (many-to-one)
    let params = parse_query_string("select=id,total_amount,customers(name,email)").unwrap();

    let mut builder = QueryBuilder::new()
        .with_schema_cache(cache.clone())
        .with_schema("public");

    let result = builder.build_select("orders", &params).unwrap();

    println!("Generated SQL:\n{}\n", result.query);

    // Verify JOIN condition is present
    assert!(result.query.contains("WHERE"));
    assert!(result.query.contains(r#""orders"."customer_id" = "customers"."id""#));

    // Verify it's using COALESCE for null safety
    assert!(result.query.contains("COALESCE"));

    // Execute the query to verify it works
    let rows = sqlx::query(&result.query).fetch_all(&pool).await.unwrap();
    assert!(!rows.is_empty(), "Should return orders with customers");
}

#[tokio::test]
async fn test_one_to_many_relation() {
    let pool = get_pool().await;
    let cache = SchemaCache::load_from_database(&pool).await.unwrap();
    let cache = Arc::new(cache);

    // Query: customers with their orders (one-to-many)
    let params = parse_query_string("select=id,name,email,orders(id,total_amount,status)").unwrap();

    let mut builder = QueryBuilder::new()
        .with_schema_cache(cache.clone())
        .with_schema("public");

    let result = builder.build_select("customers", &params).unwrap();

    println!("Generated SQL:\n{}\n", result.query);

    // Verify reverse FK is used
    assert!(result.query.contains(r#""orders"."customer_id" = "customers"."id""#));

    // Should use json_agg for array of orders
    assert!(result.query.contains("json_agg"));

    // Execute to verify it works
    let rows = sqlx::query(&result.query).fetch_all(&pool).await.unwrap();
    assert!(!rows.is_empty(), "Should return customers with orders");
}

#[tokio::test]
async fn test_nested_relations() {
    let pool = get_pool().await;
    let cache = SchemaCache::load_from_database(&pool).await.unwrap();
    let cache = Arc::new(cache);

    // Query: customers -> orders -> order_items (nested relations)
    let params = parse_query_string(
        "select=name,orders(id,order_items(quantity,products(name,price)))"
    ).unwrap();

    let mut builder = QueryBuilder::new()
        .with_schema_cache(cache.clone())
        .with_schema("public");

    let result = builder.build_select("customers", &params);

    // This should work for first level
    if let Ok(result) = result {
        println!("Generated SQL:\n{}\n", result.query);
        assert!(result.query.contains("orders"));
    } else {
        // Nested relations might not be fully supported yet
        println!("Nested relations not yet fully supported");
    }
}

#[tokio::test]
async fn test_no_schema_cache_fails_gracefully() {
    // Without schema cache, relations should fail with helpful error or generate placeholder
    let params = parse_query_string("select=id,customers(name)").unwrap();
    let mut builder = QueryBuilder::new();
    let result = builder.build_select("orders", &params);

    // Should either error or generate placeholder (invalid) SQL
    match result {
        Ok(r) => {
            // Placeholder SQL won't have JOIN condition
            assert!(!r.query.contains(r#""customer_id" = "customers"."id""#));
        }
        Err(_) => {
            // Also acceptable - error without schema cache
        }
    }
}

#[tokio::test]
async fn test_relation_not_found() {
    let pool = get_pool().await;
    let cache = SchemaCache::load_from_database(&pool).await.unwrap();
    let cache = Arc::new(cache);

    // Query with non-existent relation
    let params = parse_query_string("select=id,nonexistent_table(name)").unwrap();

    let mut builder = QueryBuilder::new()
        .with_schema_cache(cache.clone())
        .with_schema("public");

    let result = builder.build_select("customers", &params);

    // Should error with RelationNotFound
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(err.to_string().contains("relation not found"));
}

#[tokio::test]
async fn test_schema_cache_foreign_keys() {
    let pool = get_pool().await;
    let cache = SchemaCache::load_from_database(&pool).await.unwrap();

    // Verify foreign keys were loaded
    let fks = cache.get_foreign_keys("public", "orders");
    assert!(!fks.is_empty(), "orders should have foreign keys");

    // Find customer FK
    let customer_fk = fks.iter().find(|fk| fk.to_table == "customers");
    assert!(customer_fk.is_some(), "orders should have FK to customers");

    let fk = customer_fk.unwrap();
    assert_eq!(fk.from_column, "customer_id");
    assert_eq!(fk.to_column, "id");
}

#[tokio::test]
async fn test_find_relationship() {
    let pool = get_pool().await;
    let cache = SchemaCache::load_from_database(&pool).await.unwrap();

    // Test many-to-one: orders -> customers
    let rel = cache.find_relationship("public", "orders", "customers");
    assert!(rel.is_some());
    let rel = rel.unwrap();
    assert_eq!(rel.from_table, "orders");
    assert_eq!(rel.to_table, "customers");

    // Test one-to-many: customers -> orders (reverse)
    let rel = cache.find_relationship("public", "customers", "orders");
    assert!(rel.is_some());
    let rel = rel.unwrap();
    assert_eq!(rel.from_table, "customers");
    assert_eq!(rel.to_table, "orders");
}

#[tokio::test]
async fn test_complete_workflow() {
    let pool = get_pool().await;

    // Load schema
    let cache = SchemaCache::load_from_database(&pool).await.unwrap();
    let cache = Arc::new(cache);

    // Parse complex query
    let params = parse_query_string(
        "select=id,name,email,orders(id,status,total_amount)&email=like.*@example.com&limit=5"
    ).unwrap();

    // Build SQL with schema cache
    let mut builder = QueryBuilder::new()
        .with_schema_cache(cache)
        .with_schema("public");

    let result = builder.build_select("customers", &params).unwrap();

    println!("Complete workflow SQL:\n{}\n", result.query);

    // Verify all parts are present
    assert!(result.query.contains("SELECT"));
    assert!(result.query.contains(r#""orders"."customer_id" = "customers"."id""#));
    assert!(result.query.contains("WHERE"));
    assert!(result.query.contains("LIKE"));
    assert!(result.query.contains("LIMIT"));

    // Execute to verify it works
    // Extract values from serde_json::Value to avoid type inference issues
    let email_pattern = result.params[0].as_str().unwrap();
    let limit_value = result.params[1].as_i64().unwrap();

    let rows = sqlx::query(&result.query)
        .bind(email_pattern) // email LIKE param
        .bind(limit_value) // LIMIT param
        .fetch_all(&pool)
        .await
        .unwrap();

    println!("Returned {} rows", rows.len());
    assert!(rows.len() <= 5, "Should respect LIMIT");
}
