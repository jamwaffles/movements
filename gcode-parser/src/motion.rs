//! Gcodes from [modal group 1 (motion)](http://linuxcnc.org/docs/html/gcode/overview.html#_modal_groups)

use crate::word::word;
use crate::ParseInput;
use nom::combinator::map_res;
use nom::IResult;
use std::convert::{TryFrom, TryInto};

#[derive(Debug, PartialEq, Copy, Clone)]
pub enum Motion {
    /// G0
    Rapid,

    /// G1
    Feed,
}

impl TryFrom<u8> for Motion {
    type Error = ();

    fn try_from(number: u8) -> Result<Self, Self::Error> {
        match number {
            0 => Ok(Motion::Rapid),
            1 => Ok(Motion::Feed),
            _ => Err(()),
        }
    }
}

pub fn motion(i: ParseInput) -> IResult<ParseInput, Motion> {
    map_res(word::<u8, _>('G'), |word| word.value.try_into())(i)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::rem;
    use nom::error::ErrorKind;
    use nom::Err::Error;

    #[test]
    fn rapid() {
        assert_eq!(
            motion(ParseInput::new("G0")),
            Ok((rem!("", 2), Motion::Rapid))
        );
        assert_eq!(
            motion(ParseInput::new("G00")),
            Ok((rem!("", 3), Motion::Rapid))
        );
    }

    #[test]
    fn feed() {
        assert_eq!(
            motion(ParseInput::new("G1")),
            Ok((rem!("", 2), Motion::Feed))
        );
        assert_eq!(
            motion(ParseInput::new("G01")),
            Ok((rem!("", 3), Motion::Feed))
        );
    }

    #[test]
    fn ignore_unknown() {
        assert_eq!(
            motion(ParseInput::new("G17")),
            Err(Error((rem!("G17", 0), ErrorKind::MapRes)))
        );
        assert_eq!(
            motion(ParseInput::new("G90")),
            Err(Error((rem!("G90", 0), ErrorKind::MapRes)))
        );
    }
}
