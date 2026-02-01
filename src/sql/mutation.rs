use crate::ast::{
    ConflictAction, DeleteParams, InsertParams, InsertValues, OnConflict, ResolvedTable,
    SelectItem, UpdateParams,
};
use crate::error::SqlError;
use crate::sql::{QueryBuilder, QueryResult};

impl QueryBuilder {
    /// Builds an INSERT query with schema-qualified table name
    pub fn build_insert(
        &mut self,
        resolved_table: &ResolvedTable,
        params: &InsertParams,
    ) -> Result<QueryResult, SqlError> {
        if params.values.is_empty() {
            return Err(SqlError::NoInsertValues);
        }

        self.tables.push(resolved_table.name.clone());

        // INSERT INTO "schema"."table"
        self.sql
            .push_str(&format!("INSERT INTO {}", resolved_table.qualified_name()));

        // Determine columns
        let columns = if let Some(ref cols) = params.columns {
            cols.clone()
        } else {
            params.values.get_columns()
        };

        // Column list
        self.sql.push_str(" (");
        for (i, col) in columns.iter().enumerate() {
            if i > 0 {
                self.sql.push_str(", ");
            }
            self.sql.push_str(&format!("\"{}\"", col));
        }
        self.sql.push(')');

        // VALUES clause
        self.build_values_clause(&params.values, &columns)?;

        // ON CONFLICT clause
        if let Some(ref on_conflict) = params.on_conflict {
            self.build_on_conflict_clause(on_conflict)?;
        }

        // RETURNING clause
        if let Some(ref returning) = params.returning {
            self.build_returning_clause(returning)?;
        }

        Ok(QueryResult {
            query: self.sql.clone(),
            params: self.params.clone(),
            tables: self.tables.clone(),
        })
    }

    /// Builds an UPDATE query with schema-qualified table name and safety validation
    pub fn build_update(
        &mut self,
        resolved_table: &ResolvedTable,
        params: &UpdateParams,
    ) -> Result<QueryResult, SqlError> {
        // Safety validation
        self.validate_update_safety(params)?;

        if params.set_values.is_empty() {
            return Err(SqlError::NoUpdateSet);
        }

        self.tables.push(resolved_table.name.clone());

        // UPDATE "schema"."table"
        self.sql
            .push_str(&format!("UPDATE {}", resolved_table.qualified_name()));

        // SET clause
        self.build_set_clause(&params.set_values)?;

        // WHERE clause
        if !params.filters.is_empty() {
            self.build_where_clause(&params.filters)?;
        }

        // ORDER BY clause
        if !params.order.is_empty() {
            self.build_order_clause(&params.order)?;
        }

        // LIMIT clause
        if let Some(limit) = params.limit {
            self.build_limit_clause(limit)?;
        }

        // RETURNING clause
        if let Some(ref returning) = params.returning {
            self.build_returning_clause(returning)?;
        }

        Ok(QueryResult {
            query: self.sql.clone(),
            params: self.params.clone(),
            tables: self.tables.clone(),
        })
    }

    /// Builds a DELETE query with schema-qualified table name and safety validation
    pub fn build_delete(
        &mut self,
        resolved_table: &ResolvedTable,
        params: &DeleteParams,
    ) -> Result<QueryResult, SqlError> {
        // Safety validation
        self.validate_delete_safety(params)?;

        self.tables.push(resolved_table.name.clone());

        // DELETE FROM "schema"."table"
        self.sql
            .push_str(&format!("DELETE FROM {}", resolved_table.qualified_name()));

        // WHERE clause
        if !params.filters.is_empty() {
            self.build_where_clause(&params.filters)?;
        }

        // ORDER BY clause
        if !params.order.is_empty() {
            self.build_order_clause(&params.order)?;
        }

        // LIMIT clause
        if let Some(limit) = params.limit {
            self.build_limit_clause(limit)?;
        }

        // RETURNING clause
        if let Some(ref returning) = params.returning {
            self.build_returning_clause(returning)?;
        }

        Ok(QueryResult {
            query: self.sql.clone(),
            params: self.params.clone(),
            tables: self.tables.clone(),
        })
    }

