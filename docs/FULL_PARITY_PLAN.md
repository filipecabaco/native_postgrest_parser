# PostgREST Full Parity Implementation Plan

## Current Status: ~60% Parity

**Implemented:**
- ✅ SELECT (100%)
- ✅ INSERT (70%)
- ✅ UPDATE (75%)
- ✅ DELETE (75%)
- ✅ Schema resolution
- ✅ Safety validations

**Missing for 100% Parity:**
1. Prefer headers
2. RPC function calls
3. Resource embedding in mutations
4. PUT upsert
5. Advanced ON CONFLICT

---

## Phase 3: Prefer Headers Support (Week 1-2)

**Priority: HIGH** - Used in ~80% of real-world applications

### 3.1 Prefer Header Types

```rust
// src/ast/prefer.rs (~200 lines)

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ReturnRepresentation {
    Full,           // return=representation (default for POST/PATCH)
    Minimal,        // return=minimal
    HeadersOnly,    // return=headers-only
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum Resolution {
    MergeDuplicates,    // resolution=merge-duplicates (upsert)
    IgnoreDuplicates,   // resolution=ignore-duplicates
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum Count {
    Exact,      // count=exact
    Planned,    // count=planned
    Estimated,  // count=estimated
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum Plurality {
    Singular,   // plurality=singular (expect single row)
    Multiple,   // default
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum Missing {
    Default,    // missing=default (use column defaults)
    Null,       // default
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PreferOptions {
    pub return_representation: Option<ReturnRepresentation>,
    pub resolution: Option<Resolution>,
    pub count: Option<Count>,
    pub plurality: Option<Plurality>,
    pub missing: Option<Missing>,
}
```

### 3.2 Prefer Header Parser (nom combinators)

```rust
// src/parser/prefer.rs (~300 lines)

use nom::{
    branch::alt,
    bytes::complete::tag,
    character::complete::{alpha1, space0},
    combinator::{map, opt},
    multi::separated_list1,
    sequence::{delimited, preceded, tuple},
    IResult,
};

/// Parses: "return=representation"
fn parse_return(input: &str) -> IResult<&str, ReturnRepresentation> {
    preceded(
        tag("return="),
        alt((
            map(tag("representation"), |_| ReturnRepresentation::Full),
            map(tag("minimal"), |_| ReturnRepresentation::Minimal),
            map(tag("headers-only"), |_| ReturnRepresentation::HeadersOnly),
        )),
    )(input)
}

/// Parses: "resolution=merge-duplicates"
fn parse_resolution(input: &str) -> IResult<&str, Resolution> {
    preceded(
        tag("resolution="),
        alt((
            map(tag("merge-duplicates"), |_| Resolution::MergeDuplicates),
            map(tag("ignore-duplicates"), |_| Resolution::IgnoreDuplicates),
        )),
    )(input)
}

/// Parses: "count=exact"
fn parse_count(input: &str) -> IResult<&str, Count> {
    preceded(
        tag("count="),
        alt((
            map(tag("exact"), |_| Count::Exact),
            map(tag("planned"), |_| Count::Planned),
            map(tag("estimated"), |_| Count::Estimated),
        )),
    )(input)
}

/// Parses full Prefer header: "return=representation, count=exact"
pub fn parse_prefer_header(input: &str) -> Result<PreferOptions, Error> {
    let parts: Vec<&str> = input.split(',').map(|s| s.trim()).collect();

    let mut options = PreferOptions::default();

    for part in parts {
        if let Ok((_, ret)) = parse_return(part) {
            options.return_representation = Some(ret);
        } else if let Ok((_, res)) = parse_resolution(part) {
            options.resolution = Some(res);
        } else if let Ok((_, cnt)) = parse_count(part) {
            options.count = Some(cnt);
        } else if let Ok((_, plur)) = parse_plurality(part) {
            options.plurality = Some(plur);
        } else if let Ok((_, miss)) = parse_missing(part) {
            options.missing = Some(miss);
        }
    }

    Ok(options)
}
```

### 3.3 Integration with Operations

