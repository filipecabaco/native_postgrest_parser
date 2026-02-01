use super::{LogicCondition, OrderTerm, ParsedParams, PreferOptions, RpcParams, SelectItem};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Represents all supported PostgREST operations (GET, POST, PATCH, DELETE, PUT, RPC).
///
/// Each variant contains operation-specific parameters and optional `Prefer` header preferences.
///
/// # Examples
///
/// ```
/// use postgrest_parser::{parse, Operation};
///
/// // Parse a SELECT operation
/// let op = parse("GET", "users", "id=eq.123", None, None).unwrap();
/// match op {
///     Operation::Select(params, prefer) => {
///         assert!(params.has_filters());
///         assert!(prefer.is_none());
///     }
///     _ => panic!("Expected Select"),
/// }
///
/// // Parse an INSERT operation
/// let body = r#"{"name": "Alice", "email": "alice@example.com"}"#;
/// let op = parse("POST", "users", "", Some(body), None).unwrap();
/// match op {
///     Operation::Insert(params, _) => {
///         assert!(!params.values.is_empty());
///     }
///     _ => panic!("Expected Insert"),
/// }
/// ```
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Operation {
    /// SELECT query with parsed parameters and optional Prefer header
    Select(ParsedParams, Option<PreferOptions>),
    /// INSERT query with values, conflict resolution, and optional Prefer header
    Insert(InsertParams, Option<PreferOptions>),
    /// UPDATE query with SET values, filters, and optional Prefer header
    Update(UpdateParams, Option<PreferOptions>),
    /// DELETE query with filters and optional Prefer header
    Delete(DeleteParams, Option<PreferOptions>),
    /// RPC function call with arguments, filters, and optional Prefer header
    Rpc(RpcParams, Option<PreferOptions>),
}

/// A schema-qualified table name resolved from headers or defaulting to `public`.
///
/// PostgREST allows specifying schemas via `Content-Profile` and `Accept-Profile` headers.
/// This struct represents a fully qualified table reference.
///
/// # Examples
///
/// ```
/// use postgrest_parser::ResolvedTable;
///
/// let table = ResolvedTable::new("auth", "users");
/// assert_eq!(table.schema, "auth");
/// assert_eq!(table.name, "users");
/// assert_eq!(table.qualified_name(), r#""auth"."users""#);
/// ```
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ResolvedTable {
    /// Schema name (e.g., "public", "auth", "api")
    pub schema: String,
    /// Table name
    pub name: String,
}

impl ResolvedTable {
    /// Creates a new schema-qualified table reference.
    pub fn new(schema: impl Into<String>, name: impl Into<String>) -> Self {
        Self {
            schema: schema.into(),
            name: name.into(),
        }
    }

    /// Returns the SQL-quoted qualified name: `"schema"."table"`
    pub fn qualified_name(&self) -> String {
        format!("\"{}\".\"{}\"", self.schema, self.name)
    }
}

/// Parameters for an INSERT operation.
///
/// Supports single and bulk inserts, conflict resolution, and returning specific columns.
///
/// # Examples
///
/// ```
/// use postgrest_parser::{InsertParams, InsertValues, OnConflict};
/// use std::collections::HashMap;
/// use serde_json::json;
///
/// // Single insert
/// let mut values = HashMap::new();
/// values.insert("name".to_string(), json!("Alice"));
/// values.insert("email".to_string(), json!("alice@example.com"));
///
/// let params = InsertParams::new(InsertValues::Single(values));
/// assert_eq!(params.values.len(), 1);
///
/// // With conflict resolution
/// let mut values = HashMap::new();
/// values.insert("email".to_string(), json!("alice@example.com"));
///
/// let params = InsertParams::new(InsertValues::Single(values))
///     .with_on_conflict(OnConflict::do_update(vec!["email".to_string()]));
/// assert!(params.on_conflict.is_some());
/// ```
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct InsertParams {
    /// Values to insert (single or bulk)
    pub values: InsertValues,
    /// Optional explicit column list (if None, derived from values)
    pub columns: Option<Vec<String>>,
    /// Optional conflict resolution strategy
    pub on_conflict: Option<OnConflict>,
    /// Optional RETURNING clause columns
    pub returning: Option<Vec<SelectItem>>,
}

impl InsertParams {
    /// Creates new insert parameters with the given values.
    pub fn new(values: InsertValues) -> Self {
        Self {
            values,
            columns: None,
            on_conflict: None,
            returning: None,
        }
    }

