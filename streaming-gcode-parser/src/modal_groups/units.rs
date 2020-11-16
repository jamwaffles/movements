//! Group 6: units

use crate::Span;

use nom::{branch::alt, bytes::streaming::tag_no_case, combinator::map, IResult};

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
