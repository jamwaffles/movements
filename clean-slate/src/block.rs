//! Parse a block (line) populated with [`Word`]s.

use crate::spanned_word::{Span, Spanned, Word};
use nom::character::complete::space0;
use nom::{multi::many0, sequence::preceded, IResult};

#[derive(Debug)]
pub struct Block<'a> {
    words: Vec<Spanned<'a, Word>>,
}

// TODO: Trait?
impl<'a> Block<'a> {
    pub fn parse(i: Span<'a>) -> IResult<Span, Self> {
        let (i, words) = many0(preceded(space0, Word::parse))(i)?;

        Ok((i, Self { words }))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn newline() {
        insta::assert_debug_snapshot!(Block::parse("G0 G4 P2.5 ; line comment".into()));
    }
}