    /// Specifies explicit column order for the insert.
    pub fn with_columns(mut self, columns: Vec<String>) -> Self {
        self.columns = Some(columns);
        self
    }

    /// Adds conflict resolution behavior (ON CONFLICT clause).
    pub fn with_on_conflict(mut self, on_conflict: OnConflict) -> Self {
        self.on_conflict = Some(on_conflict);
        self
    }

    /// Specifies columns to return after insert (RETURNING clause).
    pub fn with_returning(mut self, returning: Vec<SelectItem>) -> Self {
        self.returning = Some(returning);
        self
    }
}

/// Insert values - either a single row or multiple rows (bulk insert).
///
/// # Examples
///
/// ```
/// use postgrest_parser::InsertValues;
/// use std::collections::HashMap;
/// use serde_json::json;
///
/// // Single row insert
/// let mut row = HashMap::new();
/// row.insert("name".to_string(), json!("Alice"));
/// let single = InsertValues::Single(row);
/// assert_eq!(single.len(), 1);
///
/// // Bulk insert
/// let mut row1 = HashMap::new();
/// row1.insert("name".to_string(), json!("Alice"));
/// let mut row2 = HashMap::new();
/// row2.insert("name".to_string(), json!("Bob"));
/// let bulk = InsertValues::Bulk(vec![row1, row2]);
/// assert_eq!(bulk.len(), 2);
/// ```
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum InsertValues {
    /// Single row insert
    Single(HashMap<String, serde_json::Value>),
    /// Bulk insert with multiple rows
    Bulk(Vec<HashMap<String, serde_json::Value>>),
}

impl InsertValues {
    /// Returns the number of rows to insert.
    pub fn len(&self) -> usize {
        match self {
            InsertValues::Single(_) => 1,
            InsertValues::Bulk(rows) => rows.len(),
        }
    }

    /// Returns true if there are no values to insert.
    pub fn is_empty(&self) -> bool {
        match self {
            InsertValues::Single(map) => map.is_empty(),
            InsertValues::Bulk(rows) => rows.is_empty(),
        }
    }

    /// Extracts column names from the values (sorted alphabetically).
    pub fn get_columns(&self) -> Vec<String> {
        match self {
            InsertValues::Single(map) => {
                let mut cols: Vec<String> = map.keys().cloned().collect();
                cols.sort();
                cols
            }
            InsertValues::Bulk(rows) => {
                if let Some(first) = rows.first() {
                    let mut cols: Vec<String> = first.keys().cloned().collect();
                    cols.sort();
                    cols
                } else {
                    Vec::new()
                }
            }
        }
    }
}

/// ON CONFLICT clause for upsert operations.
///
/// Supports both basic conflict resolution and advanced features like partial unique indexes
/// and selective column updates.
///
/// # Examples
///
/// ```
/// use postgrest_parser::{OnConflict, ConflictAction};
///
/// // Basic upsert: ON CONFLICT (email) DO UPDATE SET ...
/// let conflict = OnConflict::do_update(vec!["email".to_string()]);
/// assert_eq!(conflict.action, ConflictAction::DoUpdate);
///
/// // Ignore conflicts: ON CONFLICT (id) DO NOTHING
/// let conflict = OnConflict::do_nothing(vec!["id".to_string()]);
/// assert_eq!(conflict.action, ConflictAction::DoNothing);
///
/// // Partial unique index: ON CONFLICT (email) WHERE deleted_at IS NULL
/// use postgrest_parser::{parse_filter, LogicCondition};
/// let filter = parse_filter("deleted_at", "is.null").unwrap();
/// let conflict = OnConflict::do_update(vec!["email".to_string()])
///     .with_where_clause(vec![LogicCondition::Filter(filter)]);
/// assert!(conflict.where_clause.is_some());
///
/// // Selective update: ON CONFLICT (id) DO UPDATE SET name = EXCLUDED.name
/// let conflict = OnConflict::do_update(vec!["id".to_string()])
///     .with_update_columns(vec!["name".to_string()]);
/// assert_eq!(conflict.update_columns.unwrap().len(), 1);
/// ```
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct OnConflict {
    /// Columns that define the conflict target (unique constraint or index)
    pub columns: Vec<String>,
    /// Action to take on conflict: DO NOTHING or DO UPDATE
    pub action: ConflictAction,
    /// Optional WHERE clause for partial unique index
    ///
    /// Example: `ON CONFLICT (email) WHERE deleted_at IS NULL`
    pub where_clause: Option<Vec<LogicCondition>>,
    /// Specific columns to update on conflict (if None, all columns are updated)
    ///
    /// Example: `ON CONFLICT (id) DO UPDATE SET name = EXCLUDED.name`
    pub update_columns: Option<Vec<String>>,
}

