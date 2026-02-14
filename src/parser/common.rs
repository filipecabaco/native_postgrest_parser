use crate::ast::{Field, JsonOp};
use crate::error::ParseError;
use nom::{
    branch::alt,
    bytes::complete::{tag, take_while, take_while1},
    character::complete::{char, one_of},
    combinator::{opt, recognize},
    multi::{many0, many1, separated_list1},
    sequence::{delimited, preceded},
    IResult,
};

pub fn identifier(i: &str) -> IResult<&str, String> {
    let (i, s) = recognize(many1(alt((one_of(
        "abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789_",
    ),))))(i)?;
    Ok((i, s.to_string()))
}

pub fn json_double_arrow(i: &str) -> IResult<&str, JsonOp> {
    let (i, _) = tag("->>")(i)?;
    Ok((i, JsonOp::DoubleArrow(String::new())))
}

pub fn json_single_arrow(i: &str) -> IResult<&str, JsonOp> {
    let (i, _) = tag("->")(i)?;
    Ok((i, JsonOp::Arrow(String::new())))
}

pub fn json_operator(i: &str) -> IResult<&str, JsonOp> {
    alt((json_double_arrow, json_single_arrow))(i)
}

pub fn json_path_segment(i: &str) -> IResult<&str, JsonOp> {
    let (i, op) = json_operator(i)?;
    let (i, key) = identifier(i)?;
    let json_op = match op {
        JsonOp::Arrow(_) => JsonOp::Arrow(key),
        JsonOp::DoubleArrow(_) => JsonOp::DoubleArrow(key),
        JsonOp::ArrayIndex(idx) => JsonOp::ArrayIndex(idx),
    };
    Ok((i, json_op))
}

pub fn json_path(i: &str) -> IResult<&str, Vec<JsonOp>> {
    many0(json_path_segment)(i)
}

pub fn type_cast(i: &str) -> IResult<&str, String> {
    preceded(tag("::"), identifier)(i)
}

pub fn field(i: &str) -> IResult<&str, Field> {
    let (i, name) = identifier(i)?;
    let (i, json_path) = json_path(i)?;
    let (i, cast) = opt(type_cast)(i)?;
    Ok((
        i,
        Field {
            name,
            json_path,
            cast,
        },
    ))
}

pub fn quoted_string(i: &str) -> IResult<&str, &str> {
    delimited(char('"'), take_while(|c| c != '"'), char('"'))(i)
}

pub fn paren_list(i: &str) -> IResult<&str, Vec<String>> {
    delimited(char('('), list_items, char(')'))(i)
}

pub fn brace_list(i: &str) -> IResult<&str, Vec<String>> {
    delimited(char('{'), list_items, char('}'))(i)
}

fn list_items(i: &str) -> IResult<&str, Vec<String>> {
    let (i, items) = separated_list1(
        preceded(opt(whitespace), char(',')),
        delimited(opt(whitespace), list_item, opt(whitespace)),
    )(i)?;
    let strings: Vec<String> = items.iter().map(|s| s.to_string()).collect();
    Ok((i, strings))
}

pub fn list_item(i: &str) -> IResult<&str, &str> {
    alt((unquoted_list_item, quoted_list_item))(i)
}

pub fn unquoted_list_item(i: &str) -> IResult<&str, &str> {
    take_while1(|c| c != ',' && c != ')' && c != '}')(i)
}

pub fn quoted_list_item(i: &str) -> IResult<&str, &str> {
    quoted_string(i)
}

pub fn whitespace(i: &str) -> IResult<&str, &str> {
    take_while(char::is_whitespace)(i)
}

pub fn parse_json_path(field_str: &str) -> Result<(String, Vec<JsonOp>), ParseError> {
    if !field_str.contains("->") && !field_str.contains("->>") {
        return Ok((field_str.to_string(), Vec::new()));
    }

    let mut name = String::new();
    let mut json_path = Vec::new();
    let mut chars = field_str.chars().peekable();

    while let Some(c) = chars.next() {
        match c {
            '-' => {
                if let Some(&'>') = chars.peek() {
                    chars.next();
                    if let Some(&'>') = chars.peek() {
                        chars.next();
                        let key = take_json_identifier(&mut chars);
                        json_path.push(JsonOp::DoubleArrow(key));
                    } else {
                        let key = take_json_identifier(&mut chars);
                        json_path.push(JsonOp::Arrow(key));
                    }
                } else {
                    name.push(c);
                }
            }
            _ => name.push(c),
        }
    }

    if name.is_empty() {
        return Err(ParseError::EmptyFieldName);
    }

    Ok((name, json_path))
}

