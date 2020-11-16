//! Group 7: cutter radius compensation

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
    combinator::opt,
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
pub enum CutterCompensation {
    /// `G40`
    Off,

    /// `G41`
    Left {
        /// Optional tool number to read offset from.
        tool_number: Option<Value>,
    },

    /// `G42`
    Right {
        /// Optional tool number to read offset from.
        tool_number: Option<Value>,
    },
}

impl CutterCompensation {
    pub fn parse(i: Span) -> IResult<Span, Self> {
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
