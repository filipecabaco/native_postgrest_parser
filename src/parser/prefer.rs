use crate::ast::{Count, Missing, Plurality, PreferOptions, Resolution, ReturnRepresentation};
use crate::error::Error;
use nom::{branch::alt, bytes::complete::tag, combinator::map, sequence::preceded, IResult};

/// Parses "return=representation|minimal|headers-only"
fn parse_return(input: &str) -> IResult<&str, ReturnRepresentation> {
    preceded(
        tag("return="),
        alt((
            map(tag("representation"), |_| ReturnRepresentation::Full),
            map(tag("minimal"), |_| ReturnRepresentation::Minimal),
            map(tag("headers-only"), |_| ReturnRepresentation::HeadersOnly),
        )),
    )(input)
}

/// Parses "resolution=merge-duplicates|ignore-duplicates"
fn parse_resolution(input: &str) -> IResult<&str, Resolution> {
    preceded(
        tag("resolution="),
        alt((
            map(tag("merge-duplicates"), |_| Resolution::MergeDuplicates),
            map(tag("ignore-duplicates"), |_| Resolution::IgnoreDuplicates),
        )),
    )(input)
}

/// Parses "count=exact|planned|estimated"
fn parse_count(input: &str) -> IResult<&str, Count> {
    preceded(
        tag("count="),
        alt((
            map(tag("exact"), |_| Count::Exact),
            map(tag("planned"), |_| Count::Planned),
            map(tag("estimated"), |_| Count::Estimated),
        )),
    )(input)
}

/// Parses "plurality=singular"
fn parse_plurality(input: &str) -> IResult<&str, Plurality> {
    preceded(
        tag("plurality="),
        alt((
            map(tag("singular"), |_| Plurality::Singular),
            map(tag("multiple"), |_| Plurality::Multiple),
        )),
    )(input)
}

/// Parses "missing=default"
fn parse_missing(input: &str) -> IResult<&str, Missing> {
    preceded(
        tag("missing="),
        alt((
            map(tag("default"), |_| Missing::Default),
            map(tag("null"), |_| Missing::Null),
        )),
    )(input)
}

/// Parses a full Prefer header value
///
/// Format: "option1=value1, option2=value2, ..."
///
/// # Examples
///
/// ```
/// use postgrest_parser::parser::parse_prefer_header;
///
/// let opts = parse_prefer_header("return=representation, count=exact").unwrap();
/// ```
pub fn parse_prefer_header(input: &str) -> Result<PreferOptions, Error> {
    let parts: Vec<&str> = input.split(',').map(|s| s.trim()).collect();

    let mut options = PreferOptions::new();

    for part in parts {
        if part.is_empty() {
            continue;
        }

        // Try each parser
        if let Ok((_, ret)) = parse_return(part) {
            options.return_representation = Some(ret);
        } else if let Ok((_, res)) = parse_resolution(part) {
            options.resolution = Some(res);
        } else if let Ok((_, cnt)) = parse_count(part) {
            options.count = Some(cnt);
        } else if let Ok((_, plur)) = parse_plurality(part) {
            options.plurality = Some(plur);
        } else if let Ok((_, miss)) = parse_missing(part) {
            options.missing = Some(miss);
        } else {
            // Unknown preference - skip it (PostgREST behavior)
            continue;
        }
    }

    Ok(options)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_return_representation() {
        assert_eq!(
            parse_return("return=representation").unwrap().1,
            ReturnRepresentation::Full
        );
        assert_eq!(
            parse_return("return=minimal").unwrap().1,
            ReturnRepresentation::Minimal
        );
        assert_eq!(
            parse_return("return=headers-only").unwrap().1,
            ReturnRepresentation::HeadersOnly
        );
    }

    #[test]
    fn test_parse_resolution() {
        assert_eq!(
            parse_resolution("resolution=merge-duplicates").unwrap().1,
            Resolution::MergeDuplicates
        );
        assert_eq!(
            parse_resolution("resolution=ignore-duplicates").unwrap().1,
            Resolution::IgnoreDuplicates
        );
    }

    #[test]
    fn test_parse_count() {
        assert_eq!(parse_count("count=exact").unwrap().1, Count::Exact);
        assert_eq!(parse_count("count=planned").unwrap().1, Count::Planned);
        assert_eq!(parse_count("count=estimated").unwrap().1, Count::Estimated);
    }

    #[test]
    fn test_parse_plurality() {
        assert_eq!(
            parse_plurality("plurality=singular").unwrap().1,
            Plurality::Singular
        );
        assert_eq!(
            parse_plurality("plurality=multiple").unwrap().1,
            Plurality::Multiple
        );
    }

    #[test]
    fn test_parse_missing() {
        assert_eq!(
            parse_missing("missing=default").unwrap().1,
            Missing::Default
        );
        assert_eq!(parse_missing("missing=null").unwrap().1, Missing::Null);
    }

    #[test]
    fn test_parse_prefer_header_single() {
        let opts = parse_prefer_header("return=representation").unwrap();
        assert_eq!(opts.return_representation, Some(ReturnRepresentation::Full));
    }

    #[test]
    fn test_parse_prefer_header_multiple() {
        let opts = parse_prefer_header("return=representation, count=exact").unwrap();
        assert_eq!(opts.return_representation, Some(ReturnRepresentation::Full));
        assert_eq!(opts.count, Some(Count::Exact));
    }

    #[test]
    fn test_parse_prefer_header_all_options() {
        let input = "return=minimal, resolution=merge-duplicates, count=planned, plurality=singular, missing=default";
        let opts = parse_prefer_header(input).unwrap();

        assert_eq!(
            opts.return_representation,
            Some(ReturnRepresentation::Minimal)
        );
        assert_eq!(opts.resolution, Some(Resolution::MergeDuplicates));
        assert_eq!(opts.count, Some(Count::Planned));
        assert_eq!(opts.plurality, Some(Plurality::Singular));
        assert_eq!(opts.missing, Some(Missing::Default));
    }

    #[test]
    fn test_parse_prefer_header_with_spaces() {
        let opts = parse_prefer_header("  return=representation  ,  count=exact  ").unwrap();
        assert_eq!(opts.return_representation, Some(ReturnRepresentation::Full));
        assert_eq!(opts.count, Some(Count::Exact));
    }

    #[test]
    fn test_parse_prefer_header_unknown_option() {
        // Should skip unknown options
        let opts =
            parse_prefer_header("return=representation, unknown=value, count=exact").unwrap();
        assert_eq!(opts.return_representation, Some(ReturnRepresentation::Full));
        assert_eq!(opts.count, Some(Count::Exact));
    }

    #[test]
    fn test_parse_prefer_header_empty() {
        let opts = parse_prefer_header("").unwrap();
        assert!(opts.is_empty());
    }

    #[test]
    fn test_parse_prefer_header_empty_parts() {
        let opts = parse_prefer_header("return=representation,  , count=exact").unwrap();
        assert_eq!(opts.return_representation, Some(ReturnRepresentation::Full));
        assert_eq!(opts.count, Some(Count::Exact));
    }
}
