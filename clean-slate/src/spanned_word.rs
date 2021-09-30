//! Parse a couple of modal groups and collate them into a [`Word`] enum.
//!
//! Also contains the [`Spanned`] struct to wrap an item with its position in the input.

use crate::{
    const_generics_spanned::{literal, recognise_word},
    Span,
};
use core::time::Duration;
use nom::{
    branch::alt,
    bytes::complete::take_until,
    character::complete::anychar,
    combinator::map,
    error::{context, ContextError, ParseError},
    multi::many0,
    sequence::{delimited, preceded},
    IResult,
};
use nom::{character::complete::space0, sequence::separated_pair};
use nom_locate::position;

/// A parsed word with its input position.
#[derive(Debug, PartialEq)]
pub struct Spanned<'a, T> {
    pub start: Span<'a>,
    pub end: Span<'a>,
    pub item: T,
}

// fn spanned<'a, E, T>(
//     mut inner: impl FnMut(Span<'a>) -> IResult<Span<'a>, T>,
// ) -> impl FnMut(Span<'a>) -> IResult<Span, Spanned<'a, T>>
// where
//     E: ParseError<Span<'a>> + ContextError<Span<'a>>,
// {
//     move |i: Span<'a>| {
//         let (i, start) = position(i)?;

//         let (i, item) = inner(i)?;

//         let (i, end) = position(i)?;

//         Ok((i, Spanned { start, end, item }))
//     }
// }

#[derive(Debug)]
pub enum Word {
    /// Comment.
    Comment(Comment),

    // Modal groups.
    /// Group 0.
    NonModal(NonModal),

    /// Group 1.
    Motion(Motion),
}

// TODO: Trait?
impl Word {
    pub fn parse<'a, E>(i: Span<'a>) -> IResult<Span, Self, E>
    where
        E: ParseError<Span<'a>> + ContextError<Span<'a>>,
    {
        alt((
            context("comment", map(Comment::parse, Self::Comment)),
            map(Motion::parse::<'a, E>, Self::Motion),
            map(NonModal::parse::<'a, E>, Self::NonModal),
        ))(i)
    }
}

// TODO: Trait?
impl Word {
    pub fn parse_spanned<'a, E>(i: Span<'a>) -> IResult<Span, Spanned<Self>, E>
    where
        E: ParseError<Span<'a>> + ContextError<Span<'a>>,
    {
        // spanned(Self::parse::<'a, E>)(i)

        let (i, start) = position(i)?;

        let (i, item) = Self::parse::<'a, E>(i)?;

        let (i, end) = position(i)?;

        Ok((i, Spanned { start, end, item }))
    }
}

#[derive(Debug)]
pub enum CommentKind {
    Block,
    Inline,
}

#[derive(Debug)]
pub struct Comment {
    pub kind: CommentKind,
    pub comment: String,
}

// TODO: Trait?
impl Comment {
    pub fn parse<'a, E>(i: Span<'a>) -> IResult<Span, Self, E>
    where
        E: ParseError<Span<'a>> + ContextError<Span<'a>>,
    {
        alt((
            map(
                delimited(
                    nom::character::complete::char('('),
                    take_until(")"),
                    nom::character::complete::char(')'),
                ),
                |comment: Span| Comment {
                    kind: CommentKind::Inline,
                    comment: comment.fragment().to_string(),
                },
            ),
            map(
                preceded(nom::character::complete::char(';'), many0(anychar)),
                |comment: Vec<char>| {
                    let comment: String = comment.into_iter().collect();

                    Comment {
                        kind: CommentKind::Block,
                        comment,
                    }
                },
            ),
        ))(i)
    }
}

/// Group 1: Motion.
#[derive(Debug, PartialEq)]
pub enum Motion {
    // `G0`.
    Rapid,
}

// TODO: Trait?
impl Motion {
    pub fn parse<'a, E>(i: Span<'a>) -> IResult<Span, Self, E>
    where
        E: ParseError<Span<'a>> + ContextError<Span<'a>>,
    {
        map(recognise_word::<'a, E, 'G', 0>, |_| Self::Rapid)(i)
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
    pub fn parse<'a, E>(i: Span<'a>) -> IResult<Span, Self, E>
    where
        E: ParseError<Span<'a>> + ContextError<Span<'a>>,
    {
        context(
            "dwell",
            map(
                separated_pair(
                    recognise_word::<'a, E, 'G', 4>,
                    space0,
                    context("invalid duration", literal::<'a, E, 'P'>),
                ),
                |(_, duration)| Self::Dwell {
                    duration: Duration::from_secs_f32(duration),
                },
            ),
        )(i)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use nom::error::{ErrorKind, VerboseError};

    #[test]
    fn snapshot_motion() {
        insta::assert_debug_snapshot!(Motion::parse::<(_, ErrorKind)>("G0".into()));
    }

    #[test]
    fn snapshot_non_modal() {
        insta::assert_debug_snapshot!(NonModal::parse::<(_, ErrorKind)>("G4 P0.1".into()));
    }

    #[test]
    fn position() {
        insta::assert_debug_snapshot!(Word::parse::<(_, ErrorKind)>("G4 P0.1".into()));
    }

    #[test]
    fn position_missing_duration() {
        let out = Word::parse::<'_, VerboseError<Span>>("G0".into());

        // TODO: Proper assertions
        let out = out.unwrap_err();

        if let nom::Err::Error(nom::error::VerboseError { errors }) = out {
            dbg!(errors);
        }
    }

    #[test]
    fn block_comment() {
        insta::assert_debug_snapshot!(Comment::parse::<(_, ErrorKind)>("; absolute magic".into()));
    }

    #[test]
    fn inline_comment() {
        insta::assert_debug_snapshot!(Comment::parse::<(_, ErrorKind)>(
            "(inline comment) G0".into()
        ));
    }
}
