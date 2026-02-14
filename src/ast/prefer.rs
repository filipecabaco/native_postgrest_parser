use serde::{Deserialize, Serialize};

/// Return representation preference for mutations
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum ReturnRepresentation {
    /// Return full representation of affected rows
    #[default]
    Full,
    /// Return minimal data (just status)
    Minimal,
    /// Return only headers
    HeadersOnly,
}

/// Resolution preference for INSERT conflicts
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum Resolution {
    /// Merge duplicates (upsert)
    MergeDuplicates,
    /// Ignore duplicates
    IgnoreDuplicates,
}

/// Count preference for SELECT queries
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Count {
    /// Exact count (can be slow)
    Exact,
    /// Planned count from query planner
    Planned,
    /// Estimated count
    Estimated,
}

/// Plurality preference for responses
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum Plurality {
    /// Expect single row
    Singular,
    /// Expect multiple rows (default)
    #[default]
    Multiple,
}

/// Missing value handling for INSERT
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum Missing {
    /// Use column defaults
    Default,
    /// Use NULL (default)
    #[default]
    Null,
}

/// PostgREST Prefer header options
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PreferOptions {
    pub return_representation: Option<ReturnRepresentation>,
    pub resolution: Option<Resolution>,
    pub count: Option<Count>,
    pub plurality: Option<Plurality>,
    pub missing: Option<Missing>,
}

impl PreferOptions {
    pub fn new() -> Self {
        Self {
            return_representation: None,
            resolution: None,
            count: None,
            plurality: None,
            missing: None,
        }
    }

    pub fn with_return(mut self, ret: ReturnRepresentation) -> Self {
        self.return_representation = Some(ret);
        self
    }

    pub fn with_resolution(mut self, res: Resolution) -> Self {
        self.resolution = Some(res);
        self
    }

    pub fn with_count(mut self, count: Count) -> Self {
        self.count = Some(count);
        self
    }

    pub fn with_plurality(mut self, plurality: Plurality) -> Self {
        self.plurality = Some(plurality);
        self
    }

    pub fn with_missing(mut self, missing: Missing) -> Self {
        self.missing = Some(missing);
        self
    }

    pub fn is_empty(&self) -> bool {
        self.return_representation.is_none()
            && self.resolution.is_none()
            && self.count.is_none()
            && self.plurality.is_none()
            && self.missing.is_none()
    }
}

impl Default for PreferOptions {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_return_representation_default() {
        assert_eq!(ReturnRepresentation::default(), ReturnRepresentation::Full);
    }

    #[test]
    fn test_plurality_default() {
        assert_eq!(Plurality::default(), Plurality::Multiple);
    }

    #[test]
    fn test_missing_default() {
        assert_eq!(Missing::default(), Missing::Null);
    }

    #[test]
    fn test_prefer_options_new() {
        let opts = PreferOptions::new();
        assert!(opts.is_empty());
    }

    #[test]
    fn test_prefer_options_builder() {
        let opts = PreferOptions::new()
            .with_return(ReturnRepresentation::Minimal)
            .with_count(Count::Exact);

        assert_eq!(
            opts.return_representation,
            Some(ReturnRepresentation::Minimal)
        );
        assert_eq!(opts.count, Some(Count::Exact));
        assert!(!opts.is_empty());
    }

    #[test]
    fn test_prefer_options_serialization() {
        let opts = PreferOptions::new()
            .with_return(ReturnRepresentation::Full)
            .with_resolution(Resolution::MergeDuplicates);

        let json = serde_json::to_string(&opts).unwrap();
        assert!(json.contains("full"));
        assert!(json.contains("merge-duplicates"));
    }
}
