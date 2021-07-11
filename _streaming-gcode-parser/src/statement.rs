use crate::{
    coord::Coord,
    modal_groups::{
        CoordinateSystem, CutterCompensation, DistanceMode, FeedrateMode, Motion, NonModal,
        PlaneSelect, Spindle, Stopping, Units,
    },
    parameter::Parameter,
    value::Value,
    word::parse_word,
    Span,
};
use nom::{
    branch::alt,
    bytes::complete::{tag_no_case, take_till, take_until},
    character::complete::{anychar, char, digit1, one_of, space0},
    combinator::{map, not, verify},
    sequence::{delimited, preceded, separated_pair, terminated},
    IResult,
};
use nom_locate::position;

#[derive(Debug, PartialEq, Clone)]
pub struct Token<'a> {
    pub position: Span<'a>,
    pub statement: Statement<'a>,
}

impl<'a> Token<'a> {
    pub fn parse(i: Span<'a>) -> IResult<Span, Self> {
        let (i, statement) = Statement::parse(i)?;
        let (i, pos) = position(i)?;

        Ok((
            i,
            Self {
                statement,
                position: pos,
            },
        ))
    }
}

#[derive(Debug, PartialEq, Clone)]
pub enum CommentKind {
    /// `; comment`
    Line,

    /// `(comment)`
    Delimited,
}

#[derive(Debug, PartialEq, Clone)]
pub enum Statement<'a> {
    /// Comment.
    Comment { comment: &'a str, kind: CommentKind },

    /// Set parameter, e.g. `#5550 = [12 + 13]`.
    SetParameter {
        parameter: Parameter<'a>,
        value: Value<'a>,
    },

    /// Non modal G codes
    NonModal(NonModal<'a>),

    /// Modal group 1: motion
    Motion(Motion),

    /// Modal group 2: plane select
    PlaneSelect(PlaneSelect),

    /// Modal group 3: distance mode
    DistanceMode(DistanceMode),

    /// Modal group 4: stopping
    Stopping(Stopping),

    /// Modal group 5: feed rate mode
    FeedrateMode(FeedrateMode),

    /// Modal group 6: units
    Units(Units),

    /// Modal group 7: cutter comp
    CutterCompensation(CutterCompensation<'a>),

    /// Modal group 12: coordinate system
    CoordinateSystem(CoordinateSystem),

    /// Modal group M 7: spindle
    Spindle(Spindle),

    /// Axis value
    Coord(Coord<'a>),

    /// Tool change.
    ToolChange,

    /// Feed rate.
    FeedRate(Value<'a>),

    /// Spindle speed.
    SpindleSpeed(Value<'a>),

    /// Tool number.
    ToolNumber(Value<'a>),

    /// Dynamic token whose code is evaluated at runtime.
    ///
    /// When parsed, `letter` is transformed to lowercase.
    Dynamic { letter: char, number: Value<'a> },
}

impl Statement<'static> {
    pub(crate) fn to_token(self, offset: usize, line: u32) -> Token<'static> {
        let span = unsafe { Span::new_from_raw_offset(offset, line, "", ()) };

        Token {
            statement: self,
            position: span,
        }
    }

    fn comment(text: &'static str, kind: CommentKind) -> Self {
        Self::Comment {
            comment: text,
            kind,
        }
    }
}

impl<'s> Statement<'s> {
    fn parse_set_param(i: Span) -> IResult<Span, (Parameter, Value)> {
        separated_pair(
            Parameter::parse,
            delimited(space0, char('='), space0),
            Value::parse,
        )(i)
    }

    fn parse_comment(i: Span<'s>) -> IResult<Span, (&'s str, CommentKind)> {
        alt((
            map(
                preceded(char(';'), take_till(|c| c == '\r' || c == '\n')),
                |comment: Span| (comment.trim(), CommentKind::Line),
            ),
            map(
                delimited(char('('), take_until(")"), char(')')),
                |comment: Span| (comment.trim(), CommentKind::Delimited),
            ),
        ))(i)
    }

    /// Parses anything that isn't a line number or block delete.
    ///
    /// Line numbers and block delete hold special positions in the block, so are parsed separately.
    pub fn parse(i: Span<'s>) -> IResult<Span<'s>, Self> {
        alt((
            // ---
            // G words
            // ---
            map(PlaneSelect::parse, Self::PlaneSelect),
            map(DistanceMode::parse, Self::DistanceMode),
            map(FeedrateMode::parse, Self::FeedrateMode),
            map(Units::parse, Self::Units),
            map(CutterCompensation::parse, Self::CutterCompensation),
            map(CoordinateSystem::parse, Self::CoordinateSystem),
            // Note: order of Motion and NonModal parse is significant. E.g. `G17` matches `G1`
            // Motion::Feed instead of `PlaneSelect::XY`, so the motion (and `NonModal`) passes must
            // come after the others.
            map(Motion::parse, Self::Motion),
            map(NonModal::parse, Self::NonModal),
            // ---
            // M words
            // ---
            map(Stopping::parse, Self::Stopping),
            map(Spindle::parse, Self::Spindle),
            map(
                alt((
                    terminated(tag_no_case("M6"), not(digit1)),
                    tag_no_case("M06"),
                )),
                |_| Self::ToolChange,
            ),
            // ---
            // Other words and stuff
            // ---
            map(Coord::parse, Self::Coord),
            map(parse_word(one_of("fF")), |(_c, value)| {
                Self::FeedRate(value)
            }),
            map(parse_word(one_of("tT")), |(_c, value)| {
                Self::ToolNumber(value)
            }),
            map(parse_word(one_of("sS")), |(_c, value)| {
                Self::SpindleSpeed(value)
            }),
            map(Self::parse_comment, |(comment, kind)| Self::Comment {
                comment,
                kind,
            }),
            // Set parameter
            map(Self::parse_set_param, |(parameter, value)| {
                Self::SetParameter { parameter, value }
            }),
            // Dynamic code
            map(
                parse_word(verify(anychar, |c| c.is_ascii_alphabetic())),
                |(letter, number)| Self::Dynamic { letter, number },
            ),
        ))(i)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::assert_parse;

    #[test]
    fn assignment() {
        assert_parse!(
            Statement::parse,
            "#9 = 100;",
            (
                ";",
                Statement::SetParameter {
                    parameter: Parameter::Index(9),
                    value: 100.0.into()
                }
            )
        );
    }

    #[test]
    fn comments() {
        assert_parse!(
            Statement::parse,
            "(closed)",
            ("", Statement::comment("closed", CommentKind::Delimited))
        );
        assert_parse!(
            Statement::parse,
            "(newline and param)\n#9=0",
            (
                "\n#9=0",
                Statement::comment("newline and param", CommentKind::Delimited)
            )
        );
        assert_parse!(
            Statement::parse,
            "; Open\n",
            ("\n", Statement::comment("Open", CommentKind::Line))
        );
    }

    #[test]
    fn motion_first() {
        assert_parse!(
            Statement::parse,
            "G1\n",
            ("\n", Statement::Motion(Motion::Feed))
        );
        assert_parse!(
            Statement::parse,
            "G17",
            ("", Statement::PlaneSelect(PlaneSelect::XY))
        );
    }
}