fn take_json_identifier(chars: &mut std::iter::Peekable<std::str::Chars>) -> String {
    let mut result = String::new();
    while let Some(&c) = chars.peek() {
        if c.is_alphanumeric() || c == '_' {
            result.push(c);
            chars.next();
        } else {
            break;
        }
    }
    result
}

pub fn parse_field_fallback(field_str: &str) -> Result<Field, ParseError> {
    let parts: Vec<&str> = field_str.split("::").collect();

    match parts.as_slice() {
        [field_part, cast] => {
            let (name, json_path) = parse_json_path(field_part)?;
            Ok(Field {
                name,
                json_path,
                cast: Some(cast.to_string()),
            })
        }
        [field_part] => {
            let (name, json_path) = parse_json_path(field_part)?;
            Ok(Field {
                name,
                json_path,
                cast: None,
            })
        }
        _ => Err(ParseError::InvalidFieldName(field_str.to_string())),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_identifier_simple() {
        let result = identifier("id");
        assert!(result.is_ok());
        assert_eq!(result.unwrap().1, "id");
    }

    #[test]
    fn test_identifier_with_underscore() {
        let result = identifier("user_id");
        assert!(result.is_ok());
        assert_eq!(result.unwrap().1, "user_id");
    }

    #[test]
    fn test_json_path_segment_arrow() {
        let result = json_path_segment("->key");
        assert!(result.is_ok());
        let (_, op) = result.unwrap();
        assert_eq!(op, JsonOp::Arrow("key".to_string()));
    }

    #[test]
    fn test_json_path_segment_double_arrow() {
        let result = json_path_segment("->>key");
        assert!(result.is_ok());
        let (_, op) = result.unwrap();
        assert_eq!(op, JsonOp::DoubleArrow("key".to_string()));
    }

    #[test]
    fn test_json_path_multiple() {
        let result = json_path("->outer->inner->>final");
        assert!(result.is_ok());
        let (_, path) = result.unwrap();
        assert_eq!(path.len(), 3);
    }

    #[test]
    fn test_type_cast() {
        let result = type_cast("::text");
        assert!(result.is_ok());
        assert_eq!(result.unwrap().1, "text");
    }

    #[test]
    fn test_field_simple() {
        let result = field("id");
        assert!(result.is_ok());
        let field = result.unwrap().1;
        assert_eq!(field.name, "id");
        assert!(field.json_path.is_empty());
        assert!(field.cast.is_none());
    }

    #[test]
    fn test_field_with_json_path() {
        let result = field("data->key");
        assert!(result.is_ok());
        let field = result.unwrap().1;
        assert_eq!(field.name, "data");
        assert_eq!(field.json_path.len(), 1);
    }

    #[test]
    fn test_field_with_type_cast() {
        let result = field("price::numeric");
        assert!(result.is_ok());
        let field = result.unwrap().1;
        assert_eq!(field.cast, Some("numeric".to_string()));
    }

    #[test]
    fn test_field_with_json_path_and_cast() {
        let result = field("data->price::numeric");
        assert!(result.is_ok());
        let field = result.unwrap().1;
        assert_eq!(field.name, "data");
        assert_eq!(field.json_path.len(), 1);
        assert_eq!(field.cast, Some("numeric".to_string()));
    }

    #[test]
    fn test_paren_list() {
        let result = paren_list("(item1,item2,item3)");
        assert!(result.is_ok());
        let items = result.unwrap().1;
        assert_eq!(items.len(), 3);
    }

    #[test]
    fn test_brace_list() {
        let result = brace_list("{item1,item2,item3}");
        assert!(result.is_ok());
        let items = result.unwrap().1;
        assert_eq!(items.len(), 3);
    }

    #[test]
    fn test_quoted_string() {
        let result = quoted_string("\"test string\"");
        assert!(result.is_ok());
        assert_eq!(result.unwrap().1, "test string");
    }
}
