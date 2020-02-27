#![deny(intra_doc_link_resolution_failure)]

pub mod block;
pub mod comment;
pub mod coord;
pub mod motion;
pub mod word;

use crate::coord::coord;
use crate::coord::Coord;
use crate::motion::motion;
use crate::motion::Motion;
use crate::word::word;
use nom::branch::alt;
use nom::character::complete::char;
use nom::combinator::map;
use nom::IResult;

#[derive(Debug, PartialEq)]
pub enum Token {
    /// Block delete `/` character
    BlockDelete,

    /// Line number
    LineNumber(u32),

    /// Group 1
    Motion(Motion),

    /// Coordinate
    Coord(Coord),
}

pub fn token(i: &str) -> IResult<&str, Token> {
    alt((
        map(char('/'), |_| Token::BlockDelete),
        map(word('N'), |w| Token::LineNumber(w.value)),
        map(motion, Token::Motion),
        map(coord, Token::Coord),
    ))(i)
}