```rust
// Update Operation enum to include Prefer options
pub enum Operation {
    Select(ParsedParams, Option<PreferOptions>),
    Insert(InsertParams, Option<PreferOptions>),
    Update(UpdateParams, Option<PreferOptions>),
    Delete(DeleteParams, Option<PreferOptions>),
}

// Update parse() function
pub fn parse(
    method: &str,
    table: &str,
    query_string: &str,
    body: Option<&str>,
    headers: Option<&HashMap<String, String>>,
) -> Result<Operation, Error> {
    // Extract Prefer header
    let prefer = headers
        .and_then(|h| h.get("Prefer").or_else(|| h.get("prefer")))
        .map(|p| parse_prefer_header(p))
        .transpose()?;

    // ... rest of parsing with prefer options
}
```

### 3.4 Real-World Test Cases

```rust
#[cfg(test)]
mod prefer_tests {
    use super::*;

    #[test]
    fn test_insert_with_return_representation() {
        // Real-world: User signup returning full user object
        let headers = hashmap! {
            "Prefer".to_string() => "return=representation".to_string(),
        };
        let body = r#"{"email": "alice@example.com", "name": "Alice"}"#;
        let op = parse("POST", "users", "", Some(body), Some(&headers)).unwrap();

        match op {
            Operation::Insert(params, Some(prefer)) => {
                assert_eq!(prefer.return_representation, Some(ReturnRepresentation::Full));
            }
            _ => panic!("Expected Insert operation"),
        }
    }

    #[test]
    fn test_upsert_with_merge_duplicates() {
        // Real-world: Upsert user preferences
        let headers = hashmap! {
            "Prefer".to_string() => "resolution=merge-duplicates".to_string(),
        };
        let body = r#"{"user_id": 123, "theme": "dark"}"#;
        let op = parse("POST", "preferences", "on_conflict=user_id", Some(body), Some(&headers)).unwrap();

        match op {
            Operation::Insert(params, Some(prefer)) => {
                assert_eq!(prefer.resolution, Some(Resolution::MergeDuplicates));
                assert!(params.on_conflict.is_some());
            }
            _ => panic!("Expected Insert with resolution"),
        }
    }

    #[test]
    fn test_count_with_select() {
        // Real-world: Pagination with total count
        let headers = hashmap! {
            "Prefer".to_string() => "count=exact".to_string(),
        };
        let op = parse("GET", "users", "limit=10&offset=0", None, Some(&headers)).unwrap();

        match op {
            Operation::Select(_, Some(prefer)) => {
                assert_eq!(prefer.count, Some(Count::Exact));
            }
            _ => panic!("Expected Select with count"),
        }
    }

    #[test]
    fn test_multiple_prefer_options() {
        // Real-world: Complex mutation with multiple preferences
        let headers = hashmap! {
            "Prefer".to_string() => "return=representation, missing=default".to_string(),
        };
        let body = r#"{"name": "Bob"}"#; // age will use column default
        let op = parse("POST", "users", "", Some(body), Some(&headers)).unwrap();

        match op {
            Operation::Insert(_, Some(prefer)) => {
                assert_eq!(prefer.return_representation, Some(ReturnRepresentation::Full));
                assert_eq!(prefer.missing, Some(Missing::Default));
            }
            _ => panic!("Expected Insert with multiple preferences"),
        }
    }
}
```

---

## Phase 4: RPC Function Calls (Week 3-4)

**Priority: HIGH** - Used in ~60% of applications for stored procedures

### 4.1 RPC AST Types

```rust
// src/ast/rpc.rs (~150 lines)

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RpcParams {
    pub function_name: String,
    pub args: HashMap<String, serde_json::Value>,
    pub filters: Vec<LogicCondition>,  // PostgREST allows filtering RPC results
    pub order: Vec<OrderTerm>,
    pub limit: Option<u64>,
    pub offset: Option<u64>,
    pub returning: Option<Vec<SelectItem>>,
}

impl RpcParams {
    pub fn new(function_name: impl Into<String>, args: HashMap<String, serde_json::Value>) -> Self {
        Self {
            function_name: function_name.into(),
            args,
            filters: Vec::new(),
            order: Vec::new(),
            limit: None,
            offset: None,
            returning: None,
        }
    }
}

// Add to Operation enum
pub enum Operation {
    Select(ParsedParams, Option<PreferOptions>),
    Insert(InsertParams, Option<PreferOptions>),
    Update(UpdateParams, Option<PreferOptions>),
    Delete(DeleteParams, Option<PreferOptions>),
    Rpc(RpcParams, Option<PreferOptions>),  // NEW
}
```

