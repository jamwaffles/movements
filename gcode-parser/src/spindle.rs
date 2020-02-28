//! Gcodes from [modal group 7 (spindle)](http://linuxcnc.org/docs/html/gcode/overview.html#_modal_groups)

use crate::word::word;
use nom::combinator::map_opt;
use nom::IResult;
use std::convert::{TryFrom, TryInto};

#[derive(Debug, PartialEq)]
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

pub fn spindle(i: &str) -> IResult<&str, Spindle> {
    map_opt(word::<u8, _>('M'), |word| word.value.try_into().ok())(i)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_spindle() {
        assert_eq!(spindle("M3"), Ok(("", Spindle::Forward)));
        assert_eq!(spindle("M4"), Ok(("", Spindle::Reverse)));
        assert_eq!(spindle("M5"), Ok(("", Spindle::Stop)));
    }

    #[test]
    fn leading_zeros() {
        assert_eq!(spindle("M03"), Ok(("", Spindle::Forward)));
        assert_eq!(spindle("M04"), Ok(("", Spindle::Reverse)));
        assert_eq!(spindle("M05"), Ok(("", Spindle::Stop)));
    }
}
