//! Group 3: distance mode

use crate::Span;

use nom::{branch::alt, bytes::complete::tag_no_case, combinator::map, IResult};

#[derive(Debug, PartialEq, Clone)]
pub enum DistanceMode {
    // `G90`
    Absolute,

    // `G91`
    Incremental,
}

impl DistanceMode {
    pub fn parse(i: Span) -> IResult<Span, Self> {
        alt((
            map(tag_no_case("G90"), |_| DistanceMode::Absolute),
            map(tag_no_case("G91"), |_| DistanceMode::Incremental),
        ))(i)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::assert_parse;

    #[test]
    fn distance_modes() {
        assert_parse!(DistanceMode::parse, "g90;", (";", DistanceMode::Absolute));
        assert_parse!(
            DistanceMode::parse,
            "g91;",
            (";", DistanceMode::Incremental)
        );
        assert_parse!(DistanceMode::parse, "G90;", (";", DistanceMode::Absolute));
        assert_parse!(
            DistanceMode::parse,
            "G91;",
            (";", DistanceMode::Incremental)
        );
    }
}
