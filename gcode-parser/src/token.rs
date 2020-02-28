use crate::coord::coord;
use crate::coord::Coord;
use crate::motion::motion;
use crate::motion::Motion;
use crate::plane_select::{plane_select, PlaneSelect};
use crate::spindle::{spindle, Spindle};
use crate::units::{units, Units};
use crate::word::word;
use nom::branch::alt;
use nom::character::complete::char;
use nom::combinator::map;
use nom::combinator::verify;
use nom::IResult;

#[derive(Debug, PartialEq)]
pub enum Token {
    /// Block delete `/` character
    BlockDelete,

    /// Line number
    LineNumber(u32),

    /// Tool number
    Tool(u16),

    /// Feed rate
    FeedRate(f32),

    /// Group 1
    Motion(Motion),

    /// Coordinate
    Coord(Coord),

    /// Plane select
    PlaneSelect(PlaneSelect),

    /// Units
    Units(Units),

    /// Spindle
    Spindle(Spindle),
}

pub fn token(i: &str) -> IResult<&str, Token> {
    alt((
        map(char('/'), |_| Token::BlockDelete),
        map(word('N'), |w| Token::LineNumber(w.value)),
        map(word('T'), |w| Token::Tool(w.value)),
        map(
            verify(word::<f32, _>('F'), |w| w.value.is_sign_positive()),
            |w| Token::FeedRate(w.value),
        ),
        map(motion, Token::Motion),
        map(coord, Token::Coord),
        map(plane_select, Token::PlaneSelect),
        map(units, Token::Units),
        map(spindle, Token::Spindle),
    ))(i)
}

#[cfg(test)]
mod tests {
    use super::*;
    use nom::error::ErrorKind;
    use nom::Err::Error;

    #[test]
    fn invalid_tool_numbers() {
        assert_eq!(token("T-2"), Err(Error(("-2", ErrorKind::MapRes))));
        assert_eq!(token("T1.2"), Err(Error(("1.2", ErrorKind::MapRes))));
    }

    #[test]
    fn negative_feed() {
        assert_eq!(token("F-0.0"), Err(Error(("-0.0", ErrorKind::MapRes))));
        assert_eq!(token("F-102"), Err(Error(("-102", ErrorKind::MapRes))));
    }
}
