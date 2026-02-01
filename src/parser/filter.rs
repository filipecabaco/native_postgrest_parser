use super::common::{field, parse_field_fallback};
use crate::ast::{Field, Filter, FilterOperator, FilterValue, Quantifier};
use crate::error::ParseError;

type OperatorValueResult = (bool, String, Option<Quantifier>, Option<String>, String);

/// Parses a PostgREST filter from field and value strings.
///
/// Supports all 22+ PostgREST filter operators, quantifiers, negation, and full-text search.
///
/// # Filter Syntax
///
/// - Basic: `field=operator.value`
/// - Negated: `field=not.operator.value`
/// - Quantifiers: `field=operator(any).{val1,val2}` or `field=operator(all).{val1,val2}`
/// - FTS: `field=fts(lang).search terms`
/// - JSON: `data->key=operator.value` or `data->>key=operator.value`
///
/// # Examples
///
/// ```
/// use postgrest_parser::parse_filter;
///
/// // Basic comparison
/// let filter = parse_filter("age", "gte.18").unwrap();
/// assert!(!filter.negated);
///
/// // Negation
/// let filter = parse_filter("status", "not.eq.deleted").unwrap();
/// assert!(filter.negated);
///
/// // Array containment
/// let filter = parse_filter("tags", "cs.{rust,postgres}").unwrap();
///
/// // Quantifier with array
/// let filter = parse_filter("tags", "eq(any).{rust,elixir}").unwrap();
///
/// // Full-text search
/// let filter = parse_filter("content", "fts(english).search term").unwrap();
///
/// // JSON path
/// let filter = parse_filter("data->user->name", "eq.Alice").unwrap();
///
/// // IS NULL
/// let filter = parse_filter("deleted_at", "is.null").unwrap();
/// ```
///
/// # Errors
///
/// Returns `ParseError` if:
/// - Operator is invalid or missing
/// - Field syntax is malformed
/// - Quantifier is used with incompatible operator
/// - Value format is invalid for the operator
pub fn parse_filter(field_str: &str, value_str: &str) -> Result<Filter, ParseError> {
    let parsed_field = parse_field_string(field_str)?;
    let (negated, operator_str, quantifier, language, value) = parse_operator_value(value_str)?;

    let operator = parse_operator(&operator_str)?;
    validate_operator_quantifier(&operator, &quantifier, &language)?;
    let parsed_value = parse_value(&operator, &quantifier, &value)?;

    Ok(Filter {
        field: parsed_field,
        operator,
        value: parsed_value,
        quantifier,
        language,
        negated,
    })
}

pub fn parse_field_string(field_str: &str) -> Result<Field, ParseError> {
    match field(field_str) {
        Ok((_, field)) => Ok(field),
        Err(_) => parse_field_fallback(field_str),
    }
}


fn parse_operator_value(value_str: &str) -> Result<OperatorValueResult, ParseError> {
    let parts: Vec<&str> = value_str.split('.').collect();

    let (negated, rest) = if parts.first() == Some(&"not") {
        (true, &parts[1..])
    } else {
        (false, parts.as_slice())
    };

    if rest.is_empty() {
        return Err(ParseError::MissingOperatorOrValue);
    }

    let operator_part = rest[0];

    let (operator, mut quantifier, mut language) = if operator_part.contains('(') && operator_part.ends_with(')') {
        let open_paren = operator_part.find('(').unwrap();
        let op = &operator_part[..open_paren];
        let quant_or_lang = &operator_part[open_paren + 1..operator_part.len() - 1];

        match quant_or_lang {
            "any" => (op.to_string(), Some(Quantifier::Any), None),
            "all" => (op.to_string(), Some(Quantifier::All), None),
            _ => (op.to_string(), None, Some(quant_or_lang.to_string())),
        }
    } else {
        (operator_part.to_string(), None, None)
    };

    if rest.len() == 1 {
        return Ok((negated, operator, quantifier, language, String::new()));
    }

    let (value_quant, value_lang, value) = extract_quantifier_or_language(&rest[1..])?;

    if quantifier.is_none() {
        quantifier = value_quant;
    }
    if language.is_none() {
        language = value_lang;
    }

    Ok((negated, operator, quantifier, language, value))
}

