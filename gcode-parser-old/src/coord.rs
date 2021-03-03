use crate::Axes;
use crate::ParseInput;
use core::ops::Index;
use nom::character::complete::anychar;
use nom::character::complete::space0;
use nom::error::ParseError;
use nom::number::complete::float;
use nom::sequence::preceded;
use nom::sequence::separated_pair;
use nom::{error::ErrorKind, Err, IResult};

#[derive(Debug, PartialEq, Default, Clone)]
pub struct Coord {
    pub x: Option<f32>,
    pub y: Option<f32>,
    pub z: Option<f32>,
    pub a: Option<f32>,
    pub b: Option<f32>,
    pub c: Option<f32>,
    pub u: Option<f32>,
    pub v: Option<f32>,
    pub w: Option<f32>,
}

impl Index<usize> for Coord {
    type Output = Option<f32>;

    fn index(&self, idx: usize) -> &Self::Output {
        match idx {
            0 => &self.x,
            1 => &self.y,
            2 => &self.z,
            3 => &self.a,
            4 => &self.b,
            5 => &self.c,
            6 => &self.u,
            7 => &self.v,
            8 => &self.w,
            i => panic!(
                "Index {} is out of of bounds for coodinate. Must be in range 0-8.",
                i
            ),
        }
    }
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

    /// Convert the coordinate into a Nalgebra object
    ///
    /// Any components that are `None` will be set to `0.0`
    pub fn into_axes(self) -> Axes {
        Axes::from_fn(|i, _| self[i].unwrap_or(0.0))
    }
}

impl Coord {
    pub fn with_x(x: f32) -> Self {
        Self {
            x: Some(x),
            ..Self::default()
        }
    }

    pub fn with_xy(x: f32, y: f32) -> Self {
        Self {
            x: Some(x),
            y: Some(y),
            ..Self::default()
        }
    }
}

/// Parse a coordinate
pub fn coord<'a, E>(i: ParseInput<'a>) -> IResult<ParseInput<'a>, Coord, E>
where
    E: ParseError<ParseInput<'a>>,
{
    let mut c = Coord::default();
    let mut input = i;

    for _ in 0..9 {
        let res = preceded(space0, separated_pair(anychar, space0, float))(input);

        match res {
            Ok((i, (ch, value))) => {
                match ch.to_ascii_lowercase() {
                    'x' if c.x.is_none() => {
                        c.x = Some(value);
                        input = i;
                    }
                    'y' if c.y.is_none() => {
                        c.y = Some(value);
                        input = i;
                    }
                    'z' if c.z.is_none() => {
                        c.z = Some(value);
                        input = i;
                    }
                    //
                    'a' if c.a.is_none() => {
                        c.a = Some(value);
                        input = i;
                    }
                    'b' if c.b.is_none() => {
                        c.b = Some(value);
                        input = i;
                    }
                    'c' if c.c.is_none() => {
                        c.c = Some(value);
                        input = i;
                    }
                    //
                    'u' if c.u.is_none() => {
                        c.u = Some(value);
                        input = i;
                    }
                    'v' if c.v.is_none() => {
                        c.v = Some(value);
                        input = i;
                    }
                    'w' if c.w.is_none() => {
                        c.w = Some(value);
                        input = i;
                    }
                    // ---
                    'x' if c.x.is_some() => break,
                    'y' if c.y.is_some() => break,
                    'z' if c.z.is_some() => break,
                    //
                    'a' if c.a.is_some() => break,
                    'b' if c.b.is_some() => break,
                    'c' if c.c.is_some() => break,
                    //
                    'u' if c.u.is_some() => break,
                    'v' if c.v.is_some() => break,
                    'w' if c.w.is_some() => break,

                    _ => (),
                }
            }
            Err(Err::Error(e)) => {
                if c == Coord::default() {
                    return Err(Err::Error(E::append(input, ErrorKind::ManyMN, e)));
                } else {
                    return Ok((input, c));
                }
            }
            Err(e) => {
                return Err(e);
            }
        }
    }

    if c != Coord::default() {
        Ok((input, c))
    } else {
        Err(Err::Error(E::from_error_kind(input, ErrorKind::ManyMN)))
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
                rem!(" X3", 5),
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
