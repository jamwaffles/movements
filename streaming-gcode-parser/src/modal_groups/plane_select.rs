//! Group 2: plane select

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
pub enum PlaneSelect {
    /// `G17`
    XY,

    /// `G18`
    XZ,

    /// `G19`
    YZ,
}

impl PlaneSelect {
    pub fn parse(i: &str) -> IResult<&str, Self> {
        alt((
            map(tag_no_case("G17"), |_| PlaneSelect::XY),
            map(tag_no_case("G18"), |_| PlaneSelect::XZ),
            map(tag_no_case("G19"), |_| PlaneSelect::YZ),
        ))(i)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn lowercase() {
        assert_eq!(PlaneSelect::parse("g17;"), Ok((";", PlaneSelect::XY)));
    }

    #[test]
    fn planes() {
        assert_eq!(PlaneSelect::parse("G18;"), Ok((";", PlaneSelect::XZ)));
        assert_eq!(PlaneSelect::parse("G19;"), Ok((";", PlaneSelect::YZ)));
    }
}