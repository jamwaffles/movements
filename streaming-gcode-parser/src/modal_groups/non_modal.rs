//! Modal group 0, non-modal

use crate::Span;
use crate::{value::Value, word::parse_word};
use nom::{
    branch::alt,
    bytes::complete::tag_no_case,
    character::complete::digit1,
    character::complete::space0,
    combinator::map,
    combinator::not,
    sequence::{separated_pair, terminated},
    IResult,
};

#[derive(Debug, PartialEq, Clone)]
pub enum NonModal<'a> {
    /// `G4`
    Dwell { time: Value<'a> },
}

impl<'a> NonModal<'a> {
    pub fn parse(i: Span<'a>) -> IResult<Span<'a>, Self> {
        let short = terminated(tag_no_case("G4"), not(digit1));
        let long = tag_no_case("G04");

        map(
            separated_pair(alt((short, long)), space0, parse_word(tag_no_case("P"))),
            |(_, (_, time))| NonModal::Dwell { time },
        )(i)
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
    fn test_dwell() {
        assert_parse!(
            NonModal::parse,
            "G04 P0.5;",
            (
                ";",
                NonModal::Dwell {
                    time: Value::Literal(0.5)
                }
            )
        );
        assert_parse!(
            NonModal::parse,
            "G4 P0.5;",
            (
                ";",
                NonModal::Dwell {
                    time: Value::Literal(0.5)
                }
            )
        );

        assert_eq!(
            NonModal::parse("G01 P0.5;".into()),
            Err(Err::Error(Error::new("G01 P0.5;".into(), ErrorKind::Tag)))
        );
    }
}
