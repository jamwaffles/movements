//! Gcodes from [modal group 7 (spindle)](http://linuxcnc.org/docs/html/gcode/overview.html#_modal_groups)

use crate::word::word;
use crate::ParseInput;
use core::convert::{TryFrom, TryInto};
use nom::combinator::map_opt;
use nom::IResult;

#[derive(Debug, PartialEq, Clone)]
pub enum Spindle {
    /// M3
    Forward,

    /// M4
    Reverse,

    /// M5
    Stop,
}

impl TryFrom<u8> for Spindle {
    type Error = ();

    fn try_from(number: u8) -> Result<Self, Self::Error> {
        match number {
            3 => Ok(Spindle::Forward),
            4 => Ok(Spindle::Reverse),
            5 => Ok(Spindle::Stop),
            _ => Err(()),
        }
    }
}

pub fn spindle(i: ParseInput) -> IResult<ParseInput, Spindle> {
    map_opt(word::<u8, _>('M'), |word| word.value.try_into().ok())(i)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::rem;

    #[test]
    fn parse_spindle() {
        assert_eq!(
            spindle(ParseInput::new("M3")),
            Ok((rem!("", 2), Spindle::Forward))
        );
        assert_eq!(
            spindle(ParseInput::new("M4")),
            Ok((rem!("", 2), Spindle::Reverse))
        );
        assert_eq!(
            spindle(ParseInput::new("M5")),
            Ok((rem!("", 2), Spindle::Stop))
        );
    }

    #[test]
    fn leading_zeros() {
        assert_eq!(
            spindle(ParseInput::new("M03")),
            Ok((rem!("", 3), Spindle::Forward))
        );
        assert_eq!(
            spindle(ParseInput::new("M04")),
            Ok((rem!("", 3), Spindle::Reverse))
        );
        assert_eq!(
            spindle(ParseInput::new("M05")),
            Ok((rem!("", 3), Spindle::Stop))
        );
    }
}
