//! Gcodes from [modal group 6 (units)](http://linuxcnc.org/docs/html/gcode/overview.html#_modal_groups)

use crate::word::word;
use crate::ParseInput;
use nom::combinator::map_opt;
use nom::IResult;
use std::convert::{TryFrom, TryInto};

#[derive(Debug, PartialEq)]
pub enum Units {
    /// G20
    Inch,

    /// G21
    Mm,
}

impl TryFrom<u8> for Units {
    type Error = ();

    fn try_from(number: u8) -> Result<Self, Self::Error> {
        match number {
            20 => Ok(Units::Inch),
            21 => Ok(Units::Mm),
            _ => Err(()),
        }
    }
}

pub fn units(i: ParseInput) -> IResult<ParseInput, Units> {
    map_opt(word::<u8, _>('G'), |word| word.value.try_into().ok())(i)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::rem;

    #[test]
    fn parse_units() {
        assert_eq!(
            units(ParseInput::new("G20")),
            Ok((rem!("", 3), Units::Inch))
        );
        assert_eq!(units(ParseInput::new("G21")), Ok((rem!("", 3), Units::Mm)));
    }
}
