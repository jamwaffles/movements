use nom::{
    character::complete::{anychar, digit1, space0},
    combinator::{map_opt, verify},
    number::complete::float,
    sequence::{preceded, separated_pair},
    IResult, ParseTo,
};

pub fn recognise_word<const C: char, const N: u8>(i: &str) -> IResult<&str, ()> {
    let (i, _letter) = verify(anychar, |c| c.eq_ignore_ascii_case(&C))(i)?;

    let (i, _number): (_, u8) = verify(
        preceded(space0, map_opt(digit1, |d: &str| d.parse_to())),
        |n| *n == N,
    )(i)?;

    Ok((i, ()))
}

pub fn recognise_word_decimal<const C: char, const N: u8, const M: u8>(
    i: &str,
) -> IResult<&str, ()> {
    let (i, _) = separated_pair(
        recognise_word::<C, N>,
        space0,
        nom::character::complete::char('.'),
    )(i)?;

    let (i, _decimal): (_, u8) = preceded(
        space0,
        verify(map_opt(digit1, |d: &str| d.parse_to()), |n| *n == M),
    )(i)?;

    Ok((i, ()))
}

pub fn literal<const C: char>(i: &str) -> IResult<&str, f32> {
    let (i, _letter) = verify(anychar, |c| c.eq_ignore_ascii_case(&C))(i)?;

    preceded(space0, float)(i)
}

#[cfg(test)]
mod tests {
    use super::*;
    use nom::combinator::all_consuming;

    #[test]
    fn const_generics_int_word() {
        assert!(recognise_word::<'G', 0>("G0").is_ok());
        assert!(recognise_word::<'G', 0>("G00").is_ok());
        assert!(recognise_word::<'G', 0>("g00").is_ok());
        assert!(recognise_word::<'G', 0>("g0").is_ok());
        assert!(recognise_word::<'G', 0>("g01").is_err());

        assert!(recognise_word::<'G', 4>("G04").is_ok());
        assert!(recognise_word::<'G', 4>("G4").is_ok());
    }

    #[test]
    fn const_generics_decimal_word() {
        assert!(recognise_word_decimal::<'G', 17, 1>("G17.1").is_ok());
        assert!(recognise_word_decimal::<'G', 17, 1>("g17.1").is_ok());
        assert!(recognise_word_decimal::<'G', 17, 1>("g17 . 1").is_ok());
        assert!(recognise_word_decimal::<'G', 17, 1>("g   17   .    1").is_ok());

        assert!(recognise_word_decimal::<'G', 17, 1>("M17.1").is_err());
        assert!(recognise_word_decimal::<'G', 17, 1>("M17").is_err());
        assert!(recognise_word_decimal::<'G', 17, 1>("M17.").is_err());
        assert!(recognise_word_decimal::<'G', 17, 1>("M17.2").is_err());
        assert!(recognise_word_decimal::<'G', 17, 1>("M17.1").is_err());
    }

    #[test]
    fn literals() {
        assert_eq!(literal::<'P'>("p 0.005"), Ok(("", 0.005f32)));

        // Decimals with spaces in them are not supported.
        assert!(all_consuming(literal::<'P'>)("p 0 . 005").is_err());
    }
}
