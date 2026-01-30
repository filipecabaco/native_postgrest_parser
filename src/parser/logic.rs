use super::{common::parse_field_fallback, filter::parse_filter};
use crate::ast::{
    Field, Filter, FilterOperator, FilterValue, LogicCondition, LogicOperator, LogicTree,
};
use crate::error::ParseError;

pub fn parse_logic(key: &str, value: &str) -> Result<LogicTree, ParseError> {
    let (negated, operator) = parse_logic_key(key)?;

    let conditions_str = extract_conditions_str(value)?;
    let conditions = parse_conditions(&conditions_str)?;

    Ok(LogicTree {
        operator,
        conditions,
        negated,
    })
}

pub fn logic_key(key: &str) -> bool {
    let key_lower = key.to_lowercase();
    matches!(key_lower.as_str(), "and" | "or" | "not.and" | "not.or")
}

fn parse_logic_key(key: &str) -> Result<(bool, LogicOperator), ParseError> {
    let key_lower = key.to_lowercase();

    if let Some(rest) = key_lower.strip_prefix("not.") {
        match rest {
            "and" => Ok((true, LogicOperator::And)),
            "or" => Ok((true, LogicOperator::Or)),
            _ => Err(ParseError::InvalidLogicExpression(format!("invalid key: {}", key))),
        }
    } else {
        match key_lower.as_str() {
            "and" => Ok((false, LogicOperator::And)),
            "or" => Ok((false, LogicOperator::Or)),
            _ => Err(ParseError::InvalidLogicExpression(format!("invalid key: {}", key))),
        }
    }
}

fn extract_conditions_str(value: &str) -> Result<String, ParseError> {
    let trimmed = value.trim();

    if trimmed.starts_with('(') && trimmed.ends_with(')') {
        Ok(trimmed[1..trimmed.len() - 1].to_string())
    } else {
        Err(ParseError::LogicExpressionNotWrapped)
    }
}

fn parse_conditions(str: &str) -> Result<Vec<LogicCondition>, ParseError> {
    let parts = split_at_top_level_commas(str)?;

    parts
        .iter()
        .map(|part| parse_condition(part.trim()))
        .collect()
}

fn split_at_top_level_commas(str: &str) -> Result<Vec<String>, ParseError> {
    let mut parts = Vec::new();
    let mut current = String::new();
    let mut depth = 0;

    for c in str.chars() {
        match c {
            '(' => {
                depth += 1;
                current.push(c);
            }
            ')' => {
                depth -= 1;
                if depth < 0 {
                    return Err(ParseError::UnexpectedClosingParenthesis);
                }
                current.push(c);
            }
            ',' if depth == 0 => {
                parts.push(current.trim().to_string());
                current.clear();
            }
            _ => {
                current.push(c);
            }
        }
    }

    if !current.trim().is_empty() {
        parts.push(current.trim().to_string());
    }

    if depth > 0 {
        return Err(ParseError::UnclosedParenthesis);
    }

    Ok(parts)
}

fn parse_condition(condition_str: &str) -> Result<LogicCondition, ParseError> {
    let trimmed = condition_str.trim();

    if trimmed.is_empty() {
        return Err(ParseError::InvalidLogicExpression(
            "empty condition".to_string(),
        ));
    }

    if trimmed.starts_with("and(")
        || trimmed.starts_with("or(")
        || trimmed.starts_with("not.and(")
        || trimmed.starts_with("not.or(")
    {
        parse_nested_logic(trimmed)
    } else {
        parse_filter_condition(trimmed)
    }
}