impl OnConflict {
    /// Creates an ON CONFLICT ... DO NOTHING clause.
    pub fn do_nothing(columns: Vec<String>) -> Self {
        Self {
            columns,
            action: ConflictAction::DoNothing,
            where_clause: None,
            update_columns: None,
        }
    }

    /// Creates an ON CONFLICT ... DO UPDATE clause.
    pub fn do_update(columns: Vec<String>) -> Self {
        Self {
            columns,
            action: ConflictAction::DoUpdate,
            where_clause: None,
            update_columns: None,
        }
    }

    /// Adds a WHERE clause for partial unique index support.
    pub fn with_where_clause(mut self, where_clause: Vec<LogicCondition>) -> Self {
        self.where_clause = Some(where_clause);
        self
    }

    /// Specifies which columns to update (instead of all columns).
    pub fn with_update_columns(mut self, update_columns: Vec<String>) -> Self {
        self.update_columns = Some(update_columns);
        self
    }
}

/// Action to take when a conflict occurs during INSERT.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ConflictAction {
    /// Do nothing on conflict (ignore the insert)
    DoNothing,
    /// Update the existing row with new values
    DoUpdate,
}

/// Parameters for an UPDATE operation.
///
/// # Safety
///
/// Updates without filters are rejected to prevent accidental mass updates.
/// Use LIMIT with ORDER BY for predictable results.
///
/// # Examples
///
/// ```
/// use postgrest_parser::{UpdateParams, parse_filter, LogicCondition};
/// use std::collections::HashMap;
/// use serde_json::json;
///
/// // Update with filter
/// let mut set_values = HashMap::new();
/// set_values.insert("status".to_string(), json!("active"));
///
/// let filter = parse_filter("id", "eq.123").unwrap();
/// let params = UpdateParams::new(set_values)
///     .with_filters(vec![LogicCondition::Filter(filter)]);
///
/// assert!(params.has_filters());
/// assert_eq!(params.set_values.len(), 1);
/// ```
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct UpdateParams {
    /// Column-value pairs for the SET clause
    pub set_values: HashMap<String, serde_json::Value>,
    /// WHERE clause filters (required for safety)
    pub filters: Vec<LogicCondition>,
    /// Optional ORDER BY clause (required if using LIMIT)
    pub order: Vec<OrderTerm>,
    /// Optional LIMIT clause
    pub limit: Option<u64>,
    /// Optional RETURNING clause columns
    pub returning: Option<Vec<SelectItem>>,
}

impl UpdateParams {
    /// Creates new update parameters with the given SET values.
    pub fn new(set_values: HashMap<String, serde_json::Value>) -> Self {
        Self {
            set_values,
            filters: Vec::new(),
            order: Vec::new(),
            limit: None,
            returning: None,
        }
    }

    /// Adds WHERE clause filters.
    pub fn with_filters(mut self, filters: Vec<LogicCondition>) -> Self {
        self.filters = filters;
        self
    }

    /// Adds ORDER BY clause.
    pub fn with_order(mut self, order: Vec<OrderTerm>) -> Self {
        self.order = order;
        self
    }

    /// Adds LIMIT clause (requires ORDER BY for safety).
    pub fn with_limit(mut self, limit: u64) -> Self {
        self.limit = Some(limit);
        self
    }

    /// Specifies columns to return after update (RETURNING clause).
    pub fn with_returning(mut self, returning: Vec<SelectItem>) -> Self {
        self.returning = Some(returning);
        self
    }

    /// Returns true if filters are present.
    pub fn has_filters(&self) -> bool {
        !self.filters.is_empty()
    }

    /// Returns true if no columns are being updated.
    pub fn is_set_empty(&self) -> bool {
        self.set_values.is_empty()
    }
}

