#![deny(intra_doc_link_resolution_failure)]

use crate::block::block;
pub use crate::block::Block;
pub use crate::token::Token;
pub use crate::token::TokenType;
use nom::character::complete::line_ending;
use nom::character::complete::multispace0;
use nom::character::complete::space0;
use nom::multi::separated_list;
use nom::sequence::delimited;
use nom_locate::LocatedSpan;

pub mod block;
pub mod comment;
pub mod coord;
mod macros;
pub mod motion;
pub mod non_modal;
pub mod plane_select;
pub mod spindle;
pub mod stopping;
pub mod token;
pub mod units;
pub mod word;

pub mod tokens {
    pub use crate::stopping::Stopping;
}

pub type ParseInput<'a> = LocatedSpan<&'a str>;

#[derive(Debug, Clone)]
pub struct GcodeProgram<'a> {
    text: &'a str,

    pub blocks: Vec<Block>,
}

impl<'a> GcodeProgram<'a> {
    pub fn from_str(text: &'a str) -> Result<Self, ()> {
        let i = ParseInput::new(text);

        let (i, blocks) = delimited(
            multispace0,
            separated_list(line_ending, delimited(space0, block, space0)),
            multispace0,
        )(i)
        // TODO: Better error handling
        .map_err(|e| println!("{:?}", e))?;

        if !i.fragment().is_empty() {
            Err(())
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
#[derive(Debug, Clone, PartialEq)]
pub struct Location {
    offset: usize,
    line: u32,
    column: usize,
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
    use crate::tokens::Stopping;

    #[test]
    fn whitespace() {
        let program = GcodeProgram::from_str(
            r#"
            F500

        M2
        "#,
        )
        .unwrap();

        assert_eq!(
            program.block_iter().collect::<Vec<_>>(),
            vec![
                &Block {
                    block_delete: false,
                    tokens: vec![tok!(TokenType::FeedRate(500.0), 13, 17, 2, 2),]
                },
                &Block {
                    block_delete: false,
                    tokens: vec![]
                },
                &Block {
                    block_delete: false,
                    tokens: vec![tok!(
                        TokenType::Stopping(Stopping::EndProgram),
                        27,
                        29,
                        4,
                        4
                    )]
                },
                &Block {
                    block_delete: false,
                    tokens: vec![]
                },
            ]
        )
    }
}
