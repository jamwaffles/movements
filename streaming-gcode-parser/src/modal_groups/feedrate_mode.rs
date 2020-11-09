//! Group 5: feedrate mode

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
pub enum FeedrateMode {
    /// `G93`
    UnitsPerMinute,

    /// `G94`
    InverseTime,
}

impl FeedrateMode {
    pub fn parse(i: &str) -> IResult<&str, Self> {
        alt((
            map(tag_no_case("G93"), |_| FeedrateMode::UnitsPerMinute),
            map(tag_no_case("G94"), |_| FeedrateMode::InverseTime),
        ))(i)
    }
}
