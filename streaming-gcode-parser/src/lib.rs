mod coord;
mod expression;
mod modal_groups;
mod parameter;
mod value;
mod word;

use coord::Coord;
use modal_groups::{
    CoordinateSystem, CutterCompensation, DistanceMode, FeedrateMode, Motion, PlaneSelect,
    Stopping, Units,
};
use nom::{
    branch::alt,
    bytes::streaming::tag,
    bytes::streaming::tag_no_case,
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
use parameter::Parameter;
use std::str::FromStr;
use value::Value;

#[derive(Debug, PartialEq, Clone)]
pub struct Block {
    block_delete: bool,
    line_number: Option<u32>,
    words: Vec<Statement>,
}

// impl Block {
//     pub fn parse(i: &str) -> IResult<&str, Self> {
//         let (block_delete, i) = opt(char('/'))(i)?;

//         let (line_number, i) = opt(map(
//             separated_pair(tag_no_case("N"), space0, digit1),
//             |(_, number)| number.parse_to(),
//         ))(i)?;

//         let (words, i) = separated_list0(space0, word)(i)?;
//     }
// }

#[derive(Debug, PartialEq, Clone)]
pub enum CommentKind {
    /// `; comment`
    Line,

    /// `(comment)`
    Delimited,
}

#[derive(Debug, PartialEq, Clone)]
pub enum Statement {
    /// Block delete.
    BlockDelete,

    /// Line number.
    LineNumber(u32),

    /// Comment.
    Comment { comment: String, kind: CommentKind },

    /// Set parameter, e.g. `#5550 = [12 + 13]`.
    SetParameter { parameter: Parameter, value: Value },

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

    // NOTE: This is the only code in the group, so doesn't need its own module.
    /// Tool change.
    ToolChange,

    /// Dynamic token whose code is evaluated at runtime.
    ///
    /// When parsed, `letter` is transformed to lowercase.
    Dynamic { letter: char, number: Value },
}

impl Statement {
    fn parse_block_delete(i: &str) -> IResult<&str, Option<Self>> {
        opt(map(char('/'), |_| Statement::BlockDelete))(i)
    }

    fn parse_line_number(i: &str) -> IResult<&str, Option<Self>> {
        opt(map_res(
            separated_pair(tag_no_case("N"), space0, digit1),
            |(_, number)| u32::from_str(number).map(Statement::LineNumber),
        ))(i)
    }

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
                preceded(char(';'), take_while(|c| c != '\r' && c != '\n')),
                |comment: &str| (comment.trim().to_string(), CommentKind::Line),
            ),
            map(
                delimited(char('('), take_until(")"), char(')')),
                |comment: &str| (comment.trim().to_string(), CommentKind::Delimited),
            ),
        ))(i)
    }

    pub fn parse(i: &str) -> IResult<&str, Self> {
        alt((
            map(Motion::parse, Self::Motion),
            map(PlaneSelect::parse, Self::PlaneSelect),
            map(DistanceMode::parse, Self::DistanceMode),
            map(Stopping::parse, Self::Stopping),
            map(FeedrateMode::parse, Self::FeedrateMode),
            map(Units::parse, Self::Units),
            map(CutterCompensation::parse, Self::CutterCompensation),
            map(CoordinateSystem::parse, Self::CoordinateSystem),
            map(Coord::parse, Self::Coord),
            map(Self::parse_comment, |(comment, kind)| Self::Comment {
                comment,
                kind,
            }),
            // Tool change (M6)
            map(alt((tag_no_case("M6"), tag_no_case("M06"))), |_| {
                Self::ToolChange
            }),
            // Set parameter
            map(Self::parse_set_param, |(parameter, value)| {
                Self::SetParameter { parameter, value }
            }),
            // Dynamic code
            map(
                separated_pair(
                    map(anychar, |c| c.to_ascii_lowercase()),
                    space0,
                    Value::parse,
                ),
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

pub fn end_of_line(i: &str) -> IResult<&str, &str> {
    if i.is_empty() {
        Ok((i, i))
    } else {
        line_ending(i)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn comments() {
        assert_eq!(
            Statement::parse("(closed)"),
            Ok(("", Statement::comment("closed", CommentKind::Delimited)))
        );
        assert_eq!(
            Statement::parse("; Open\n"),
            Ok(("\n", Statement::comment("Open", CommentKind::Line)))
        );
    }
}
