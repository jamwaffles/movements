//! Gcodes from [M code modal group 4 (stopping)](http://linuxcnc.org/docs/html/gcode/overview.html#_modal_groups)

use crate::word::word;
use crate::ParseInput;
use nom::combinator::map_opt;
use nom::IResult;
use std::convert::{TryFrom, TryInto};

#[derive(Debug, PartialEq)]
pub enum Stopping {
    /// M2
    EndProgram,
}

impl TryFrom<u8> for Stopping {
    type Error = ();

    fn try_from(number: u8) -> Result<Self, Self::Error> {
        match number {
            2 => Ok(Stopping::EndProgram),
            _ => Err(()),
        }
    }
}

pub fn stopping(i: ParseInput) -> IResult<ParseInput, Stopping> {
    map_opt(word::<u8, _>('M'), |word| word.value.try_into().ok())(i)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::rem;

    #[test]
    fn parse_program_end() {
        assert_eq!(
            stopping(ParseInput::new("M2")),
            Ok((rem!("", 2), Stopping::EndProgram))
        );
        assert_eq!(
            stopping(ParseInput::new("M02")),
            Ok((rem!("", 3), Stopping::EndProgram))
        );
    }
}