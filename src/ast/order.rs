use super::Field;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Direction {
    Asc,
    Desc,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Nulls {
    First,
    Last,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct OrderTerm {
    pub field: Field,
    pub direction: Direction,
    pub nulls: Option<Nulls>,
}

impl OrderTerm {
    pub fn new(field: Field) -> Self {
        Self {
            field,
            direction: Direction::Asc,
            nulls: None,
        }
    }

    pub fn with_direction(mut self, direction: Direction) -> Self {
        self.direction = direction;
        self
    }

    pub fn with_nulls(mut self, nulls: Nulls) -> Self {
        self.nulls = Some(nulls);
        self
    }

    pub fn desc(mut self) -> Self {
        self.direction = Direction::Desc;
        self
    }

    pub fn asc(mut self) -> Self {
        self.direction = Direction::Asc;
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_order_term_new() {
        let field = Field::new("id");
        let term = OrderTerm::new(field);
        assert_eq!(term.direction, Direction::Asc);
        assert!(term.nulls.is_none());
    }

    #[test]
    fn test_order_term_desc() {
        let field = Field::new("created_at");
        let term = OrderTerm::new(field).desc();
        assert_eq!(term.direction, Direction::Desc);
    }

    #[test]
    fn test_order_term_with_nulls() {
        let field = Field::new("name");
        let term = OrderTerm::new(field).with_nulls(Nulls::Last);
        assert_eq!(term.nulls, Some(Nulls::Last));
    }

    #[test]
    fn test_order_term_serialization() {
        let field = Field::new("id");
        let term = OrderTerm::new(field).desc();
        let json = serde_json::to_string(&term).unwrap();
        assert!(json.contains("desc"));
    }
}
