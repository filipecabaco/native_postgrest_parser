use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ItemType {
    Field,
    Relation,
    Spread,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(untagged)]
pub enum ItemHint {
    Inner(String),
    JsonPathCast(Vec<String>, String),
    JsonPath(Vec<String>),
    Cast(String),
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SelectItem {
    pub item_type: ItemType,
    pub name: String,
    pub alias: Option<String>,
    pub children: Option<Vec<SelectItem>>,
    pub hint: Option<ItemHint>,
}

impl SelectItem {
    pub fn field(name: impl Into<String>) -> Self {
        Self {
            item_type: ItemType::Field,
            name: name.into(),
            alias: None,
            children: None,
            hint: None,
        }
    }

    pub fn with_alias(mut self, alias: impl Into<String>) -> Self {
        self.alias = Some(alias.into());
        self
    }

    pub fn with_children(mut self, children: Vec<SelectItem>) -> Self {
        self.children = Some(children);
        self
    }

    pub fn with_hint(mut self, hint: ItemHint) -> Self {
        self.hint = Some(hint);
        self
    }

    pub fn relation(name: impl Into<String>) -> Self {
        Self {
            item_type: ItemType::Relation,
            name: name.into(),
            alias: None,
            children: None,
            hint: None,
        }
    }

    pub fn spread(name: impl Into<String>) -> Self {
        Self {
            item_type: ItemType::Spread,
            name: name.into(),
            alias: None,
            children: None,
            hint: None,
        }
    }

    pub fn wildcard() -> Self {
        Self::field("*".to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_select_item_field() {
        let item = SelectItem::field("id");
        assert_eq!(item.item_type, ItemType::Field);
        assert_eq!(item.name, "id");
        assert!(item.alias.is_none());
    }

    #[test]
    fn test_select_item_with_alias() {
        let item = SelectItem::field("name").with_alias("user_name");
        assert_eq!(item.alias, Some("user_name".to_string()));
    }

    #[test]
    fn test_select_item_with_children() {
        let item = SelectItem::relation("client")
            .with_children(vec![SelectItem::field("id"), SelectItem::field("name")]);
        assert_eq!(item.item_type, ItemType::Relation);
        assert_eq!(item.children.as_ref().unwrap().len(), 2);
    }

    #[test]
    fn test_select_item_wildcard() {
        let item = SelectItem::wildcard();
        assert_eq!(item.name, "*");
        assert_eq!(item.item_type, ItemType::Field);
    }

    #[test]
    fn test_select_item_spread() {
        let item = SelectItem::spread("profile");
        assert_eq!(item.item_type, ItemType::Spread);
        assert_eq!(item.name, "profile");
    }

    #[test]
    fn test_select_item_serialization() {
        let item = SelectItem::field("id").with_alias("user_id");
        let json = serde_json::to_string(&item).unwrap();
        assert!(json.contains("id"));
        assert!(json.contains("user_id"));
    }
}
