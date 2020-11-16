use crate::{value::Value, word::parse_word, Span};
use nom::{character::streaming::one_of, combinator::map_res, IResult};

#[derive(Debug, Clone, PartialEq)]
pub enum Coord {
    X(Value),
    Y(Value),
    Z(Value),
    A(Value),
    B(Value),
    C(Value),
    U(Value),
    V(Value),
    W(Value),
}

impl Coord {
    fn from_char(c: char, value: Value) -> Result<Self, char> {
        // PERF: Benchmark using `'x' | 'X'`, etc, instead of to_ascii_lowercase()
        match c.to_ascii_lowercase() {
            'x' => Ok(Self::X(value)),
            'y' => Ok(Self::Y(value)),
            'z' => Ok(Self::Z(value)),
            'a' => Ok(Self::A(value)),
            'b' => Ok(Self::B(value)),
            'c' => Ok(Self::C(value)),
            'u' => Ok(Self::U(value)),
            'v' => Ok(Self::V(value)),
            'w' => Ok(Self::W(value)),
            other => Err(other),
        }
    }

    pub fn parse<'a>(i: Span) -> IResult<Span, Self> {
        map_res(parse_word(one_of("xyzabcuvwXYZABCUVW")), |(c, value)| {
            Self::from_char(c, value)
        })(i)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::assert_parse;

    #[test]
    fn check_coord() {
        assert_parse!(Coord::parse, "x10;", (";", Coord::X(10.0.into())));
        assert_parse!(Coord::parse, "x10 y20", (" y20", Coord::X(10.0.into())));
    }

    #[test]
    fn caps() {
        assert_parse!(Coord::parse, "Z10.1;", (";", Coord::Z(10.1.into())));
    }

    #[test]
    fn spaces() {
        assert_parse!(Coord::parse, "u -12.3;", (";", Coord::U((-12.3).into())));
    }
}
