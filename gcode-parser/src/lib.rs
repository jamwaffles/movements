#![deny(broken_intra_doc_links)]

use crate::block::block;
pub use crate::block::Block;
pub use crate::token::Token;
pub use crate::token::TokenType;
use core::fmt;
use nalgebra::VectorN;
use nalgebra::U9;
use nom::character::complete::line_ending;
use nom::character::complete::multispace0;
use nom::multi::separated_list0;
use nom::sequence::delimited;
use nom_locate::LocatedSpan;

pub mod block;
pub mod comment;
pub mod coord;
pub mod cutter_compensation;
pub mod distance_mode;
mod macros;
pub mod motion;
pub mod non_modal;
pub mod plane_select;
pub mod spindle;
pub mod stopping;
pub mod token;
pub mod tool_change;
pub mod units;
pub mod word;

pub mod tokens {
    pub use crate::motion::Motion;
    pub use crate::stopping::Stopping;
}

pub type Axes = VectorN<f32, U9>;

pub type ParseInput<'a> = LocatedSpan<&'a str>;

#[derive(Debug, Clone)]
pub struct GcodeProgram<'a> {
    pub text: &'a str,

    pub blocks: Vec<Block>,
}

pub struct ProgramParseError<'a> {
    /// Failure point
    failure_point: ParseInput<'a>,

    /// Complete file input
    input: ParseInput<'a>,
}

impl<'a> fmt::Display for ProgramParseError<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // Location lines are 1-indexed so subtract 1 to get 0-index
        let failing_line = self.failure_point.location_line() as usize - 1;

        let whole_line = self.input.fragment().lines().nth(failing_line);

        write!(
            f,
            "Error on line {line}:\n\n{code}\n{marker_padding}^",
            line = self.failure_point.location_line(),
            code = whole_line.unwrap_or("( unknown input )"),
            marker_padding = " ".repeat(self.failure_point.get_utf8_column() - 1)
        )
    }
}

impl<'a> fmt::Debug for ProgramParseError<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let all_bytes = self.input.fragment().len();
        let remaining = self.failure_point.fragment().len();

        write!(
            f,
            "Failed to parse program.\n\n{}\n\n{} of {} bytes remaining ({} bytes consumed)",
            self.to_string(),
            remaining,
            all_bytes,
            all_bytes - remaining
        )
    }
}

impl<'a> GcodeProgram<'a> {
    pub fn from_str(text: &'a str) -> Result<Self, ProgramParseError> {
        let file = ParseInput::new(text);

        let (remaining, blocks) = delimited(
            multispace0,
            separated_list0(line_ending, block),
            multispace0,
        )(file)
        .map_err(|e| match e {
            nom::Err::Incomplete(_n) => unreachable!(),
            nom::Err::Error(e) | nom::Err::Failure(e) => ProgramParseError {
                failure_point: e.input,
                input: file,
            },
        })?;

        if !remaining.fragment().is_empty() {
            Err(ProgramParseError {
                failure_point: remaining,
                input: file,
            })
        } else {
            Ok(Self { blocks, text })
        }
    }

    /// Get an iterator over every block (line) in the program
    pub fn block_iter(&self) -> impl DoubleEndedIterator<Item = &Block> {
        self.blocks.iter()
    }

    /// Get an iterator over every token in the program
    pub fn token_iter(&self) -> impl DoubleEndedIterator<Item = &Token> {
        self.blocks.iter().map(|b| b.tokens.iter()).flatten()
    }
}

/// A span of text within the program
///
/// This is used instead of nom_locate's `LocatedSpan` as it does not hold a reference to the input.
#[derive(Debug, Clone, PartialEq)]
pub struct Location {
    /// Offset from the beginning of the input
    offset: usize,

    /// Line, 1-indexed
    line: u32,

    /// Column, 1-indexed
    column: usize,
}

impl Location {
    pub fn new(offset: usize, line: u32, column: usize) -> Self {
        Self {
            offset,
            line,
            column,
        }
    }
}

impl From<ParseInput<'_>> for Location {
    fn from(other: ParseInput) -> Self {
        Self {
            offset: other.location_offset(),
            line: other.location_line(),
            column: other.get_utf8_column(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::comment::{Comment, CommentType};
    use crate::stopping::Stopping;
    use crate::units::Units;

    #[test]
    fn whitespace() {
        let program_text = r#"
            F500

        M2
        "#;

        let program = GcodeProgram::from_str(program_text).unwrap();

        assert_eq!(
            program.block_iter().collect::<Vec<_>>(),
            vec![
                &Block {
                    block_delete: false,
                    tokens: vec![tok!(
                        TokenType::FeedRate(500.0),
                        offs = (13, 17),
                        line = (2, 2),
                        col = (13, 17)
                    ),]
                },
                &Block {
                    block_delete: false,
                    tokens: vec![]
                },
                &Block {
                    block_delete: false,
                    tokens: vec![tok!(
                        TokenType::Stopping(Stopping::EndProgram),
                        offs = (27, 29),
                        line = (4, 4),
                        col = (9, 11)
                    )]
                },
                &Block {
                    block_delete: false,
                    tokens: vec![]
                },
            ]
        )
    }

    #[test]
    fn windows() {
        let program_text = "(SuperCam Ver 2.2a SPINDLE)\r\nN1 G20\t( set inches mode - ash )";

        let program = GcodeProgram::from_str(program_text).unwrap();

        assert_eq!(
            program.block_iter().collect::<Vec<_>>(),
            vec![
                &Block {
                    block_delete: false,
                    tokens: vec![tok!(
                        TokenType::Comment(Comment::new(
                            "SuperCam Ver 2.2a SPINDLE",
                            CommentType::Parens
                        )),
                        offs = (0, 27),
                        line = (1, 1),
                        col = (1, 28)
                    ),]
                },
                &Block {
                    block_delete: false,
                    tokens: vec![
                        tok!(
                            TokenType::LineNumber(1),
                            offs = (29, 31),
                            line = (2, 2),
                            col = (1, 3)
                        ),
                        tok!(
                            TokenType::Units(Units::Inch),
                            offs = (32, 35),
                            line = (2, 2),
                            col = (4, 7)
                        ),
                        tok!(
                            TokenType::Comment(Comment::new(
                                "set inches mode - ash",
                                CommentType::Parens
                            )),
                            offs = (36, 61),
                            line = (2, 2),
                            col = (8, 33)
                        ),
                    ]
                },
            ]
        )
    }
}
