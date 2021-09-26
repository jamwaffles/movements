use crate::const_generics_test::{literal, recognise_word};
use core::time::Duration;
use nom::{
    character::complete::{anychar, digit1, space0},
    combinator::{map, map_opt, map_res, verify},
    number::complete::float,
    sequence::{preceded, separated_pair},
    IResult,
};
use nom_locate::LocatedSpan;

pub type Span<'a> = LocatedSpan<&'a str>;

/// A parsed word with its input position.
#[derive(Debug, PartialEq)]
pub struct Word<'a> {
    pub position: Span<'a>,
    pub word: ModalGroup,
}

/// Modal groups.
#[derive(Debug, PartialEq)]
pub enum ModalGroup {
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
    pub fn parse(i: &str) -> IResult<&str, Self> {
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
    pub fn parse(i: &str) -> IResult<&str, Self> {
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
    fn motion() {
        assert_eq!(Motion::parse("G0"), Ok(("", Motion::Rapid)));
    }

    #[test]
    fn non_modal() {
        assert_eq!(
            NonModal::parse("G4 P0.1"),
            Ok((
                "",
                NonModal::Dwell {
                    duration: Duration::from_millis(100)
                }
            ))
        );
    }
}
