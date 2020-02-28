#![deny(intra_doc_link_resolution_failure)]

use crate::block::block;
use crate::block::Block;
use crate::token::Token;
use nom::character::complete::line_ending;
use nom::multi::separated_list;
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

pub type ParseInput<'a> = LocatedSpan<&'a str>;

pub struct GcodeProgram<'a> {
    text: ParseInput<'a>,

    blocks: Vec<Block<'a>>,
}

impl<'a> GcodeProgram<'a> {
    pub fn from_str(i: ParseInput<'a>) -> Result<Self, ()> {
        // TODO: Better error handling
        let (i, blocks) = separated_list(line_ending, block)(i).map_err(|_e| ())?;

        if !i.fragment().is_empty() {
            Err(())
        } else {
            Ok(Self { blocks, text: i })
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