fn extract_quantifier_or_language(
    parts: &[&str],
) -> Result<(Option<Quantifier>, Option<String>, String), ParseError> {
    let (quantifier, language, value) = match parts {
        [rest] => {
            if rest.is_empty() {
                (None, None, String::new())
            } else {
                (None, None, rest.to_string())
            }
        }
        [quant_or_lang, value] => {
            if quant_or_lang.ends_with(')') {
                if quant_or_lang.starts_with('(') {
                    let inner = &quant_or_lang[1..quant_or_lang.len() - 1];
                    if inner == "any" {
                        (Some(Quantifier::Any), None, value.to_string())
                    } else if inner == "all" {
                        (Some(Quantifier::All), None, value.to_string())
                    } else {
                        (None, Some(inner.to_string()), value.to_string())
                    }
                } else {
                    (None, None, format!("{}.{}", quant_or_lang, value))
                }
            } else {
                (None, None, format!("{}.{}", quant_or_lang, value))
            }
        }
        [quantifier, language, value] => {
            let quant = if quantifier.ends_with(')') {
                match *quantifier {
                    "(any)" => Some(Quantifier::Any),
                    "(all)" => Some(Quantifier::All),
                    _ => return Err(ParseError::InvalidQuantifier(quantifier.to_string())),
                }
            } else {
                return Err(ParseError::InvalidQuantifier(quantifier.to_string()));
            };

            if language.ends_with(')') {
                let inner = &language[1..language.len() - 1];
                (quant, Some(inner.to_string()), value.to_string())
            } else {
                (quant, None, format!("{}.{}", language, value))
            }
        }
        _ => return Err(ParseError::InvalidFilterFormat(format!("{:?}", parts))),
    };

    Ok((quantifier, language, value))
}

