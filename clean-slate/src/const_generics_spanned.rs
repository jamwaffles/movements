use crate::parse_modal_spans::Span;
use nom::{
    character::complete::{anychar, digit1, space0},
    combinator::{map_opt, verify},
    number::complete::float,
    sequence::{preceded, separated_pair},
    IResult, ParseTo,
};

pub fn recognise_word<const C: char, const N: u8>(i: Span) -> IResult<Span, ()> {
    let (i, _letter) = verify(anychar, |c| c.eq_ignore_ascii_case(&C))(i)?;

    let (i, _number): (_, u8) = verify(
        preceded(space0, map_opt(digit1, |d: Span| d.parse_to())),
        |n| *n == N,
    )(i)?;

    Ok((i, ()))
}

pub fn recognise_word_decimal<const C: char, const N: u8, const M: u8>(
    i: Span,
) -> IResult<Span, ()> {
    let (i, _) = separated_pair(
        recognise_word::<C, N>,
        space0,
        nom::character::complete::char('.'),
    )(i)?;

    let (i, _decimal): (_, u8) = preceded(
        space0,
        verify(map_opt(digit1, |d: Span| d.parse_to()), |n| *n == M),
    )(i)?;

    Ok((i, ()))
}

pub fn literal<const C: char>(i: Span) -> IResult<Span, f32> {
    let (i, _letter) = verify(anychar, |c| c.eq_ignore_ascii_case(&C))(i)?;

    preceded(space0, float)(i)
}

#[cfg(test)]
mod tests {
    use super::*;
    use nom::combinator::all_consuming;

    #[test]
    fn const_generics_int_word() {
        assert!(recognise_word::<'G', 0>("G0".into()).is_ok());
        assert!(recognise_word::<'G', 0>("G00".into()).is_ok());
        assert!(recognise_word::<'G', 0>("g00".into()).is_ok());
        assert!(recognise_word::<'G', 0>("g0".into()).is_ok());
        assert!(recognise_word::<'G', 0>("g01".into()).is_err());

        assert!(recognise_word::<'G', 4>("G04".into()).is_ok());
        assert!(recognise_word::<'G', 4>("G4".into()).is_ok());
    }

    #[test]
    fn const_generics_decimal_word() {
        assert!(recognise_word_decimal::<'G', 17, 1>("G17.1".into()).is_ok());
        assert!(recognise_word_decimal::<'G', 17, 1>("g17.1".into()).is_ok());
        assert!(recognise_word_decimal::<'G', 17, 1>("g17 . 1".into()).is_ok());
        assert!(recognise_word_decimal::<'G', 17, 1>("g   17   .    1".into()).is_ok());

        assert!(recognise_word_decimal::<'G', 17, 1>("M17.1".into()).is_err());
        assert!(recognise_word_decimal::<'G', 17, 1>("M17".into()).is_err());
        assert!(recognise_word_decimal::<'G', 17, 1>("M17.".into()).is_err());
        assert!(recognise_word_decimal::<'G', 17, 1>("M17.2".into()).is_err());
        assert!(recognise_word_decimal::<'G', 17, 1>("M17.1".into()).is_err());
    }

    #[test]
    fn literals() {
        assert_eq!(
            literal::<'P'>("p 0.005".into()),
            Ok((unsafe { Span::new_from_raw_offset(7, 1, "", ()) }, 0.005f32))
        );

        // Decimals with spaces in them are not supported.
        assert!(all_consuming(literal::<'P'>)("p 0 . 005".into()).is_err());
    }
}
