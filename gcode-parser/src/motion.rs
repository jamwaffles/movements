//! Gcodes from [modal group 1 (motion)](http://linuxcnc.org/docs/html/gcode/overview.html#_modal_groups)

use crate::word::word;
use nom::combinator::map_opt;
use nom::IResult;
use std::convert::{TryFrom, TryInto};

#[derive(Debug, PartialEq)]
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

pub fn motion(i: &str) -> IResult<&str, Motion> {
    map_opt(word::<u8, _>('G'), |word| word.value.try_into().ok())(i)
}

#[cfg(test)]
mod tests {
    use super::*;
    use nom::error::ErrorKind;
    use nom::Err::Error;

    #[test]
    fn rapid() {
        assert_eq!(motion("G0"), Ok(("", Motion::Rapid)));
        assert_eq!(motion("G00"), Ok(("", Motion::Rapid)));
    }

    #[test]
    fn feed() {
        assert_eq!(motion("G1"), Ok(("", Motion::Feed)));
        assert_eq!(motion("G01"), Ok(("", Motion::Feed)));
    }

    #[test]
    fn ignore_unknown() {
        assert_eq!(motion("G17"), Err(Error(("G17", ErrorKind::MapOpt))));
        assert_eq!(motion("G90"), Err(Error(("G90", ErrorKind::MapOpt))));
    }
}