### 4.2 RPC Parser

```rust
// src/parser/rpc.rs (~200 lines)

/// Parses RPC parameters from query string and body
///
/// PostgREST format:
/// POST /rpc/function_name?filter=value
/// Body: {"arg1": "value1", "arg2": "value2"}
pub fn parse_rpc_params(
    function_name: &str,
    query_string: &str,
    body: Option<&str>,
) -> Result<RpcParams, Error> {
    // Parse arguments from body
    let args = if let Some(body_str) = body {
        let json_value = parse_json_body(body_str)?;
        validate_rpc_args(json_value)?
    } else {
        HashMap::new()
    };

    let mut params = RpcParams::new(function_name, args);

    // Parse query parameters (filters, order, limit, etc.)
    let query_params = parse_query_params(query_string);

    // Parse filters
    let filters = parse_params_from_pairs(
        query_params
            .iter()
            .filter(|(k, _)| !is_reserved_key(k))
            .map(|(k, v)| (k.clone(), v.clone()))
            .collect(),
    )?;

    params = params.with_filters(filters.filters);

    // Parse order, limit, offset
    if let Some(order_str) = query_params.get("order") {
        params = params.with_order(parse_order(order_str)?);
    }

    if let Some(limit_str) = query_params.get("limit") {
        params = params.with_limit(limit_str.parse()?);
    }

    Ok(params)
}

fn validate_rpc_args(value: Value) -> Result<HashMap<String, Value>, Error> {
    match value {
        Value::Object(map) => {
            let mut hash_map = HashMap::new();
            for (k, v) in map {
                hash_map.insert(k, v);
            }
            Ok(hash_map)
        }
        _ => Err(Error::Parse(ParseError::InvalidJsonBody(
            "RPC arguments must be an object".to_string(),
        ))),
    }
}
```

### 4.3 RPC SQL Generation

```rust
// src/sql/rpc.rs (~250 lines)

impl QueryBuilder {
    pub fn build_rpc(
        &mut self,
        resolved_table: &ResolvedTable,
        params: &RpcParams,
    ) -> Result<QueryResult, SqlError> {
        // SELECT * FROM schema.function_name(arg1 := $1, arg2 := $2)
        self.sql.push_str("SELECT * FROM ");
        self.sql.push_str(&format!(
            "\"{}\".\"{}\"(",
            resolved_table.schema,
            params.function_name
        ));

        // Build named arguments
        let mut sorted_args: Vec<(&String, &Value)> = params.args.iter().collect();
        sorted_args.sort_by_key(|(k, _)| *k); // Deterministic order

        for (i, (name, value)) in sorted_args.iter().enumerate() {
            if i > 0 {
                self.sql.push_str(", ");
            }
            let param = self.add_param((*value).clone());
            self.sql.push_str(&format!("\"{}\" := {}", name, param));
        }

        self.sql.push(')');

        // Add filters on RPC result
        if !params.filters.is_empty() {
            self.build_where_clause(&params.filters)?;
        }

        // Add ORDER BY
        if !params.order.is_empty() {
            self.build_order_clause(&params.order)?;
        }

        // Add LIMIT/OFFSET
        self.build_limit_offset(params.limit, params.offset)?;

        Ok(QueryResult {
            query: self.sql.clone(),
            params: self.params.clone(),
            tables: vec![params.function_name.clone()],
        })
    }
}
```

### 4.4 Real-World RPC Test Cases

