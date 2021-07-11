//! Modal group 1, Spindle.

use crate::Span;

use nom::{
    branch::alt, bytes::complete::tag_no_case, character::complete::digit1, combinator::map,
    combinator::not, sequence::terminated, IResult,
};

#[derive(Debug, PartialEq, Clone)]
pub enum Spindle {
    /// `M3`, clockwise
    Forward,

    /// `M4`, anticlockwise
    Reverse,

    /// `M5`, stop
    Stop,
}

impl Spindle {
    pub fn parse(i: Span) -> IResult<Span, Self> {
        let short_m3 = terminated(tag_no_case("M3"), not(digit1));
        let short_m4 = terminated(tag_no_case("M4"), not(digit1));
        let short_m5 = terminated(tag_no_case("M5"), not(digit1));

        alt((
            map(tag_no_case("M03"), |_| Spindle::Forward),
            map(tag_no_case("M04"), |_| Spindle::Reverse),
            map(tag_no_case("M05"), |_| Spindle::Stop),
            map(short_m3, |_| Spindle::Forward),
            map(short_m4, |_| Spindle::Reverse),
            map(short_m5, |_| Spindle::Stop),
        ))(i)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::assert_parse;
    use nom::{
        error::{Error, ErrorKind},
        Err,
    };

    #[test]
    fn check_spindle() {
        assert_parse!(Spindle::parse, "m3;", (";", Spindle::Forward));
        assert_parse!(Spindle::parse, "m03;", (";", Spindle::Forward));
        assert_parse!(Spindle::parse, "M4;", (";", Spindle::Reverse));
        assert_parse!(Spindle::parse, "M04;", (";", Spindle::Reverse));
        assert_parse!(Spindle::parse, "M5;", (";", Spindle::Stop));
        assert_parse!(Spindle::parse, "M05;", (";", Spindle::Stop));
    }
}
