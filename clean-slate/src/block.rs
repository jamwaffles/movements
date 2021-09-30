//! Parse a block (line) populated with [`Word`]s.

use crate::spanned_word::{Spanned, Word};
use crate::Span;
use nom::character::complete::space0;
use nom::error::{ContextError, ParseError};
use nom::{multi::many0, sequence::preceded, IResult};

#[derive(Debug)]
pub struct Block<'a> {
    words: Vec<Spanned<'a, Word>>,
}

// TODO: Trait?
impl<'a> Block<'a> {
    pub fn parse<E>(i: Span<'a>) -> IResult<Span, Self, E>
    where
        E: ParseError<Span<'a>> + ContextError<Span<'a>>,
    {
        let (i, words) = many0(preceded(space0, Word::parse_spanned::<'a, E>))(i)?;

        Ok((i, Self { words }))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use nom::error::ErrorKind;

    #[test]
    fn newline() {
        insta::assert_debug_snapshot!(Block::parse::<(_, ErrorKind)>(
            "G0 G4 P2.5 ; line comment".into()
        ));
    }
}