```rust
#[cfg(test)]
mod rpc_tests {
    use super::*;

    #[test]
    fn test_rpc_simple_function() {
        // Real-world: Call search function
        let body = r#"{"query": "rust programming"}"#;
        let op = parse("POST", "rpc/search_posts", "", Some(body), None).unwrap();

        match op {
            Operation::Rpc(params, _) => {
                assert_eq!(params.function_name, "search_posts");
                assert_eq!(params.args.len(), 1);
            }
            _ => panic!("Expected RPC operation"),
        }

        let result = operation_to_sql("rpc/search_posts", &op).unwrap();
        assert!(result.query.contains("SELECT * FROM"));
        assert!(result.query.contains("search_posts"));
        assert!(result.query.contains("\"query\" :="));
    }

    #[test]
    fn test_rpc_with_filters() {
        // Real-world: Call function and filter results
        let body = r#"{"user_id": 123}"#;
        let op = parse("POST", "rpc/get_user_posts", "status=eq.published&limit=10", Some(body), None).unwrap();

        match op {
            Operation::Rpc(params, _) => {
                assert!(params.filters.len() > 0);
                assert_eq!(params.limit, Some(10));
            }
            _ => panic!("Expected RPC with filters"),
        }
    }

    #[test]
    fn test_rpc_get_method() {
        // Real-world: GET request to RPC (no body, args in query)
        // Note: PostgREST supports GET for RPC with query params
        let op = parse("GET", "rpc/current_user_id", "", None, None).unwrap();

        match op {
            Operation::Rpc(params, _) => {
                assert_eq!(params.function_name, "current_user_id");
                assert!(params.args.is_empty());
            }
            _ => panic!("Expected RPC operation"),
        }
    }

    #[test]
    fn test_rpc_complex_args() {
        // Real-world: Complex nested JSON arguments
        let body = r#"{
            "filters": {
                "status": "active",
                "tags": ["rust", "database"]
            },
            "page_size": 20
        }"#;
        let op = parse("POST", "rpc/advanced_search", "", Some(body), None).unwrap();

        match op {
            Operation::Rpc(params, _) => {
                assert!(params.args.contains_key("filters"));
                assert!(params.args.contains_key("page_size"));
            }
            _ => panic!("Expected RPC with complex args"),
        }
    }
}
```

---

## Phase 5: Resource Embedding in Mutations (Week 5)

**Priority: MEDIUM** - Used in ~30% of applications

### 5.1 Extend INSERT/UPDATE/DELETE with Select

```rust
// Modify existing params to include embedded selects

impl InsertParams {
    pub fn with_select(mut self, select: Vec<SelectItem>) -> Self {
        self.returning = Some(select);
        self
    }
}

// Parser changes
pub fn parse_insert_params(query_string: &str, body: &str) -> Result<InsertParams, Error> {
    // ... existing code ...

    // Parse select parameter (for embedding)
    if let Some(select_str) = query_params.get("select") {
        let select = parse_select(select_str)?;
        params = params.with_select(select);
    }

    // ... rest of code ...
}
```

### 5.2 SQL Generation with CTEs

```rust
// For resource embedding, use CTEs (Common Table Expressions)

impl QueryBuilder {
    pub fn build_insert_with_embedding(
        &mut self,
        resolved_table: &ResolvedTable,
        params: &InsertParams,
    ) -> Result<QueryResult, SqlError> {
        // WITH inserted AS (
        //   INSERT INTO users (...) VALUES (...) RETURNING *
        // )
        // SELECT inserted.*, posts.* FROM inserted
        // LEFT JOIN posts ON posts.user_id = inserted.id

        // This requires relationship metadata - defer to later phase
        // For now, just return the inserted row with selected columns
        self.build_insert(resolved_table, params)
    }
}
```

### 5.3 Real-World Test Cases

```rust
#[test]
fn test_insert_with_embedding() {
    // Real-world: Insert user and return with related posts
    let body = r#"{"email": "alice@example.com", "name": "Alice"}"#;
    let op = parse("POST", "users", "select=id,name,posts(id,title)", Some(body), None).unwrap();

    // Note: Full embedding requires relationship metadata
    // For now, test that select is parsed correctly
    match op {
        Operation::Insert(params, _) => {
            assert!(params.returning.is_some());
            let select = params.returning.unwrap();
            assert!(select.iter().any(|s| s.item_type == ItemType::Relation && s.name == "posts"));
        }
        _ => panic!("Expected Insert with embedding"),
    }
}
```

