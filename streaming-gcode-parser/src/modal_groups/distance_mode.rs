//! Group 3: distance mode

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
    pub fn parse(i: &str) -> IResult<&str, Self> {
        alt((
            map(tag_no_case("G90"), |_| DistanceMode::Absolute),
            map(tag_no_case("G91"), |_| DistanceMode::Incremental),
        ))(i)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn distance_modes() {
        assert_eq!(
            DistanceMode::parse("g90;"),
            Ok((";", DistanceMode::Absolute))
        );
        assert_eq!(
            DistanceMode::parse("g91;"),
            Ok((";", DistanceMode::Incremental))
        );
        assert_eq!(
            DistanceMode::parse("G90;"),
            Ok((";", DistanceMode::Absolute))
        );
        assert_eq!(
            DistanceMode::parse("G91;"),
            Ok((";", DistanceMode::Incremental))
        );
    }
}
