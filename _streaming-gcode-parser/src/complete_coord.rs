use crate::{value::Value, word::parse_word};
use core::ops::{Index, IndexMut};
use nom::{
    branch::alt,
    bytes::complete::tag,
    bytes::complete::take_until,
    character::complete::char,
    character::complete::digit1,
    character::complete::one_of,
    character::complete::space0,
    character::complete::{alpha1, anychar, multispace0},
    combinator::map,
    combinator::map_opt,
    combinator::map_res,
    combinator::peek,
    combinator::{cond, verify},
    multi::many0,
    multi::many_m_n,
    multi::separated_list0,
    sequence::delimited,
    sequence::preceded,
    sequence::{separated_pair, terminated},
    IResult,
};

#[derive(Debug, Clone, PartialEq, Default)]
pub struct Coord {
    pub x: Option<Value>,
    pub y: Option<Value>,
    pub z: Option<Value>,
    pub a: Option<Value>,
    pub b: Option<Value>,
    pub c: Option<Value>,
    pub u: Option<Value>,
    pub v: Option<Value>,
    pub w: Option<Value>,
}

impl Index<char> for Coord {
    type Output = Option<Value>;

    fn index(&self, index: char) -> &Self::Output {
        match index {
            'x' => &self.x,
            'y' => &self.y,
            'z' => &self.z,
            'a' => &self.a,
            'b' => &self.b,
            'c' => &self.c,
            'u' => &self.u,
            'v' => &self.v,
            'w' => &self.w,
            bad => panic!("Index {} not recognised", bad),
        }
    }
}

impl IndexMut<char> for Coord {
    fn index_mut(&mut self, index: char) -> &mut Self::Output {
        match index {
            'x' => &mut self.x,
            'y' => &mut self.y,
            'z' => &mut self.z,
            'a' => &mut self.a,
            'b' => &mut self.b,
            'c' => &mut self.c,
            'u' => &mut self.u,
            'v' => &mut self.v,
            'w' => &mut self.w,
            bad => panic!("Index {} not recognised", bad),
        }
    }
}

impl Coord {
    pub fn parse<'a>(i: &'a str) -> IResult<&'a str, Self> {
        let mut coord = Coord::default();

        // PERF: Benchmark against HashSet, BTreeSet and normal array.
        let mut remaining = String::from("xyzabcuvw");
        let mut i = i;

        for _ in 0..9 {
            let result = preceded(
                space0,
                parse_word(verify(map(anychar, |c| c.to_ascii_lowercase()), |c| {
                    remaining.contains(*c)
                })),
            )(i);

            match result {
                Ok((_i, (axis, value))) => {
                    remaining = remaining.chars().filter(|c| *c != axis).collect();

                    coord[axis] = Some(value);

                    i = _i;
                }
                Err(nom::Err::Error(e)) => {
                    if coord == Coord::default() {
                        return Err(nom::Err::Error(nom::error::ParseError::append(
                            i,
                            nom::error::ErrorKind::ManyMN,
                            e,
                        )));
                    } else {
                        break;
                    }
                }
                Err(e) => {
                    return Err(e);
                }
            }
        }

        Ok((i, coord))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn stop_at_dupe() {
        let expected = Coord {
            x: Some(Value::Literal(10.0)),
            y: Some(Value::Literal(20.0)),
            ..Coord::default()
        };

        assert_eq!(Coord::parse("X10 Y20 X15;"), Ok((" X15;", expected)));
    }

    #[test]
    fn random_order() {
        let expected = Coord {
            x: Some(Value::Literal(9.0)),
            y: Some(Value::Literal(10.0)),
            u: Some(Value::Literal(5.0)),
            c: Some(Value::Literal(4.0)),
            ..Coord::default()
        };

        assert_eq!(Coord::parse("U5 Y10 X9 C4;"), Ok((";", expected)));
    }
}
