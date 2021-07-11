//! Group 2: plane select

use crate::Span;

use nom::{branch::alt, bytes::complete::tag_no_case, combinator::map, IResult};

#[derive(Debug, PartialEq, Clone)]
pub enum PlaneSelect {
    /// `G17`
    XY,

    /// `G18`
    XZ,

    /// `G19`
    YZ,
}

impl PlaneSelect {
    pub fn parse(i: Span) -> IResult<Span, Self> {
        alt((
            map(tag_no_case("G17"), |_| PlaneSelect::XY),
            map(tag_no_case("G18"), |_| PlaneSelect::XZ),
            map(tag_no_case("G19"), |_| PlaneSelect::YZ),
        ))(i)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::assert_parse;

    #[test]
    fn lowercase() {
        assert_parse!(PlaneSelect::parse, "g17;", (";", PlaneSelect::XY));
    }

    #[test]
    fn planes() {
        assert_parse!(PlaneSelect::parse, "G18;", (";", PlaneSelect::XZ));
        assert_parse!(PlaneSelect::parse, "G19;", (";", PlaneSelect::YZ));
    }
}
