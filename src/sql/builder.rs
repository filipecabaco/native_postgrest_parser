use crate::ast::*;
use crate::error::SqlError;

#[derive(Debug, Clone, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct QueryResult {
    pub query: String,
    pub params: Vec<serde_json::Value>,
    pub tables: Vec<String>,
}

pub struct QueryBuilder {
    pub sql: String,
    pub params: Vec<serde_json::Value>,
    pub param_index: usize,
    pub tables: Vec<String>,
}

impl Default for QueryBuilder {
    fn default() -> Self {
        Self::new()
    }
}

impl QueryBuilder {
    pub fn new() -> Self {
        Self {
            sql: String::new(),
            params: Vec::new(),
            param_index: 0,
            tables: Vec::new(),
        }
    }

    pub fn build_select(
        &mut self,
        table: &str,
        params: &ParsedParams,
    ) -> Result<QueryResult, SqlError> {
        self.tables.push(table.to_string());

        if let Some(select) = &params.select {
            self.build_select_clause(select)?;
        } else {
            self.sql.push_str("SELECT *");
        }

        self.build_from_clause(table)?;

        if !params.filters.is_empty() {
            self.build_where_clause(&params.filters)?;
        }

        if !params.order.is_empty() {
            self.build_order_clause(&params.order)?;
        }

        self.build_limit_offset(params.limit, params.offset)?;

        Ok(QueryResult {
            query: self.sql.clone(),
            params: self.params.clone(),
            tables: self.tables.clone(),
        })
    }

    fn build_select_clause(&mut self, items: &[SelectItem]) -> Result<(), SqlError> {
        if items.is_empty() {
            return Err(SqlError::NoSelectItems);
        }

        let columns: Vec<String> = items
            .iter()
            .map(|item| self.select_item_to_sql(item))
            .collect::<Result<Vec<_>, _>>()?;
        self.sql.push_str("SELECT ");
        self.sql.push_str(&columns.join(", "));
        Ok(())
    }

    fn select_item_to_sql(&self, item: &SelectItem) -> Result<String, SqlError> {
        match item.item_type {
            ItemType::Field => {
                if item.name == "*" {
                    Ok("*".to_string())
                } else {
                    let field_sql = self.field_to_sql(&Field::new(&item.name));
                    if let Some(alias) = &item.alias {
                        Ok(format!("{} AS {}", field_sql, self.quote_identifier(alias)))
                    } else {
                        Ok(field_sql)
                    }
                }
            }
            ItemType::Relation | ItemType::Spread => self.build_relation_sql(item),
        }
    }

    fn build_relation_sql(&self, item: &SelectItem) -> Result<String, SqlError> {
        let rel_alias = &item.name;

        if let Some(children) = &item.children {
            let child_columns: Vec<String> = children
                .iter()
                .filter(|c| c.item_type == ItemType::Field)
                .map(|c| {
                    if c.name == "*" {
                        format!("{}.*", rel_alias)
                    } else {
                        format!("{}.{}", rel_alias, self.quote_identifier(&c.name))
                    }
                })
                .collect();

            if child_columns.is_empty() {
                Ok(format!(
                    "(SELECT json_agg(row_to_json({}.*)) AS {} FROM {})",
                    rel_alias, rel_alias, rel_alias
                ))
            } else {
                Ok(format!(
                    "(SELECT json_agg(row_to_json({})) AS {} FROM {})",
                    child_columns.join(", "),
                    rel_alias,
                    rel_alias
                ))
            }
        } else {
            Ok(format!(
                "(SELECT json_agg(row_to_json({}.*)) AS {} FROM {})",
                rel_alias, rel_alias, rel_alias
            ))
        }
    }

    fn build_from_clause(&mut self, table: &str) -> Result<(), SqlError> {
        if table.is_empty() {
            return Err(SqlError::EmptyTableName);
        }

        self.sql.push_str(" FROM ");
        self.sql.push_str(&self.quote_identifier(table));
        Ok(())
    }

    pub fn build_where_clause(&mut self, filters: &[LogicCondition]) -> Result<(), SqlError> {
        self.sql.push_str(" WHERE ");

        let clauses: Result<Vec<String>, SqlError> = filters
            .iter()
            .map(|filter| self.build_filter(filter))
            .collect();

        match clauses {
            Ok(clauses) => {
                self.sql.push_str(&clauses.join(" AND "));
                Ok(())
            }
            Err(e) => Err(e),
        }
    }

