pub mod parse;
pub mod sql;

pub use parse::ParseError;
pub use sql::SqlError;

#[derive(Debug, PartialEq, Clone)]
pub enum Error {
    Parse(ParseError),
    Sql(SqlError),
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::Parse(e) => write!(f, "parse error: {}", e),
            Error::Sql(e) => write!(f, "SQL error: {}", e),
        }
    }
}

impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Error::Parse(e) => Some(e),
            Error::Sql(e) => Some(e),
        }
    }
}

impl From<ParseError> for Error {
    fn from(err: ParseError) -> Self {
        Error::Parse(err)
    }
}

impl From<SqlError> for Error {
    fn from(err: SqlError) -> Self {
        Error::Sql(err)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_from_parse() {
        let parse_err = ParseError::UnclosedParenthesis;
        let err = Error::from(parse_err.clone());
        assert!(matches!(err, Error::Parse(_)));
    }

    #[test]
    fn test_error_from_sql() {
        let sql_err = SqlError::TableNotFound("users".to_string());
        let err = Error::from(sql_err.clone());
        assert!(matches!(err, Error::Sql(_)));
    }

    #[test]
    fn test_error_display() {
        let err = Error::Parse(ParseError::UnclosedParenthesis);
        assert!(err.to_string().contains("parse error"));
    }
}
