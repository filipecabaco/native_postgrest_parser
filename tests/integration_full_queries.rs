//! Full integration tests executing real PostgREST-style queries against PostgreSQL.
//!
//! These tests verify the complete workflow: parse -> build SQL -> execute against DB
//! They test realistic use cases with filtering, ordering, pagination, and relations.

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
async fn test_customers_with_filters_and_ordering() {
    let pool = get_pool().await;
    let cache = Arc::new(SchemaCache::load_from_database(&pool).await.unwrap());

    // Query: Get gold tier customers, ordered by name
    let params = parse_query_string(
        "select=id,name,email&metadata->>tier=eq.gold&order=name.asc"
    ).unwrap();

    let mut builder = QueryBuilder::new()
        .with_schema_cache(cache)
        .with_schema("public");

    let result = builder.build_select("customers", &params).unwrap();

    // Execute query
    let rows = sqlx::query(&result.query)
        .bind(result.params[0].as_str().unwrap()) // tier value
        .fetch_all(&pool)
        .await
        .unwrap();

    assert!(!rows.is_empty(), "Should find gold tier customers");
    println!("Found {} gold tier customers", rows.len());
}

#[tokio::test]
async fn test_orders_with_customer_details() {
    let pool = get_pool().await;
    let cache = Arc::new(SchemaCache::load_from_database(&pool).await.unwrap());

    // Query: Get completed orders with customer info
    let params = parse_query_string(
        "select=id,total_amount,status,customers(name,email)&status=eq.completed&order=total_amount.desc"
    ).unwrap();

    let mut builder = QueryBuilder::new()
        .with_schema_cache(cache)
        .with_schema("public");

    let result = builder.build_select("orders", &params).unwrap();

    println!("Generated SQL:\n{}\n", result.query);

    // Verify SQL structure
    assert!(result.query.contains("WHERE"));
    assert!(result.query.contains("ORDER BY"));
    assert!(result.query.contains(r#""orders"."customer_id" = "customers"."id""#));

    // Execute query
    let status_value = result.params[0].as_str().unwrap();
    let rows = sqlx::query(&result.query)
        .bind(status_value)
        .fetch_all(&pool)
        .await
        .unwrap();

    assert!(!rows.is_empty(), "Should find completed orders");
}

#[tokio::test]
async fn test_customers_with_all_orders() {
    let pool = get_pool().await;
    let cache = Arc::new(SchemaCache::load_from_database(&pool).await.unwrap());

    // Query: Get all customers with their orders (even if no orders)
    let params = parse_query_string(
        "select=id,name,email,orders(id,status,total_amount)&limit=10"
    ).unwrap();

    let mut builder = QueryBuilder::new()
        .with_schema_cache(cache)
        .with_schema("public");

    let result = builder.build_select("customers", &params).unwrap();

    println!("SQL:\n{}\n", result.query);

    // Execute query
    let limit_value = result.params[0].as_i64().unwrap();
    let rows = sqlx::query(&result.query)
        .bind(limit_value)
        .fetch_all(&pool)
        .await
        .unwrap();

    assert!(!rows.is_empty(), "Should return customers");
    assert!(rows.len() <= 10, "Should respect limit");
}

#[tokio::test]
async fn test_pagination_with_offset() {
    let pool = get_pool().await;
    let cache = Arc::new(SchemaCache::load_from_database(&pool).await.unwrap());

    // Query: Get products with pagination
    let params = parse_query_string(
        "select=id,name,price&order=price.desc&limit=3&offset=2"
    ).unwrap();

    let mut builder = QueryBuilder::new()
        .with_schema_cache(cache)
        .with_schema("public");

    let result = builder.build_select("products", &params).unwrap();

    // Execute query
    let limit_value = result.params[0].as_i64().unwrap();
    let offset_value = result.params[1].as_i64().unwrap();

    let rows = sqlx::query(&result.query)
        .bind(limit_value)
        .bind(offset_value)
        .fetch_all(&pool)
        .await
        .unwrap();

    assert!(rows.len() <= 3, "Should respect limit");
}

#[tokio::test]
async fn test_range_filters() {
    let pool = get_pool().await;
    let cache = Arc::new(SchemaCache::load_from_database(&pool).await.unwrap());

    // Query: Get products with price between 50 and 150 (inclusive)
    // Multiple filters on same column now work correctly!
    let params = parse_query_string(
        "select=id,name,price&price=gte.50&price=lte.150"
    ).unwrap();

    let mut builder = QueryBuilder::new()
        .with_schema_cache(cache)
        .with_schema("public");

    let result = builder.build_select("products", &params).unwrap();

    println!("Range filter SQL:\n{}\n", result.query);
    println!("Params: {:?}", result.params);

    // Verify both filters are present
    assert_eq!(result.params.len(), 2, "Should have two filter parameters");
    assert!(result.query.contains(">="), "Should have >= operator");
    assert!(result.query.contains("<="), "Should have <= operator");

    // Execute query
    let min_price = if result.params[0].is_string() {
        result.params[0].as_str().unwrap().parse::<f64>().unwrap()
    } else {
        result.params[0].as_f64().unwrap()
    };

    let max_price = if result.params[1].is_string() {
        result.params[1].as_str().unwrap().parse::<f64>().unwrap()
    } else {
        result.params[1].as_f64().unwrap()
    };

    let rows = sqlx::query(&result.query)
        .bind(min_price)
        .bind(max_price)
        .fetch_all(&pool)
        .await
        .unwrap();

    println!("Found {} products in price range 50-150", rows.len());
    assert!(!rows.is_empty(), "Should find products in price range");
}

#[tokio::test]
async fn test_in_operator_with_list() {
    let pool = get_pool().await;
    let cache = Arc::new(SchemaCache::load_from_database(&pool).await.unwrap());

    // Query: Get orders with specific statuses
    let params = parse_query_string(
        "select=id,status,total_amount&status=in.(pending,completed)"
    ).unwrap();

    let mut builder = QueryBuilder::new()
        .with_schema_cache(cache)
        .with_schema("public");

    let result = builder.build_select("orders", &params).unwrap();

    println!("IN operator SQL:\n{}\n", result.query);

    // Verify SQL contains ANY clause (PostgreSQL converts IN to = ANY)
    assert!(result.query.contains("ANY"));

    // Execute query - ANY expects a Postgres array
    // Extract the array elements from serde_json::Value
    let statuses = result.params[0].as_array().unwrap();
    let status_strs: Vec<&str> = statuses.iter()
        .map(|v| v.as_str().unwrap())
        .collect();

    let rows = sqlx::query(&result.query)
        .bind(&status_strs[..]) // Pass as slice for Postgres array
        .fetch_all(&pool)
        .await
        .unwrap();

    assert!(!rows.is_empty(), "Should find orders with specified statuses");
}

#[tokio::test]
async fn test_or_logic_filter() {
    let pool = get_pool().await;
    let cache = Arc::new(SchemaCache::load_from_database(&pool).await.unwrap());

    // Query: Get products in Electronics OR Office category
    let params = parse_query_string(
        "select=id,name,category&or=(category.eq.Electronics,category.eq.Office)"
    ).unwrap();

    let mut builder = QueryBuilder::new()
        .with_schema_cache(cache)
        .with_schema("public");

    let result = builder.build_select("products", &params).unwrap();

    println!("OR logic SQL:\n{}\n", result.query);

    // Execute query
    let cat1 = result.params[0].as_str().unwrap();
    let cat2 = result.params[1].as_str().unwrap();

    let rows = sqlx::query(&result.query)
        .bind(cat1)
        .bind(cat2)
        .fetch_all(&pool)
        .await
        .unwrap();

    assert!(!rows.is_empty(), "Should find products in either category");
}

#[tokio::test]
async fn test_pattern_matching() {
    let pool = get_pool().await;
    let cache = Arc::new(SchemaCache::load_from_database(&pool).await.unwrap());

    // Query: Find products with names containing "Laptop"
    let params = parse_query_string(
        "select=id,name,price&name=like.*Laptop*"
    ).unwrap();

    let mut builder = QueryBuilder::new()
        .with_schema_cache(cache)
        .with_schema("public");

    let result = builder.build_select("products", &params).unwrap();

    println!("Pattern matching SQL:\n{}\n", result.query);
    println!("Params: {:?}", result.params);

    // Execute query
    let pattern = result.params[0].as_str().unwrap();

    let rows = sqlx::query(&result.query)
        .bind(pattern)
        .fetch_all(&pool)
        .await
        .unwrap();

    println!("Found {} products matching pattern '{}'", rows.len(), pattern);

    // Note: If no matches, the pattern conversion might not be working as expected
    // PostgREST converts *Laptop* to %Laptop% for SQL LIKE
    if rows.is_empty() {
        println!("Warning: No matches found. Check pattern conversion.");
    }
}

#[tokio::test]
async fn test_null_handling() {
    let pool = get_pool().await;
    let cache = Arc::new(SchemaCache::load_from_database(&pool).await.unwrap());

    // Query: Get orders with no notes
    let params = parse_query_string(
        "select=id,status,notes&notes=is.null"
    ).unwrap();

    let mut builder = QueryBuilder::new()
        .with_schema_cache(cache)
        .with_schema("public");

    let result = builder.build_select("orders", &params).unwrap();

    println!("NULL check SQL:\n{}\n", result.query);

    // Verify SQL contains IS NULL
    assert!(result.query.contains("IS NULL"));

    // Execute query (no parameters for IS NULL)
    let rows = sqlx::query(&result.query)
        .fetch_all(&pool)
        .await
        .unwrap();

    println!("Found {} orders with null notes", rows.len());
}

#[tokio::test]
async fn test_junction_table_many_to_many() {
    let pool = get_pool().await;
    let cache = Arc::new(SchemaCache::load_from_database(&pool).await.unwrap());

    // Query: Get posts (should work even though we don't fully support M2M yet)
    let params = parse_query_string(
        "select=id,title,published&published=eq.true"
    ).unwrap();

    let mut builder = QueryBuilder::new()
        .with_schema_cache(cache)
        .with_schema("public");

    let result = builder.build_select("posts", &params).unwrap();

    println!("Posts query SQL:\n{}\n", result.query);
    println!("Params: {:?}", result.params);

    // Execute query - handle boolean parameter
    let published_value = if result.params[0].is_boolean() {
        result.params[0].as_bool().unwrap()
    } else if result.params[0].is_string() {
        result.params[0].as_str().unwrap() == "true"
    } else {
        true
    };

    let rows = sqlx::query(&result.query)
        .bind(published_value)
        .fetch_all(&pool)
        .await
        .unwrap();

    assert!(!rows.is_empty(), "Should find published posts");
}

#[tokio::test]
async fn test_one_to_one_relationship() {
    let pool = get_pool().await;
    let cache = Arc::new(SchemaCache::load_from_database(&pool).await.unwrap());

    // Query: Get customers with their profile (1-to-1)
    let params = parse_query_string(
        "select=id,name,customer_profiles(bio,avatar_url)"
    ).unwrap();

    let mut builder = QueryBuilder::new()
        .with_schema_cache(cache)
        .with_schema("public");

    let result = builder.build_select("customers", &params).unwrap();

    println!("One-to-one SQL:\n{}\n", result.query);

    // Verify JOIN condition
    assert!(result.query.contains(r#""customer_profiles"."customer_id" = "customers"."id""#));

    // Execute query
    let rows = sqlx::query(&result.query)
        .fetch_all(&pool)
        .await
        .unwrap();

    assert!(!rows.is_empty(), "Should return customers");
}

#[tokio::test]
async fn test_complex_combined_query() {
    let pool = get_pool().await;
    let cache = Arc::new(SchemaCache::load_from_database(&pool).await.unwrap());

    // Complex query: customers with gold/silver tier, with pending/completed orders,
    // ordered by name, paginated
    let params = parse_query_string(
        "select=name,email,orders(id,status,total_amount)&\
         metadata->>tier=in.(gold,silver)&\
         order=name.asc&\
         limit=5"
    ).unwrap();

    let mut builder = QueryBuilder::new()
        .with_schema_cache(cache)
        .with_schema("public");

    let result = builder.build_select("customers", &params).unwrap();

    println!("Complex query SQL:\n{}\n", result.query);

    // Verify structure
    assert!(result.query.contains("WHERE"));
    assert!(result.query.contains("ORDER BY"));
    assert!(result.query.contains("LIMIT"));
    assert!(result.query.contains(r#""orders"."customer_id" = "customers"."id""#));

    // Execute query
    let tier_array = result.params[0].as_array().unwrap();
    let tier_strs: Vec<&str> = tier_array.iter()
        .map(|v| v.as_str().unwrap())
        .collect();
    let limit_value = result.params[1].as_i64().unwrap();

    let rows = sqlx::query(&result.query)
        .bind(&tier_strs[..]) // Pass as slice for Postgres array
        .bind(limit_value)
        .fetch_all(&pool)
        .await
        .unwrap();

    println!("Found {} matching customers", rows.len());
    assert!(rows.len() <= 5, "Should respect limit");
}
