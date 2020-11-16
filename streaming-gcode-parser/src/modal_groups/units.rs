//! Group 6: units

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
pub enum Units {
    /// `G20`
    Inches,

    /// `G21`
    Mm,
}

impl Units {
    pub fn parse(i: Span) -> IResult<Span, Self> {
        alt((
            map(tag_no_case("G20"), |_| Units::Inches),
            map(tag_no_case("G21"), |_| Units::Mm),
        ))(i)
    }
}
