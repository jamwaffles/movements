use crate::token::token_parser;
use crate::token::{token, Token, TokenType};
use crate::word::word;
use crate::ParseInput;
use nom::character::complete::char;
use nom::character::complete::space0;
use nom::combinator::map;
use nom::combinator::opt;
use nom::multi::many0;
use nom::sequence::delimited;
use nom::sequence::preceded;
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
    let (i, block_delete) = delimited(
        space0,
        opt(token_parser(map(char('/'), |_| TokenType::BlockDelete))),
        space0,
    )(i)?;

    let (i, line_number) = terminated(
        opt(token_parser(map(word('N'), |w| {
            TokenType::LineNumber(w.value)
        }))),
        space0,
    )(i)?;

    // TODO: Fix and benchmark `separated_list()` if I can get it to support zero length separators
    let (i, mut rest) = preceded(space0, many0(terminated(token, space0)))(i)?;

    // TODO: Add file position context thing

    let mut tokens = Vec::new();

    let has_block_delete = block_delete.is_some();

    if let Some(del) = block_delete {
        tokens.push(del);
    }

    if let Some(n) = line_number {
        tokens.push(n);
    }

    tokens.append(&mut rest);

    Ok((
        i,
        Block {
            block_delete: has_block_delete,
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
                        tok!(TokenType::BlockDelete, offs = (0, 1)),
                        tok!(TokenType::Motion(Motion::Feed), offs = (2, 4)),
                        tok!(TokenType::Coord(Coord::with_x(10.0)), offs = (5, 8))
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
                        tok!(TokenType::LineNumber(1234u32), offs = (0, 5)),
                        tok!(TokenType::Motion(Motion::Feed), offs = (6, 8)),
                        tok!(TokenType::Coord(Coord::with_x(10.0)), offs = (9, 12))
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
                        tok!(TokenType::BlockDelete, offs = (0, 1)),
                        tok!(TokenType::LineNumber(1234u32), offs = (2, 7)),
                        tok!(TokenType::Motion(Motion::Feed), offs = (8, 10)),
                        tok!(TokenType::Coord(Coord::with_x(10.0)), offs = (11, 14)),
                    ]
                }
            ))
        );
    }
}
