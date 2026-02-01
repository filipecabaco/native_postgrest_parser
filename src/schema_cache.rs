///! Schema introspection and caching for relation resolution.
///!
///! This module provides database schema introspection to resolve foreign key
///! relationships, enabling proper JOIN generation for resource embedding.

use std::collections::HashMap;
use serde::{Deserialize, Serialize};

#[cfg(feature = "postgres")]
use sqlx::{PgPool, Row};

/// A foreign key relationship between two tables.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ForeignKey {
    /// Source table schema
    pub from_schema: String,
    /// Source table name
    pub from_table: String,
    /// Source column name
    pub from_column: String,
    /// Target table schema
    pub to_schema: String,
    /// Target table name
    pub to_table: String,
    /// Target column name
    pub to_column: String,
    /// Foreign key constraint name
    pub constraint_name: String,
}

impl ForeignKey {
    /// Returns true if this FK goes from `from_table` to `to_table`
    pub fn links(&self, from_table: &str, to_table: &str) -> bool {
        self.from_table == from_table && self.to_table == to_table
    }

    /// Returns the JOIN ON clause for this foreign key
    pub fn join_condition(&self, from_alias: &str, to_alias: &str) -> String {
        format!(
            "\"{}\".\"{}\" = \"{}\".\"{}\"",
            from_alias,
            &self.from_column,
            to_alias,
            &self.to_column
        )
    }
}

/// Type of relationship between tables
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RelationType {
    /// Many-to-One: many rows in source table reference one row in target
    ManyToOne,
    /// One-to-Many: one row in source table is referenced by many rows in target
    OneToMany,
    /// Many-to-Many: through a junction table
    ManyToMany {
        /// The junction table name
        junction_table: &'static str,
    },
}

/// A resolved relationship between two tables
#[derive(Debug, Clone)]
pub struct Relationship {
    /// Source table
    pub from_table: String,
    /// Target table (the relation name used in queries)
    pub to_table: String,
    /// Foreign key used for the relationship
    pub foreign_key: ForeignKey,
    /// Type of relationship
    pub relation_type: RelationType,
}

/// Cache of database schema information
#[derive(Debug, Clone, Default)]
pub struct SchemaCache {
    /// All foreign keys in the database, indexed by (schema, table)
    foreign_keys: HashMap<(String, String), Vec<ForeignKey>>,
    /// Reverse lookup: which tables reference this table
    reverse_fks: HashMap<(String, String), Vec<ForeignKey>>,
}

impl SchemaCache {
    /// Creates an empty schema cache
    pub fn new() -> Self {
        Self::default()
    }

    #[cfg(feature = "postgres")]
    /// Loads schema information from a PostgreSQL database
    pub async fn load_from_database(pool: &PgPool) -> Result<Self, sqlx::Error> {
        let mut cache = Self::new();

        // Query all foreign keys from pg_catalog (more reliable than information_schema)
        // Based on PostgREST's approach using system catalogs
        let fks = sqlx::query(
            r#"
            SELECT
                con.conname AS constraint_name,
                sn.nspname AS from_schema,
                sc.relname AS from_table,
                sa.attname AS from_column,
                tn.nspname AS to_schema,
                tc.relname AS to_table,
                ta.attname AS to_column
            FROM pg_constraint con
            JOIN pg_class sc ON sc.oid = con.conrelid
            JOIN pg_namespace sn ON sn.oid = sc.relnamespace
            JOIN pg_class tc ON tc.oid = con.confrelid
            JOIN pg_namespace tn ON tn.oid = tc.relnamespace
            JOIN pg_attribute sa ON sa.attrelid = sc.oid AND sa.attnum = con.conkey[1]
            JOIN pg_attribute ta ON ta.attrelid = tc.oid AND ta.attnum = con.confkey[1]
            WHERE con.contype = 'f'
              AND sn.nspname NOT IN ('pg_catalog', 'information_schema')
              AND array_length(con.conkey, 1) = 1
            ORDER BY sn.nspname, sc.relname, con.conname
            "#,
        )
        .fetch_all(pool)
        .await?;

        for row in fks {
            let fk = ForeignKey {
                from_schema: row.get("from_schema"),
                from_table: row.get("from_table"),
                from_column: row.get("from_column"),
                to_schema: row.get("to_schema"),
                to_table: row.get("to_table"),
                to_column: row.get("to_column"),
                constraint_name: row.get("constraint_name"),
            };

            // Index by source table
            cache
                .foreign_keys
                .entry((fk.from_schema.clone(), fk.from_table.clone()))
                .or_default()
                .push(fk.clone());

            // Index by target table (reverse lookup)
            cache
                .reverse_fks
                .entry((fk.to_schema.clone(), fk.to_table.clone()))
                .or_default()
                .push(fk);
        }

        Ok(cache)
    }

