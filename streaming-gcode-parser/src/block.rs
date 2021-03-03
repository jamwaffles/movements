use crate::statement::Token;
use crate::Span;
use core::str::FromStr;
use nom::{
    bytes::complete::tag_no_case,
    character::complete::{char, digit1, space0},
    combinator::{map, map_res, opt},
    multi::many1,
    sequence::{preceded, separated_pair},
    IResult,
};

#[derive(Debug, PartialEq, Clone)]
pub struct Block<'a> {
    block_delete: bool,
    line_number: Option<u32>,
    // TODO: Un-pub
    pub words: Vec<Token<'a>>,
}

impl<'a> Default for Block<'a> {
    fn default() -> Self {
        Self {
            block_delete: false,
            line_number: None,
            words: Vec::new(),
        }
    }
}

impl<'a> Block<'a> {
    fn tokens(tokens: Vec<Token<'a>>) -> Self {
        Self {
            words: tokens,
            ..Self::default()
        }
    }

    fn parse_block_delete(i: Span) -> IResult<Span, Option<()>> {
        opt(map(char('/'), |_| ()))(i)
    }

    fn parse_line_number(i: Span) -> IResult<Span, Option<u32>> {
        opt(map_res(
            separated_pair(tag_no_case("N"), space0, digit1),
            |(_, number): (_, Span)| u32::from_str(number.fragment()),
        ))(i)
    }

    fn parse_words(i: Span) -> IResult<Span, Vec<Token>> {
        let (i, items) = opt(many1(preceded(space0, Token::parse)))(i)?;

        Ok((i, items.unwrap_or_default()))
    }

    pub fn parse(i: Span<'a>) -> IResult<Span<'a>, Self> {
        let (i, block_delete) = Self::parse_block_delete(i)?;

        let (i, line_number) = Self::parse_line_number(i)?;

        let (i, words) = Self::parse_words(i)?;

        Ok((
            i,
            Self {
                block_delete: block_delete.is_some(),
                line_number,
                words,
            },
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        assert_parse,
        coord::Coord,
        modal_groups::{DistanceMode, Motion, Units},
        statement::Statement,
    };

    #[test]
    fn empty_line() {
        assert_parse!(Block::parse, "\n", ("\n", Block::default()));
    }

    #[test]
    fn check_block() {
        assert_parse!(
            Block::parse,
            "g21 g90",
            (
                "",
                Block::tokens(vec![
                    Statement::Units(Units::Mm).to_token(3, 1),
                    Statement::DistanceMode(DistanceMode::Absolute).to_token(7, 1),
                ])
            )
        );
    }

    #[test]
    fn check_block_no_spaces() {
        assert_parse!(
            Block::parse,
            "G21G90",
            (
                "",
                Block::tokens(vec![
                    Statement::Units(Units::Mm).to_token(3, 1),
                    Statement::DistanceMode(DistanceMode::Absolute).to_token(6, 1),
                ])
            )
        );
    }

    #[test]
    fn no_ending() {
        assert_parse!(
            Block::parse,
            "g1 z10",
            (
                "",
                Block::tokens(vec![
                    Statement::Motion(Motion::Feed).to_token(2, 1),
                    Statement::Coord(Coord::Z(10.0.into())).to_token(6, 1),
                ])
            )
        );
    }
}
