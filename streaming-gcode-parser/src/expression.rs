use crate::value::Value;
use nom::{
    branch::alt,
    character::streaming::char,
    character::streaming::multispace0,
    character::streaming::space0,
    combinator::map,
    multi::many0,
    multi::separated_list0,
    multi::separated_list1,
    sequence::{delimited, terminated},
    IResult,
};

/// Something wrapped in square braces, e.g. `[200 - 3]`.
#[derive(Debug, PartialEq, Clone)]
pub struct Expression {
    pub(crate) tokens: Vec<ExpressionToken>,
}

impl Expression {
    pub fn parse(i: &str) -> IResult<&str, Self> {
        let (i, tokens) = delimited(
            char('['),
            many0(terminated(ExpressionToken::parse, space0)),
            char(']'),
        )(i)?;

        Ok((i, Self { tokens }))
    }
}

#[derive(Debug, PartialEq, Clone)]
pub enum ExpressionToken {
    Value(Value),
    Expression(Expression),
    Operator(Operator),
}

impl ExpressionToken {
    fn parse(i: &str) -> IResult<&str, ExpressionToken> {
        alt((
            map(Value::parse, ExpressionToken::Value),
            map(Operator::parse, ExpressionToken::Operator),
            map(Expression::parse, ExpressionToken::Expression),
        ))(i)
    }
}

#[derive(Debug, PartialEq, Clone)]
pub enum Operator {
    Add,
    Sub,
    Mul,
    Div,
    // etc
}

impl Operator {
    fn parse(i: &str) -> IResult<&str, Operator> {
        alt((
            map(char('+'), |_| Operator::Add),
            map(char('-'), |_| Operator::Sub),
            map(char('*'), |_| Operator::Mul),
            map(char('/'), |_| Operator::Div),
        ))(i)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn simple() {
        assert_eq!(
            Expression::parse("[100 + 200]"),
            Ok((
                "",
                Expression {
                    tokens: vec![
                        ExpressionToken::Value(Value::Literal(100.0)),
                        ExpressionToken::Operator(Operator::Add),
                        ExpressionToken::Value(Value::Literal(200.0))
                    ]
                }
            ))
        );
    }
}