    /// Finds a foreign key relationship from one table to another
    pub fn find_relationship(
        &self,
        from_schema: &str,
        from_table: &str,
        to_table: &str,
    ) -> Option<Relationship> {
        // Try forward FK (Many-to-One): from_table has FK to to_table
        if let Some(fks) = self.foreign_keys.get(&(from_schema.to_string(), from_table.to_string())) {
            if let Some(fk) = fks.iter().find(|fk| fk.to_table == to_table) {
                return Some(Relationship {
                    from_table: from_table.to_string(),
                    to_table: to_table.to_string(),
                    foreign_key: fk.clone(),
                    relation_type: RelationType::ManyToOne,
                });
            }
        }

        // Try reverse FK (One-to-Many): to_table has FK to from_table
        if let Some(fks) = self.reverse_fks.get(&(from_schema.to_string(), from_table.to_string())) {
            if let Some(fk) = fks.iter().find(|fk| fk.from_table == to_table) {
                return Some(Relationship {
                    from_table: from_table.to_string(),
                    to_table: to_table.to_string(),
                    foreign_key: fk.clone(),
                    relation_type: RelationType::OneToMany,
                });
            }
        }

        // TODO: Detect Many-to-Many through junction tables
        None
    }

    /// Gets all foreign keys from a table
    pub fn get_foreign_keys(&self, schema: &str, table: &str) -> Vec<&ForeignKey> {
        self.foreign_keys
            .get(&(schema.to_string(), table.to_string()))
            .map(|fks| fks.iter().collect())
            .unwrap_or_default()
    }

    /// Gets all tables that reference this table
    pub fn get_referencing_tables(&self, schema: &str, table: &str) -> Vec<&ForeignKey> {
        self.reverse_fks
            .get(&(schema.to_string(), table.to_string()))
            .map(|fks| fks.iter().collect())
            .unwrap_or_default()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_foreign_key_links() {
        let fk = ForeignKey {
            from_schema: "public".to_string(),
            from_table: "orders".to_string(),
            from_column: "customer_id".to_string(),
            to_schema: "public".to_string(),
            to_table: "customers".to_string(),
            to_column: "id".to_string(),
            constraint_name: "orders_customer_id_fkey".to_string(),
        };

        assert!(fk.links("orders", "customers"));
        assert!(!fk.links("customers", "orders"));
    }

    #[test]
    fn test_join_condition() {
        let fk = ForeignKey {
            from_schema: "public".to_string(),
            from_table: "orders".to_string(),
            from_column: "customer_id".to_string(),
            to_schema: "public".to_string(),
            to_table: "customers".to_string(),
            to_column: "id".to_string(),
            constraint_name: "orders_customer_id_fkey".to_string(),
        };

        let condition = fk.join_condition("orders", "customers");
        assert_eq!(condition, r#""orders"."customer_id" = "customers"."id""#);
    }

    #[test]
    fn test_schema_cache_empty() {
        let cache = SchemaCache::new();
        assert_eq!(cache.get_foreign_keys("public", "users").len(), 0);
    }
}
