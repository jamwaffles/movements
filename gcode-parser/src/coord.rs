use crate::word::word;
use nom::branch::alt;
use nom::character::complete::space0;
use nom::combinator::verify;
use nom::error::ParseError;
use nom::multi::fold_many_m_n;
use nom::sequence::terminated;
use nom::IResult;

#[derive(Debug, PartialEq, Default, Clone)]
pub struct Coord {
    x: Option<f32>,
    y: Option<f32>,
    z: Option<f32>,
    a: Option<f32>,
    b: Option<f32>,
    c: Option<f32>,
    u: Option<f32>,
    v: Option<f32>,
    w: Option<f32>,
}

impl Coord {
    /// Create a coord where all components are `Some`
    pub fn all(x: f32, y: f32, z: f32, a: f32, b: f32, c: f32, u: f32, v: f32, w: f32) -> Self {
        Self {
            x: Some(x),
            y: Some(y),
            z: Some(z),
            a: Some(a),
            b: Some(b),
            c: Some(c),
            u: Some(u),
            v: Some(v),
            w: Some(w),
        }
    }
}

/// Parse a coordinate
///
/// TODO: Fail when more than one of each axis is encountered
pub fn coord<'a, E>(i: &'a str) -> IResult<&'a str, Coord, E>
where
    E: ParseError<&'a str>,
{
    verify(
        fold_many_m_n(
            1,
            9,
            terminated(
                alt((
                    word::<f32, _>('X'),
                    word::<f32, _>('Y'),
                    word::<f32, _>('Z'),
                    word::<f32, _>('A'),
                    word::<f32, _>('B'),
                    word::<f32, _>('C'),
                    word::<f32, _>('U'),
                    word::<f32, _>('V'),
                    word::<f32, _>('W'),
                )),
                space0,
            ),
            Coord::default(),
            |mut carry, part| {
                match part.letter {
                    'X' => carry.x = Some(part.value),
                    'Y' => carry.y = Some(part.value),
                    'Z' => carry.z = Some(part.value),
                    'A' => carry.a = Some(part.value),
                    'B' => carry.b = Some(part.value),
                    'C' => carry.c = Some(part.value),
                    'U' => carry.u = Some(part.value),
                    'V' => carry.v = Some(part.value),
                    'W' => carry.w = Some(part.value),
                    c => panic!("Character '{}' is not a recognised axis letter", c),
                };
                carry
            },
        ),
        |coord| coord != &Coord::default(),
    )(i)
}

#[cfg(test)]
mod tests {
    use super::*;
    use nom::error::ErrorKind;
    use nom::Err::Error;

    #[test]
    fn empty() {
        assert_eq!(coord(""), Err(Error(("", ErrorKind::ManyMN))));
    }

    #[test]
    fn full() {
        assert_eq!(
            coord::<()>("X1 Y2 Z3 A4 B5 C6 U7 V8 W9"),
            Ok(("", Coord::all(1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0, 9.0)))
        );
    }

    #[test]
    fn partial() {
        assert_eq!(
            coord::<()>("X1 Y2 V8 W9"),
            Ok((
                "",
                Coord {
                    x: Some(1.0),
                    y: Some(2.0),
                    v: Some(8.0),
                    w: Some(9.0),
                    ..Coord::default()
                }
            ))
        );
    }

    #[test]
    fn random_order() {
        assert_eq!(
            coord::<()>("U7 C6 Y2 X1 A4 B5 Z3 W9 V8"),
            Ok(("", Coord::all(1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0, 9.0)))
        );
    }
}
