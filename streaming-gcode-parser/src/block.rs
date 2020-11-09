use crate::statement::Statement;
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
    multi::many_m_n,
    multi::separated_list0,
    sequence::delimited,
    sequence::preceded,
    sequence::{separated_pair, terminated},
    IResult, InputIter, InputLength, Parser,
};
use std::str::FromStr;

pub fn end_of_line(i: &str) -> IResult<&str, &str> {
    if i.is_empty() {
        Ok((i, i))
    } else {
        line_ending(i)
    }
}

#[derive(Debug, PartialEq, Clone)]
pub struct Block {
    block_delete: bool,
    line_number: Option<u32>,
    words: Vec<Statement>,
}

fn parse_words(i: &str) -> IResult<&str, Vec<Statement>> {
    let mut i = i;
    let mut res = Vec::new();

    loop {
        match dbg!(preceded(space0, Statement::parse)(i)) {
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

    fn parse_block_delete(i: &str) -> IResult<&str, Option<()>> {
        opt(map(char('/'), |_| ()))(i)
    }

    fn parse_line_number(i: &str) -> IResult<&str, Option<u32>> {
        opt(map_res(
            separated_pair(tag_no_case("N"), space0, digit1),
            |(_, number)| u32::from_str(number),
        ))(i)
    }

    pub fn parse(i: &str) -> IResult<&str, Self> {
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
    use crate::modal_groups::{DistanceMode, Units};

    use super::*;
    use nom::{
        error::{Error, ErrorKind},
        Err,
    };

    #[test]
    fn check_block() {
        assert_eq!(
            Block::parse("g21 g90;"),
            Ok((
                ";",
                Block::words(vec![
                    Statement::Units(Units::Mm),
                    Statement::DistanceMode(DistanceMode::Absolute),
                ])
            ))
        );
    }

    #[test]
    fn check_block_no_spaces() {
        assert_eq!(
            Block::parse("G21G90;"),
            Ok((
                ";",
                Block::words(vec![
                    Statement::Units(Units::Mm),
                    Statement::DistanceMode(DistanceMode::Absolute),
                ])
            ))
        );
    }
}
