//! Group 12: coordinate system

use crate::{value::Value, word::parse_word};
use nom::{
    branch::alt,
    bytes::streaming::tag,
    bytes::streaming::tag_no_case,
    bytes::streaming::take_until,
    character::streaming::char,
    character::streaming::digit1,
    character::streaming::one_of,
    character::streaming::space0,
    character::streaming::{alpha1, anychar, multispace0},
    combinator::map,
    combinator::map_opt,
    combinator::map_res,
    combinator::peek,
    combinator::{cond, verify},
    multi::many0,
    multi::many_m_n,
    multi::separated_list0,
    sequence::delimited,
    sequence::preceded,
    sequence::{separated_pair, terminated},
    IResult,
};

#[derive(Debug, PartialEq, Clone)]
pub enum CoordinateSystem {
    G54 = 0,
    G55 = 1,
    G56 = 2,
    G57 = 3,
    G58 = 4,
    G59 = 5,
    G59_1 = 6,
    G59_2 = 7,
    G59_3 = 8,
}

impl CoordinateSystem {
    pub fn parse(i: &str) -> IResult<&str, Self> {
        alt((
            // Decimal offsets must be parsed first to consume the decimal.
            map(tag_no_case("G59.1"), |_| CoordinateSystem::G59_1),
            map(tag_no_case("G59.2"), |_| CoordinateSystem::G59_2),
            map(tag_no_case("G59.3"), |_| CoordinateSystem::G59_3),
            map(tag_no_case("G54"), |_| CoordinateSystem::G54),
            map(tag_no_case("G55"), |_| CoordinateSystem::G55),
            map(tag_no_case("G56"), |_| CoordinateSystem::G56),
            map(tag_no_case("G57"), |_| CoordinateSystem::G57),
            map(tag_no_case("G58"), |_| CoordinateSystem::G58),
            map(tag_no_case("G59"), |_| CoordinateSystem::G59),
        ))(i)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn lowercase() {
        assert_eq!(
            CoordinateSystem::parse("g54;"),
            Ok((";", CoordinateSystem::G54))
        );
    }

    #[test]
    fn decimal() {
        assert_eq!(
            CoordinateSystem::parse("G59.1;"),
            Ok((";", CoordinateSystem::G59_1))
        );
    }
}