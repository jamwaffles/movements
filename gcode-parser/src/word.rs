use nom::character::complete::alpha1;
use nom::character::complete::space0;
use nom::combinator::map_res;
use nom::number::complete::recognize_float;
use nom::sequence::preceded;
use nom::IResult;
use std::str::FromStr;

/// A gcode word
///
/// A word consists of a letter and a number like `G1`, `M199` or `T6`. The numberic part has different meanings depending on letter (and sometimes context).
#[derive(Debug, PartialEq)]
pub struct Word<V> {
    /// Single code letter, uppercased by the [`word`] parser
    letter: char,

    /// Value or identifier after letter
    value: V,
}

impl<V> Word<V> {
    /// Create a new `Word` from a letter and given value type
    pub fn new(letter: char, value: V) -> Self {
        Self { letter, value }
    }
}

/// Parse a word
pub fn word<'a, V, E>(i: &str) -> IResult<&str, Word<V>>
where
    V: FromStr + Default,
{
    let (i, letter_str) = alpha1(i)?;

    let letter = letter_str
        .chars()
        .next()
        // alpha1() collects ascii only, so this is safe to do
        .map(|c| c.to_ascii_uppercase())
        // alpha1() will fail before the unwrap() here does
        .unwrap();

    let (i, value): (&str, V) =
        map_res(preceded(space0, recognize_float), |s: &str| s.parse::<V>())(i)?;

    Ok((i, Word { letter, value }))
}

#[cfg(test)]
mod tests {
    use super::*;
    use nom::error::ErrorKind;
    use nom::Err::Error;

    #[test]
    fn canonical() {
        assert_eq!(word::<_, ()>("G1"), Ok(("", Word::<u8>::new('G', 1u8))));
    }

    #[test]
    fn spaces() {
        assert_eq!(word::<_, ()>("G \t 1"), Ok(("", Word::<u8>::new('G', 1u8))));
    }

    #[test]
    fn leading_zeros() {
        assert_eq!(word::<_, ()>("G00"), Ok(("", Word::<u8>::new('G', 0u8))));
        assert_eq!(word::<_, ()>("G01"), Ok(("", Word::<u8>::new('G', 1u8))));
    }

    #[test]
    fn float() {
        assert_eq!(
            word::<_, ()>("X12.45"),
            Ok(("", Word::<f32>::new('X', 12.45f32)))
        );

        // Fail to parse to an integer due to trailing characters
        assert_eq!(
            word::<u8, ()>("X12.45"),
            Err(Error(("12.45", ErrorKind::MapRes)))
        );
    }
}
