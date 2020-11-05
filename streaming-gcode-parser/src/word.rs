use crate::value::Value;
use nom::{
    branch::alt,
    bytes::streaming::tag,
    bytes::streaming::take_until,
    character::streaming::char,
    character::streaming::digit1,
    character::streaming::one_of,
    character::streaming::space0,
    character::streaming::{alpha1, anychar, multispace0},
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

pub fn parse_word<'a, W, R>(recognizer: R) -> impl FnMut(&'a str) -> IResult<&'a str, (W, Value)>
where
    R: FnMut(&'a str) -> IResult<&'a str, W> + FnOnce(&'a str) -> IResult<&'a str, W>,
{
    separated_pair(recognizer, space0, Value::parse)
}
