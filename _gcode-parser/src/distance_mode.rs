//! Gcodes from [modal group 3 (distance mode)](http://linuxcnc.org/docs/html/gcode/overview.html#_modal_groups)

use crate::word::word;
use crate::ParseInput;
use core::str::FromStr;
use nom::combinator::map;
use nom::IResult;

#[derive(Debug, PartialEq, Clone)]
pub enum DistanceMode {
    /// G90
    Absolute,

    /// G91
    Incremental,
}

impl FromStr for DistanceMode {
    type Err = ();

    fn from_str(number: &str) -> Result<Self, Self::Err> {
        match number {
            "90" => Ok(Self::Absolute),
            "91" => Ok(Self::Incremental),
            _ => Err(()),
        }
    }
}

pub fn distance_mode(i: ParseInput) -> IResult<ParseInput, DistanceMode> {
    map(word::<DistanceMode, _>('G'), |word| word.value)(i)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::rem;

    #[test]
    fn check_distance_mode() {
        assert_eq!(
            distance_mode(ParseInput::new("G90")),
            Ok((rem!("", 3), DistanceMode::Absolute))
        );

        assert_eq!(
            distance_mode(ParseInput::new("G91")),
            Ok((rem!("", 3), DistanceMode::Incremental))
        );
    }
}
