//! Group 3: distance mode

use crate::Span;
use crate::{value::Value, word::parse_word};
use nom::{
    branch::alt,
    bytes::streaming::tag,
    bytes::streaming::tag_no_case,
    bytes::streaming::take_until,
    character::streaming::char,
    character::streaming::digit1,
    character::streaming::one_of,
    character::streaming::space0,
    character::streaming::{alpha1, anychar, multispace0},
    combinator::map,
    combinator::map_opt,
    combinator::map_res,
    combinator::peek,
    combinator::{cond, verify},
    multi::many0,
    multi::many_m_n,
    multi::separated_list0,
    sequence::delimited,
    sequence::preceded,
    sequence::{separated_pair, terminated},
    IResult,
};

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
