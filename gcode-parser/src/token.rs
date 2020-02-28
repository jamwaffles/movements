use crate::coord::coord;
use crate::coord::Coord;
use crate::motion::motion;
use crate::motion::Motion;
use crate::plane_select::{plane_select, PlaneSelect};
use crate::spindle::{spindle, Spindle};
use crate::stopping::stopping;
use crate::stopping::Stopping;
use crate::units::{units, Units};
use crate::word::word;
use crate::ParseInput;
use nom::branch::alt;
use nom::character::complete::char;
use nom::combinator::map;
use nom::combinator::verify;
use nom::IResult;
use nom_locate::position;

#[derive(Debug, PartialEq)]
pub struct Token<'a> {
    pub start_pos: ParseInput<'a>,

    pub end_pos: ParseInput<'a>,

    pub token: TokenType,
}

#[derive(Debug, PartialEq)]
pub enum TokenType {
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

    /// Stopping
    Stopping(Stopping),
}

// TODO: Rename to `command`?
pub fn token(i: ParseInput) -> IResult<ParseInput, Token> {
    let (i, start_pos) = position(i)?;

    let (i, token_type) = alt((
        map(char('/'), |_| TokenType::BlockDelete),
        map(word('N'), |w| TokenType::LineNumber(w.value)),
        map(word('T'), |w| TokenType::Tool(w.value)),
        map(
            verify(word::<f32, _>('F'), |w| w.value.is_sign_positive()),
            |w| TokenType::FeedRate(w.value),
        ),
        map(motion, TokenType::Motion),
        map(coord, TokenType::Coord),
        map(plane_select, TokenType::PlaneSelect),
        map(units, TokenType::Units),
        map(spindle, TokenType::Spindle),
        map(stopping, TokenType::Stopping),
    ))(i)?;

    let (i, end_pos) = position(i)?;

    Ok((
        i,
        Token {
            start_pos,
            end_pos,
            token: token_type,
        },
    ))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::rem;
    use nom::error::ErrorKind;
    use nom::Err::Error;

    #[test]
    fn invalid_tool_numbers() {
        assert_eq!(
            token(ParseInput::new("T-2")),
            Err(Error((rem!("-2", 1), ErrorKind::MapRes)))
        );
        assert_eq!(
            token(ParseInput::new("T1.2")),
            Err(Error((rem!("1.2", 1), ErrorKind::MapRes)))
        );
    }

    #[test]
    fn negative_feed() {
        assert_eq!(
            token(ParseInput::new("F-0.0")),
            Err(Error((rem!("-0.0", 1), ErrorKind::MapRes)))
        );
        assert_eq!(
            token(ParseInput::new("F-102")),
            Err(Error((rem!("-102", 1), ErrorKind::MapRes)))
        );
    }
}
