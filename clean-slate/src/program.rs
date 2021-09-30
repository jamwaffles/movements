//! Parse a block (line) populated with [`Word`]s.

use crate::block::Block;
use crate::Span;
use nom::character::complete::line_ending;
use nom::error::{ContextError, ParseError};
use nom::multi::separated_list0;
use nom::IResult;

#[derive(Debug)]
pub struct Program<'a> {
    blocks: Vec<Block<'a>>,
}

// TODO: Trait?
impl<'a> Program<'a> {
    pub fn parse<E>(i: Span<'a>) -> IResult<Span, Self, E>
    where
        E: ParseError<Span<'a>> + ContextError<Span<'a>>,
    {
        let (i, blocks) = separated_list0(line_ending, Block::parse::<E>)(i)?;

        Ok((i, Self { blocks }))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use nom::error::ErrorKind;

    #[test]
    fn newline() {
        let program = r#"(begin)
        G0
        G4 P2.5
        G0
        ;end"#;

        insta::assert_debug_snapshot!(Program::parse::<(_, ErrorKind)>(program.into()));
    }
}
