//! Modal group 1, motion.

use crate::Span;

use nom::{
    branch::alt, bytes::complete::tag_no_case, character::complete::digit1, combinator::map,
    combinator::not, sequence::terminated, IResult,
};

/// Arc direction
#[derive(Debug, PartialEq, Clone)]
pub enum ArcDirection {
    /// Clockwise
    CW,

    /// Counter-clockwise.
    CCW,
}

#[derive(Debug, PartialEq, Clone)]
pub enum Motion {
    /// `G0`
    Rapid,

    /// `G1`
    Feed,

    /// `G2`/`G3` CW/CCW arc
    Arc(ArcDirection),
}

impl Motion {
    pub fn parse(i: Span) -> IResult<Span, Self> {
        let short_g0 = terminated(tag_no_case("G0"), not(digit1));
        let short_g1 = terminated(tag_no_case("G1"), not(digit1));
        let short_g2 = terminated(tag_no_case("G2"), not(digit1));
        let short_g3 = terminated(tag_no_case("G3"), not(digit1));

        alt((
            map(tag_no_case("G00"), |_| Motion::Rapid),
            map(tag_no_case("G01"), |_| Motion::Feed),
            map(tag_no_case("G02"), |_| Motion::Arc(ArcDirection::CW)),
            map(tag_no_case("G03"), |_| Motion::Arc(ArcDirection::CCW)),
            map(short_g0, |_| Motion::Rapid),
            map(short_g1, |_| Motion::Feed),
            map(short_g2, |_| Motion::Arc(ArcDirection::CW)),
            map(short_g3, |_| Motion::Arc(ArcDirection::CCW)),
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
        assert_parse!(Motion::parse, "g2;", (";", Motion::Arc(ArcDirection::CW)));
        assert_parse!(Motion::parse, "g02;", (";", Motion::Arc(ArcDirection::CW)));
        assert_parse!(Motion::parse, "G3;", (";", Motion::Arc(ArcDirection::CCW)));
        assert_parse!(Motion::parse, "G03;", (";", Motion::Arc(ArcDirection::CCW)));

        assert_eq!(
            Motion::parse("G04;".into()),
            Err(Err::Error(Error::new("G04;".into(), ErrorKind::Tag)))
        );
    }
}
