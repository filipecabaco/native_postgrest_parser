use super::Filter;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum LogicOperator {
    And,
    Or,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum LogicCondition {
    Filter(Filter),
    Logic(LogicTree),
}

impl From<Filter> for LogicCondition {
    fn from(filter: Filter) -> Self {
        LogicCondition::Filter(filter)
    }
}

impl From<LogicTree> for LogicCondition {
    fn from(tree: LogicTree) -> Self {
        LogicCondition::Logic(tree)
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LogicTree {
    pub operator: LogicOperator,
    pub conditions: Vec<LogicCondition>,
    pub negated: bool,
}

impl LogicTree {
    pub fn new(operator: LogicOperator) -> Self {
        Self {
            operator,
            conditions: Vec::new(),
            negated: false,
        }
    }

    pub fn with_conditions(mut self, conditions: Vec<LogicCondition>) -> Self {
        self.conditions = conditions;
        self
    }

    pub fn and() -> Self {
        Self::new(LogicOperator::And)
    }

    pub fn or() -> Self {
        Self::new(LogicOperator::Or)
    }

    pub fn negated(mut self) -> Self {
        self.negated = true;
        self
    }

    pub fn add_condition(mut self, condition: LogicCondition) -> Self {
        self.conditions.push(condition);
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ast::{Field, FilterOperator, FilterValue};

    #[test]
    fn test_logic_tree_and() {
        let tree = LogicTree::and();
        assert_eq!(tree.operator, LogicOperator::And);
        assert!(!tree.negated);
        assert!(tree.conditions.is_empty());
    }

    #[test]
    fn test_logic_tree_or() {
        let tree = LogicTree::or();
        assert_eq!(tree.operator, LogicOperator::Or);
    }

    #[test]
    fn test_logic_tree_with_conditions() {
        let filter1 = Filter::new(
            Field::new("id"),
            FilterOperator::Eq,
            FilterValue::Single("1".to_string()),
        );
        let filter2 = Filter::new(
            Field::new("status"),
            FilterOperator::Eq,
            FilterValue::Single("active".to_string()),
        );

        let tree = LogicTree::and().with_conditions(vec![filter1.into(), filter2.into()]);

        assert_eq!(tree.conditions.len(), 2);
    }

    #[test]
    fn test_logic_tree_negated() {
        let tree = LogicTree::and().negated();
        assert!(tree.negated);
    }

    #[test]
    fn test_logic_condition_from_filter() {
        let filter = Filter::new(
            Field::new("id"),
            FilterOperator::Eq,
            FilterValue::Single("1".to_string()),
        );
        let condition = LogicCondition::from(filter);
        assert!(matches!(condition, LogicCondition::Filter(_)));
    }

    #[test]
    fn test_logic_condition_from_tree() {
        let tree = LogicTree::or();
        let condition = LogicCondition::from(tree);
        assert!(matches!(condition, LogicCondition::Logic(_)));
    }

    #[test]
    fn test_logic_tree_serialization() {
        let tree = LogicTree::and().negated();
        let json = serde_json::to_string(&tree).unwrap();
        assert!(json.contains("and"));
        assert!(json.contains("negated"));
    }
}
