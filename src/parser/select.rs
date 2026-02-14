use crate::ast::{ItemType, JsonOp, SelectItem};
use crate::error::ParseError;

/// Parses a PostgREST select clause into a list of select items.
///
/// Supports column selection, renaming, JSON path navigation, type casting,
/// and nested resource embedding (relations).
///
/// # Syntax
///
/// - Columns: `col1,col2,col3`
/// - Wildcard: `*`
/// - Rename: `alias:column` (note: alias comes first)
/// - JSON path: `data->key` or `data->>key`
/// - Type cast: `price::numeric`
/// - Nested relations: `users(id,name,posts(title))`
/// - Spread operator: `...foreign_table(col1,col2)`
///
/// # Examples
///
/// ```
/// use postgrest_parser::parse_select;
///
/// // Simple columns
/// let items = parse_select("id,name,email").unwrap();
/// assert_eq!(items.len(), 3);
///
/// // With alias (alias:field_name syntax)
/// let items = parse_select("full_name:name,user_email:email").unwrap();
/// assert_eq!(items.len(), 2);
/// assert_eq!(items[0].name, "name");
/// assert_eq!(items[0].alias, Some("full_name".to_string()));
///
/// // Wildcard
/// let items = parse_select("*").unwrap();
/// assert_eq!(items.len(), 1);
///
/// // JSON path
/// let items = parse_select("data->user->name,metadata->>key").unwrap();
/// assert_eq!(items.len(), 2);
///
/// // Nested relation
/// let items = parse_select("id,name,orders(id,total,items(product_id))").unwrap();
/// assert_eq!(items.len(), 3);
///
/// // Type cast
/// let items = parse_select("price::numeric,created_at::text").unwrap();
/// assert_eq!(items.len(), 2);
/// ```
///
/// # Errors
///
/// Returns `ParseError` if:
/// - Parentheses are unclosed
/// - Relation syntax is malformed
/// - Field names are invalid
pub fn parse_select(select_str: &str) -> Result<Vec<SelectItem>, ParseError> {
    if select_str.is_empty() {
        return Ok(Vec::new());
    }

    if select_str.trim() == "*" {
        return Ok(vec![SelectItem::wildcard()]);
    }

    tokenize_and_parse(select_str)
}

fn tokenize_and_parse(select_str: &str) -> Result<Vec<SelectItem>, ParseError> {
    let tokens = tokenize(select_str)?;
    parse_items(&tokens)
}

fn tokenize(select_str: &str) -> Result<Vec<SelectToken>, ParseError> {
    let mut tokens = Vec::new();
    let mut current = String::new();
    let mut depth = 0;

    for c in select_str.chars() {
        match c {
            '(' => {
                if !current.is_empty() {
                    tokens.push(SelectToken::Text(current.clone()));
                }
                tokens.push(SelectToken::OpenParen);
                current.clear();
                depth += 1;
            }
            ')' => {
                if !current.is_empty() {
                    tokens.push(SelectToken::Text(current.clone()));
                    current.clear();
                }
                tokens.push(SelectToken::CloseParen);
                depth -= 1;
            }
            ',' => {
                if !current.is_empty() {
                    tokens.push(SelectToken::Text(current.clone()));
                    current.clear();
                }
                tokens.push(SelectToken::Comma);
            }
            _ => {
                current.push(c);
            }
        }
    }

    if !current.is_empty() {
        tokens.push(SelectToken::Text(current));
    }

    if depth != 0 {
        return Err(ParseError::UnclosedParenthesisInSelect);
    }

    Ok(tokens)
}

#[derive(Debug, Clone, PartialEq)]
enum SelectToken {
    Text(String),
    OpenParen,
    CloseParen,
    Comma,
}

fn parse_items(tokens: &[SelectToken]) -> Result<Vec<SelectItem>, ParseError> {
    let mut items = Vec::new();
    let mut index = 0;

    while index < tokens.len() {
        match &tokens[index] {
            SelectToken::Text(text) => {
                let has_children =
                    index + 1 < tokens.len() && matches!(tokens[index + 1], SelectToken::OpenParen);

                let item = parse_item_text(text, has_children)?;

                if matches!(item.item_type, ItemType::Relation | ItemType::Spread) {
                    if !has_children {
                        return Err(ParseError::ExpectedParenthesisAfterRelation);
                    }

                    let (children, next_index) = parse_nested_children(tokens, index + 2)?;
                    let item_with_children = item.with_children(children);
                    items.push(item_with_children);
                    index = next_index;
                } else {
                    items.push(item);
                    index += 1;
                }
            }
            SelectToken::OpenParen => {
                return Err(ParseError::UnexpectedToken("(".to_string()));
            }
            SelectToken::CloseParen => {
                return Err(ParseError::UnexpectedClosingParenthesis);
            }
            SelectToken::Comma => {
                index += 1;
            }
        }
    }

    Ok(items)
}

