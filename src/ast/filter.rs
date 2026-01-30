use super::Field;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FilterOperator {
    Eq,
    Neq,
    Gt,
    Gte,
    Lt,
    Lte,
    Like,
    Ilike,
    Match,
    Imatch,
    In,
    Is,
    Fts,
    Plfts,
    Phfts,
    Wfts,
    Cs,
    Cd,
    Ov,
    Sl,
    Sr,
    Nxl,
    Nxr,
    Adj,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Quantifier {
    Any,
    All,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum FilterValue {
    Single(String),
    List(Vec<String>),
}

impl FilterValue {
    pub fn as_str(&self) -> Option<&str> {
        match self {
            FilterValue::Single(s) => Some(s.as_str()),
            FilterValue::List(_) => None,
        }
    }

    pub fn as_list(&self) -> Option<&[String]> {
        match self {
            FilterValue::Single(_) => None,
            FilterValue::List(list) => Some(list.as_slice()),
        }
    }

    pub fn to_json(&self) -> serde_json::Value {
        match self {
            FilterValue::Single(s) => serde_json::Value::String(s.clone()),
            FilterValue::List(list) => serde_json::Value::Array(
                list.iter()
                    .map(|s| {
                        if let Ok(i) = s.parse::<i64>() {
                            serde_json::Value::Number(i.into())
                        } else if let Ok(f) = s.parse::<f64>() {
                            if let Some(num) = serde_json::Number::from_f64(f) {
                                serde_json::Value::Number(num)
                            } else {
                                serde_json::Value::String(s.clone())
                            }
                        } else {
                            serde_json::Value::String(s.clone())
                        }
                    })
                    .collect(),
            ),
        }
    }
}

impl From<&str> for FilterValue {
    fn from(s: &str) -> Self {
        FilterValue::Single(s.to_string())
    }
}

impl From<String> for FilterValue {
    fn from(s: String) -> Self {
        FilterValue::Single(s)
    }
}

impl From<Vec<String>> for FilterValue {
    fn from(v: Vec<String>) -> Self {
        FilterValue::List(v)
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Filter {
    pub field: Field,
    pub operator: FilterOperator,
    pub value: FilterValue,
    pub quantifier: Option<Quantifier>,
    pub language: Option<String>,
    pub negated: bool,
}

impl Filter {
    pub fn new(field: Field, operator: FilterOperator, value: FilterValue) -> Self {
        Self {
            field,
            operator,
            value,
            quantifier: None,
            language: None,
            negated: false,
        }
    }

    pub fn with_quantifier(mut self, quantifier: Quantifier) -> Self {
        self.quantifier = Some(quantifier);
        self
    }

    pub fn with_language(mut self, language: impl Into<String>) -> Self {
        self.language = Some(language.into());
        self
    }

    pub fn negated(mut self) -> Self {
        self.negated = true;
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_filter_new() {
        let field = Field::new("id");
        let filter = Filter::new(
            field,
            FilterOperator::Eq,
            FilterValue::Single("1".to_string()),
        );

        assert_eq!(filter.operator, FilterOperator::Eq);
        assert!(!filter.negated);
        assert!(filter.quantifier.is_none());
    }

    #[test]
    fn test_filter_with_quantifier() {
        let field = Field::new("status");
        let filter = Filter::new(
            field,
            FilterOperator::Eq,
            FilterValue::List(vec!["active".to_string(), "pending".to_string()]),
        )
        .with_quantifier(Quantifier::Any);

        assert_eq!(filter.quantifier, Some(Quantifier::Any));
    }

    #[test]
    fn test_filter_negated() {
        let field = Field::new("status");
        let filter = Filter::new(
            field,
            FilterOperator::Eq,
            FilterValue::Single("deleted".to_string()),
        )
        .negated();

        assert!(filter.negated);
    }

    #[test]
    fn test_filter_value_as_str() {
        let value = FilterValue::Single("test".to_string());
        assert_eq!(value.as_str(), Some("test"));
    }

    #[test]
    fn test_filter_value_as_list() {
        let value = FilterValue::List(vec!["a".to_string(), "b".to_string()]);
        let expected: Vec<String> = vec!["a".to_string(), "b".to_string()];
        assert_eq!(value.as_list(), Some(expected.as_slice()));
    }

    #[test]
    fn test_filter_value_to_json() {
        let value = FilterValue::Single("test".to_string());
        let json = value.to_json();
        assert_eq!(json, serde_json::Value::String("test".to_string()));
    }

    #[test]
    fn test_filter_value_list_to_json() {
        let value = FilterValue::List(vec!["1".to_string(), "2".to_string(), "3".to_string()]);
        let json = value.to_json();
        match json {
            serde_json::Value::Array(arr) => {
                assert_eq!(arr.len(), 3);
                assert_eq!(arr[0], serde_json::Value::Number(1.into()));
            }
            _ => panic!("Expected array"),
        }
    }
}
