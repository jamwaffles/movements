//! Parse a block (line) populated with [`Word`]s.

use crate::block::Block;
use crate::spanned_word::Span;
use nom::character::complete::line_ending;
use nom::combinator::all_consuming;
use nom::multi::separated_list0;
use nom::IResult;

#[derive(Debug)]
pub struct Program<'a> {
    blocks: Vec<Block<'a>>,
}

// TODO: Trait?
impl<'a> Program<'a> {
    pub fn parse(i: Span<'a>) -> IResult<Span, Self> {
        let (i, blocks) = all_consuming(separated_list0(line_ending, Block::parse))(i)?;

        Ok((i, Self { blocks }))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn newline() {
        let program = r#"(begin)
        G0
        G4 P2.5
        G0
        ;end"#;

        insta::assert_debug_snapshot!(Program::parse(program.into()));
    }
}