fn parse_nested_children(
    tokens: &[SelectToken],
    start: usize,
) -> Result<(Vec<SelectItem>, usize), ParseError> {
    let mut children = Vec::new();
    let mut index = start;
    let mut depth = 1;

    while index < tokens.len() && depth > 0 {
        match &tokens[index] {
            SelectToken::Text(text) => {
                let has_children =
                    index + 1 < tokens.len() && matches!(tokens[index + 1], SelectToken::OpenParen);

                let item = parse_item_text(text, has_children)?;

                if matches!(item.item_type, ItemType::Relation | ItemType::Spread) {
                    if !has_children {
                        return Err(ParseError::ExpectedParenthesisAfterRelation);
                    }

                    let (nested_children, next_index) = parse_nested_children(tokens, index + 2)?;
                    let item_with_children = item.with_children(nested_children);
                    children.push(item_with_children);
                    index = next_index;
                } else {
                    children.push(item);
                    index += 1;
                }
            }
            SelectToken::OpenParen => {
                depth += 1;
                index += 1;
            }
            SelectToken::CloseParen => {
                depth -= 1;
                if depth == 0 {
                    index += 1;
                    break;
                }
                index += 1;
            }
            SelectToken::Comma => {
                index += 1;
            }
        }
    }

    Ok((children, index))
}

fn parse_item_text(text: &str, has_children: bool) -> Result<SelectItem, ParseError> {
    let trimmed = text.trim();

    if trimmed.is_empty() {
        return Err(ParseError::EmptyFieldName);
    }

    let is_spread = trimmed.starts_with("...");
    let (name_part, alias) = extract_alias(if is_spread { &trimmed[3..] } else { trimmed })?;

    let (name, hint) = extract_hint(&name_part)?;

    if name.is_empty() {
        return Err(ParseError::EmptyFieldName);
    }

    let item_type = if is_spread {
        ItemType::Spread
    } else if has_children {
        ItemType::Relation
    } else {
        ItemType::Field
    };

    let mut item = match item_type {
        ItemType::Field => SelectItem::field(name.clone()),
        ItemType::Relation => SelectItem::relation(name.clone()),
        ItemType::Spread => SelectItem::spread(name.clone()),
    };

    if let Some(alias_name) = alias {
        item = item.with_alias(alias_name);
    }

    if let Some(h) = hint {
        item = item.with_hint(h);
    }

    Ok(item)
}

fn extract_alias(text: &str) -> Result<(String, Option<String>), ParseError> {
    if text.contains(':') {
        let parts: Vec<&str> = text.splitn(2, ':').collect();
        if parts.len() == 2 {
            Ok((
                parts[1].trim().to_string(),
                Some(parts[0].trim().to_string()),
            ))
        } else {
            Ok((text.to_string(), None))
        }
    } else {
        Ok((text.to_string(), None))
    }
}

fn extract_hint(text: &str) -> Result<(String, Option<crate::ast::ItemHint>), ParseError> {
    if let Some(pos) = text.find('!') {
        let name = text[..pos].to_string();
        let hint_str = text[pos + 1..].to_string();

        let hint = parse_field_for_hint(&name, &hint_str)?;
        Ok((name, Some(hint)))
    } else {
        Ok((text.to_string(), None))
    }
}

