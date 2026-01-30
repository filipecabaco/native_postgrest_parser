use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Field {
    pub name: String,
    pub json_path: Vec<JsonOp>,
    pub cast: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum JsonOp {
    Arrow(String),
    DoubleArrow(String),
    ArrayIndex(i32),
}

impl Field {
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            json_path: Vec::new(),
            cast: None,
        }
    }

    pub fn with_json_path(mut self, ops: Vec<JsonOp>) -> Self {
        self.json_path = ops;
        self
    }

    pub fn with_cast(mut self, cast: impl Into<String>) -> Self {
        self.cast = Some(cast.into());
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_field_new() {
        let field = Field::new("id");
        assert_eq!(field.name, "id");
        assert!(field.json_path.is_empty());
        assert!(field.cast.is_none());
    }

    #[test]
    fn test_field_with_json_path() {
        let field = Field::new("data").with_json_path(vec![JsonOp::Arrow("key".to_string())]);
        assert_eq!(field.name, "data");
        assert_eq!(field.json_path.len(), 1);
    }

    #[test]
    fn test_field_with_cast() {
        let field = Field::new("price").with_cast("numeric");
        assert_eq!(field.name, "price");
        assert_eq!(field.cast, Some("numeric".to_string()));
    }

    #[test]
    fn test_json_op_serialization() {
        let op = JsonOp::Arrow("key".to_string());
        let json = serde_json::to_string(&op).unwrap();
        assert!(json.contains("arrow"));
    }
}