    fn build_filter(&mut self, condition: &LogicCondition) -> Result<String, SqlError> {
        match condition {
            LogicCondition::Filter(filter) => self.build_single_filter(filter),
            LogicCondition::Logic(tree) => self.build_logic_tree(tree),
        }
    }

    fn build_single_filter(&mut self, filter: &Filter) -> Result<String, SqlError> {
        let field_sql = self.field_to_sql(&filter.field);
        let (clause, _) = self.operator_to_sql(&field_sql, filter)?;
        Ok(clause)
    }

    fn build_logic_tree(&mut self, tree: &LogicTree) -> Result<String, SqlError> {
        let joiner = if tree.operator == LogicOperator::And {
            " AND "
        } else {
            " OR "
        };

        let conditions: Result<Vec<String>, SqlError> = tree
            .conditions
            .iter()
            .map(|c| self.build_filter(c))
            .collect();

        let conditions_sql = conditions?.join(joiner);

        if tree.negated {
            Ok(format!("NOT ({})", conditions_sql))
        } else {
            Ok(format!("({})", conditions_sql))
        }
    }

    fn build_order_clause(&mut self, order_terms: &[OrderTerm]) -> Result<(), SqlError> {
        let clauses: Result<Vec<String>, SqlError> = order_terms
            .iter()
            .map(|term| self.order_term_to_sql(term))
            .collect();

        match clauses {
            Ok(clauses) => {
                self.sql.push_str(" ORDER BY ");
                self.sql.push_str(&clauses.join(", "));
                Ok(())
            }
            Err(e) => Err(e),
        }
    }

    fn order_term_to_sql(&self, term: &OrderTerm) -> Result<String, SqlError> {
        let field_sql = self.field_to_sql(&term.field);
        let dir_sql = if term.direction == Direction::Desc {
            " DESC"
        } else {
            " ASC"
        };

        let nulls_sql = match term.nulls {
            Some(Nulls::First) => " NULLS FIRST",
            Some(Nulls::Last) => " NULLS LAST",
            None => "",
        };

        Ok(format!("{}{}{}", field_sql, dir_sql, nulls_sql))
    }

    fn build_limit_offset(
        &mut self,
        limit: Option<u64>,
        offset: Option<u64>,
    ) -> Result<(), SqlError> {
        match (limit, offset) {
            (Some(lim), Some(off)) => {
                let lim_ref = self.add_param(serde_json::Value::Number(lim.into()));
                let off_ref = self.add_param(serde_json::Value::Number(off.into()));
                self.sql
                    .push_str(&format!(" LIMIT {} OFFSET {}", lim_ref, off_ref));
            }
            (Some(lim), None) => {
                let lim_ref = self.add_param(serde_json::Value::Number(lim.into()));
                self.sql.push_str(&format!(" LIMIT {}", lim_ref));
            }
            (None, Some(off)) => {
                let off_ref = self.add_param(serde_json::Value::Number(off.into()));
                self.sql.push_str(&format!(" OFFSET {}", off_ref));
            }
            (None, None) => {}
        }

        Ok(())
    }

