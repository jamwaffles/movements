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
    character::complete::line_ending,
    multi::{many1, separated_list0},
    IResult,
};
use nom_locate::LocatedSpan;
pub use statement::Statement;
pub use value::Value;

pub type Span<'a> = LocatedSpan<&'a str>;

#[derive(Debug, Clone)]
pub struct Program<'a> {
    // TODO: Un-pub
    pub blocks: Vec<Block<'a>>,
}

impl<'a> Program<'a> {
    pub fn parse_complete(i: &'a str) -> IResult<Span, Self> {
        let i = Span::new(i);
        // let mut blocks = Vec::new();

        let (i, blocks) = separated_list0(many1(line_ending), Block::parse)(i)?;

        // let (i, blocks) = loop {
        //     match terminated(Block::parse, many0(line_ending))(i) {
        //         Err(nom::Err::Error(e)) => {
        //             break Ok((i, blocks));
        //         }
        //         Err(e) => {
        //             if blocks.is_empty() {
        //                 break Err(e);
        //             } else {
        //                 break Ok((i, blocks));
        //             }
        //         }
        //         Ok((i1, o)) => {
        //             blocks.push(o);
        //             i = i1;
        //         }
        //     }
        // }?;

        // dbg!(&res);

        // let (i, blocks) = res.unwrap();

        // println!("{}", i);
        // println!("{:#?}", blocks);

        // debug_assert!(i.is_empty(), "Remaining input: {}", i);

        Ok((i, Self { blocks }))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn simple_complete_parse() {
        let result = Program::parse_complete("G0 x0 y0\ng1 z10");

        println!("{:#?}", result);
    }
}
