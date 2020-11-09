//! Modal group 1, motion.

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
pub enum Motion {
    /// `G0`
    Rapid,

    /// `G1`
    Feed,
}

impl Motion {
    pub fn parse(i: &str) -> IResult<&str, Self> {
        alt((
            map(tag_no_case("G0"), |_| Motion::Rapid),
            map(tag_no_case("G00"), |_| Motion::Rapid),
            map(tag_no_case("G1"), |_| Motion::Feed),
            map(tag_no_case("G01"), |_| Motion::Feed),
        ))(i)
    }
}