    fn operator_to_sql(
        &mut self,
        field: &str,
        filter: &Filter,
    ) -> Result<(String, usize), SqlError> {
        let (op_sql, value) = match (&filter.operator, &filter.quantifier, &filter.value) {
            // Eq with quantifiers
            (FilterOperator::Eq, Some(Quantifier::Any), FilterValue::List(ref _vals)) => {
                let param_ref = self.add_param(filter.value.to_json());
                (format!("{} = ANY({})", field, param_ref), param_ref.len())
            }
            (FilterOperator::Eq, Some(Quantifier::All), FilterValue::List(ref _vals)) => {
                let param_ref = self.add_param(filter.value.to_json());
                (format!("{} = ALL({})", field, param_ref), param_ref.len())
            }
            (FilterOperator::Eq, _, FilterValue::Single(ref _val)) => {
                let param_ref = self.add_param(filter.value.to_json());
                let op_sql = if filter.negated { "<>" } else { "=" };
                (format!("{} {} {}", field, op_sql, param_ref), 1)
            }

            // Neq
            (FilterOperator::Neq, _, FilterValue::Single(_)) => {
                let param_ref = self.add_param(filter.value.to_json());
                let op_sql = if filter.negated { "=" } else { "<>" };
                (format!("{} {} {}", field, op_sql, param_ref), 1)
            }

            // Comparison operators
            (FilterOperator::Gt, _, FilterValue::Single(_)) => {
                let param_ref = self.add_param(filter.value.to_json());
                let op_sql = if filter.negated { "<=" } else { ">" };
                (format!("{} {} {}", field, op_sql, param_ref), 1)
            }
            (FilterOperator::Gte, _, FilterValue::Single(_)) => {
                let param_ref = self.add_param(filter.value.to_json());
                let op_sql = if filter.negated { "<" } else { ">=" };
                (format!("{} {} {}", field, op_sql, param_ref), 1)
            }
            (FilterOperator::Lt, _, FilterValue::Single(_)) => {
                let param_ref = self.add_param(filter.value.to_json());
                let op_sql = if filter.negated { ">=" } else { "<" };
                (format!("{} {} {}", field, op_sql, param_ref), 1)
            }
            (FilterOperator::Lte, _, FilterValue::Single(_)) => {
                let param_ref = self.add_param(filter.value.to_json());
                let op_sql = if filter.negated { ">" } else { "<=" };
                (format!("{} {} {}", field, op_sql, param_ref), 1)
            }

            // IN operator
            (FilterOperator::In, _, FilterValue::List(_)) => {
                let param_ref = self.add_param(filter.value.to_json());
                let not_prefix = if filter.negated { "NOT " } else { "" };
                (format!("{}{} = ANY({})", field, not_prefix, param_ref), 1)
            }

            // IS operator
            (FilterOperator::Is, _, FilterValue::Single(ref val)) => {
                let clause = self.build_is_clause(field, val, filter.negated)?;
                (clause, 0)
            }

            // LIKE/ILIKE operators
            (FilterOperator::Like | FilterOperator::Ilike, Some(Quantifier::Any), FilterValue::List(_)) => {
                let param_ref = self.add_param(filter.value.to_json());
                let not_prefix = if filter.negated { "NOT " } else { "" };
                let op_str = if filter.operator == FilterOperator::Like {
                    "LIKE"
                } else {
                    "ILIKE"
                };
                (format!("{}{} {} ANY({})", field, not_prefix, op_str, param_ref), 1)
            }
            (FilterOperator::Like | FilterOperator::Ilike, _, FilterValue::Single(_)) => {
                let param_ref = self.add_param(filter.value.to_json());
                let not_prefix = if filter.negated { "NOT " } else { "" };
                let op_str = if filter.operator == FilterOperator::Like {
                    "LIKE"
                } else {
                    "ILIKE"
                };
                (format!("{}{} {} {}", field, not_prefix, op_str, param_ref), 1)
            }

            // Regex match operators
            (FilterOperator::Match, _, FilterValue::Single(_)) => {
                let param_ref = self.add_param(filter.value.to_json());
                let op_sql = if filter.negated { "!~" } else { "~" };
                (format!("{} {} {}", field, op_sql, param_ref), 1)
            }
            (FilterOperator::Imatch, _, FilterValue::Single(_)) => {
                let param_ref = self.add_param(filter.value.to_json());
                let op_sql = if filter.negated { "!~*" } else { "~*" };
                (format!("{} {} {}", field, op_sql, param_ref), 1)
            }

            // Full-text search operators
            (FilterOperator::Fts | FilterOperator::Plfts | FilterOperator::Phfts | FilterOperator::Wfts, _, FilterValue::Single(_)) => {
                let param_ref = self.add_param(filter.value.to_json());
                let lang = filter.language.as_deref().unwrap_or("english");
                let ts_fn = match filter.operator {
                    FilterOperator::Fts | FilterOperator::Plfts => "plainto_tsquery",
                    FilterOperator::Phfts => "phraseto_tsquery",
                    FilterOperator::Wfts => "websearch_to_tsquery",
                    _ => unreachable!(),
                };
                let not_prefix = if filter.negated { "NOT " } else { "" };
                (format!("{}to_tsvector('{}', {}) @@ {}('{}', {})",
                    not_prefix, lang, field, ts_fn, lang, param_ref), 1)
            }

            // Array/Range operators
            (FilterOperator::Cs, _, FilterValue::Single(_)) => {
                let param_ref = self.add_param(filter.value.to_json());
                let op_sql = if filter.negated { "NOT @>" } else { "@>" };
                (format!("{} {} {}", field, op_sql, param_ref), 1)
            }
            (FilterOperator::Cd, _, FilterValue::Single(_)) => {
                let param_ref = self.add_param(filter.value.to_json());
                let op_sql = if filter.negated { "NOT <@" } else { "<@" };
                (format!("{} {} {}", field, op_sql, param_ref), 1)
            }
            (FilterOperator::Ov, _, FilterValue::List(_)) => {
                let param_ref = self.add_param(filter.value.to_json());
                let not_prefix = if filter.negated { "NOT " } else { "" };
                (format!("{}{} && {}", field, not_prefix, param_ref), 1)
            }

            // Range operators
            (FilterOperator::Sl, _, FilterValue::Single(_)) => {
                let param_ref = self.add_param(filter.value.to_json());
                let op_sql = if filter.negated { "NOT <<" } else { "<<" };
                (format!("{} {} {}", field, op_sql, param_ref), 1)
            }
            (FilterOperator::Sr, _, FilterValue::Single(_)) => {
                let param_ref = self.add_param(filter.value.to_json());
                let op_sql = if filter.negated { "NOT >>" } else { ">>" };
                (format!("{} {} {}", field, op_sql, param_ref), 1)
            }
            (FilterOperator::Nxl, _, FilterValue::Single(_)) => {
                let param_ref = self.add_param(filter.value.to_json());
                let op_sql = if filter.negated { "NOT &<" } else { "&<" };
                (format!("{} {} {}", field, op_sql, param_ref), 1)
            }
            (FilterOperator::Nxr, _, FilterValue::Single(_)) => {
                let param_ref = self.add_param(filter.value.to_json());
                let op_sql = if filter.negated { "NOT &>" } else { "&>" };
                (format!("{} {} {}", field, op_sql, param_ref), 1)
            }
            (FilterOperator::Adj, _, FilterValue::Single(_)) => {
                let param_ref = self.add_param(filter.value.to_json());
                let op_sql = if filter.negated { "NOT -|-" } else { "-|-" };
                (format!("{} {} {}", field, op_sql, param_ref), 1)
            }

            // Fallback
            _ => {
                return Err(SqlError::InvalidParameter(format!(
                    "unsupported operator/value combination: {:?} with {:?}",
                    filter.operator, filter.value
                )));
            }
        };

        Ok((op_sql, value))
    }

