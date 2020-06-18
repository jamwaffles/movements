//! Gcodes from [modal group 0 (non modal codes)](http://linuxcnc.org/docs/html/gcode/overview.html#_modal_groups)

use crate::coord::{coord, Coord};
use crate::word::word;
use crate::ParseInput;
use nom::branch::alt;
use nom::character::complete::space0;
use nom::combinator::map;
use nom::combinator::verify;
use nom::sequence::separated_pair;
use nom::IResult;

#[derive(Debug, PartialEq, Clone)]
pub enum NonModal {
    /// G4
    Dwell { duration: f32 },

    /// G92
    CoordinateSystemOffset { position: Coord },
}

fn dwell(i: ParseInput) -> IResult<ParseInput, NonModal> {
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

fn coord_system_offset(i: ParseInput) -> IResult<ParseInput, NonModal> {
    map(
        separated_pair(verify(word::<u8, _>('G'), |w| w.value == 92), space0, coord),
        |(_g92, coord)| NonModal::CoordinateSystemOffset { position: coord },
    )(i)
}

pub fn non_modal(i: ParseInput) -> IResult<ParseInput, NonModal> {
    alt((coord_system_offset, dwell))(i)
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
            non_modal(ParseInput::new("G4 P12.34")),
            Ok((rem!("", 9), NonModal::Dwell { duration: 12.34 }))
        );
    }

    #[test]
    fn zero_seconds() {
        assert_eq!(
            non_modal(ParseInput::new("G4 P0")),
            Ok((rem!("", 5), NonModal::Dwell { duration: 0.0 }))
        );
    }

    #[test]
    fn requires_p_word() {
        assert_eq!(
            non_modal(ParseInput::new("G4")),
            Err(Error((rem!("", 2), ErrorKind::Eof)))
        );
    }

    #[test]
    fn negative() {
        assert_eq!(
            non_modal(ParseInput::new("G4 P-1.0")),
            Err(Error((rem!("P-1.0", 3), ErrorKind::Verify)))
        );
    }

    #[test]
    fn g92_correct() {
        assert_eq!(
            non_modal(ParseInput::new("G92 Y0.326 X0.000")),
            Ok((
                rem!("", 17),
                NonModal::CoordinateSystemOffset {
                    position: Coord::with_xy(0.0, 0.326)
                }
            ))
        );
    }
}
