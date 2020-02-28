#![deny(intra_doc_link_resolution_failure)]

use crate::block::block;
use crate::block::Block;
use nom::character::complete::line_ending;
use nom::multi::separated_list;

pub mod block;
pub mod comment;
pub mod coord;
pub mod motion;
pub mod non_modal;
pub mod plane_select;
pub mod spindle;
pub mod token;
pub mod units;
pub mod word;

pub struct Program {
    blocks: Vec<Block>,
}

impl Program {
    pub fn from_str(i: &str) -> Result<Self, ()> {
        let (i, blocks) = separated_list(line_ending, block)(i).map_err(|e| ())?;

        if !i.is_empty() {
            Err(())
        } else {
            Ok(Self { blocks })
        }
    }
}
