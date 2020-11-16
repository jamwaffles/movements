use crate::statement::Statement;
use crate::Span;
use nom::{
    branch::alt,
    bytes::streaming::tag,
    bytes::streaming::tag_no_case,
    bytes::streaming::take_till,
    bytes::streaming::take_until,
    bytes::streaming::take_while,
    character::streaming::char,
    character::streaming::digit1,
    character::streaming::not_line_ending,
    character::streaming::one_of,
    character::streaming::space0,
    character::streaming::space1,
    character::{
        complete::line_ending,
        streaming::{alpha1, anychar, multispace0},
    },
    combinator::map,
    combinator::map_res,
    combinator::opt,
    combinator::peek,
    combinator::recognize,
    combinator::{cond, verify},
    combinator::{eof, map_opt},
    error::ParseError,
    multi::many0,
    multi::many1,
    multi::many_m_n,
    multi::separated_list0,
    sequence::delimited,
    sequence::preceded,
    sequence::{separated_pair, terminated},
    IResult, InputIter, InputLength, Parser,
};
use std::str::FromStr;

pub fn end_of_line(i: Span) -> IResult<Span, Span> {
    if i.is_empty() {
        Ok((i, i))
    } else {
        line_ending(i)
    }
}

#[derive(Debug, PartialEq, Clone, Default)]
pub struct Block {
    block_delete: bool,
    line_number: Option<u32>,
    // TODO: Un-pub
    pub words: Vec<Statement>,
}

fn parse_words(i: Span) -> IResult<Span, Vec<Statement>> {
    let mut i = i;
    let mut res = Vec::new();

    loop {
        match preceded(space0, Statement::parse)(i) {
            Err(nom::Err::Error(_)) => {
                break Ok((i, res));
            }
            Err(e) => {
                if res.is_empty() {
                    break Err(e);
                } else {
                    break Ok((i, res));
                }
            }
            Ok((i1, o)) => {
                res.push(o);
                i = i1;
            }
        }
    }
}

impl Block {
    pub fn words(words: Vec<Statement>) -> Self {
        Self {
            block_delete: false,
            line_number: None,
            words,
        }
    }

    fn parse_block_delete(i: Span) -> IResult<Span, Option<()>> {
        opt(map(char('/'), |_| ()))(i)
    }

    fn parse_line_number(i: Span) -> IResult<Span, Option<u32>> {
        opt(map_res(
            separated_pair(tag_no_case("N"), space0, digit1),
            |(_, number): (_, Span)| u32::from_str(&number.to_string()),
        ))(i)
    }

    pub fn parse(i: Span) -> IResult<Span, Self> {
        let (i, block_delete) = Self::parse_block_delete(i)?;

        let (i, line_number) = Self::parse_line_number(i)?;

        let (i, words) = parse_words(i)?;

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
        modal_groups::{DistanceMode, Units},
    };

    #[test]
    fn empty_line() {
        assert_parse!(Block::parse, "\n", ("\n", Block::default()));
    }

    #[test]
    fn check_block() {
        assert_parse!(
            Block::parse,
            "g21 g90;",
            (
                ";",
                Block::words(vec![
                    Statement::Units(Units::Mm),
                    Statement::DistanceMode(DistanceMode::Absolute),
                ])
            )
        );
    }

    #[test]
    fn check_block_no_spaces() {
        assert_parse!(
            Block::parse,
            "G21G90;",
            (
                ";",
                Block::words(vec![
                    Statement::Units(Units::Mm),
                    Statement::DistanceMode(DistanceMode::Absolute),
                ])
            )
        );
    }
}