/// Parameters for a DELETE operation.
///
/// # Safety
///
/// Deletes without filters are rejected to prevent accidental mass deletions.
/// Use LIMIT with ORDER BY for predictable results.
///
/// # Examples
///
/// ```
/// use postgrest_parser::{DeleteParams, parse_filter, LogicCondition};
///
/// // Delete with filter
/// let filter = parse_filter("status", "eq.inactive").unwrap();
/// let params = DeleteParams::new()
///     .with_filters(vec![LogicCondition::Filter(filter)]);
///
/// assert!(params.has_filters());
///
/// // Delete with LIMIT and ORDER BY
/// use postgrest_parser::parse_order;
/// let filter = parse_filter("created_at", "lt.2020-01-01").unwrap();
/// let order = parse_order("created_at.asc").unwrap();
/// let params = DeleteParams::new()
///     .with_filters(vec![LogicCondition::Filter(filter)])
///     .with_order(order)
///     .with_limit(100);
///
/// assert_eq!(params.limit, Some(100));
/// ```
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DeleteParams {
    /// WHERE clause filters (required for safety)
    pub filters: Vec<LogicCondition>,
    /// Optional ORDER BY clause (required if using LIMIT)
    pub order: Vec<OrderTerm>,
    /// Optional LIMIT clause
    pub limit: Option<u64>,
    /// Optional RETURNING clause columns (returns deleted rows)
    pub returning: Option<Vec<SelectItem>>,
}

impl DeleteParams {
    /// Creates new delete parameters with no filters.
    pub fn new() -> Self {
        Self {
            filters: Vec::new(),
            order: Vec::new(),
            limit: None,
            returning: None,
        }
    }

    /// Adds WHERE clause filters.
    pub fn with_filters(mut self, filters: Vec<LogicCondition>) -> Self {
        self.filters = filters;
        self
    }

    /// Adds ORDER BY clause.
    pub fn with_order(mut self, order: Vec<OrderTerm>) -> Self {
        self.order = order;
        self
    }

    /// Adds LIMIT clause (requires ORDER BY for safety).
    pub fn with_limit(mut self, limit: u64) -> Self {
        self.limit = Some(limit);
        self
    }

    /// Specifies columns to return from deleted rows (RETURNING clause).
    pub fn with_returning(mut self, returning: Vec<SelectItem>) -> Self {
        self.returning = Some(returning);
        self
    }

    /// Returns true if filters are present.
    pub fn has_filters(&self) -> bool {
        !self.filters.is_empty()
    }
}

impl Default for DeleteParams {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_resolved_table_new() {
        let table = ResolvedTable::new("public", "users");
        assert_eq!(table.schema, "public");
        assert_eq!(table.name, "users");
    }

    #[test]
    fn test_resolved_table_qualified_name() {
        let table = ResolvedTable::new("auth", "users");
        assert_eq!(table.qualified_name(), "\"auth\".\"users\"");
    }

    #[test]
    fn test_insert_params_new() {
        let mut map = HashMap::new();
        map.insert("name".to_string(), json!("Alice"));
        let params = InsertParams::new(InsertValues::Single(map));
        assert!(params.columns.is_none());
        assert!(params.on_conflict.is_none());
        assert!(params.returning.is_none());
    }

    #[test]
    fn test_insert_params_with_columns() {
        let mut map = HashMap::new();
        map.insert("name".to_string(), json!("Alice"));
        let params = InsertParams::new(InsertValues::Single(map))
            .with_columns(vec!["name".to_string(), "email".to_string()]);
        assert_eq!(params.columns.as_ref().unwrap().len(), 2);
    }

    #[test]
    fn test_insert_params_with_on_conflict() {
        let mut map = HashMap::new();
        map.insert("email".to_string(), json!("alice@example.com"));
        let conflict = OnConflict::do_update(vec!["email".to_string()]);
        let params = InsertParams::new(InsertValues::Single(map)).with_on_conflict(conflict);
        assert!(params.on_conflict.is_some());
        assert_eq!(
            params.on_conflict.unwrap().action,
            ConflictAction::DoUpdate
        );
    }

    #[test]
    fn test_insert_values_single_len() {
        let mut map = HashMap::new();
        map.insert("name".to_string(), json!("Alice"));
        let values = InsertValues::Single(map);
        assert_eq!(values.len(), 1);
        assert!(!values.is_empty());
    }

    #[test]
    fn test_insert_values_bulk_len() {
        let mut map1 = HashMap::new();
        map1.insert("name".to_string(), json!("Alice"));
        let mut map2 = HashMap::new();
        map2.insert("name".to_string(), json!("Bob"));
        let values = InsertValues::Bulk(vec![map1, map2]);
        assert_eq!(values.len(), 2);
        assert!(!values.is_empty());
    }

    #[test]
    fn test_insert_values_get_columns() {
        let mut map = HashMap::new();
        map.insert("name".to_string(), json!("Alice"));
        map.insert("age".to_string(), json!(30));
        let values = InsertValues::Single(map);
        let columns = values.get_columns();
        assert_eq!(columns.len(), 2);
        assert!(columns.contains(&"name".to_string()));
        assert!(columns.contains(&"age".to_string()));
    }

