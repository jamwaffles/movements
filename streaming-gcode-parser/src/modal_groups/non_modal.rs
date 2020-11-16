//! Modal group 0, non-modal

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
    combinator::not,
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
pub enum NonModal {
    /// `G4`
    Dwell { time: Value },
}

impl NonModal {
    pub fn parse(i: Span) -> IResult<Span, Self> {
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
