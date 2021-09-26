use crate::const_generics_spanned::{literal, recognise_word};
use core::time::Duration;
use nom::{character::complete::space0, combinator::map, sequence::separated_pair, IResult};
use nom_locate::{position, LocatedSpan};

pub type Span<'a> = LocatedSpan<&'a str>;

/// A parsed word with its input position.
#[derive(Debug, PartialEq)]
pub struct Spanned<'a, T> {
    pub start: Span<'a>,
    pub end: Span<'a>,
    pub item: T,
}

pub fn spanned<'a, T>(
    mut inner: impl FnMut(Span<'a>) -> IResult<Span<'a>, T>,
) -> impl FnMut(Span<'a>) -> IResult<Span, Spanned<'a, T>> {
    move |i: Span<'a>| {
        let (i, start) = position(i)?;

        let (i, item) = inner(i)?;

        let (i, end) = position(i)?;

        Ok((i, Spanned { start, end, item }))
    }
}

/// Modal groups.
#[derive(Debug, PartialEq)]
pub enum WordType {
    NonModal(NonModal),
    Motion(Motion),
}

/// Group 1: Motion.
#[derive(Debug, PartialEq)]
pub enum Motion {
    // `G0`.
    Rapid,
}

// TODO: Trait?
impl Motion {
    pub fn parse(i: Span) -> IResult<Span, Spanned<Self>> {
        spanned(map(recognise_word::<'G', 0>, |_| Self::Rapid))(i)
    }
}

/// Group 0: Non-modal.
#[derive(Debug, PartialEq)]
pub enum NonModal {
    // `G4 Pn`.
    Dwell { duration: Duration },
}

// TODO: Trait?
impl NonModal {
    pub fn parse<'a>(i: Span) -> IResult<Span, Spanned<Self>> {
        spanned(map(
            separated_pair(recognise_word::<'G', 4>, space0, literal::<'P'>),
            |(_, duration)| Self::Dwell {
                duration: Duration::from_secs_f32(duration),
            },
        ))(i)
    }
}

#[cfg(test)]
mod tests {
    #[allow(unused_imports)]
    use super::*;

    #[cfg(feature = "std")]
    #[test]
    fn snapshot_motion() {
        insta::assert_debug_snapshot!(Motion::parse("G0".into()));
    }

    #[cfg(feature = "std")]
    #[test]
    fn snapshot_non_modal() {
        insta::assert_debug_snapshot!(NonModal::parse("G4 P0.1".into()));
    }
}