fn parse_operator(op_str: &str) -> Result<FilterOperator, ParseError> {
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

fn validate_operator_quantifier(
    operator: &FilterOperator,
    quantifier: &Option<Quantifier>,
    language: &Option<String>,
) -> Result<(), ParseError> {
    match (operator, quantifier, language) {
        (FilterOperator::In, Some(_), _)
        | (FilterOperator::Cs, Some(_), _)
        | (FilterOperator::Cd, Some(_), _)
        | (FilterOperator::Ov, Some(_), _)
        | (FilterOperator::Sl, Some(_), _)
        | (FilterOperator::Sr, Some(_), _)
        | (FilterOperator::Nxl, Some(_), _)
        | (FilterOperator::Nxr, Some(_), _)
        | (FilterOperator::Adj, Some(_), _)
        | (FilterOperator::Is, Some(_), _) => Err(ParseError::QuantifierNotSupported),
        (
            FilterOperator::Fts
            | FilterOperator::Plfts
            | FilterOperator::Phfts
            | FilterOperator::Wfts,
            Some(Quantifier::Any | Quantifier::All),
            _,
        ) => Err(ParseError::InvalidFtsLanguage(
            "any/all not supported for FTS".to_string(),
        )),
        _ => Ok(()),
    }
}

fn parse_value(
    operator: &FilterOperator,
    quantifier: &Option<Quantifier>,
    value_str: &str,
) -> Result<FilterValue, ParseError> {
    match (operator, quantifier) {
        (FilterOperator::In, _) => parse_list_value(value_str, '(', ')'),
        (FilterOperator::Cs | FilterOperator::Cd, _) => {
            Ok(FilterValue::Single(value_str.to_string()))
        }
        (FilterOperator::Ov, _) => parse_list_value(value_str, '(', ')'),
        (
            FilterOperator::Eq
            | FilterOperator::Neq
            | FilterOperator::Gt
            | FilterOperator::Gte
            | FilterOperator::Lt
            | FilterOperator::Lte
            | FilterOperator::Like
            | FilterOperator::Ilike
            | FilterOperator::Match
            | FilterOperator::Imatch,
            Some(Quantifier::Any | Quantifier::All),
        ) => parse_list_value(value_str, '{', '}'),
        _ => Ok(FilterValue::Single(value_str.to_string())),
    }
}

fn parse_list_value(value_str: &str, open: char, close: char) -> Result<FilterValue, ParseError> {
    if value_str.starts_with(open) && value_str.ends_with(close) {
        let inner = &value_str[1..value_str.len() - 1];
        let items: Vec<String> = inner.split(',').map(|s| s.trim().to_string()).collect();
        Ok(FilterValue::List(items))
    } else {
        Err(ParseError::ExpectedListFormat(format!(
            "expected list with {} and {}",
            open, close
        )))
    }
}

pub fn reserved_key(key: &str) -> bool {
    matches!(
        key,
        "select" | "order" | "limit" | "offset" | "on_conflict" | "columns" | "returning"
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_filter_eq() {
        let result = parse_filter("id", "eq.1");
        assert!(result.is_ok());
        let filter = result.unwrap();
        assert_eq!(filter.operator, FilterOperator::Eq);
        assert_eq!(filter.value, FilterValue::Single("1".to_string()));
        assert!(!filter.negated);
    }

    #[test]
    fn test_parse_filter_negated() {
        let result = parse_filter("status", "not.eq.active");
        assert!(result.is_ok());
        let filter = result.unwrap();
        assert!(filter.negated);
    }

    #[test]
    fn test_parse_filter_in_operator() {
        let result = parse_filter("status", "in.(active,pending)");
        assert!(result.is_ok());
        let filter = result.unwrap();
        assert_eq!(filter.operator, FilterOperator::In);
        match filter.value {
            FilterValue::List(items) => {
                assert_eq!(items.len(), 2);
                assert!(items.contains(&"active".to_string()));
            }
            _ => panic!("Expected list value"),
        }
    }

    #[test]
    fn test_parse_filter_with_quantifier() {
        let result = parse_filter("status", "eq(any).{active,pending}");
        assert!(result.is_ok());
        let filter = result.unwrap();
        assert_eq!(filter.quantifier, Some(Quantifier::Any));
        assert!(matches!(filter.value, FilterValue::List(_)));
    }

    #[test]
    fn test_parse_filter_with_json_path() {
        let result = parse_filter("data->name", "eq.test");
        assert!(result.is_ok());
        let filter = result.unwrap();
        assert_eq!(filter.field.name, "data");
        assert_eq!(filter.field.json_path.len(), 1);
    }

    #[test]
    fn test_parse_filter_with_type_cast() {
        let result = parse_filter("price", "eq.100");
        assert!(result.is_ok());
        assert!(result.is_ok());
    }

    #[test]
    fn test_parse_filter_fts_with_language() {
        let result = parse_filter("content", "fts(english).search");
        assert!(result.is_ok());
        let filter = result.unwrap();
        assert_eq!(filter.language, Some("english".to_string()));
    }

    #[test]
    fn test_parse_filter_unknown_operator() {
        let result = parse_filter("id", "invalid.1");
        assert!(matches!(result, Err(ParseError::UnknownOperator(_))));
    }

    #[test]
    fn test_parse_filter_invalid_quantifier() {
        let result = parse_filter("status", "is(any).null");
        assert!(matches!(result, Err(ParseError::QuantifierNotSupported)));
    }

    #[test]
    fn test_parse_filter_unclosed_parenthesis() {
        let result = parse_filter("status", "in.(active");
        assert!(matches!(result, Err(ParseError::ExpectedListFormat(_))));
    }

    #[test]
    fn test_reserved_key() {
        assert!(reserved_key("select"));
        assert!(reserved_key("order"));
        assert!(reserved_key("limit"));
        assert!(!reserved_key("id"));
    }

    #[test]
    fn test_parse_filter_with_whitespace_in_list() {
        let result = parse_filter("status", "in.(active, pending, closed)");
        assert!(result.is_ok());
        let filter = result.unwrap();
        assert_eq!(filter.operator, FilterOperator::In);
        match filter.value {
            FilterValue::List(items) => {
                assert_eq!(items.len(), 3);
                assert!(items.contains(&"active".to_string()));
                assert!(items.contains(&"pending".to_string()));
                assert!(items.contains(&"closed".to_string()));
            }
            _ => panic!("Expected list value"),
        }
    }

    #[test]
    fn test_parse_filter_comparison_operators() {
        // GT operator
        let result = parse_filter("age", "gt.18");
        assert!(result.is_ok());
        let filter = result.unwrap();
        assert_eq!(filter.operator, FilterOperator::Gt);
        assert_eq!(filter.value, FilterValue::Single("18".to_string()));

        // GTE operator
        let result = parse_filter("age", "gte.21");
        assert!(result.is_ok());
        assert_eq!(result.unwrap().operator, FilterOperator::Gte);

        // LT operator
        let result = parse_filter("age", "lt.65");
        assert!(result.is_ok());
        assert_eq!(result.unwrap().operator, FilterOperator::Lt);

        // LTE operator
        let result = parse_filter("age", "lte.65");
        assert!(result.is_ok());
        assert_eq!(result.unwrap().operator, FilterOperator::Lte);

        // NEQ operator
        let result = parse_filter("status", "neq.inactive");
        assert!(result.is_ok());
        assert_eq!(result.unwrap().operator, FilterOperator::Neq);
    }

    #[test]
    fn test_parse_filter_match_operators() {
        // Match operator
        let result = parse_filter("name", "match.^John");
        assert!(result.is_ok());
        let filter = result.unwrap();
        assert_eq!(filter.operator, FilterOperator::Match);

        // Imatch operator
        let result = parse_filter("name", "imatch.^john");
        assert!(result.is_ok());
        assert_eq!(result.unwrap().operator, FilterOperator::Imatch);
    }

    #[test]
    fn test_parse_filter_array_operators() {
        // CS operator (contains)
        let result = parse_filter("tags", "cs.{rust}");
        assert!(result.is_ok());
        assert_eq!(result.unwrap().operator, FilterOperator::Cs);

        // CD operator (contained in)
        let result = parse_filter("tags", "cd.{rust,elixir}");
        assert!(result.is_ok());
        assert_eq!(result.unwrap().operator, FilterOperator::Cd);

        // OV operator (overlaps)
        let result = parse_filter("tags", "ov.(rust,elixir)");
        assert!(result.is_ok());
        assert_eq!(result.unwrap().operator, FilterOperator::Ov);
    }

    #[test]
    fn test_parse_filter_range_operators() {
        // SL operator (strictly left)
        let result = parse_filter("range", "sl.[1,10)");
        assert!(result.is_ok());
        assert_eq!(result.unwrap().operator, FilterOperator::Sl);

        // SR operator (strictly right)
        let result = parse_filter("range", "sr.[1,10)");
        assert!(result.is_ok());
        assert_eq!(result.unwrap().operator, FilterOperator::Sr);

        // NXL operator
        let result = parse_filter("range", "nxl.[1,10)");
        assert!(result.is_ok());
        assert_eq!(result.unwrap().operator, FilterOperator::Nxl);

        // NXR operator
        let result = parse_filter("range", "nxr.[1,10)");
        assert!(result.is_ok());
        assert_eq!(result.unwrap().operator, FilterOperator::Nxr);

        // ADJ operator (adjacent)
        let result = parse_filter("range", "adj.[1,10)");
        assert!(result.is_ok());
        assert_eq!(result.unwrap().operator, FilterOperator::Adj);
    }

    #[test]
    fn test_parse_filter_fts_operators() {
        // PLFTS operator
        let result = parse_filter("content", "plfts.search");
        assert!(result.is_ok());
        assert_eq!(result.unwrap().operator, FilterOperator::Plfts);

        // PHFTS operator
        let result = parse_filter("content", "phfts.search phrase");
        assert!(result.is_ok());
        assert_eq!(result.unwrap().operator, FilterOperator::Phfts);

        // WFTS operator
        let result = parse_filter("content", "wfts.search query");
        assert!(result.is_ok());
        assert_eq!(result.unwrap().operator, FilterOperator::Wfts);
    }

    #[test]
    fn test_parse_filter_negated_operators() {
        // Negated GT
        let result = parse_filter("age", "not.gt.18");
        assert!(result.is_ok());
        let filter = result.unwrap();
        assert!(filter.negated);
        assert_eq!(filter.operator, FilterOperator::Gt);

        // Negated IN
        let result = parse_filter("status", "not.in.(active,pending)");
        assert!(result.is_ok());
        let filter = result.unwrap();
        assert!(filter.negated);
        assert_eq!(filter.operator, FilterOperator::In);

        // Negated LIKE
        let result = parse_filter("name", "not.like.*John*");
        assert!(result.is_ok());
        let filter = result.unwrap();
        assert!(filter.negated);
        assert_eq!(filter.operator, FilterOperator::Like);
    }
}
