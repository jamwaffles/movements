use crate::expression::Expression;
use nom::{
    branch::alt,
    bytes::streaming::tag,
    bytes::streaming::take_until,
    character::streaming::char,
    character::streaming::digit1,
    character::streaming::{alpha1, anychar, multispace0},
    combinator::map,
    combinator::map_res,
    multi::many0,
    multi::separated_list0,
    sequence::delimited,
    sequence::preceded,
    IResult,
};

#[derive(Debug, PartialEq, Clone)]
pub enum Parameter {
    /// `#<local>`
    Local(String),
    /// `#<_global>`
    Global(String),
    /// `#5520`
    Index(usize),
    /// `#[220 + 5]`
    /// `#[220 + #50]`
    Dynamic(Expression),
}

impl Parameter {
    pub fn parse(i: &str) -> IResult<&str, Self> {
        let (i, param) = preceded(
            char('#'),
            alt((
                map(
                    delimited(tag("<_"), take_until(">"), char('>')),
                    |name: &str| Self::Global(name.to_string()),
                ),
                map(
                    delimited(char('<'), take_until(">"), char('>')),
                    |name: &str| Self::Local(name.to_string()),
                ),
                map_res(digit1, |bytes: &str| bytes.parse().map(Self::Index)),
                map(Expression::parse, Self::Dynamic),
            )),
        )(i)?;

        Ok((i, param))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        expression::{ExpressionToken, Operator},
        value::Value,
    };

    #[test]
    fn local() {
        assert_eq!(
            Parameter::parse("#<testo>"),
            Ok(("", Parameter::Local(String::from("testo"))))
        );
    }

    #[test]
    fn global() {
        assert_eq!(
            Parameter::parse("#<_testo>"),
            Ok(("", Parameter::Global(String::from("testo"))))
        );
    }

    #[test]
    fn index() {
        assert_eq!(
            Parameter::parse("#5535;"),
            Ok((";", Parameter::Index(5535)))
        );
    }

    #[test]
    fn dynamic() {
        assert_eq!(
            Parameter::parse("#[100 + 200]"),
            Ok((
                "",
                Parameter::Dynamic(Expression {
                    tokens: vec![
                        ExpressionToken::Value(Value::Literal(100.0)),
                        ExpressionToken::Operator(Operator::Add),
                        ExpressionToken::Value(Value::Literal(200.0))
                    ]
                })
            ))
        );
    }
}
