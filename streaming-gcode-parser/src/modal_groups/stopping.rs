//! Group 4: stopping

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
    pub fn parse(i: &str) -> IResult<&str, Self> {
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

    #[test]
    fn lowercase() {
        assert_eq!(Stopping::parse("m0;"), Ok((";", Stopping::Pause)));
        assert_eq!(Stopping::parse("m1;"), Ok((";", Stopping::OptionalPause)));
        assert_eq!(
            Stopping::parse("m60;"),
            Ok((";", Stopping::ChangePalletPause))
        );
        assert_eq!(Stopping::parse("m2;"), Ok((";", Stopping::EndProgram)));
        assert_eq!(
            Stopping::parse("m30;"),
            Ok((";", Stopping::ChangePalletEndProgram))
        );
    }
}
