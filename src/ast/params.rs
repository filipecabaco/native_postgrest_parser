use super::{LogicCondition, OrderTerm, SelectItem};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ParsedParams {
    pub select: Option<Vec<SelectItem>>,
    pub filters: Vec<LogicCondition>,
    pub order: Vec<OrderTerm>,
    pub limit: Option<u64>,
    pub offset: Option<u64>,
}

impl ParsedParams {
    pub fn new() -> Self {
        Self {
            select: None,
            filters: Vec::new(),
            order: Vec::new(),
            limit: None,
            offset: None,
        }
    }

    pub fn with_select(mut self, select: Vec<SelectItem>) -> Self {
        self.select = Some(select);
        self
    }

    pub fn with_filters(mut self, filters: Vec<LogicCondition>) -> Self {
        self.filters = filters;
        self
    }

    pub fn with_order(mut self, order: Vec<OrderTerm>) -> Self {
        self.order = order;
        self
    }

    pub fn with_limit(mut self, limit: u64) -> Self {
        self.limit = Some(limit);
        self
    }

    pub fn with_offset(mut self, offset: u64) -> Self {
        self.offset = Some(offset);
        self
    }

    pub fn is_empty(&self) -> bool {
        self.select.is_none()
            && self.filters.is_empty()
            && self.order.is_empty()
            && self.limit.is_none()
            && self.offset.is_none()
    }

    pub fn has_filters(&self) -> bool {
        !self.filters.is_empty()
    }

    pub fn has_select(&self) -> bool {
        self.select.is_some()
    }
}

impl Default for ParsedParams {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ast::{Field, Filter, FilterOperator, FilterValue};

    #[test]
    fn test_parsed_params_new() {
        let params = ParsedParams::new();
        assert!(params.select.is_none());
        assert!(params.filters.is_empty());
        assert!(params.order.is_empty());
        assert!(params.limit.is_none());
        assert!(params.offset.is_none());
    }

    #[test]
    fn test_parsed_params_with_select() {
        let select = vec![SelectItem::field("id"), SelectItem::field("name")];
        let params = ParsedParams::new().with_select(select);
        assert!(params.has_select());
        assert_eq!(params.select.as_ref().unwrap().len(), 2);
    }

    #[test]
    fn test_parsed_params_with_limit() {
        let params = ParsedParams::new().with_limit(100);
        assert_eq!(params.limit, Some(100));
    }

    #[test]
    fn test_parsed_params_with_offset() {
        let params = ParsedParams::new().with_offset(20);
        assert_eq!(params.offset, Some(20));
    }

    #[test]
    fn test_parsed_params_is_empty() {
        let params = ParsedParams::new();
        assert!(params.is_empty());
    }

    #[test]
    fn test_parsed_params_has_filters() {
        let filter = LogicCondition::Filter(Filter::new(
            Field::new("id"),
            FilterOperator::Eq,
            FilterValue::Single("1".to_string()),
        ));
        let params = ParsedParams::new().with_filters(vec![filter]);
        assert!(params.has_filters());
    }

    #[test]
    fn test_parsed_params_default() {
        let params = ParsedParams::default();
        assert!(params.is_empty());
    }

    #[test]
    fn test_parsed_params_serialization() {
        let params = ParsedParams::new().with_limit(10);
        let json = serde_json::to_string(&params).unwrap();
        assert!(json.contains("limit"));
    }
}
