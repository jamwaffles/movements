use crate::word::word;
use crate::ParseInput;
use nom::branch::alt;
use nom::character::complete::space0;
use nom::error::ParseError;
use nom::sequence::terminated;
use nom::{error::ErrorKind, Err, IResult};

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

impl Coord {
    pub fn with_x(x: f32) -> Self {
        Self {
            x: Some(x),
            ..Self::default()
        }
    }
}

/// Parse a coordinate
pub fn coord<'a, E>(i: ParseInput<'a>) -> IResult<ParseInput<'a>, Coord, E>
where
    E: ParseError<ParseInput<'a>>,
{
    let parser = terminated::<_, _, _, E, _, _>(
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
    );

    let mut i = i.clone();
    let mut matched_count = 0;
    let mut coord = Coord::default();

    loop {
        let res = parser(i.clone());

        match res {
            Ok((i1, part)) => {
                // Nothing was consumed, we're done
                if i == i1 {
                    return Ok((i1, coord));
                }

                let axis = match part.letter {
                    'X' => &mut coord.x,
                    'Y' => &mut coord.y,
                    'Z' => &mut coord.z,
                    'A' => &mut coord.a,
                    'B' => &mut coord.b,
                    'C' => &mut coord.c,
                    'U' => &mut coord.u,
                    'V' => &mut coord.v,
                    'W' => &mut coord.w,
                    c => panic!("Character '{}' is not a recognised axis letter", c),
                };

                // If we've already parsed this axis, we should complete and return, allowing the
                // next coord to be parsed.
                if axis.is_some() {
                    return Ok((i, coord));
                } else {
                    *axis = Some(part.value);
                    matched_count += 1;
                }

                // Remove the parsed characters from the input
                i = i1;
            }
            Err(Err::Error(e)) => {
                if matched_count == 0 {
                    return Err(Err::Error(E::append(i, ErrorKind::ManyMN, e)));
                } else {
                    return Ok((i, coord));
                }
            }
            Err(e) => {
                return Err(e);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::rem;
    use nom::error::ErrorKind;
    use nom::multi::many1;
    use nom::Err::Error;

    #[test]
    fn empty() {
        assert_eq!(
            coord(ParseInput::new("")),
            Err(Error((rem!(""), ErrorKind::Eof)))
        );
    }

    #[test]
    fn full() {
        assert_eq!(
            coord::<()>(ParseInput::new("X1 Y2 Z3 A4 B5 C6 U7 V8 W9")),
            Ok((
                rem!("", 26),
                Coord::all(1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0, 9.0)
            ))
        );
    }

    #[test]
    fn partial() {
        assert_eq!(
            coord::<()>(ParseInput::new("X1 Y2 V8 W9")),
            Ok((
                rem!("", 11),
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
            coord::<()>(ParseInput::new("U7 C6 Y2 X1 A4 B5 Z3 W9 V8")),
            Ok((
                rem!("", 26),
                Coord::all(1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0, 9.0)
            ))
        );
    }

    #[test]
    fn no_repeats() {
        assert_eq!(
            coord::<()>(ParseInput::new("X1 Y2 X3")),
            Ok((
                rem!("X3", 6),
                Coord {
                    x: Some(1.0),
                    y: Some(2.0),
                    ..Coord::default()
                }
            ))
        );
    }

    #[test]
    fn multi() {
        assert_eq!(
            many1::<_, _, (), _>(coord)(ParseInput::new("X1 Y2 X3")),
            Ok((
                rem!("", 8),
                vec![
                    Coord {
                        x: Some(1.0),
                        y: Some(2.0),
                        ..Coord::default()
                    },
                    Coord {
                        x: Some(3.0),
                        ..Coord::default()
                    },
                ]
            ))
        );
    }
}