    fn build_values_clause(
        &mut self,
        values: &InsertValues,
        columns: &[String],
    ) -> Result<(), SqlError> {
        self.sql.push_str(" VALUES ");

        match values {
            InsertValues::Single(map) => {
                self.sql.push('(');
                for (i, col) in columns.iter().enumerate() {
                    if i > 0 {
                        self.sql.push_str(", ");
                    }
                    let value = map.get(col).unwrap_or(&serde_json::Value::Null);
                    let param = self.add_param(value.clone());
                    self.sql.push_str(&param);
                }
                self.sql.push(')');
            }
            InsertValues::Bulk(rows) => {
                for (row_idx, row) in rows.iter().enumerate() {
                    if row_idx > 0 {
                        self.sql.push_str(", ");
                    }
                    self.sql.push('(');
                    for (i, col) in columns.iter().enumerate() {
                        if i > 0 {
                            self.sql.push_str(", ");
                        }
                        let value = row.get(col).unwrap_or(&serde_json::Value::Null);
                        let param = self.add_param(value.clone());
                        self.sql.push_str(&param);
                    }
                    self.sql.push(')');
                }
            }
        }

        Ok(())
    }

    fn build_on_conflict_clause(&mut self, on_conflict: &OnConflict) -> Result<(), SqlError> {
        self.sql.push_str(" ON CONFLICT (");
        for (i, col) in on_conflict.columns.iter().enumerate() {
            if i > 0 {
                self.sql.push_str(", ");
            }
            self.sql.push_str(&format!("\"{}\"", col));
        }
        self.sql.push(')');

        // Add WHERE clause for partial unique index
        if let Some(ref where_conditions) = on_conflict.where_clause {
            self.sql.push_str(" WHERE ");
            for (i, condition) in where_conditions.iter().enumerate() {
                if i > 0 {
                    self.sql.push_str(" AND ");
                }
                let condition_sql = self.build_filter(condition)?;
                self.sql.push_str(&condition_sql);
            }
        }

        match on_conflict.action {
            ConflictAction::DoNothing => {
                self.sql.push_str(" DO NOTHING");
            }
            ConflictAction::DoUpdate => {
                self.sql.push_str(" DO UPDATE SET ");

                // Determine which columns to update
                let columns_to_update = if let Some(ref update_cols) = on_conflict.update_columns {
                    // Use specified columns
                    update_cols.clone()
                } else {
                    // Default: update all columns (same as conflict columns for now)
                    on_conflict.columns.clone()
                };

                // Update specified columns
                let mut first = true;
                for col in columns_to_update.iter() {
                    if !first {
                        self.sql.push_str(", ");
                    }
                    self.sql
                        .push_str(&format!("\"{}\" = EXCLUDED.\"{}\"", col, col));
                    first = false;
                }
            }
        }

        Ok(())
    }

    fn build_set_clause(
        &mut self,
        set_values: &std::collections::HashMap<String, serde_json::Value>,
    ) -> Result<(), SqlError> {
        self.sql.push_str(" SET ");

        let mut sorted_keys: Vec<&String> = set_values.keys().collect();
        sorted_keys.sort(); // Sort for deterministic output

        for (i, key) in sorted_keys.iter().enumerate() {
            if i > 0 {
                self.sql.push_str(", ");
            }
            let value = set_values.get(*key).unwrap();
            let param = self.add_param(value.clone());
            self.sql.push_str(&format!("\"{}\" = {}", key, param));
        }

        Ok(())
    }

    fn build_returning_clause(&mut self, items: &[SelectItem]) -> Result<(), SqlError> {
        self.sql.push_str(" RETURNING ");

        for (i, item) in items.iter().enumerate() {
            if i > 0 {
                self.sql.push_str(", ");
            }

            // Relations are not supported in RETURNING for now
            if matches!(item.item_type, crate::ast::ItemType::Relation) {
                return Err(SqlError::FailedToBuildSelectClause);
            }

            self.sql.push_str(&format!("\"{}\"", item.name));
            if let Some(ref alias) = item.alias {
                self.sql.push_str(&format!(" AS \"{}\"", alias));
            }
        }

        Ok(())
    }

    fn build_limit_clause(&mut self, limit: u64) -> Result<(), SqlError> {
        let param = self.add_param(serde_json::Value::Number(limit.into()));
        self.sql.push_str(&format!(" LIMIT {}", param));
        Ok(())
    }

    fn validate_update_safety(&self, params: &UpdateParams) -> Result<(), SqlError> {
        if params.filters.is_empty() {
            return Err(SqlError::UnsafeUpdate);
        }

        if params.limit.is_some() && params.order.is_empty() {
            return Err(SqlError::LimitWithoutOrder);
        }

        Ok(())
    }

