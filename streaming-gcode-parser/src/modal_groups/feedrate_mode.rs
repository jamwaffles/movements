//! Group 5: feedrate mode

use crate::Span;

use nom::{branch::alt, bytes::streaming::tag_no_case, combinator::map, IResult};

#[derive(Debug, PartialEq, Clone)]
pub enum FeedrateMode {
    /// `G93`
    UnitsPerMinute,

    /// `G94`
    InverseTime,
}

impl FeedrateMode {
    pub fn parse(i: Span) -> IResult<Span, Self> {
        alt((
            map(tag_no_case("G93"), |_| FeedrateMode::UnitsPerMinute),
            map(tag_no_case("G94"), |_| FeedrateMode::InverseTime),
        ))(i)
    }
}
