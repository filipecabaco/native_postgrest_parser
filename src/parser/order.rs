use super::common::{field, parse_field_fallback};
use crate::ast::{Direction, Field, Nulls, OrderTerm};
use crate::error::ParseError;

fn is_direction(s: &str) -> bool {
    matches!(s, "asc" | "desc")
}

fn is_nulls_option(s: &str) -> bool {
    matches!(s, "nullsfirst" | "nullslast")
}

/// Parses a PostgREST order clause into a list of order terms.
///
/// # Syntax
///
/// - Single column: `column_name.asc` or `column_name.desc`
/// - Multiple columns: `col1.asc,col2.desc,col3`
/// - With nulls handling: `column.desc.nullsfirst` or `column.asc.nullslast`
/// - JSON fields: `data->created_at.desc`
/// - Type casts: `price::numeric.desc`
///
/// # Examples
///
/// ```
/// use postgrest_parser::parse_order;
///
/// // Single column ascending
/// let terms = parse_order("name.asc").unwrap();
/// assert_eq!(terms.len(), 1);
///
/// // Multiple columns
/// let terms = parse_order("created_at.desc,name.asc").unwrap();
/// assert_eq!(terms.len(), 2);
///
/// // With nulls handling
/// let terms = parse_order("updated_at.desc.nullsfirst").unwrap();
/// assert_eq!(terms.len(), 1);
///
/// // JSON field ordering
/// let terms = parse_order("data->timestamp.desc").unwrap();
/// assert_eq!(terms.len(), 1);
///
/// // Default direction (ascending)
/// let terms = parse_order("id").unwrap();
/// assert_eq!(terms.len(), 1);
/// ```
///
/// # Errors
///
/// Returns `ParseError` if:
/// - Field name is invalid or empty
/// - Direction is not `asc` or `desc`
/// - Nulls option is not `nullsfirst` or `nullslast`
pub fn parse_order(order_str: &str) -> Result<Vec<OrderTerm>, ParseError> {
    if order_str.is_empty() || order_str.trim().is_empty() {
        return Ok(Vec::new());
    }

    let items: Vec<&str> = order_str.split(',').map(|s| s.trim()).collect();

    items
        .iter()
        .map(|item_str| parse_order_term(item_str))
        .collect()
}

/// Parses a single order term from a string.
///
/// # Examples
///
/// ```
/// use postgrest_parser::{parse_order_term, Direction};
///
/// let term = parse_order_term("created_at.desc").unwrap();
/// assert_eq!(term.field.name, "created_at");
/// assert_eq!(term.direction, Direction::Desc);
/// ```
pub fn parse_order_term(term_str: &str) -> Result<OrderTerm, ParseError> {
    let parts: Vec<&str> = term_str.split('.').collect();

    if parts.is_empty() || parts[0].is_empty() {
        return Err(ParseError::InvalidOrderOptions(term_str.to_string()));
    }

    let (field_parts, option_parts) = split_field_and_options(&parts);

    let field_str = field_parts.join(".");
    let field = parse_order_field(&field_str)?;

    let (direction, nulls) = parse_options(&option_parts)?;

    let mut term = OrderTerm::new(field).with_direction(direction);
    if let Some(n) = nulls {
        term = term.with_nulls(n);
    }

    Ok(term)
}

fn split_field_and_options(parts: &[&str]) -> (Vec<String>, Vec<String>) {
    if parts.is_empty() {
        return (Vec::new(), Vec::new());
    }

    let mut field_parts = vec![parts[0].to_string()];
    let mut option_parts = Vec::new();
    let mut seen_option = false;

    for part in &parts[1..] {
        if is_direction(part) || is_nulls_option(part) {
            option_parts.push(part.to_string());
            seen_option = true;
        } else if !seen_option && (part.contains("->") || part.contains("::")) {
            field_parts.push(part.to_string());
        } else {
            option_parts.push(part.to_string());
        }
    }

    (field_parts, option_parts)
}

fn parse_order_field(field_str: &str) -> Result<Field, ParseError> {
    match field(field_str) {
        Ok((_, field)) => Ok(field),
        Err(_) => parse_field_fallback(field_str),
    }
}


fn parse_options(option_parts: &[String]) -> Result<(Direction, Option<Nulls>), ParseError> {
    let mut direction = Direction::Asc;
    let mut nulls: Option<Nulls> = None;

    for part in option_parts {
        let lower = part.to_lowercase();
        match lower.as_str() {
            "asc" => direction = Direction::Asc,
            "desc" => direction = Direction::Desc,
            "nullsfirst" => nulls = Some(Nulls::First),
            "nullslast" => nulls = Some(Nulls::Last),
            _ => return Err(ParseError::InvalidOrderOptions(lower.to_string())),
        }
    }

    Ok((direction, nulls))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_order_simple() {
        let result = parse_order("id");
        assert!(result.is_ok());
        let terms = result.unwrap();
        assert_eq!(terms.len(), 1);
        assert_eq!(terms[0].direction, Direction::Asc);
    }

    #[test]
    fn test_parse_order_desc() {
        let result = parse_order("id.desc");
        assert!(result.is_ok());
        let terms = result.unwrap();
        assert_eq!(terms[0].direction, Direction::Desc);
    }

    #[test]
    fn test_parse_order_with_nulls() {
        let result = parse_order("id.desc.nullslast");
        assert!(result.is_ok());
        let terms = result.unwrap();
        assert_eq!(terms[0].direction, Direction::Desc);
        assert_eq!(terms[0].nulls, Some(Nulls::Last));
    }

    #[test]
    fn test_parse_order_multiple() {
        let result = parse_order("id.desc,name.asc");
        assert!(result.is_ok());
        let terms = result.unwrap();
        assert_eq!(terms.len(), 2);
        assert_eq!(terms[0].direction, Direction::Desc);
        assert_eq!(terms[1].direction, Direction::Asc);
    }

    #[test]
    fn test_parse_order_with_json_path() {
        let result = parse_order("data->key.desc");
        assert!(result.is_ok());
        let terms = result.unwrap();
        assert_eq!(terms[0].field.name, "data");
        assert_eq!(terms[0].field.json_path.len(), 1);
    }

    #[test]
    fn test_parse_order_empty() {
        let result = parse_order("");
        assert!(result.is_ok());
        assert!(result.unwrap().is_empty());
    }

    #[test]
    fn test_parse_order_invalid() {
        let result = parse_order("id.invalid");
        assert!(matches!(result, Err(ParseError::InvalidOrderOptions(_))));
    }

    #[test]
    fn test_parse_order_with_cast() {
        let result = parse_order("price::numeric.desc");
        assert!(result.is_ok());
        let terms = result.unwrap();
        assert_eq!(terms[0].field.cast, Some("numeric".to_string()));
    }
}