---

## Phase 6: PUT Upsert (Week 6)

**Priority: LOW** - Alternative to POST with on_conflict

### 6.1 PUT Method Handling

```rust
// Update parse() to handle PUT
pub fn parse(
    method: &str,
    table: &str,
    query_string: &str,
    body: Option<&str>,
    headers: Option<&HashMap<String, String>>,
) -> Result<Operation, Error> {
    match method.to_uppercase().as_str() {
        "GET" => { /* ... */ }
        "POST" => { /* ... */ }
        "PUT" => {
            // PUT is upsert: INSERT with automatic ON CONFLICT
            let body = body.ok_or_else(|| /* ... */)?;
            let mut params = parse_insert_params(query_string, body)?;

            // Auto-detect conflict columns from filters or use primary key
            if params.on_conflict.is_none() {
                // Extract columns from query filters to use as conflict target
                let conflict_columns = extract_conflict_columns_from_query(query_string)?;
                params = params.with_on_conflict(OnConflict::do_update(conflict_columns));
            }

            Ok(Operation::Insert(params, prefer))
        }
        "PATCH" => { /* ... */ }
        "DELETE" => { /* ... */ }
        _ => Err(/* ... */)
    }
}
```

### 6.2 Real-World Test Cases

```rust
#[test]
fn test_put_upsert() {
    // Real-world: Upsert user by email
    let body = r#"{"email": "alice@example.com", "name": "Alice Updated"}"#;
    let op = parse("PUT", "users", "email=eq.alice@example.com", Some(body), None).unwrap();

    match op {
        Operation::Insert(params, _) => {
            assert!(params.on_conflict.is_some());
            let conflict = params.on_conflict.unwrap();
            assert_eq!(conflict.action, ConflictAction::DoUpdate);
        }
        _ => panic!("Expected Insert (upsert) operation"),
    }
}
```

---

## Phase 7: Advanced ON CONFLICT (Week 7)

**Priority: LOW** - For complex upsert scenarios

### 7.1 Enhanced ON CONFLICT AST

```rust
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct OnConflict {
    pub columns: Vec<String>,
    pub action: ConflictAction,
    pub where_clause: Option<Vec<LogicCondition>>,  // NEW: partial unique index
    pub update_columns: Option<Vec<String>>,        // NEW: specify which columns to update
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ConflictAction {
    DoNothing,
    DoUpdate,
    DoUpdateWhere,  // NEW: conditional update
}
```

### 7.2 Real-World Test Cases

```rust
#[test]
fn test_on_conflict_with_where() {
    // Real-world: Upsert only if certain condition met
    let body = r#"{"email": "alice@example.com", "name": "Alice"}"#;
    let query = "on_conflict=email.do_update.where(updated_at.lt.2024-01-01)";
    let op = parse("POST", "users", query, Some(body), None).unwrap();

    // SQL: ON CONFLICT (email) DO UPDATE SET ... WHERE updated_at < '2024-01-01'
}
```

---

## Real-World Test Scenarios (All Phases)

### E-Commerce Application

```rust
#[test]
fn test_ecommerce_workflow() {
    // 1. Create order with items (bulk insert)
    let body = r#"[
        {"product_id": 1, "quantity": 2, "price": 29.99},
        {"product_id": 3, "quantity": 1, "price": 49.99}
    ]"#;
    let headers = hashmap! {
        "Prefer".to_string() => "return=representation".to_string(),
        "Content-Profile".to_string() => "sales".to_string(),
    };
    let op = parse("POST", "order_items", "returning=*", Some(body), Some(&headers)).unwrap();

    // 2. Update order status
    let body = r#"{"status": "shipped", "shipped_at": "2024-01-15"}"#;
    let op = parse("PATCH", "orders", "id=eq.123", Some(body), None).unwrap();

    // 3. Calculate total with RPC
    let body = r#"{"order_id": 123}"#;
    let op = parse("POST", "rpc/calculate_order_total", "", Some(body), None).unwrap();
}
```

### Social Media Application

