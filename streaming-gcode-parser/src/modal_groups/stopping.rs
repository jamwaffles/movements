//! Group 4: stopping

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
pub enum Stopping {
    /// `M0`
    Pause,

    /// `M1`
    OptionalPause,

    /// `M60`
    ChangePalletPause,

    /// `M2`
    EndProgram,

    /// `M30`
    ChangePalletEndProgram,
}

impl Stopping {
    pub fn parse(i: Span) -> IResult<Span, Self> {
        alt((
            map(tag_no_case("M0"), |_| Stopping::Pause),
            map(tag_no_case("M1"), |_| Stopping::OptionalPause),
            map(tag_no_case("M60"), |_| Stopping::ChangePalletPause),
            map(tag_no_case("M2"), |_| Stopping::EndProgram),
            map(tag_no_case("M30"), |_| Stopping::ChangePalletEndProgram),
        ))(i)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::assert_parse;

    #[test]
    fn lowercase() {
        assert_parse!(Stopping::parse, "m0;", (";", Stopping::Pause));
        assert_parse!(Stopping::parse, "m1;", (";", Stopping::OptionalPause));
        assert_parse!(Stopping::parse, "m60;", (";", Stopping::ChangePalletPause));
        assert_parse!(Stopping::parse, "m2;", (";", Stopping::EndProgram));
        assert_parse!(
            Stopping::parse,
            "m30;",
            (";", Stopping::ChangePalletEndProgram)
        );
    }
}