    fn validate_delete_safety(&self, params: &DeleteParams) -> Result<(), SqlError> {
        if params.filters.is_empty() {
            return Err(SqlError::UnsafeDelete);
        }

        if params.limit.is_some() && params.order.is_empty() {
            return Err(SqlError::LimitWithoutOrder);
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ast::{Field, LogicCondition};
    use serde_json::json;
    use std::collections::HashMap;

    #[test]
    fn test_build_insert_single() {
        let mut builder = QueryBuilder::new();
        let table = ResolvedTable::new("public", "users");

        let mut values = HashMap::new();
        values.insert("name".to_string(), json!("Alice"));
        values.insert("age".to_string(), json!(30));

        let params = InsertParams::new(InsertValues::Single(values));
        let result = builder.build_insert(&table, &params).unwrap();

        assert!(result.query.contains("INSERT INTO \"public\".\"users\""));
        assert!(result.query.contains("\"age\""));
        assert!(result.query.contains("\"name\""));
        assert_eq!(result.params.len(), 2);
    }

    #[test]
    fn test_build_insert_bulk() {
        let mut builder = QueryBuilder::new();
        let table = ResolvedTable::new("public", "users");

        let mut row1 = HashMap::new();
        row1.insert("name".to_string(), json!("Alice"));
        let mut row2 = HashMap::new();
        row2.insert("name".to_string(), json!("Bob"));

        let params = InsertParams::new(InsertValues::Bulk(vec![row1, row2]));
        let result = builder.build_insert(&table, &params).unwrap();

        assert!(result.query.contains("VALUES"));
        assert_eq!(result.params.len(), 2);
    }

    #[test]
    fn test_build_insert_with_on_conflict() {
        let mut builder = QueryBuilder::new();
        let table = ResolvedTable::new("auth", "users");

        let mut values = HashMap::new();
        values.insert("email".to_string(), json!("alice@example.com"));

        let conflict = OnConflict::do_update(vec!["email".to_string()]);
        let params = InsertParams::new(InsertValues::Single(values)).with_on_conflict(conflict);

        let result = builder.build_insert(&table, &params).unwrap();

        assert!(result.query.contains("ON CONFLICT"));
        assert!(result.query.contains("DO UPDATE"));
        assert!(result.query.contains("EXCLUDED"));
    }

    #[test]
    fn test_build_update_with_filters() {
        let mut builder = QueryBuilder::new();
        let table = ResolvedTable::new("public", "users");

        let mut set_values = HashMap::new();
        set_values.insert("status".to_string(), json!("active"));

        let filter = LogicCondition::Filter(crate::ast::Filter::new(
            Field::new("id"),
            crate::ast::FilterOperator::Eq,
            crate::ast::FilterValue::Single("123".to_string()),
        ));

        let params = UpdateParams::new(set_values).with_filters(vec![filter]);
        let result = builder.build_update(&table, &params).unwrap();

        assert!(result.query.contains("UPDATE \"public\".\"users\""));
        assert!(result.query.contains("SET"));
        assert!(result.query.contains("WHERE"));
    }

    #[test]
    fn test_build_update_without_filters_fails() {
        let mut builder = QueryBuilder::new();
        let table = ResolvedTable::new("public", "users");

        let mut set_values = HashMap::new();
        set_values.insert("status".to_string(), json!("active"));

        let params = UpdateParams::new(set_values);
        let result = builder.build_update(&table, &params);

        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), SqlError::UnsafeUpdate));
    }

    #[test]
    fn test_build_delete_with_filters() {
        let mut builder = QueryBuilder::new();
        let table = ResolvedTable::new("public", "users");

        let filter = LogicCondition::Filter(crate::ast::Filter::new(
            Field::new("status"),
            crate::ast::FilterOperator::Eq,
            crate::ast::FilterValue::Single("deleted".to_string()),
        ));

        let params = DeleteParams::new().with_filters(vec![filter]);
        let result = builder.build_delete(&table, &params).unwrap();

        assert!(result.query.contains("DELETE FROM \"public\".\"users\""));
        assert!(result.query.contains("WHERE"));
    }

    #[test]
    fn test_build_delete_without_filters_fails() {
        let mut builder = QueryBuilder::new();
        let table = ResolvedTable::new("public", "users");

        let params = DeleteParams::new();
        let result = builder.build_delete(&table, &params);

        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), SqlError::UnsafeDelete));
    }

    #[test]
    fn test_update_limit_without_order_fails() {
        let mut builder = QueryBuilder::new();
        let table = ResolvedTable::new("public", "users");

        let mut set_values = HashMap::new();
        set_values.insert("status".to_string(), json!("active"));

        let filter = LogicCondition::Filter(crate::ast::Filter::new(
            Field::new("id"),
            crate::ast::FilterOperator::Eq,
            crate::ast::FilterValue::Single("123".to_string()),
        ));

        let params = UpdateParams::new(set_values)
            .with_filters(vec![filter])
            .with_limit(10);

        let result = builder.build_update(&table, &params);
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), SqlError::LimitWithoutOrder));
    }
}
