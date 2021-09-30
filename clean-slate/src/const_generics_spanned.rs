//! Functions to parse words and literals using `Span`s.

use crate::Span;
use nom::{
    character::complete::{digit1, satisfy, space0},
    combinator::{map_opt, verify},
    error::ParseError,
    number::complete::float,
    sequence::{preceded, separated_pair},
    IResult, ParseTo,
};

// pub fn parse<'a, E: ParseError<Span<'a>>>(i: Span<'a>) -> IResult<Span<'a>, Self, E> {

pub fn recognise_word<'a, E, const C: char, const N: u8>(i: Span<'a>) -> IResult<Span, (), E>
where
    E: ParseError<Span<'a>>,
{
    let (i, _letter) = satisfy(|c| c.eq_ignore_ascii_case(&C))(i)?;

    let (i, _number): (_, u8) = verify(
        preceded(space0, map_opt(digit1, |d: Span| d.parse_to())),
        |n| *n == N,
    )(i)?;

    Ok((i, ()))
}

pub fn recognise_word_decimal<'a, E, const C: char, const N: u8, const M: u8>(
    i: Span<'a>,
) -> IResult<Span, (), E>
where
    E: ParseError<Span<'a>>,
{
    let (i, _) = separated_pair(
        recognise_word::<'_, E, C, N>,
        space0,
        nom::character::complete::char('.'),
    )(i)?;

    let (i, _decimal): (_, u8) = preceded(
        space0,
        verify(map_opt(digit1, |d: Span| d.parse_to()), |n| *n == M),
    )(i)?;

    Ok((i, ()))
}

pub fn literal<'a, E, const C: char>(i: Span<'a>) -> IResult<Span, f32, E>
where
    E: ParseError<Span<'a>>,
{
    let (i, _letter) = satisfy(|c| c.eq_ignore_ascii_case(&C))(i)?;

    preceded(space0, float)(i)
}

#[cfg(test)]
mod tests {
    use super::*;
    use nom::{combinator::all_consuming, error::ErrorKind};

    #[test]
    fn const_generics_int_word() {
        assert!(recognise_word::<(_, ErrorKind), 'G', 0>("G0".into()).is_ok());
        assert!(recognise_word::<(_, ErrorKind), 'G', 0>("G00".into()).is_ok());
        assert!(recognise_word::<(_, ErrorKind), 'G', 0>("g00".into()).is_ok());
        assert!(recognise_word::<(_, ErrorKind), 'G', 0>("g0".into()).is_ok());
        assert!(recognise_word::<(_, ErrorKind), 'G', 0>("g01".into()).is_err());

        assert!(recognise_word::<(_, ErrorKind), 'G', 4>("G04".into()).is_ok());
        assert!(recognise_word::<(_, ErrorKind), 'G', 4>("G4".into()).is_ok());
    }

    #[test]
    fn const_generics_decimal_word() {
        assert!(recognise_word_decimal::<(_, ErrorKind), 'G', 17, 1>("G17.1".into()).is_ok());
        assert!(recognise_word_decimal::<(_, ErrorKind), 'G', 17, 1>("g17.1".into()).is_ok());
        assert!(recognise_word_decimal::<(_, ErrorKind), 'G', 17, 1>("g17 . 1".into()).is_ok());
        assert!(
            recognise_word_decimal::<(_, ErrorKind), 'G', 17, 1>("g   17   .    1".into()).is_ok()
        );

        assert!(recognise_word_decimal::<(_, ErrorKind), 'G', 17, 1>("M17.1".into()).is_err());
        assert!(recognise_word_decimal::<(_, ErrorKind), 'G', 17, 1>("M17".into()).is_err());
        assert!(recognise_word_decimal::<(_, ErrorKind), 'G', 17, 1>("M17.".into()).is_err());
        assert!(recognise_word_decimal::<(_, ErrorKind), 'G', 17, 1>("M17.2".into()).is_err());
        assert!(recognise_word_decimal::<(_, ErrorKind), 'G', 17, 1>("M17.1".into()).is_err());
    }

    #[test]
    fn literals() {
        assert_eq!(
            literal::<(_, ErrorKind), 'P'>("p 0.005".into()),
            Ok((unsafe { Span::new_from_raw_offset(7, 1, "", ()) }, 0.005f32))
        );

        // Decimals with spaces in them are not supported.
        assert!(all_consuming(literal::<(_, ErrorKind), 'P'>)("p 0 . 005".into()).is_err());
    }
}
