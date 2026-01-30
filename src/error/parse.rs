use thiserror::Error;

#[derive(Error, Debug, PartialEq, Clone)]
pub enum ParseError {
    #[error("unknown operator: {0}")]
    UnknownOperator(String),

    #[error("invalid filter format: {0}")]
    InvalidFilterFormat(String),

    #[error("invalid operator: {0}")]
    InvalidOperator(String),

    #[error("expected operator: {0}")]
    ExpectedOperator(String),

    #[error("missing operator or value")]
    MissingOperatorOrValue,

    #[error("invalid quantifier: {0}")]
    InvalidQuantifier(String),

    #[error("operator does not support quantifiers")]
    QuantifierNotSupported,

    #[error("invalid FTS language: {0}")]
    InvalidFtsLanguage(String),

    #[error("expected list format: {0}")]
    ExpectedListFormat(String),

    #[error("unclosed parenthesis")]
    UnclosedParenthesis,

    #[error("unexpected closing parenthesis")]
    UnexpectedClosingParenthesis,

    #[error("invalid field name: {0}")]
    InvalidFieldName(String),

    #[error("empty field name")]
    EmptyFieldName,

    #[error("invalid JSON path syntax")]
    InvalidJsonPathSyntax,

    #[error("invalid type cast: {0}")]
    InvalidTypeCast(String),

    #[error("invalid select item: {0}")]
    InvalidSelectItem(String),

    #[error("unexpected '(' after field")]
    UnexpectedParenthesisAfterField,

    #[error("expected '(' after relation name")]
    ExpectedParenthesisAfterRelation,

    #[error("unclosed parenthesis in select")]
    UnclosedParenthesisInSelect,

    #[error("unexpected token: {0}")]
    UnexpectedToken(String),

    #[error("unexpected token in nested select")]
    UnexpectedTokenInNestedSelect,

    #[error("invalid order options: {0}")]
    InvalidOrderOptions(String),

    #[error("invalid logic expression: {0}")]
    InvalidLogicExpression(String),

    #[error("logic expression must be wrapped in parentheses")]
    LogicExpressionNotWrapped,

    #[error("invalid nulls option: {0}")]
    InvalidNullsOption(String),

    #[error("invalid direction: {0}")]
    InvalidDirection(String),

    #[error("invalid limit value: {0}")]
    InvalidLimit(String),

    #[error("invalid offset value: {0}")]
    InvalidOffset(String),

    #[error("invalid integer value: {0}")]
    InvalidInteger(String),

    #[error("reserved key: {0}")]
    ReservedKey(String),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_error_unknown_operator() {
        let err = ParseError::UnknownOperator("invalid".to_string());
        assert!(err.to_string().contains("unknown operator"));
    }

    #[test]
    fn test_parse_error_unclosed_parenthesis() {
        let err = ParseError::UnclosedParenthesis;
        assert!(err.to_string().contains("unclosed"));
    }

    #[test]
    fn test_parse_error_eq() {
        let err1 = ParseError::UnknownOperator("test".to_string());
        let err2 = ParseError::UnknownOperator("test".to_string());
        assert_eq!(err1, err2);
    }

    #[test]
    fn test_parse_error_clone() {
        let err = ParseError::UnclosedParenthesis;
        let cloned = err.clone();
        assert_eq!(err, cloned);
    }
}
