use crate::{
    coord::Coord,
    modal_groups::{
        CoordinateSystem, CutterCompensation, DistanceMode, FeedrateMode, Motion, NonModal,
        PlaneSelect, Stopping, Units,
    },
    parameter::Parameter,
    value::Value,
    word::parse_word,
    Span,
};
use nom::{
    branch::alt,
    bytes::streaming::tag_no_case,
    bytes::streaming::take_till,
    bytes::streaming::take_until,
    character::streaming::anychar,
    character::streaming::char,
    character::streaming::digit1,
    character::streaming::one_of,
    character::streaming::space0,
    combinator::map,
    combinator::not,
    combinator::verify,
    sequence::delimited,
    sequence::preceded,
    sequence::{separated_pair, terminated},
    IResult,
};
use nom_locate::position;

#[derive(Debug, PartialEq, Clone)]
pub struct Token<'a> {
    pub position: Span<'a>,
    pub statement: Statement,
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
pub enum Statement {
    /// Comment.
    Comment { comment: String, kind: CommentKind },

    /// Set parameter, e.g. `#5550 = [12 + 13]`.
    SetParameter { parameter: Parameter, value: Value },

    /// Non modal G codes
    NonModal(NonModal),

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
    CutterCompensation(CutterCompensation),

    /// Modal group 12: coordinate system
    CoordinateSystem(CoordinateSystem),

    /// Axis value
    Coord(Coord),

    /// Tool change.
    ToolChange,

    /// Feed rate.
    FeedRate(Value),

    /// Spindle speed.
    SpindleSpeed(Value),

    /// Tool number.
    ToolNumber(Value),

    /// Dynamic token whose code is evaluated at runtime.
    ///
    /// When parsed, `letter` is transformed to lowercase.
    Dynamic { letter: char, number: Value },
}

impl Statement {
    pub(crate) fn to_token<'a>(self, offset: usize, line: u32) -> Token<'a> {
        let span = unsafe { Span::new_from_raw_offset(offset, line, "", ()) };

        Token {
            statement: self,
            position: span,
        }
    }

    fn parse_set_param(i: Span) -> IResult<Span, (Parameter, Value)> {
        separated_pair(
            Parameter::parse,
            delimited(space0, char('='), space0),
            Value::parse,
        )(i)
    }

    fn parse_comment(i: Span) -> IResult<Span, (String, CommentKind)> {
        alt((
            map(
                preceded(char(';'), take_till(|c| c == '\r' || c == '\n')),
                |comment: Span| (comment.trim().to_string(), CommentKind::Line),
            ),
            map(
                delimited(char('('), take_until(")"), char(')')),
                |comment: Span| (comment.trim().to_string(), CommentKind::Delimited),
            ),
        ))(i)
    }

    /// Parses anything that isn't a line number or block delete.
    ///
    /// Line numbers and block delete hold special positions in the block, so are parsed separately.
    pub fn parse(i: Span) -> IResult<Span, Self> {
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

    pub fn comment(text: &str, kind: CommentKind) -> Self {
        Self::Comment {
            comment: text.to_string(),
            kind,
        }
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
