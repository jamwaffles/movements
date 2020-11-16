use crate::value::Value;
use crate::Span;
use nom::{character::complete::space0, sequence::separated_pair, IResult};

pub fn parse_word<'a, W, R>(recognizer: R) -> impl FnMut(Span<'a>) -> IResult<Span<'a>, (W, Value)>
where
    R: FnMut(Span<'a>) -> IResult<Span<'a>, W> + FnOnce(Span<'a>) -> IResult<Span<'a>, W>,
{
    separated_pair(recognizer, space0, Value::parse)
}
