use crate::word::word;
use crate::Token;
use nom::character::complete::char;
use nom::character::complete::space0;
use nom::combinator::opt;
use nom::error::ParseError;
use nom::sequence::terminated;
use nom::IResult;
use std::str::FromStr;

#[derive(Debug, PartialEq)]
pub struct Block {
    /// Whether the line has a leading `/` block delete character present
    block_delete: bool,

    /// Line number (optional)
    line_number: Option<u32>,

    /// All tokens in this block, including optional block delete and line numbers
    tokens: Vec<Token>,
}

/// Parse a block (single line of gcode)
pub fn block(i: &str) -> IResult<&str, Block> {
    let (i, block_delete_token) = opt(terminated(char('/'), space0))(i)?;
    let (i, line_number_token) = opt(terminated(word('N'), space0))(i)?;

    Ok((
        i,
        Block {
            block_delete: block_delete_token.is_some(),
            line_number: line_number_token.map(|word| word.value),
            tokens: Vec::new(),
        },
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn simple_block() {}

    #[test]
    fn block_delete() {
        assert_eq!(
            block("/ G1 X0"),
            Ok((
                "G1 X0",
                Block {
                    block_delete: true,
                    line_number: None,
                    tokens: Vec::new()
                }
            ))
        );
    }

    #[test]
    fn line_number() {
        assert_eq!(
            block("N1234 G1 X0"),
            Ok((
                "G1 X0",
                Block {
                    block_delete: false,
                    line_number: Some(1234),
                    tokens: Vec::new()
                }
            ))
        );
    }

    #[test]
    fn line_number_and_block_delete() {
        assert_eq!(
            block("/ N1234 G1 X0"),
            Ok((
                "G1 X0",
                Block {
                    block_delete: true,
                    line_number: Some(1234),
                    tokens: Vec::new()
                }
            ))
        );
    }
}
