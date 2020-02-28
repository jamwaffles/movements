use crate::ParseInput;
use nom::branch::alt;
use nom::bytes::complete::is_not;
use nom::character::complete::char;
use nom::character::complete::not_line_ending;
use nom::combinator::map;
use nom::sequence::delimited;
use nom::sequence::preceded;
use nom::IResult;

#[derive(Debug, PartialEq)]
pub enum CommentType {
    /// A comment beginning with a `;` semicolon character, ending at a newline
    LineEnd,

    /// A comment delimited by parentheses
    Parens,
}

#[derive(Debug, PartialEq)]
pub struct Comment {
    text: String,

    comment_type: CommentType,
}

pub fn comment(i: ParseInput) -> IResult<ParseInput, Comment> {
    alt((
        map(
            delimited(char('('), is_not(")"), char(')')),
            |text: ParseInput| Comment {
                text: text.fragment().trim().to_string(),
                comment_type: CommentType::Parens,
            },
        ),
        map(preceded(char(';'), not_line_ending), |text: ParseInput| {
            Comment {
                text: text.fragment().trim().to_string(),
                comment_type: CommentType::LineEnd,
            }
        }),
    ))(i)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::rem;

    #[test]
    fn parens_comment() {
        assert_eq!(
            comment(ParseInput::new("( hello world )")),
            Ok((
                rem!("", 15),
                Comment {
                    text: "hello world".to_string(),
                    comment_type: CommentType::Parens
                }
            ))
        );
    }

    #[test]
    fn line_ending_comment() {
        assert_eq!(
            comment(ParseInput::new("; hello world")),
            Ok((
                rem!("", 13),
                Comment {
                    text: "hello world".to_string(),
                    comment_type: CommentType::LineEnd
                }
            ))
        );
    }
}