fn parse_nested_logic(str: &str) -> Result<LogicCondition, ParseError> {
    let (negated, rest) = if let Some(stripped) = str.strip_prefix("not.") {
        (true, stripped)
    } else {
        (false, str)
    };

    let (operator, inner) = if let Some(rest) = rest.strip_prefix("and(") {
        if !rest.ends_with(')') {
            return Err(ParseError::InvalidLogicExpression(format!(
                "invalid nested logic: {}",
                str
            )));
        }
        (LogicOperator::And, &rest[..rest.len() - 1])
    } else if let Some(rest) = rest.strip_prefix("or(") {
        if !rest.ends_with(')') {
            return Err(ParseError::InvalidLogicExpression(format!(
                "invalid nested logic: {}",
                str
            )));
        }
        (LogicOperator::Or, &rest[..rest.len() - 1])
    } else {
        return Err(ParseError::InvalidLogicExpression(format!(
            "invalid nested logic: {}",
            str
        )));
    };

    let conditions = parse_conditions(inner)?;

    Ok(LogicCondition::Logic(LogicTree {
        operator,
        conditions,
        negated,
    }))
}

fn parse_filter_condition(str: &str) -> Result<LogicCondition, ParseError> {
    if str.contains('=') {
        parse_equals_notation(str)
    } else {
        parse_dot_notation(str)
    }
}

fn parse_equals_notation(str: &str) -> Result<LogicCondition, ParseError> {
    let parts: Vec<&str> = str.splitn(2, '=').collect();

    if parts.len() == 2 {
        let field_str = parts[0].trim();
        let operator_value = parts[1].trim();

        let filter = parse_filter(field_str, operator_value)?;
        Ok(LogicCondition::Filter(filter))
    } else {
        Err(ParseError::InvalidFilterFormat(format!(
            "invalid equals notation: {}",
            str
        )))
    }
}

fn parse_dot_notation(str: &str) -> Result<LogicCondition, ParseError> {
    let parts: Vec<&str> = str.split('.').collect();

    if parts.len() == 3 {
        let field_str = parts[0];
        let operator_str = parts[1];
        let value_str = parts[2];

        let operator = parse_filter_operator(operator_str)?;
        let value = FilterValue::Single(value_str.to_string());

        let field = parse_filter_field(field_str)?;

        Ok(LogicCondition::Filter(Filter {
            field,
            operator,
            value,
            quantifier: None,
            language: None,
            negated: false,
        }))
    } else if parts.len() == 4 && parts[1] == "not" {
        let field_str = parts[0];
        let operator_str = parts[2];
        let value_str = parts[3];

        let operator = parse_filter_operator(operator_str)?;
        let value = FilterValue::Single(value_str.to_string());

        let field = parse_filter_field(field_str)?;

        Ok(LogicCondition::Filter(Filter {
            field,
            operator,
            value,
            quantifier: None,
            language: None,
            negated: true,
        }))
    } else {
        Err(ParseError::InvalidFilterFormat(format!(
            "invalid dot notation: {}",
            str
        )))
    }
}

fn parse_filter_operator(op_str: &str) -> Result<FilterOperator, ParseError> {
    match op_str.to_lowercase().as_str() {
        "eq" => Ok(FilterOperator::Eq),
        "neq" => Ok(FilterOperator::Neq),
        "gt" => Ok(FilterOperator::Gt),
        "gte" => Ok(FilterOperator::Gte),
        "lt" => Ok(FilterOperator::Lt),
        "lte" => Ok(FilterOperator::Lte),
        "like" => Ok(FilterOperator::Like),
        "ilike" => Ok(FilterOperator::Ilike),
        "match" => Ok(FilterOperator::Match),
        "imatch" => Ok(FilterOperator::Imatch),
        "in" => Ok(FilterOperator::In),
        "is" => Ok(FilterOperator::Is),
        "fts" => Ok(FilterOperator::Fts),
        "plfts" => Ok(FilterOperator::Plfts),
        "phfts" => Ok(FilterOperator::Phfts),
        "wfts" => Ok(FilterOperator::Wfts),
        "cs" => Ok(FilterOperator::Cs),
        "cd" => Ok(FilterOperator::Cd),
        "ov" => Ok(FilterOperator::Ov),
        "sl" => Ok(FilterOperator::Sl),
        "sr" => Ok(FilterOperator::Sr),
        "nxl" => Ok(FilterOperator::Nxl),
        "nxr" => Ok(FilterOperator::Nxr),
        "adj" => Ok(FilterOperator::Adj),
        _ => Err(ParseError::UnknownOperator(op_str.to_string())),
    }
}

