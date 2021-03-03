//! Group 7: cutter radius compensation

use crate::Span;
use crate::{value::Value, word::parse_word};
use nom::{
    branch::alt, bytes::complete::tag_no_case, character::complete::space0, combinator::map,
    combinator::opt, sequence::separated_pair, IResult,
};

#[derive(Debug, PartialEq, Clone)]
pub enum CutterCompensation<'a> {
    /// `G40`
    Off,

    /// `G41`
    Left {
        /// Optional tool number to read offset from.
        tool_number: Option<Value<'a>>,
    },

    /// `G42`
    Right {
        /// Optional tool number to read offset from.
        tool_number: Option<Value<'a>>,
    },
}

impl<'a> CutterCompensation<'a> {
    pub fn parse(i: Span<'a>) -> IResult<Span<'a>, Self> {
        let tool_number = |tag: &'static str| {
            map(
                separated_pair(tag_no_case(tag), space0, opt(parse_word(tag_no_case("D")))),
                |(_, d)| d.map(|(_c, value)| value),
            )
        };

        alt((
            map(tag_no_case("G40"), |_| CutterCompensation::Off),
            map(tool_number("G41"), |tool_number| CutterCompensation::Left {
                tool_number,
            }),
            map(tool_number("G42"), |tool_number| {
                CutterCompensation::Right { tool_number }
            }),
        ))(i)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::assert_parse;

    #[test]
    fn no_d() {
        assert_parse!(
            CutterCompensation::parse,
            "G41;",
            (";", CutterCompensation::Left { tool_number: None })
        );
        assert_parse!(
            CutterCompensation::parse,
            "G42;",
            (";", CutterCompensation::Right { tool_number: None })
        );
    }

    #[test]
    fn with_d() {
        assert_parse!(
            CutterCompensation::parse,
            "G41 D13;",
            (
                ";",
                CutterCompensation::Left {
                    tool_number: Some(13.into())
                }
            )
        );
        assert_parse!(
            CutterCompensation::parse,
            "G42 D1;",
            (
                ";",
                CutterCompensation::Right {
                    tool_number: Some(1.into())
                }
            )
        );
    }
}
