use crate::comment::comment;
use crate::comment::Comment;
use crate::coord::coord;
use crate::coord::Coord;
use crate::cutter_compensation::cutter_compensation;
use crate::cutter_compensation::CutterCompensation;
use crate::distance_mode::distance_mode;
use crate::distance_mode::DistanceMode;
use crate::motion::motion;
use crate::motion::Motion;
use crate::plane_select::{plane_select, PlaneSelect};
use crate::spindle::{spindle, Spindle};
use crate::stopping::stopping;
use crate::stopping::Stopping;
use crate::units::{units, Units};
use crate::word::word;
use crate::Location;
use crate::ParseInput;
use nom::branch::alt;
use nom::character::complete::char;
use nom::combinator::map;
use nom::combinator::verify;
use nom::IResult;
use nom_locate::position;

#[derive(Debug, PartialEq, Clone)]
pub struct Token {
    pub start_pos: Location,

    pub end_pos: Location,

    pub token: TokenType,
}

#[derive(Debug, PartialEq, Clone)]
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

    /// Distance mode
    DistanceMode(DistanceMode),

    /// Comment
    Comment(Comment),

    /// Cutter radius compensation
    CutterCompensation(CutterCompensation),
}

// TODO: Rename to `command`?
pub fn token(i: ParseInput) -> IResult<ParseInput, Token> {
    let (i, start_pos) = position(i)?;

    let (i, token_type) = alt((
        map(coord, TokenType::Coord),
        map(motion, TokenType::Motion),
        map(plane_select, TokenType::PlaneSelect),
        map(units, TokenType::Units),
        map(spindle, TokenType::Spindle),
        map(stopping, TokenType::Stopping),
        map(distance_mode, TokenType::DistanceMode),
        map(cutter_compensation, TokenType::CutterCompensation),
        map(char('/'), |_| TokenType::BlockDelete),
        map(word('N'), |w| TokenType::LineNumber(w.value)),
        map(word('T'), |w| TokenType::Tool(w.value)),
        map(comment, TokenType::Comment),
        map(
            verify(word::<f32, _>('F'), |w| w.value.is_sign_positive()),
            |w| TokenType::FeedRate(w.value),
        ),
    ))(i)?;

    let (i, end_pos) = position(i)?;

    Ok((
        i,
        Token {
            start_pos: start_pos.into(),
            end_pos: end_pos.into(),
            token: token_type,
        },
    ))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{rem, tok};
    use nom::error::ErrorKind;
    use nom::Err::Error;

    #[test]
    fn invalid_tool_numbers() {
        assert_eq!(
            token(ParseInput::new("T-2")),
            Err(Error((rem!("T-2"), ErrorKind::Verify)))
        );
        assert_eq!(
            token(ParseInput::new("T1.2")),
            Err(Error((rem!("T1.2"), ErrorKind::Verify)))
        );
    }

    #[test]
    fn negative_feed() {
        assert_eq!(
            token(ParseInput::new("F-0.0")),
            Err(Error((rem!("F-0.0"), ErrorKind::Verify)))
        );
        assert_eq!(
            token(ParseInput::new("F-102")),
            Err(Error((rem!("F-102"), ErrorKind::Verify)))
        );
    }

    #[test]
    fn line_number() {
        assert_eq!(
            token(ParseInput::new("N1 G40")),
            Ok((
                rem!(" G40", 2),
                tok!(TokenType::LineNumber(1), offs = (0, 2), line = (1, 1))
            ))
        );
    }
}