    fn build_is_clause(&self, field: &str, value: &str, negated: bool) -> Result<String, SqlError> {
        match (value.to_lowercase().as_str(), negated) {
            ("null", false) => Ok(format!("{} IS NULL", field)),
            ("null", true) => Ok(format!("{} IS NOT NULL", field)),
            ("not_null", false) => Ok(format!("{} IS NOT NULL", field)),
            ("not_null", true) => Ok(format!("{} IS NULL", field)),
            ("true", false) => Ok(format!("{} IS TRUE", field)),
            ("true", true) => Ok(format!("{} IS NOT TRUE", field)),
            ("false", false) => Ok(format!("{} IS FALSE", field)),
            ("false", true) => Ok(format!("{} IS NOT FALSE", field)),
            ("unknown", false) => Ok(format!("{} IS UNKNOWN", field)),
            ("unknown", true) => Ok(format!("{} IS NOT UNKNOWN", field)),
            _ => Err(SqlError::InvalidParameter(format!(
                "invalid IS value: {}",
                value
            ))),
        }
    }

    fn field_to_sql(&self, field: &Field) -> String {
        let base = self.quote_identifier(&field.name);

        match (&field.json_path[..], &field.cast) {
            ([], None) => base,
            ([], Some(cast)) => format!("{}::{}", base, cast),
            (json_path, cast_opt) => {
                let json_path_sql: Vec<String> = json_path
                    .iter()
                    .map(|op| match op {
                        JsonOp::Arrow(key) => format!("->'{}'", key),
                        JsonOp::DoubleArrow(key) => format!("->>'{}'", key),
                        JsonOp::ArrayIndex(idx) => format!("->{}", idx),
                    })
                    .collect();

                let json_path_str = json_path_sql.join("");

                if let Some(cast) = cast_opt {
                    format!("({}{})::{}", base, json_path_str, cast)
                } else {
                    format!("{}{}", base, json_path_str)
                }
            }
        }
    }

