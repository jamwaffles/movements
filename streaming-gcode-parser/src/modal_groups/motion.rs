//! Modal group 1, motion.

use crate::Span;

use nom::{
    branch::alt, bytes::streaming::tag_no_case, character::streaming::digit1, combinator::map,
    combinator::not, sequence::terminated, IResult,
};

#[derive(Debug, PartialEq, Clone)]
pub enum Motion {
    /// `G0`
    Rapid,

    /// `G1`
    Feed,
}

impl Motion {
    pub fn parse(i: Span) -> IResult<Span, Self> {
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
    use crate::assert_parse;
    use nom::{
        error::{Error, ErrorKind},
        Err,
    };

    #[test]
    fn motions() {
        assert_parse!(Motion::parse, "g0;", (";", Motion::Rapid));
        assert_parse!(Motion::parse, "g00;", (";", Motion::Rapid));
        assert_parse!(Motion::parse, "G1;", (";", Motion::Feed));
        assert_parse!(Motion::parse, "G01;", (";", Motion::Feed));

        assert_eq!(
            Motion::parse("G04;".into()),
            Err(Err::Error(Error::new("G04;".into(), ErrorKind::Tag)))
        );
    }
}
