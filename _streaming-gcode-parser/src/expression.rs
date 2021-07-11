use crate::{value::Value, Span};
use nom::{
    branch::alt,
    character::complete::char,
    character::complete::space0,
    combinator::map,
    multi::many0,
    sequence::{delimited, terminated},
    IResult,
};

/// Something wrapped in square braces, e.g. `[200 - 3]`.
#[derive(Debug, PartialEq, Clone)]
pub struct Expression<'a> {
    pub(crate) tokens: Vec<ExpressionToken<'a>>,
}

impl<'a> Expression<'a> {
    pub fn parse(i: Span<'a>) -> IResult<Span<'a>, Expression> {
        let (i, tokens) = delimited(
            char('['),
            many0(terminated(ExpressionToken::parse, space0)),
            char(']'),
        )(i)?;

        Ok((i, Self { tokens }))
    }
}

#[derive(Debug, PartialEq, Clone)]
pub enum ExpressionToken<'a> {
    Value(Value<'a>),
    Expression(Expression<'a>),
    Operator(Operator),
}

impl ExpressionToken<'_> {
    fn parse(i: Span) -> IResult<Span, ExpressionToken> {
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
    fn parse(i: Span) -> IResult<Span, Operator> {
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
    use crate::assert_parse;

    #[test]
    fn simple() {
        assert_parse!(
            Expression::parse,
            "[100 + 200]",
            (
                "",
                Expression {
                    tokens: vec![
                        ExpressionToken::Value(Value::Literal(100.0)),
                        ExpressionToken::Operator(Operator::Add),
                        ExpressionToken::Value(Value::Literal(200.0))
                    ]
                }
            )
        );
    }
}