    #[test]
    fn test_on_conflict_do_nothing() {
        let conflict = OnConflict::do_nothing(vec!["email".to_string()]);
        assert_eq!(conflict.columns.len(), 1);
        assert_eq!(conflict.action, ConflictAction::DoNothing);
    }

    #[test]
    fn test_on_conflict_do_update() {
        let conflict = OnConflict::do_update(vec!["email".to_string()]);
        assert_eq!(conflict.action, ConflictAction::DoUpdate);
    }

    #[test]
    fn test_update_params_new() {
        let mut map = HashMap::new();
        map.insert("status".to_string(), json!("active"));
        let params = UpdateParams::new(map);
        assert_eq!(params.set_values.len(), 1);
        assert!(params.filters.is_empty());
        assert!(params.order.is_empty());
        assert!(params.limit.is_none());
    }

    #[test]
    fn test_update_params_with_limit() {
        let mut map = HashMap::new();
        map.insert("status".to_string(), json!("active"));
        let params = UpdateParams::new(map).with_limit(10);
        assert_eq!(params.limit, Some(10));
    }

    #[test]
    fn test_update_params_has_filters() {
        let mut map = HashMap::new();
        map.insert("status".to_string(), json!("active"));
        let params = UpdateParams::new(map);
        assert!(!params.has_filters());
    }

    #[test]
    fn test_update_params_is_set_empty() {
        let params = UpdateParams::new(HashMap::new());
        assert!(params.is_set_empty());
    }

    #[test]
    fn test_delete_params_new() {
        let params = DeleteParams::new();
        assert!(params.filters.is_empty());
        assert!(params.order.is_empty());
        assert!(params.limit.is_none());
        assert!(params.returning.is_none());
    }

    #[test]
    fn test_delete_params_with_limit() {
        let params = DeleteParams::new().with_limit(5);
        assert_eq!(params.limit, Some(5));
    }

    #[test]
    fn test_delete_params_has_filters() {
        let params = DeleteParams::new();
        assert!(!params.has_filters());
    }

    #[test]
    fn test_delete_params_default() {
        let params = DeleteParams::default();
        assert!(params.filters.is_empty());
    }

    #[test]
    fn test_resolved_table_serialization() {
        let table = ResolvedTable::new("public", "users");
        let json = serde_json::to_string(&table).unwrap();
        assert!(json.contains("public"));
        assert!(json.contains("users"));
    }

    #[test]
    fn test_insert_params_serialization() {
        let mut map = HashMap::new();
        map.insert("name".to_string(), json!("Alice"));
        let params = InsertParams::new(InsertValues::Single(map));
        let json = serde_json::to_string(&params).unwrap();
        assert!(json.contains("Alice"));
    }

    #[test]
    fn test_update_params_serialization() {
        let mut map = HashMap::new();
        map.insert("status".to_string(), json!("active"));
        let params = UpdateParams::new(map);
        let json = serde_json::to_string(&params).unwrap();
        assert!(json.contains("active"));
    }

    #[test]
    fn test_delete_params_serialization() {
        let params = DeleteParams::new();
        let json = serde_json::to_string(&params).unwrap();
        assert!(json.contains("filters"));
    }

    #[test]
    fn test_conflict_action_serialization() {
        let action = ConflictAction::DoNothing;
        let json = serde_json::to_string(&action).unwrap();
        assert!(json.contains("DoNothing"));
    }

    #[test]
    fn test_insert_values_empty_map() {
        let values = InsertValues::Single(HashMap::new());
        assert!(values.is_empty());
        assert_eq!(values.len(), 1); // Still counts as 1 row, even if empty
    }

    #[test]
    fn test_insert_values_empty_bulk() {
        let values = InsertValues::Bulk(Vec::new());
        assert!(values.is_empty());
        assert_eq!(values.len(), 0);
    }

    #[test]
    fn test_bulk_insert_get_columns() {
        let mut map1 = HashMap::new();
        map1.insert("name".to_string(), json!("Alice"));
        map1.insert("age".to_string(), json!(30));
        let mut map2 = HashMap::new();
        map2.insert("name".to_string(), json!("Bob"));
        map2.insert("age".to_string(), json!(25));
        let values = InsertValues::Bulk(vec![map1, map2]);
        let columns = values.get_columns();
        assert_eq!(columns.len(), 2);
    }
}
