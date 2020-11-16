mod block;
mod coord;
mod expression;
mod macros;
mod modal_groups;
mod parameter;
mod statement;
mod value;
mod word;

use block::Block;
use nom::{
    character::streaming::line_ending,
    combinator::all_consuming,
    multi::{many0, many1, separated_list0},
    sequence::terminated,
    IResult,
};
use nom_locate::LocatedSpan;
pub use statement::Statement;
pub use value::Value;

pub type Span<'a> = LocatedSpan<&'a str>;

#[derive(Debug, Clone)]
pub struct Program {
    // TODO: Un-pub
    pub blocks: Vec<Block>,
}

impl Program {
    pub fn parse_complete(i: &str) -> IResult<Span, Self> {
        let i = Span::new(i);

        let (i, blocks) = separated_list0(many1(line_ending), Block::parse)(i)?;

        // println!("{:#?}", blocks);

        // debug_assert!(i.is_empty(), "Remaining input: {}", i);

        Ok((i, Self { blocks }))
    }
}
