use crate::ParseInput;
use nom::character::complete::anychar;
use nom::character::complete::space0;
use nom::combinator::map;
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

pub fn word<'a, V, E>(
    search: char,
) -> impl Fn(ParseInput<'a>) -> IResult<ParseInput<'a>, Word<V>, E>
where
    E: ParseError<ParseInput<'a>>,
    V: FromStr,
{
    verify(
        map(
            separated_pair(
                anychar,
                space0,
                map_res(recognize_float, |s: ParseInput| s.fragment().parse::<V>()),
            ),
            |(letter, value)| Word {
                letter: letter.to_ascii_uppercase(),
                value,
            },
        ),
        // move |w| w.letter.to_ascii_uppercase() == search.to_ascii_uppercase(),
        move |w| w.letter.eq_ignore_ascii_case(&search),
    )
}

/// Recognise a word starting with a given character

#[cfg(test)]
mod tests {
    use super::*;
    use crate::rem;
    use nom::error::ErrorKind;
    use nom::Err::Error;

    #[test]
    fn canonical() {
        assert_eq!(
            word::<_, ()>('G')(ParseInput::new("G1")),
            Ok((rem!("", 2), Word::<u8>::new('G', 1u8)))
        );
    }

    #[test]
    fn spaces() {
        assert_eq!(
            word::<_, ()>('G')(ParseInput::new("G \t 1")),
            Ok((rem!("", 5), Word::<u8>::new('G', 1u8)))
        );
    }

    #[test]
    fn leading_zeros() {
        assert_eq!(
            word::<_, ()>('G')(ParseInput::new("G00")),
            Ok((rem!("", 3), Word::<u8>::new('G', 0u8)))
        );
        assert_eq!(
            word::<_, ()>('G')(ParseInput::new("G01")),
            Ok((rem!("", 3), Word::<u8>::new('G', 1u8)))
        );
    }

    #[test]
    fn float() {
        assert_eq!(
            word::<_, ()>('X')(ParseInput::new("X12.45")),
            Ok((rem!("", 6), Word::<f32>::new('X', 12.45f32)))
        );

        // Fail to parse to an integer due to trailing characters
        assert_eq!(
            word::<u8, _>('X')(ParseInput::new("X12.45")),
            Err(Error((rem!("12.45", 1), ErrorKind::MapRes)))
        );
    }

    #[test]
    fn non_matching() {
        assert_eq!(
            word::<u8, _>('X')(ParseInput::new("G1")),
            Err(Error((ParseInput::new("G1"), ErrorKind::Verify)))
        );
    }

    #[test]
    fn force_uppercase() {
        assert_eq!(
            word::<_, ()>('g')(ParseInput::new("g1")),
            Ok((rem!("", 2), Word::<u8>::new('G', 1u8)))
        );

        assert_eq!(
            word::<_, ()>('G')(ParseInput::new("g1")),
            Ok((rem!("", 2), Word::<u8>::new('G', 1u8)))
        );

        assert_eq!(
            word::<_, ()>('g')(ParseInput::new("G1")),
            Ok((rem!("", 2), Word::<u8>::new('G', 1u8)))
        );
    }
}
