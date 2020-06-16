//! Gcodes from [modal group 2 (plane select)](http://linuxcnc.org/docs/html/gcode/overview.html#_modal_groups)

use crate::word::word;
use crate::ParseInput;
use nom::combinator::map;
use nom::IResult;
use std::str::FromStr;

#[derive(Debug, PartialEq, Clone)]
pub enum PlaneSelect {
    /// G17
    XY,

    /// G18
    ZX,

    /// G19
    YZ,

    /// G17.1
    UV,

    /// G18.1
    WU,

    /// G19.1
    VW,
}

impl FromStr for PlaneSelect {
    type Err = ();

    fn from_str(number: &str) -> Result<Self, Self::Err> {
        match number {
            "17" => Ok(Self::XY),
            "18" => Ok(Self::ZX),
            "19" => Ok(Self::YZ),
            "17.1" => Ok(Self::UV),
            "18.1" => Ok(Self::WU),
            "19.1" => Ok(Self::VW),
            _ => Err(()),
        }
    }
}

pub fn plane_select(i: ParseInput) -> IResult<ParseInput, PlaneSelect> {
    map(word::<PlaneSelect, _>('G'), |word| word.value)(i)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::rem;

    #[test]
    fn xy_plane_select() {
        assert_eq!(
            plane_select(ParseInput::new("G17")),
            Ok((rem!("", 3), PlaneSelect::XY))
        );
    }

    #[test]
    fn extra() {
        assert_eq!(
            plane_select(ParseInput::new("G17.1")),
            Ok((rem!("", 5), PlaneSelect::UV))
        );
    }
}
