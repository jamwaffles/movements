use crate::{
    coord::Coord,
    modal_groups::{
        CoordinateSystem, CutterCompensation, DistanceMode, FeedrateMode, Motion, NonModal,
        PlaneSelect, Stopping, Units,
    },
    parameter::Parameter,
    value::Value,
    word::parse_word,
};
use nom::{
    branch::alt,
    bytes::streaming::tag,
    bytes::streaming::tag_no_case,
    bytes::streaming::take,
    bytes::streaming::take_till,
    bytes::streaming::take_until,
    bytes::streaming::take_while,
    character::streaming::char,
    character::streaming::digit1,
    character::streaming::not_line_ending,
    character::streaming::one_of,
    character::streaming::space0,
    character::{
        complete::line_ending,
        streaming::{alpha1, anychar, multispace0},
    },
    combinator::map,
    combinator::map_res,
    combinator::not,
    combinator::opt,
    combinator::peek,
    combinator::recognize,
    combinator::{cond, verify},
    combinator::{eof, map_opt},
    multi::many0,
    multi::many_m_n,
    multi::separated_list0,
    sequence::delimited,
    sequence::preceded,
    sequence::{separated_pair, terminated},
    IResult,
};

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
    fn parse_set_param(i: &str) -> IResult<&str, (Parameter, Value)> {
        separated_pair(
            Parameter::parse,
            delimited(space0, char('='), space0),
            Value::parse,
        )(i)
    }

    fn parse_comment(i: &str) -> IResult<&str, (String, CommentKind)> {
        alt((
            map(
                preceded(char(';'), take_till(|c| c == '\r' || c == '\n')),
                |comment: &str| (comment.trim().to_string(), CommentKind::Line),
            ),
            map(
                delimited(char('('), take_until(")"), char(')')),
                |comment: &str| (comment.trim().to_string(), CommentKind::Delimited),
            ),
        ))(i)
    }

    /// Parses anything that isn't a line number or block delete.
    ///
    /// Line numbers and block delete hold special positions in the block, so are parsed separately.
    pub fn parse(i: &str) -> IResult<&str, Self> {
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

    #[test]
    fn assignment() {
        assert_eq!(
            Statement::parse("#9 = 100;"),
            Ok((
                ";",
                Statement::SetParameter {
                    parameter: Parameter::Index(9),
                    value: 100.0.into()
                }
            ))
        );
    }

    #[test]
    fn comments() {
        assert_eq!(
            Statement::parse("(closed)"),
            Ok(("", Statement::comment("closed", CommentKind::Delimited)))
        );
        assert_eq!(
            Statement::parse("(newline and param)\n#9=0"),
            Ok((
                "\n#9=0",
                Statement::comment("newline and param", CommentKind::Delimited)
            ))
        );
        assert_eq!(
            Statement::parse("; Open\n"),
            Ok(("\n", Statement::comment("Open", CommentKind::Line)))
        );
    }

    #[test]
    fn motion_first() {
        assert_eq!(
            Statement::parse("G1\n"),
            Ok(("\n", Statement::Motion(Motion::Feed)))
        );
        assert_eq!(
            Statement::parse("G17"),
            Ok(("", Statement::PlaneSelect(PlaneSelect::XY)))
        );
    }
}
