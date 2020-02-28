//! Gcodes from [modal group 0 (non modal codes)](http://linuxcnc.org/docs/html/gcode/overview.html#_modal_groups)

use crate::word::word;
use crate::ParseInput;
use nom::character::complete::space0;
use nom::combinator::map;
use nom::combinator::verify;
use nom::sequence::separated_pair;
use nom::IResult;

#[derive(Debug, PartialEq)]
pub enum NonModal {
    /// G4
    Dwell { duration: f32 },
}

pub fn motion(i: ParseInput) -> IResult<ParseInput, NonModal> {
    map(
        separated_pair(
            verify(word::<u8, _>('G'), |w| w.value == 4),
            space0,
            verify(word::<f32, _>('P'), |w| w.value.is_sign_positive()),
        ),
        |(_g4, duration)| NonModal::Dwell {
            duration: duration.value,
        },
    )(i)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::rem;
    use nom::error::ErrorKind;
    use nom::Err::Error;

    #[test]
    fn float_dwell() {
        assert_eq!(
            motion(ParseInput::new("G4 P12.34")),
            Ok((rem!("", 9), NonModal::Dwell { duration: 12.34 }))
        );
    }

    #[test]
    fn zero_seconds() {
        assert_eq!(
            motion(ParseInput::new("G4 P0")),
            Ok((rem!("", 5), NonModal::Dwell { duration: 0.0 }))
        );
    }

    #[test]
    fn requires_p_word() {
        assert_eq!(
            motion(ParseInput::new("G4")),
            Err(Error((rem!("", 2), ErrorKind::Eof)))
        );
    }

    #[test]
    fn negative() {
        assert_eq!(
            motion(ParseInput::new("G4 P-1.0")),
            Err(Error((rem!("P-1.0", 3), ErrorKind::Verify)))
        );
    }
}