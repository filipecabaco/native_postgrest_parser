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
