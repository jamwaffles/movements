//! Gcodes from [modal group 3 (distance mode)](http://linuxcnc.org/docs/html/gcode/overview.html#_modal_groups)

use crate::word::word;
use crate::ParseInput;
use nom::combinator::map;
use nom::IResult;
use std::str::FromStr;

#[derive(Debug, PartialEq, Clone)]
pub enum CutterCompensation {
    /// G40
    Disabled,

    /// G41
    Left,

    /// G42
    Right,
}

impl FromStr for CutterCompensation {
    type Err = ();

    fn from_str(number: &str) -> Result<Self, Self::Err> {
        match number {
            "40" => Ok(Self::Disabled),
            "41" => Ok(Self::Left),
            "42" => Ok(Self::Right),
            _ => Err(()),
        }
    }
}

pub fn cutter_compensation(i: ParseInput) -> IResult<ParseInput, CutterCompensation> {
    map(word::<CutterCompensation, _>('G'), |word| word.value)(i)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::rem;

    #[test]
    fn cutter_comp() {
        assert_eq!(
            cutter_compensation(ParseInput::new("G40")),
            Ok((rem!("", 3), CutterCompensation::Disabled))
        );

        assert_eq!(
            cutter_compensation(ParseInput::new("G41")),
            Ok((rem!("", 3), CutterCompensation::Left))
        );

        assert_eq!(
            cutter_compensation(ParseInput::new("G42")),
            Ok((rem!("", 3), CutterCompensation::Right))
        );
    }
}
