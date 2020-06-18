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
use crate::tool_change::tool_change;
use crate::tool_change::ToolChange;
use crate::units::{units, Units};
use crate::word::word;
use crate::Location;
use crate::ParseInput;
use nom::branch::alt;
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

    /// Spindle speed (RPM)
    SpindleSpeed(f32),

    /// Stopping
    Stopping(Stopping),

    /// Distance mode
    DistanceMode(DistanceMode),

    /// Comment
    Comment(Comment),

    /// Cutter radius compensation
    CutterCompensation(CutterCompensation),

    /// Tool change
    ToolChange(ToolChange),
}

pub fn token_parser<'a, P>(parser: P) -> impl Fn(ParseInput<'a>) -> IResult<ParseInput<'a>, Token>
where
    P: Fn(ParseInput<'a>) -> IResult<ParseInput<'a>, TokenType>,
{
    move |i| {
        let (i, start_pos) = position(i)?;

        let (i, token_type) = parser(i)?;

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
}

// TODO: Rename to `command`?
pub fn token(i: ParseInput) -> IResult<ParseInput, Token> {
    token_parser(alt((
        map(coord, TokenType::Coord),
        map(motion, TokenType::Motion),
        map(plane_select, TokenType::PlaneSelect),
        map(units, TokenType::Units),
        map(spindle, TokenType::Spindle),
        map(stopping, TokenType::Stopping),
        map(distance_mode, TokenType::DistanceMode),
        map(cutter_compensation, TokenType::CutterCompensation),
        map(tool_change, TokenType::ToolChange),
        map(comment, TokenType::Comment),
        map(word('T'), |w| TokenType::Tool(w.value)),
        map(verify(word('S'), |w| w.value >= 0.0), |w| {
            TokenType::SpindleSpeed(w.value)
        }),
        map(
            verify(word::<f32, _>('F'), |w| w.value.is_sign_positive()),
            |w| TokenType::FeedRate(w.value),
        ),
    )))(i)
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
    fn does_not_parse_line_number() {
        assert_eq!(
            token(ParseInput::new("N1 G40")),
            Err(Error((rem!("N1 G40", 0, 1), nom::error::ErrorKind::Verify)))
        );
    }

    #[test]
    fn spindle_speed() {
        assert_eq!(
            token(ParseInput::new("S0")),
            Ok((
                rem!("", 2),
                tok!(TokenType::SpindleSpeed(0.0), offs = (0, 2)),
            ))
        );

        assert_eq!(
            token(ParseInput::new("S5000")),
            Ok((
                rem!("", 5),
                tok!(TokenType::SpindleSpeed(5000.0), offs = (0, 5)),
            ))
        );

        assert_eq!(
            token(ParseInput::new("S340.6")),
            Ok((
                rem!("", 6),
                tok!(TokenType::SpindleSpeed(340.6), offs = (0, 6)),
            ))
        );

        assert_eq!(
            token(ParseInput::new("S-200.0")),
            Err(Error((
                rem!("S-200.0", 0, 1),
                nom::error::ErrorKind::Verify
            )))
        );
    }
}
