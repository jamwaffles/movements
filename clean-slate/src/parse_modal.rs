use crate::{
    const_generics_spanned::{literal, recognise_word},
    parse_modal_spans::Span,
};
use core::time::Duration;
use nom::{character::complete::space0, combinator::map, sequence::separated_pair, IResult};

/// Group 1: Motion.
#[derive(Debug, PartialEq)]
pub enum Motion {
    // `G0`.
    Rapid,
}

// TODO: Trait?
impl Motion {
    pub fn parse(i: Span) -> IResult<Span, Self> {
        map(recognise_word::<'G', 0>, |_| Self::Rapid)(i)
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
    pub fn parse<'a>(i: Span) -> IResult<Span, Self> {
        map(
            separated_pair(recognise_word::<'G', 4>, space0, literal::<'P'>),
            |(_, duration)| Self::Dwell {
                duration: Duration::from_secs_f32(duration),
            },
        )(i)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn snapshot_motion() {
        insta::assert_debug_snapshot!(Motion::parse("G0".into()));
    }

    #[test]
    fn snapshot_non_modal() {
        insta::assert_debug_snapshot!(NonModal::parse("G4 P0.1".into()));
    }
}
