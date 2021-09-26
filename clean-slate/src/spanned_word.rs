use std::convert::TryFrom;

use crate::{
    parse_modal::{Motion, NonModal},
    parse_modal_spans::{spanned, Span, Spanned},
};
use nom::{
    branch::alt,
    bytes::complete::take_until,
    character::complete::anychar,
    combinator::{eof, map, map_res},
    multi::many0,
    sequence::{delimited, preceded},
    IResult,
};

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
    pub fn parse(i: Span) -> IResult<Span, Spanned<Self>> {
        spanned(alt((
            map(Comment::parse, Self::Comment),
            map(Motion::parse, Self::Motion),
            map(NonModal::parse, Self::NonModal),
        )))(i)
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
    pub fn parse(i: Span) -> IResult<Span, Self> {
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn position() {
        insta::assert_debug_snapshot!(Word::parse("G4 P0.1".into()));
    }

    #[test]
    fn block_comment() {
        insta::assert_debug_snapshot!(Comment::parse("; absolute magic".into()));
    }

    #[test]
    fn inline_comment() {
        insta::assert_debug_snapshot!(Comment::parse("(inline comment) G0".into()));
    }
}
