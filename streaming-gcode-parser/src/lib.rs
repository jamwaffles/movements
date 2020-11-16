mod block;
mod coord;
mod expression;
mod modal_groups;
mod parameter;
mod statement;
mod value;
mod word;

use block::Block;
use nom::{
    character::streaming::line_ending,
    combinator::all_consuming,
    multi::{many0, separated_list0},
    sequence::terminated,
    IResult,
};
pub use statement::Statement;
pub use value::Value;

#[derive(Debug, Clone)]
pub struct Program {
    // TODO: Un-pub
    pub blocks: Vec<Block>,
}

impl Program {
    pub fn parse_complete(i: &str) -> IResult<&str, Self> {
        let mut i = i;
        let mut blocks = Vec::new();

        loop {
            match terminated(Block::parse, line_ending)(i) {
                Err(nom::Err::Error(_)) => break,
                Err(e) => return Err(e),
                Ok((i1, o)) => {
                    blocks.push(o);
                    i = i1;
                }
            }

            if i.is_empty() {
                break;
            }
        }

        // debug_assert!(i.is_empty(), "Remaining input: {}", i);

        Ok((i, Self { blocks }))
    }
}
