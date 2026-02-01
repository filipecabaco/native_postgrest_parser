use thiserror::Error;

#[derive(Error, Debug, PartialEq, Clone)]
pub enum SqlError {
    #[error("table not found: {0}")]
    TableNotFound(String),

    #[error("relationship not found: {0}")]
    RelationshipNotFound(String),

    #[error("relationship is ambiguous, use hint: {0}")]
    RelationshipAmbiguous(String),

    #[error("invalid table name: {0}")]
    InvalidTableName(String),

    #[error("empty table name")]
    EmptyTableName,

    #[error("no select items specified")]
    NoSelectItems,

    #[error("invalid parameter: {0}")]
    InvalidParameter(String),

    #[error("failed to build WHERE clause")]
    FailedToBuildWhereClause,

    #[error("failed to build SELECT clause")]
    FailedToBuildSelectClause,

    #[error("failed to build ORDER BY clause")]
    FailedToBuildOrderByClause,

    #[error("failed to build LIMIT/OFFSET clause")]
    FailedToBuildLimitOffset,

    #[error("failed to build LATERAL JOIN")]
    FailedToBuildLateralJoin,

    #[error("invalid JSON path for SQL generation")]
    InvalidJsonPathForSql,

    #[error("invalid type cast for SQL generation")]
    InvalidTypeCastForSql,

    // Mutation safety errors
    #[error("unsafe UPDATE: no WHERE clause specified. Updates without filters are not allowed for safety reasons")]
    UnsafeUpdate,

    #[error("unsafe DELETE: no WHERE clause specified. Deletes without filters are not allowed for safety reasons")]
    UnsafeDelete,

    #[error("LIMIT without ORDER BY: non-deterministic result set. Use ORDER BY when using LIMIT")]
    LimitWithoutOrder,

    #[error("no values provided for INSERT")]
    NoInsertValues,

    #[error("no SET clause for UPDATE")]
    NoUpdateSet,

    // Relation resolution errors
    #[error("no table context for relation resolution")]
    NoTableContext,

    #[error("relation not found: no foreign key between '{from_table}' and '{to_table}'")]
    RelationNotFound {
        from_table: String,
        to_table: String,
    },

    #[error("many-to-many relationships not yet supported (junction table: '{junction_table}')")]
    ManyToManyNotYetSupported { junction_table: String },
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sql_error_table_not_found() {
        let err = SqlError::TableNotFound("users".to_string());
        assert!(err.to_string().contains("not found"));
    }

    #[test]
    fn test_sql_error_eq() {
        let err1 = SqlError::TableNotFound("test".to_string());
        let err2 = SqlError::TableNotFound("test".to_string());
        assert_eq!(err1, err2);
    }

    #[test]
    fn test_sql_error_clone() {
        let err = SqlError::TableNotFound("users".to_string());
        let cloned = err.clone();
        assert_eq!(err, cloned);
    }

    #[test]
    fn test_sql_error_relationship_ambiguous() {
        let err = SqlError::RelationshipAmbiguous("client".to_string());
        assert!(err.to_string().contains("ambiguous"));
    }
}
