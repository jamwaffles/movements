//! Group 12: coordinate system

use crate::Span;

use nom::{branch::alt, bytes::complete::tag_no_case, combinator::map, IResult};

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
    pub fn parse(i: Span) -> IResult<Span, Self> {
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
    use crate::assert_parse;

    #[test]
    fn lowercase() {
        assert_parse!(
            CoordinateSystem::parse,
            "g54;",
            (";", CoordinateSystem::G54)
        );
    }

    #[test]
    fn decimal() {
        assert_parse!(
            CoordinateSystem::parse,
            "G59.1;",
            (";", CoordinateSystem::G59_1)
        );
    }
}
