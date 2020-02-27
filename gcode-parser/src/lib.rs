#![deny(intra_doc_link_resolution_failure)]

pub mod block;
pub mod comment;
pub mod coord;
pub mod motion;
pub mod plane_select;
pub mod units;
pub mod word;

use crate::coord::coord;
use crate::coord::Coord;
use crate::motion::motion;
use crate::motion::Motion;
use crate::plane_select::{plane_select, PlaneSelect};
use crate::units::{units, Units};
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

    /// Plane select
    PlaneSelect(PlaneSelect),

    /// Units
    Units(Units),
}

pub fn token(i: &str) -> IResult<&str, Token> {
    alt((
        map(char('/'), |_| Token::BlockDelete),
        map(word('N'), |w| Token::LineNumber(w.value)),
        map(motion, Token::Motion),
        map(coord, Token::Coord),
        map(plane_select, Token::PlaneSelect),
        map(units, Token::Units),
    ))(i)
}
