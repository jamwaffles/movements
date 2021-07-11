use nom::{
    branch::alt,
    bytes::complete::{is_not, tag, take, take_until, take_while},
    character::complete::{
        alpha1, alphanumeric1, anychar, char, digit1, not_line_ending, one_of, space0,
    },
    combinator::{iterator, map, map_res, ParserIterator},
    number::{
        self,
        complete::{float, recognize_float},
    },
    sequence::{delimited, separated_pair, terminated},
    IResult,
};
use std::{
    error::Error, fmt::Display, hint::unreachable_unchecked, marker::PhantomData, str::FromStr,
};

#[derive(Debug)]
pub enum TokenErrorKind {
    OpeningParen,
    ClosingParen,
    Other,
}

#[derive(Debug)]
pub struct TokenError<'a> {
    context: Option<nom::Err<nom::error::Error<&'a str>>>,
    kind: TokenErrorKind,
}

impl<'a> TokenError<'a> {
    fn from_nom(
        kind: TokenErrorKind,
    ) -> impl FnOnce(nom::Err<nom::error::Error<&'_ str>>) -> TokenError {
        |e| TokenError {
            context: Some(e),
            kind,
        }
    }
}

impl<'a> Display for TokenError<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("Todo lol")
    }
}

#[derive(Debug, Copy, Clone)]
pub enum Operator {
    Add,
    Subtract,
    Multiply,
    Divide,
}

impl Operator {
    fn parse(i: &str) -> IResult<&str, Self> {
        let (i, op) = one_of("+-/*")(i)?;

        let res = match op {
            '+' => Self::Add,
            '-' => Self::Subtract,
            '/' => Self::Divide,
            '*' => Self::Multiply,
            _ => unreachable!(),
        };

        Ok((i, res))
    }
}

/// Parameter.
///
/// Nested parameteres (e.g. `##2`) are not supported.
#[derive(Debug, Copy, Clone)]
pub enum Parameter<'a> {
    /// LinuxCNC-specific: params local to a function.
    Local(&'a str),

    /// Global parameters
    Global(&'a str),

    /// Numbered parameters
    Numbered(u16),
    // TODO
    // Expression(Expression),
}

impl<'a> Parameter<'a> {
    fn parse(i: &'a str) -> IResult<&'a str, Self> {
        separated_pair(
            char('#'),
            space0,
            alt((
                map_res(digit1, |digits: &str| {
                    digits.parse::<u16>().map(Self::Numbered)
                }),
                map(
                    delimited(char('<'), take_until(">"), char('>')),
                    Self::Local,
                ),
                map(
                    delimited(tag("<_"), take_until(">"), char('>')),
                    Self::Global,
                ),
            )),
        )(i)
        .map(|(i, (_, param))| (i, param))
    }
}

#[derive(Debug, Clone)]
pub enum Token<'a> {
    Value(f32),
    Parameter(Parameter<'a>),
    Operator(Operator),
    Expression(Expression<'a>),
}

impl<'a> Token<'a> {
    fn parse(i: &'a str) -> IResult<&'a str, Self> {
        alt((
            map(float, Token::Value),
            map(Operator::parse, Self::Operator),
            map(Parameter::parse, Self::Parameter),
            map(Expression::parse, Self::Expression),
        ))(i)
    }
}

#[derive(Debug, Clone)]
pub struct Expression<'a, F: FnMut(&'a str) -> nom::Err<nom::error::Error<&'a str>>> {
    iter: ParserIterator<&'a str, nom::Err<nom::error::Error<&'a str>>, F>,
}

impl<'a> Expression<'a> {
    pub fn parse(i: &'a str) -> Result<(), TokenError<'a>> {
        let (rest, i) = delimited(
            char('['),
            delimited(space0, take_until("]"), space0),
            char(']'),
        )(i)
        .map_err(TokenError::from_nom(TokenErrorKind::Other))?;

        // Check if rest is empty - if not, there's trailing junk

        // let (i, _opener) = terminated(char('['), space0)(i)
        //     // .map_err(|e| TokenError::from_nom(e, TokenErrorKind::OpeningParen))?;
        //     .map_err(TokenError::from_nom(TokenErrorKind::OpeningParen))?;

        let mut tokens = iterator(i, terminated(Token::parse, space0)(i));

        // let ass = tokens.collect::<Vec<_>>();

        // dbg!(ass);

        for t in &mut tokens {
            dbg!(t);
        }

        // let (i, _closer) = char(']')(i).map_err(TokenError::from_nom(TokenErrorKind::ClosingParen))?;

        tokens
            .finish()
            .map_err(TokenError::from_nom(TokenErrorKind::Other))?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ngc_grammar() {
        let input = "[ 10 + 20 / <local> + [<_global> * 14.0] ]";
        // let input = "[ 10 + 20.1 / #<local> ]";

        Expression::parse(input).unwrap();
    }
}