fn parse_field_for_hint(name: &str, hint_str: &str) -> Result<crate::ast::ItemHint, ParseError> {
    match crate::parser::common::field(name) {
        Ok((_, field)) => {
            let json_path_vec = || {
                field
                    .json_path
                    .iter()
                    .map(|op| match op {
                        JsonOp::Arrow(s) | JsonOp::DoubleArrow(s) => s.clone(),
                        JsonOp::ArrayIndex(i) => i.to_string(),
                    })
                    .collect()
            };

            match (field.json_path.is_empty(), field.cast) {
                (true, None) => Ok(crate::ast::ItemHint::Inner(hint_str.to_string())),
                (true, Some(cast)) => Ok(crate::ast::ItemHint::Cast(cast.to_string())),
                (false, None) => Ok(crate::ast::ItemHint::JsonPath(json_path_vec())),
                (false, Some(cast)) => Ok(crate::ast::ItemHint::JsonPathCast(
                    json_path_vec(),
                    cast.to_string(),
                )),
            }
        }
        Err(_) => Ok(crate::ast::ItemHint::Inner(hint_str.to_string())),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_select_simple() {
        let result = parse_select("id,name,email");
        assert!(result.is_ok());
        let items = result.unwrap();
        assert_eq!(items.len(), 3);
        assert_eq!(items[0].name, "id");
    }

    #[test]
    fn test_parse_select_wildcard() {
        let result = parse_select("*");
        assert!(result.is_ok());
        let items = result.unwrap();
        assert_eq!(items.len(), 1);
        assert_eq!(items[0].name, "*");
    }

    #[test]
    fn test_parse_select_with_alias() {
        let result = parse_select("user_name:name,user_email:email");
        assert!(result.is_ok());
        let items = result.unwrap();
        assert_eq!(items[0].alias, Some("user_name".to_string()));
    }

    #[test]
    fn test_parse_select_with_relation() {
        let result = parse_select("id,client(id,name)");
        assert!(result.is_ok());
        let items = result.unwrap();
        assert_eq!(items.len(), 2);
        assert_eq!(items[1].item_type, ItemType::Relation);
        assert_eq!(items[1].children.as_ref().unwrap().len(), 2);
    }

    #[test]
    fn test_parse_select_with_spread() {
        let result = parse_select("id,...profile(name)");
        assert!(result.is_ok());
        let items = result.unwrap();
        assert_eq!(items[1].item_type, ItemType::Spread);
    }

    #[test]
    fn test_parse_select_with_hint() {
        let result = parse_select("author!inner,client!left");
        assert!(result.is_ok());
        let items = result.unwrap();
        assert!(items[0].hint.is_some());
    }

    #[test]
    fn test_parse_select_nested_relations() {
        let result = parse_select("id,client(id,orders(id,total))");
        assert!(result.is_ok());
        let items = result.unwrap();
        assert_eq!(items.len(), 2);

        let client_children = items[1].children.as_ref().unwrap();
        assert_eq!(client_children.len(), 2);
        assert_eq!(client_children[1].item_type, ItemType::Relation);
    }

    #[test]
    fn test_parse_select_empty() {
        let result = parse_select("");
        assert!(result.is_ok());
        assert!(result.unwrap().is_empty());
    }

    #[test]
    fn test_parse_select_unclosed_parenthesis() {
        let result = parse_select("client(id,name");
        assert!(matches!(
            result,
            Err(ParseError::UnclosedParenthesisInSelect)
        ));
    }

    // Resource embedding use cases (PostgREST select syntax)

    #[test]
    fn test_many_to_one_join_via_fk() {
        // select("*, profiles(username, avatar_url)")
        let items = parse_select("*, profiles(username, avatar_url)").unwrap();
        assert_eq!(items.len(), 2);
        assert_eq!(items[0].name, "*");
        assert_eq!(items[0].item_type, ItemType::Field);
        assert_eq!(items[1].name, "profiles");
        assert_eq!(items[1].item_type, ItemType::Relation);
        let children = items[1].children.as_ref().unwrap();
        assert_eq!(children.len(), 2);
        assert_eq!(children[0].name, "username");
        assert_eq!(children[1].name, "avatar_url");
    }

    #[test]
    fn test_one_to_many_join() {
        // select("title, comments(id, body)")
        let items = parse_select("title, comments(id, body)").unwrap();
        assert_eq!(items.len(), 2);
        assert_eq!(items[0].name, "title");
        assert_eq!(items[1].name, "comments");
        assert_eq!(items[1].item_type, ItemType::Relation);
        let children = items[1].children.as_ref().unwrap();
        assert_eq!(children.len(), 2);
        assert_eq!(children[0].name, "id");
        assert_eq!(children[1].name, "body");
    }

    #[test]
    fn test_aliased_relation() {
        // select("*, author:profiles(name)") — aliased relation
        let items = parse_select("*, author:profiles(name)").unwrap();
        assert_eq!(items.len(), 2);
        assert_eq!(items[1].name, "profiles");
        assert_eq!(items[1].alias, Some("author".to_string()));
        assert_eq!(items[1].item_type, ItemType::Relation);
        let children = items[1].children.as_ref().unwrap();
        assert_eq!(children.len(), 1);
        assert_eq!(children[0].name, "name");
    }

    #[test]
    fn test_nested_embedding_with_alias() {
        // select("*, comments(id, author:profiles(name))") — nested embedding with alias
        let items = parse_select("*, comments(id, author:profiles(name))").unwrap();
        assert_eq!(items.len(), 2);
        assert_eq!(items[1].name, "comments");
        assert_eq!(items[1].item_type, ItemType::Relation);

        let children = items[1].children.as_ref().unwrap();
        assert_eq!(children.len(), 2);
        assert_eq!(children[0].name, "id");

        // The nested aliased relation
        assert_eq!(children[1].name, "profiles");
        assert_eq!(children[1].alias, Some("author".to_string()));
        assert_eq!(children[1].item_type, ItemType::Relation);
        let nested = children[1].children.as_ref().unwrap();
        assert_eq!(nested.len(), 1);
        assert_eq!(nested[0].name, "name");
    }

    #[test]
    fn test_fk_hint_on_relation() {
        // select("*, author:profiles!author_id_fkey(name)") — FK hint for disambiguation
        let items = parse_select("*, author:profiles!author_id_fkey(name)").unwrap();
        assert_eq!(items.len(), 2);
        assert_eq!(items[1].name, "profiles");
        assert_eq!(items[1].alias, Some("author".to_string()));
        assert_eq!(items[1].item_type, ItemType::Relation);
        assert!(items[1].hint.is_some());
        assert_eq!(
            items[1].hint,
            Some(crate::ast::ItemHint::Inner("author_id_fkey".to_string()))
        );
        let children = items[1].children.as_ref().unwrap();
        assert_eq!(children.len(), 1);
        assert_eq!(children[0].name, "name");
    }

    #[test]
    fn test_fk_hint_without_alias() {
        // select("*, profiles!author_id_fkey(name)") — FK hint without alias
        let items = parse_select("*, profiles!author_id_fkey(name)").unwrap();
        assert_eq!(items.len(), 2);
        assert_eq!(items[1].name, "profiles");
        assert_eq!(items[1].alias, None);
        assert_eq!(items[1].item_type, ItemType::Relation);
        assert!(items[1].hint.is_some());
        assert_eq!(
            items[1].hint,
            Some(crate::ast::ItemHint::Inner("author_id_fkey".to_string()))
        );
    }

    #[test]
    fn test_multiple_relations() {
        // select("id, author:profiles(name), comments(id, body)")
        let items = parse_select("id, author:profiles(name), comments(id, body)").unwrap();
        assert_eq!(items.len(), 3);
        assert_eq!(items[0].name, "id");
        assert_eq!(items[0].item_type, ItemType::Field);

        assert_eq!(items[1].name, "profiles");
        assert_eq!(items[1].alias, Some("author".to_string()));
        assert_eq!(items[1].item_type, ItemType::Relation);

        assert_eq!(items[2].name, "comments");
        assert_eq!(items[2].item_type, ItemType::Relation);
        assert_eq!(items[2].children.as_ref().unwrap().len(), 2);
    }

    #[test]
    fn test_deeply_nested_relations() {
        // select("*, posts(id, comments(id, author:profiles(name, avatar_url)))")
        let items =
            parse_select("*, posts(id, comments(id, author:profiles(name, avatar_url)))").unwrap();
        assert_eq!(items.len(), 2);

        let posts = &items[1];
        assert_eq!(posts.name, "posts");
        let post_children = posts.children.as_ref().unwrap();
        assert_eq!(post_children.len(), 2);

        let comments = &post_children[1];
        assert_eq!(comments.name, "comments");
        let comment_children = comments.children.as_ref().unwrap();
        assert_eq!(comment_children.len(), 2);

        let author = &comment_children[1];
        assert_eq!(author.name, "profiles");
        assert_eq!(author.alias, Some("author".to_string()));
        let author_children = author.children.as_ref().unwrap();
        assert_eq!(author_children.len(), 2);
        assert_eq!(author_children[0].name, "name");
        assert_eq!(author_children[1].name, "avatar_url");
    }
}
