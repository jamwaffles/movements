use crate::{expression::Expression, parameter::Parameter};
use nom::{
    branch::alt, character::streaming::char, character::streaming::multispace0, combinator::map,
    multi::many0, multi::separated_list0, number::streaming::double, sequence::delimited, IResult,
};
/// Value.
#[derive(Debug, PartialEq, Clone)]
pub enum Value {
    /// `100.2`
    Literal(f64),
    /// `#2250`
    Parameter(Parameter),
    /// `[900 + 3 / #2250]`
    Expression(Expression),
}

impl Value {
    pub fn parse(i: &str) -> IResult<&str, Self> {
        alt((
            map(double, Value::Literal),
            map(Parameter::parse, Value::Parameter),
            map(Expression::parse, Value::Expression),
        ))(i)
    }
}

impl From<f64> for Value {
    fn from(other: f64) -> Self {
        Self::Literal(other)
    }
}

impl From<f32> for Value {
    fn from(other: f32) -> Self {
        Self::Literal(other.into())
    }
}

impl From<i32> for Value {
    fn from(other: i32) -> Self {
        Self::Literal(other.into())
    }
}

impl From<u32> for Value {
    fn from(other: u32) -> Self {
        Self::Literal(other.into())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::expression::{ExpressionToken, Operator};

    #[test]
    fn literal() {
        assert_eq!(Value::parse("100.0;"), Ok((";", Value::Literal(100.0))));
        assert_eq!(Value::parse("100;"), Ok((";", Value::Literal(100.0))));
    }
}
