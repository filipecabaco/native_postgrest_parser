use crate::ast::{ResolvedTable, RpcParams};
use crate::error::SqlError;
use crate::sql::{QueryBuilder, QueryResult};

impl QueryBuilder {
    /// Builds an RPC (function call) query with schema-qualified function name
    ///
    /// Generates: SELECT * FROM "schema"."function_name"(arg1 := $1, arg2 := $2)
    /// PostgREST allows filtering, ordering, and pagination of function results
    pub fn build_rpc(
        &mut self,
        resolved_table: &ResolvedTable,
        params: &RpcParams,
    ) -> Result<QueryResult, SqlError> {
        self.tables.push(params.function_name.clone());

        // SELECT clause (use returning if specified, otherwise *)
        if let Some(ref returning) = params.returning {
            self.build_select_clause(returning)?;
        } else {
            self.sql.push_str("SELECT *");
        }

        // FROM schema.function_name(args)
        self.sql.push_str(" FROM ");
        self.sql.push_str(&format!(
            "\"{}\".\"{}\"(",
            resolved_table.schema, resolved_table.name
        ));

        // Build named arguments in deterministic order
        if !params.args.is_empty() {
            let mut sorted_args: Vec<(&String, &serde_json::Value)> = params.args.iter().collect();
            sorted_args.sort_by_key(|(k, _)| *k);

            for (i, (name, value)) in sorted_args.iter().enumerate() {
                if i > 0 {
                    self.sql.push_str(", ");
                }
                let param_placeholder = self.add_param((*value).clone());
                self.sql
                    .push_str(&format!("\"{}\" := {}", name, param_placeholder));
            }
        }

        self.sql.push(')');

        // WHERE clause for filtering function results
        if !params.filters.is_empty() {
            self.build_where_clause(&params.filters)?;
        }

        // ORDER BY clause
        if !params.order.is_empty() {
            self.build_order_clause(&params.order)?;
        }

        // LIMIT and OFFSET
        self.build_limit_offset(params.limit, params.offset)?;

        Ok(QueryResult {
            query: self.sql.clone(),
            params: self.params.clone(),
            tables: self.tables.clone(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ast::{LogicCondition, ResolvedTable, RpcParams};
    use crate::parser::{parse_filter, parse_order};
    use serde_json::Value;
    use std::collections::HashMap;

    #[test]
    fn test_build_rpc_simple() {
        let mut builder = QueryBuilder::new();
        let resolved = ResolvedTable::new("public", "get_user_profile");

        let mut args = HashMap::new();
        args.insert("user_id".to_string(), Value::Number(123.into()));

        let params = RpcParams::new("get_user_profile", args);

        let result = builder.build_rpc(&resolved, &params).unwrap();

        assert_eq!(
            result.query,
            r#"SELECT * FROM "public"."get_user_profile"("user_id" := $1)"#
        );
        assert_eq!(result.params.len(), 1);
    }

    #[test]
    fn test_build_rpc_no_args() {
        let mut builder = QueryBuilder::new();
        let resolved = ResolvedTable::new("api", "health_check");

        let params = RpcParams::new("health_check", HashMap::new());

        let result = builder.build_rpc(&resolved, &params).unwrap();

        assert_eq!(
            result.query,
            r#"SELECT * FROM "api"."health_check"()"#
        );
        assert!(result.params.is_empty());
    }

    #[test]
    fn test_build_rpc_multiple_args() {
        let mut builder = QueryBuilder::new();
        let resolved = ResolvedTable::new("public", "find_employees");

        let mut args = HashMap::new();
        args.insert("department".to_string(), Value::String("IT".to_string()));
        args.insert("min_salary".to_string(), Value::Number(50000.into()));

        let params = RpcParams::new("find_employees", args);

        let result = builder.build_rpc(&resolved, &params).unwrap();

        assert!(result.query.contains(r#""department" := $1"#));
        assert!(result.query.contains(r#""min_salary" := $2"#));
        assert_eq!(result.params.len(), 2);
    }

    #[test]
    fn test_build_rpc_with_filters() {
        let mut builder = QueryBuilder::new();
        let resolved = ResolvedTable::new("public", "get_recent_posts");

        let filter = parse_filter("status", "eq.published").unwrap();
        let params = RpcParams::new("get_recent_posts", HashMap::new())
            .with_filters(vec![LogicCondition::Filter(filter)]);

        let result = builder.build_rpc(&resolved, &params).unwrap();

        assert!(result.query.contains("WHERE"));
        assert!(result.query.contains(r#""status" = $1"#));
    }

    #[test]
    fn test_build_rpc_with_order() {
        let mut builder = QueryBuilder::new();
        let resolved = ResolvedTable::new("public", "get_posts");

        let params = RpcParams::new("get_posts", HashMap::new())
            .with_order(parse_order("created_at.desc").unwrap());

        let result = builder.build_rpc(&resolved, &params).unwrap();

        assert!(result.query.contains("ORDER BY"));
        assert!(result.query.contains(r#""created_at" DESC"#));
    }

    #[test]
    fn test_build_rpc_with_limit_offset() {
        let mut builder = QueryBuilder::new();
        let resolved = ResolvedTable::new("public", "list_users");

        let params = RpcParams::new("list_users", HashMap::new())
            .with_limit(10)
            .with_offset(20);

        let result = builder.build_rpc(&resolved, &params).unwrap();

        assert!(result.query.contains("LIMIT $1"));
        assert!(result.query.contains("OFFSET $2"));
        assert_eq!(result.params.len(), 2);
    }

    #[test]
    fn test_build_rpc_complex() {
        let mut builder = QueryBuilder::new();
        let resolved = ResolvedTable::new("api", "search_products");

        let mut args = HashMap::new();
        args.insert("query".to_string(), Value::String("laptop".to_string()));
        args.insert(
            "min_price".to_string(),
            Value::Number(500.into()),
        );

        let filter = parse_filter("in_stock", "eq.true").unwrap();
        let params = RpcParams::new("search_products", args)
            .with_filters(vec![LogicCondition::Filter(filter)])
            .with_order(parse_order("price.asc").unwrap())
            .with_limit(20);

        let result = builder.build_rpc(&resolved, &params).unwrap();

        assert!(result.query.contains(r#"FROM "api"."search_products"("#));
        assert!(result.query.contains("WHERE"));
        assert!(result.query.contains("ORDER BY"));
        assert!(result.query.contains("LIMIT"));
        assert!(result.params.len() >= 3);
    }
}