fn parse_filter_field(field_str: &str) -> Result<Field, ParseError> {
    match crate::parser::common::field(field_str) {
        Ok((_, field)) => Ok(field),
        Err(_) => parse_field_fallback(field_str),
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_logic_and() {
        let result = parse_logic("and", "(id.eq.1,name.eq.john)");
        assert!(result.is_ok());
        let tree = result.unwrap();
        assert_eq!(tree.operator, LogicOperator::And);
        assert!(!tree.negated);
        assert_eq!(tree.conditions.len(), 2);
    }

    #[test]
    fn test_parse_logic_or() {
        let result = parse_logic("or", "(id.eq.1,id.eq.2)");
        assert!(result.is_ok());
        let tree = result.unwrap();
        assert_eq!(tree.operator, LogicOperator::Or);
    }

    #[test]
    fn test_parse_logic_negated() {
        let result = parse_logic("not.and", "(id.eq.1,name.eq.john)");
        assert!(result.is_ok());
        let tree = result.unwrap();
        assert!(tree.negated);
    }

    #[test]
    fn test_parse_logic_nested() {
        let result = parse_logic("and", "(id.eq.1,or(id.eq.2,id.eq.3))");
        assert!(result.is_ok());
        let tree = result.unwrap();
        assert_eq!(tree.conditions.len(), 2);

        assert!(matches!(&tree.conditions[1], LogicCondition::Logic(_)));
    }

    #[test]
    fn test_logic_key() {
        assert!(logic_key("and"));
        assert!(logic_key("or"));
        assert!(logic_key("not.and"));
        assert!(logic_key("not.or"));
        assert!(!logic_key("id"));
    }

    #[test]
    fn test_parse_condition_filter() {
        let result = parse_condition("id.eq.1");
        assert!(result.is_ok());
        assert!(matches!(result.unwrap(), LogicCondition::Filter(_)));
    }

    #[test]
    fn test_parse_condition_nested() {
        let result = parse_condition("and(id.eq.1,name.eq.john)");
        assert!(result.is_ok());
        assert!(matches!(result.unwrap(), LogicCondition::Logic(_)));
    }

    #[test]
    fn test_parse_condition_equals_notation() {
        let result = parse_condition("id=eq.1");
        assert!(result.is_ok());
        assert!(matches!(result.unwrap(), LogicCondition::Filter(_)));
    }

    #[test]
    fn test_parse_condition_invalid() {
        let result = parse_condition("invalid");
        assert!(matches!(result, Err(ParseError::InvalidFilterFormat(_))));
    }

    #[test]
    fn test_split_at_top_level_commas() {
        let result = split_at_top_level_commas("id.eq.1,name.eq.john,or(x.eq.1,y.eq.2)");
        assert!(result.is_ok());
        let parts = result.unwrap();
        assert_eq!(parts.len(), 3);
    }

    #[test]
    fn test_parse_nested_logic() {
        let result = parse_nested_logic("and(id.eq.1,name.eq.john)");
        assert!(result.is_ok());
        let condition = result.unwrap();
        assert!(matches!(condition, LogicCondition::Logic(_)));
    }

    #[test]
    fn test_parse_filter_condition_equals() {
        let result = parse_equals_notation("id=eq.1");
        assert!(result.is_ok());
        assert!(matches!(result.unwrap(), LogicCondition::Filter(_)));
    }

    #[test]
    fn test_parse_filter_condition_dot() {
        let result = parse_dot_notation("id.eq.1");
        assert!(result.is_ok());
        assert!(matches!(result.unwrap(), LogicCondition::Filter(_)));
    }
}
