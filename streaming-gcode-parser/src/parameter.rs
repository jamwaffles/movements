use crate::{expression::Expression, Span};
use nom::{
    branch::alt, bytes::complete::tag, bytes::complete::take_until, character::complete::char,
    character::complete::digit1, combinator::map, combinator::map_res, sequence::delimited,
    sequence::preceded, IResult,
};

#[derive(Debug, PartialEq, Clone)]
pub enum Parameter<'a> {
    /// `#<local>`
    Local(&'a str),
    /// `#<_global>`
    Global(&'a str),
    /// `#5520`
    Index(usize),
    /// `#[220 + 5]`
    /// `#[220 + #50]`
    Dynamic(Expression<'a>),
}

impl<'a> Parameter<'a> {
    pub fn parse(i: Span<'a>) -> IResult<Span<'a>, Self> {
        let (i, param) = preceded(
            char('#'),
            alt((
                map(
                    delimited(tag("<_"), take_until(">"), char('>')),
                    |name: Span| Self::Global(name.fragment()),
                ),
                map(
                    delimited(char('<'), take_until(">"), char('>')),
                    |name: Span| Self::Local(name.fragment()),
                ),
                map_res(digit1, |bytes: Span| bytes.parse().map(Self::Index)),
                map(Expression::parse, Self::Dynamic),
            )),
        )(i)?;

        Ok((i, param))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::assert_parse;
    use crate::expression::Expression;
    use crate::expression::ExpressionToken;
    use crate::expression::Operator;
    use crate::Value;

    #[test]
    fn local() {
        assert_parse!(
            Parameter::parse,
            "#<testo>",
            ("", Parameter::Local("testo"))
        );
    }

    #[test]
    fn global() {
        assert_parse!(
            Parameter::parse,
            "#<_testo>",
            ("", Parameter::Global("testo"))
        );
    }

    #[test]
    fn index() {
        assert_parse!(Parameter::parse, "#5535;", (";", Parameter::Index(5535)));
    }

    #[test]
    fn dynamic() {
        assert_parse!(
            Parameter::parse,
            "#[100 + 200]",
            (
                "",
                Parameter::Dynamic(Expression {
                    tokens: vec![
                        ExpressionToken::Value(Value::Literal(100.0)),
                        ExpressionToken::Operator(Operator::Add),
                        ExpressionToken::Value(Value::Literal(200.0))
                    ]
                })
            )
        );
    }
}
