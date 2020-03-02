use crate::token::{token, Token, TokenType};
use crate::ParseInput;
use nom::character::complete::space0;
use nom::multi::many0;
use nom::sequence::terminated;
use nom::IResult;

#[derive(Debug, Clone, PartialEq)]
pub struct Block {
    /// Whether the line has a leading `/` block delete character present
    pub block_delete: bool,

    /// All tokens in this block, including optional block delete and line numbers
    pub tokens: Vec<Token>,
}

/// Parse a block (single line of gcode)
pub fn block(i: ParseInput) -> IResult<ParseInput, Block> {
    // TODO: Fix and benchmark `separated_list()` if I can get it to support zero length separators
    let (i, tokens) = many0(terminated(token, space0))(i)?;

    Ok((
        i,
        Block {
            block_delete: tokens.first().map(|t| &t.token) == Some(&TokenType::BlockDelete),
            tokens,
        },
    ))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{coord::Coord, motion::Motion};
    use crate::{rem, tok};

    #[test]
    fn block_delete() {
        assert_eq!(
            block(ParseInput::new("/ G1 X10")),
            Ok((
                rem!("", 8),
                Block {
                    block_delete: true,
                    tokens: vec![
                        tok!(TokenType::BlockDelete, 0, 1),
                        tok!(TokenType::Motion(Motion::Feed), 2, 4),
                        tok!(TokenType::Coord(Coord::with_x(10.0)), 5, 8)
                    ]
                }
            ))
        );
    }

    #[test]
    fn line_number() {
        assert_eq!(
            block(ParseInput::new("N1234 G1 X10")),
            Ok((
                rem!("", 12),
                Block {
                    block_delete: false,
                    tokens: vec![
                        tok!(TokenType::LineNumber(1234u32), 0, 5),
                        tok!(TokenType::Motion(Motion::Feed), 6, 8),
                        tok!(TokenType::Coord(Coord::with_x(10.0)), 9, 12)
                    ]
                }
            ))
        );
    }

    #[test]
    fn line_number_and_block_delete() {
        assert_eq!(
            block(ParseInput::new("/ N1234 G1 X10")),
            Ok((
                rem!("", 14),
                Block {
                    block_delete: true,
                    tokens: vec![
                        tok!(TokenType::BlockDelete, 0, 1),
                        tok!(TokenType::LineNumber(1234u32), 2, 7),
                        tok!(TokenType::Motion(Motion::Feed), 8, 10),
                        tok!(TokenType::Coord(Coord::with_x(10.0)), 11, 14),
                    ]
                }
            ))
        );
    }
}
