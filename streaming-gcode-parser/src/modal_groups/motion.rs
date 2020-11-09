//! Modal group 1, motion.

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
pub enum Motion {
    /// `G0`
    Rapid,

    /// `G1`
    Feed,
}

impl Motion {
    pub fn parse(i: &str) -> IResult<&str, Self> {
        let short_g0 = terminated(tag_no_case("G0"), not(digit1));
        let short_g1 = terminated(tag_no_case("G1"), not(digit1));

        alt((
            map(tag_no_case("G00"), |_| Motion::Rapid),
            map(tag_no_case("G01"), |_| Motion::Feed),
            map(short_g0, |_| Motion::Rapid),
            map(short_g1, |_| Motion::Feed),
        ))(i)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use nom::{
        error::{Error, ErrorKind},
        Err,
    };

    #[test]
    fn motions() {
        assert_eq!(Motion::parse("g0;"), Ok((";", Motion::Rapid)));
        assert_eq!(Motion::parse("g00;"), Ok((";", Motion::Rapid)));
        assert_eq!(Motion::parse("G1;"), Ok((";", Motion::Feed)));
        assert_eq!(Motion::parse("G01;"), Ok((";", Motion::Feed)));

        assert_eq!(
            Motion::parse("G04;"),
            Err(Err::Error(Error::new("G04;", ErrorKind::Tag)))
        );
    }
}
