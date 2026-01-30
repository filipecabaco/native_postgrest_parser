use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Table {
    pub schema: String,
    pub name: String,
    pub columns: Vec<Column>,
    pub primary_key: Vec<String>,
    pub is_view: bool,
}

impl Table {
    pub fn new(schema: impl Into<String>, name: impl Into<String>) -> Self {
        Self {
            schema: schema.into(),
            name: name.into(),
            columns: Vec::new(),
            primary_key: Vec::new(),
            is_view: false,
        }
    }

    pub fn with_columns(mut self, columns: Vec<Column>) -> Self {
        self.columns = columns;
        self
    }

    pub fn with_primary_key(mut self, primary_key: Vec<String>) -> Self {
        self.primary_key = primary_key;
        self
    }

    pub fn as_view(mut self) -> Self {
        self.is_view = true;
        self
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Column {
    pub name: String,
    pub type_: String,
    pub nullable: bool,
    pub has_default: bool,
    pub position: usize,
}

impl Column {
    pub fn new(name: impl Into<String>, type_: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            type_: type_.into(),
            nullable: false,
            has_default: false,
            position: 0,
        }
    }

    pub fn with_position(mut self, position: usize) -> Self {
        self.position = position;
        self
    }

    pub fn nullable(mut self, nullable: bool) -> Self {
        self.nullable = nullable;
        self
    }

    pub fn with_default(mut self, has_default: bool) -> Self {
        self.has_default = has_default;
        self
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Cardinality {
    ManyToOne,
    OneToMany,
    OneToOne,
    ManyToMany,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Junction {
    pub schema: String,
    pub table: String,
    pub source_columns: Vec<String>,
    pub target_columns: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Relationship {
    pub constraint_name: String,
    pub source_schema: String,
    pub source_table: String,
    pub source_columns: Vec<String>,
    pub target_schema: String,
    pub target_table: String,
    pub target_columns: Vec<String>,
    pub cardinality: Cardinality,
    pub junction: Option<Junction>,
}

impl Relationship {
    pub fn new(
        constraint_name: impl Into<String>,
        source_schema: impl Into<String>,
        source_table: impl Into<String>,
        target_schema: impl Into<String>,
        target_table: impl Into<String>,
        cardinality: Cardinality,
    ) -> Self {
        Self {
            constraint_name: constraint_name.into(),
            source_schema: source_schema.into(),
            source_table: source_table.into(),
            source_columns: Vec::new(),
            target_schema: target_schema.into(),
            target_table: target_table.into(),
            target_columns: Vec::new(),
            cardinality,
            junction: None,
        }
    }

    pub fn with_source_columns(mut self, columns: Vec<String>) -> Self {
        self.source_columns = columns;
        self
    }

    pub fn with_target_columns(mut self, columns: Vec<String>) -> Self {
        self.target_columns = columns;
        self
    }

    pub fn with_junction(mut self, junction: Junction) -> Self {
        self.junction = Some(junction);
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_table_new() {
        let table = Table::new("public", "users");
        assert_eq!(table.schema, "public");
        assert_eq!(table.name, "users");
    }

    #[test]
    fn test_table_with_columns() {
        let columns = vec![Column::new("id", "integer"), Column::new("name", "text")];
        let table = Table::new("public", "users").with_columns(columns);
        assert_eq!(table.columns.len(), 2);
    }

    #[test]
    fn test_table_as_view() {
        let table = Table::new("public", "user_stats").as_view();
        assert!(table.is_view);
    }

    #[test]
    fn test_column_new() {
        let column = Column::new("id", "integer");
        assert_eq!(column.name, "id");
        assert_eq!(column.type_, "integer");
    }

    #[test]
    fn test_relationship_new() {
        let rel = Relationship::new(
            "fk_user_client",
            "public",
            "users",
            "public",
            "clients",
            Cardinality::ManyToOne,
        );
        assert_eq!(rel.source_table, "users");
        assert_eq!(rel.target_table, "clients");
        assert_eq!(rel.cardinality, Cardinality::ManyToOne);
    }

    #[test]
    fn test_relationship_with_junction() {
        let junction = Junction {
            schema: "public".to_string(),
            table: "post_tags".to_string(),
            source_columns: vec!["post_id".to_string()],
            target_columns: vec!["tag_id".to_string()],
        };

        let rel = Relationship::new(
            "pk_post_tags",
            "public",
            "posts",
            "public",
            "tags",
            Cardinality::ManyToMany,
        )
        .with_junction(junction);

        assert!(rel.junction.is_some());
    }

    #[test]
    fn test_table_serialization() {
        let table = Table::new("public", "users");
        let json = serde_json::to_string(&table).unwrap();
        assert!(json.contains("users"));
    }

    #[test]
    fn test_relationship_serialization() {
        let rel = Relationship::new(
            "fk",
            "public",
            "users",
            "public",
            "clients",
            Cardinality::ManyToOne,
        );
        let json = serde_json::to_string(&rel).unwrap();
        assert!(json.contains("many_to_one"));
    }
}
