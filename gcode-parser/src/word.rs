use nom::character::complete::anychar;
use nom::character::complete::space0;
use nom::combinator::map_res;
use nom::combinator::verify;
use nom::error::ParseError;
use nom::number::complete::recognize_float;
use nom::sequence::separated_pair;
use nom::IResult;
use std::fmt;
use std::str::FromStr;

/// A gcode word
///
/// A word consists of a letter and a number like `G1`, `M199` or `T6`. The numberic part has different meanings depending on letter (and sometimes context).
#[derive(Debug, PartialEq)]
pub struct Word<V> {
    /// Single code letter, uppercased by the [`word`] parser
    pub letter: char,

    /// Value or identifier after letter
    pub value: V,
}

impl<V> Word<V> {
    /// Create a new `Word` from a letter and given value type
    pub fn new(letter: char, value: V) -> Self {
        Self { letter, value }
    }
}

impl<V> fmt::Display for Word<V>
where
    V: fmt::Display,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}{}", self.letter, self.value)
    }
}

/// Parse a word
pub fn word<'a, V, E>(search: char) -> impl Fn(&'a str) -> IResult<&'a str, Word<V>, E>
where
    E: ParseError<&'a str>,
    V: FromStr + Default,
{
    verify(
        map_res::<_, _, _, _, E, _, _>(
            separated_pair(
                anychar,
                space0,
                map_res(recognize_float, |s: &str| s.parse::<V>()),
            ),
            |(letter, value)| Ok(Word { letter, value }),
        ),
        // move |w| w.letter.to_ascii_uppercase() == search.to_ascii_uppercase(),
        move |w| w.letter.eq_ignore_ascii_case(&search),
    )
}

/// Recognise a word starting with a given character

#[cfg(test)]
mod tests {
    use super::*;
    use nom::error::ErrorKind;
    use nom::Err::Error;

    #[test]
    fn canonical() {
        assert_eq!(
            word::<_, ()>('G')("G1"),
            Ok(("", Word::<u8>::new('G', 1u8)))
        );
    }

    #[test]
    fn spaces() {
        assert_eq!(
            word::<_, ()>('G')("G \t 1"),
            Ok(("", Word::<u8>::new('G', 1u8)))
        );
    }

    #[test]
    fn leading_zeros() {
        assert_eq!(
            word::<_, ()>('G')("G00"),
            Ok(("", Word::<u8>::new('G', 0u8)))
        );
        assert_eq!(
            word::<_, ()>('G')("G01"),
            Ok(("", Word::<u8>::new('G', 1u8)))
        );
    }

    #[test]
    fn float() {
        assert_eq!(
            word::<_, ()>('X')("X12.45"),
            Ok(("", Word::<f32>::new('X', 12.45f32)))
        );

        // Fail to parse to an integer due to trailing characters
        assert_eq!(
            word::<u8, _>('X')("X12.45"),
            Err(Error(("12.45", ErrorKind::MapRes)))
        );
    }

    #[test]
    fn non_matching() {
        assert_eq!(
            word::<u8, _>('X')("G1"),
            Err(Error(("G1", ErrorKind::Verify)))
        );
    }
}
