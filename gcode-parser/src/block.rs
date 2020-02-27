use crate::token;
use crate::Token;
use nom::character::complete::space0;
use nom::multi::many0;
use nom::sequence::terminated;
use nom::IResult;

#[derive(Debug, PartialEq)]
pub struct Block {
    /// Whether the line has a leading `/` block delete character present
    block_delete: bool,

    /// All tokens in this block, including optional block delete and line numbers
    tokens: Vec<Token>,
}

/// Parse a block (single line of gcode)
pub fn block(i: &str) -> IResult<&str, Block> {
    // TODO: Fix and benchmark `separated_list()` if I can get it to support zero length separators
    let (i, tokens) = many0(terminated(token, space0))(i)?;

    Ok((
        i,
        Block {
            block_delete: tokens.first() == Some(&Token::BlockDelete),
            tokens,
        },
    ))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{coord::Coord, motion::Motion};

    #[test]
    fn simple_block() {}

    #[test]
    fn block_delete() {
        assert_eq!(
            block("/ G1 X10"),
            Ok((
                "",
                Block {
                    block_delete: true,
                    tokens: vec![
                        Token::BlockDelete,
                        Token::Motion(Motion::Feed),
                        Token::Coord(Coord::with_x(10.0))
                    ]
                }
            ))
        );
    }

    #[test]
    fn line_number() {
        assert_eq!(
            block("N1234 G1 X10"),
            Ok((
                "",
                Block {
                    block_delete: false,
                    tokens: vec![
                        Token::LineNumber(1234u32),
                        Token::Motion(Motion::Feed),
                        Token::Coord(Coord::with_x(10.0))
                    ]
                }
            ))
        );
    }

    #[test]
    fn line_number_and_block_delete() {
        assert_eq!(
            block("/ N1234 G1 X10"),
            Ok((
                "",
                Block {
                    block_delete: true,
                    tokens: vec![
                        Token::BlockDelete,
                        Token::LineNumber(1234u32),
                        Token::Motion(Motion::Feed),
                        Token::Coord(Coord::with_x(10.0))
                    ]
                }
            ))
        );
    }
}