    fn add_param(&mut self, value: serde_json::Value) -> String {
        let idx = self.param_index + 1;
        self.param_index = idx;
        self.params.push(value);
        format!("${}", idx)
    }

    fn quote_identifier(&self, name: &str) -> String {
        let escaped = name.replace('"', "\"\"");
        format!("\"{}\"", escaped)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_query_builder_new() {
        let builder = QueryBuilder::new();
        assert!(builder.sql.is_empty());
        assert!(builder.params.is_empty());
        assert_eq!(builder.param_index, 0);
    }

    #[test]
    fn test_add_param() {
        let mut builder = QueryBuilder::new();
        let param_ref = builder.add_param(serde_json::Value::String("test".to_string()));
        assert_eq!(param_ref, "$1");
        assert_eq!(builder.params.len(), 1);
        assert_eq!(builder.param_index, 1);
    }

    #[test]
    fn test_quote_identifier() {
        let builder = QueryBuilder::new();
        assert_eq!(builder.quote_identifier("id"), "\"id\"");
        assert_eq!(builder.quote_identifier("user\"id"), "\"user\"\"id\"");
    }

    #[test]
    fn test_field_to_sql_simple() {
        let builder = QueryBuilder::new();
        let field = Field::new("id");
        assert_eq!(builder.field_to_sql(&field), "\"id\"");
    }

    #[test]
    fn test_field_to_sql_with_json_path() {
        let builder = QueryBuilder::new();
        let field = Field::new("data").with_json_path(vec![JsonOp::Arrow("key".to_string())]);
        let sql = builder.field_to_sql(&field);
        assert!(sql.contains("\"data\"->'key'"));
    }

    #[test]
    fn test_field_to_sql_with_cast() {
        let builder = QueryBuilder::new();
        let field = Field::new("price").with_cast("numeric");
        assert_eq!(builder.field_to_sql(&field), "\"price\"::numeric");
    }

    #[test]
    fn test_build_is_clause() {
        let builder = QueryBuilder::new();

        assert_eq!(
            builder.build_is_clause("\"id\"", "null", false).unwrap(),
            "\"id\" IS NULL"
        );

        assert_eq!(
            builder.build_is_clause("\"id\"", "null", true).unwrap(),
            "\"id\" IS NOT NULL"
        );
    }

    #[test]
    fn test_operator_to_sql_comparison() {
        let mut builder = QueryBuilder::new();

        // GT operator
        let filter = Filter::new(Field::new("age"), FilterOperator::Gt, FilterValue::Single("18".to_string()));
        let (sql, _) = builder.operator_to_sql("\"age\"", &filter).unwrap();
        assert_eq!(sql, "\"age\" > $1");
        assert_eq!(builder.params[0], serde_json::Value::String("18".to_string()));

        // GTE operator
        builder = QueryBuilder::new();
        let filter = Filter::new(Field::new("age"), FilterOperator::Gte, FilterValue::Single("18".to_string()));
        let (sql, _) = builder.operator_to_sql("\"age\"", &filter).unwrap();
        assert_eq!(sql, "\"age\" >= $1");

        // LT operator
        builder = QueryBuilder::new();
        let filter = Filter::new(Field::new("age"), FilterOperator::Lt, FilterValue::Single("65".to_string()));
        let (sql, _) = builder.operator_to_sql("\"age\"", &filter).unwrap();
        assert_eq!(sql, "\"age\" < $1");

        // LTE operator
        builder = QueryBuilder::new();
        let filter = Filter::new(Field::new("age"), FilterOperator::Lte, FilterValue::Single("65".to_string()));
        let (sql, _) = builder.operator_to_sql("\"age\"", &filter).unwrap();
        assert_eq!(sql, "\"age\" <= $1");

        // NEQ operator
        builder = QueryBuilder::new();
        let filter = Filter::new(Field::new("status"), FilterOperator::Neq, FilterValue::Single("active".to_string()));
        let (sql, _) = builder.operator_to_sql("\"status\"", &filter).unwrap();
        assert_eq!(sql, "\"status\" <> $1");
    }

    #[test]
    fn test_operator_to_sql_in_operator() {
        let mut builder = QueryBuilder::new();
        let filter = Filter::new(
            Field::new("status"),
            FilterOperator::In,
            FilterValue::List(vec!["active".to_string(), "pending".to_string()]),
        );
        let (sql, _) = builder.operator_to_sql("\"status\"", &filter).unwrap();
        assert_eq!(sql, "\"status\" = ANY($1)");
        assert!(matches!(builder.params[0], serde_json::Value::Array(_)));
    }

    #[test]
    fn test_operator_to_sql_pattern_matching() {
        let mut builder = QueryBuilder::new();

        // Match operator (regex)
        let filter = Filter::new(Field::new("name"), FilterOperator::Match, FilterValue::Single("^John".to_string()));
        let (sql, _) = builder.operator_to_sql("\"name\"", &filter).unwrap();
        assert_eq!(sql, "\"name\" ~ $1");

        // Imatch operator (case-insensitive regex)
        builder = QueryBuilder::new();
        let filter = Filter::new(Field::new("name"), FilterOperator::Imatch, FilterValue::Single("^john".to_string()));
        let (sql, _) = builder.operator_to_sql("\"name\"", &filter).unwrap();
        assert_eq!(sql, "\"name\" ~* $1");
    }

    #[test]
    fn test_operator_to_sql_fts() {
        let mut builder = QueryBuilder::new();

        // FTS operator without language (defaults to english)
        let filter = Filter::new(Field::new("content"), FilterOperator::Fts, FilterValue::Single("search".to_string()));
        let (sql, _) = builder.operator_to_sql("\"content\"", &filter).unwrap();
        assert_eq!(sql, "to_tsvector('english', \"content\") @@ plainto_tsquery('english', $1)");

        // FTS operator with custom language
        builder = QueryBuilder::new();
        let mut filter = Filter::new(Field::new("content"), FilterOperator::Fts, FilterValue::Single("search".to_string()));
        filter.language = Some("french".to_string());
        let (sql, _) = builder.operator_to_sql("\"content\"", &filter).unwrap();
        assert_eq!(sql, "to_tsvector('french', \"content\") @@ plainto_tsquery('french', $1)");

        // PHFTS operator (phrase search)
        builder = QueryBuilder::new();
        let filter = Filter::new(Field::new("content"), FilterOperator::Phfts, FilterValue::Single("search phrase".to_string()));
        let (sql, _) = builder.operator_to_sql("\"content\"", &filter).unwrap();
        assert_eq!(sql, "to_tsvector('english', \"content\") @@ phraseto_tsquery('english', $1)");

        // WFTS operator (websearch)
        builder = QueryBuilder::new();
        let filter = Filter::new(Field::new("content"), FilterOperator::Wfts, FilterValue::Single("search query".to_string()));
        let (sql, _) = builder.operator_to_sql("\"content\"", &filter).unwrap();
        assert_eq!(sql, "to_tsvector('english', \"content\") @@ websearch_to_tsquery('english', $1)");
    }

    #[test]
    fn test_operator_to_sql_array_operators() {
        let mut builder = QueryBuilder::new();

        // CS operator (contains)
        let filter = Filter::new(Field::new("tags"), FilterOperator::Cs, FilterValue::Single("{rust}".to_string()));
        let (sql, _) = builder.operator_to_sql("\"tags\"", &filter).unwrap();
        assert_eq!(sql, "\"tags\" @> $1");

        // CD operator (contained in)
        builder = QueryBuilder::new();
        let filter = Filter::new(Field::new("tags"), FilterOperator::Cd, FilterValue::Single("{rust,elixir}".to_string()));
        let (sql, _) = builder.operator_to_sql("\"tags\"", &filter).unwrap();
        assert_eq!(sql, "\"tags\" <@ $1");

        // OV operator (overlaps)
        builder = QueryBuilder::new();
        let filter = Filter::new(
            Field::new("tags"),
            FilterOperator::Ov,
            FilterValue::List(vec!["rust".to_string(), "elixir".to_string()]),
        );
        let (sql, _) = builder.operator_to_sql("\"tags\"", &filter).unwrap();
        assert_eq!(sql, "\"tags\" && $1");
    }

    #[test]
    fn test_operator_to_sql_range_operators() {
        let mut builder = QueryBuilder::new();

        // SL operator (strictly left)
        let filter = Filter::new(Field::new("range"), FilterOperator::Sl, FilterValue::Single("[1,10)".to_string()));
        let (sql, _) = builder.operator_to_sql("\"range\"", &filter).unwrap();
        assert_eq!(sql, "\"range\" << $1");

        // SR operator (strictly right)
        builder = QueryBuilder::new();
        let filter = Filter::new(Field::new("range"), FilterOperator::Sr, FilterValue::Single("[1,10)".to_string()));
        let (sql, _) = builder.operator_to_sql("\"range\"", &filter).unwrap();
        assert_eq!(sql, "\"range\" >> $1");

        // NXL operator (does not extend to right)
        builder = QueryBuilder::new();
        let filter = Filter::new(Field::new("range"), FilterOperator::Nxl, FilterValue::Single("[1,10)".to_string()));
        let (sql, _) = builder.operator_to_sql("\"range\"", &filter).unwrap();
        assert_eq!(sql, "\"range\" &< $1");

        // NXR operator (does not extend to left)
        builder = QueryBuilder::new();
        let filter = Filter::new(Field::new("range"), FilterOperator::Nxr, FilterValue::Single("[1,10)".to_string()));
        let (sql, _) = builder.operator_to_sql("\"range\"", &filter).unwrap();
        assert_eq!(sql, "\"range\" &> $1");

        // ADJ operator (adjacent)
        builder = QueryBuilder::new();
        let filter = Filter::new(Field::new("range"), FilterOperator::Adj, FilterValue::Single("[1,10)".to_string()));
        let (sql, _) = builder.operator_to_sql("\"range\"", &filter).unwrap();
        assert_eq!(sql, "\"range\" -|- $1");
    }

    #[test]
    fn test_operator_to_sql_negated() {
        let mut builder = QueryBuilder::new();

        // Negated EQ becomes NEQ
        let mut filter = Filter::new(Field::new("status"), FilterOperator::Eq, FilterValue::Single("active".to_string()));
        filter.negated = true;
        let (sql, _) = builder.operator_to_sql("\"status\"", &filter).unwrap();
        assert_eq!(sql, "\"status\" <> $1");

        // Negated GT becomes LTE
        builder = QueryBuilder::new();
        let mut filter = Filter::new(Field::new("age"), FilterOperator::Gt, FilterValue::Single("18".to_string()));
        filter.negated = true;
        let (sql, _) = builder.operator_to_sql("\"age\"", &filter).unwrap();
        assert_eq!(sql, "\"age\" <= $1");

        // Negated FTS
        builder = QueryBuilder::new();
        let mut filter = Filter::new(Field::new("content"), FilterOperator::Fts, FilterValue::Single("search".to_string()));
        filter.negated = true;
        let (sql, _) = builder.operator_to_sql("\"content\"", &filter).unwrap();
        assert_eq!(sql, "NOT to_tsvector('english', \"content\") @@ plainto_tsquery('english', $1)");
    }

    #[test]
    fn test_operator_to_sql_with_quantifiers() {
        let mut builder = QueryBuilder::new();

        // EQ with ANY quantifier
        let mut filter = Filter::new(
            Field::new("status"),
            FilterOperator::Eq,
            FilterValue::List(vec!["active".to_string(), "pending".to_string()]),
        );
        filter.quantifier = Some(Quantifier::Any);
        let (sql, _) = builder.operator_to_sql("\"status\"", &filter).unwrap();
        assert_eq!(sql, "\"status\" = ANY($1)");

        // EQ with ALL quantifier
        builder = QueryBuilder::new();
        let mut filter = Filter::new(
            Field::new("status"),
            FilterOperator::Eq,
            FilterValue::List(vec!["active".to_string()]),
        );
        filter.quantifier = Some(Quantifier::All);
        let (sql, _) = builder.operator_to_sql("\"status\"", &filter).unwrap();
        assert_eq!(sql, "\"status\" = ALL($1)");
    }
}