```rust
#[test]
fn test_social_media_workflow() {
    // 1. Create post with return representation
    let body = r#"{"content": "Hello World!", "user_id": 456}"#;
    let headers = hashmap! {
        "Prefer".to_string() => "return=representation".to_string(),
    };
    let op = parse("POST", "posts", "select=*,user(*)", Some(body), Some(&headers)).unwrap();

    // 2. Upsert like (PUT)
    let body = r#"{"user_id": 789, "post_id": 123}"#;
    let op = parse("PUT", "likes", "user_id=eq.789&post_id=eq.123", Some(body), None).unwrap();

    // 3. Delete old posts (with limit)
    let op = parse("DELETE", "posts", "created_at=lt.2020-01-01&order=created_at.asc&limit=100", None, None).unwrap();
}
```

### Analytics Application

```rust
#[test]
fn test_analytics_workflow() {
    // 1. Bulk upsert metrics
    let body = r#"[
        {"metric": "pageviews", "value": 1234, "date": "2024-01-15"},
        {"metric": "signups", "value": 56, "date": "2024-01-15"}
    ]"#;
    let headers = hashmap! {
        "Prefer".to_string() => "resolution=merge-duplicates".to_string(),
    };
    let op = parse("POST", "metrics", "on_conflict=metric,date", Some(body), Some(&headers)).unwrap();

    // 2. Get aggregated stats with RPC
    let body = r#"{"start_date": "2024-01-01", "end_date": "2024-01-31"}"#;
    let op = parse("POST", "rpc/get_monthly_stats", "", Some(body), None).unwrap();

    // 3. Count with prefer header
    let headers = hashmap! {
        "Prefer".to_string() => "count=exact".to_string(),
    };
    let op = parse("GET", "events", "created_at=gte.2024-01-01", None, Some(&headers)).unwrap();
}
```

---

## Implementation Strategy

### Week-by-Week Breakdown

**Week 1-2: Prefer Headers**
- [ ] Create AST types for Prefer options
- [ ] Implement nom-based parser for Prefer header
- [ ] Integrate with Operation enum
- [ ] Add SQL generation changes for return representation
- [ ] Write 30+ tests covering all prefer options

**Week 3-4: RPC Function Calls**
- [ ] Create RPC AST types
- [ ] Implement RPC parser (handle GET and POST)
- [ ] Build RPC SQL generation with named parameters
- [ ] Support filtering RPC results
- [ ] Write 25+ tests with real-world stored procedure examples

**Week 5: Resource Embedding**
- [ ] Extend parsers to handle select in mutations
- [ ] Research CTE approach for embedding
- [ ] Implement basic embedding (defer complex relations)
- [ ] Write 15+ tests

**Week 6: PUT Upsert**
- [ ] Add PUT method handling
- [ ] Auto-detect conflict columns from filters
- [ ] Write 10+ tests

**Week 7: Advanced ON CONFLICT**
- [ ] Extend OnConflict AST
- [ ] Parse WHERE clauses in conflict
- [ ] Update SQL generation
- [ ] Write 15+ tests

---

## Testing Philosophy

1. **Test Real PostgREST Examples**
   - Use actual PostgREST documentation examples
   - Test against real-world application patterns
   - Include edge cases from GitHub issues

2. **Parser Combinator Tests**
   - Test each nom parser in isolation
   - Test composed parsers
   - Test error cases and recovery

3. **Integration Tests**
   - Full request → SQL generation flow
   - Verify SQL output matches PostgREST
   - Test with actual Postgres (optional)

4. **Performance Tests**
   - Benchmark parsing performance
   - Ensure <100μs for typical requests
   - Profile hot paths

---

## Success Criteria

- [ ] 100% parity with PostgREST mutation features
- [ ] All real-world test scenarios passing
- [ ] <100μs parsing performance (95th percentile)
- [ ] 500+ total tests (currently 236)
- [ ] Documentation with examples for all features
- [ ] WASM bindings for all new features

---

## Notes

- Use nom parser combinators consistently
- Follow existing architectural patterns
- Maintain backward compatibility
- Keep safety validations
- Prioritize commonly-used features
- Test with real PostgreSQL when possible
