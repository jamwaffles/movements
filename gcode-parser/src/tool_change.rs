//! Gcodes from [modal group 6 (tool change)](http://linuxcnc.org/docs/html/gcode/overview.html#_modal_groups)

use crate::word::word;
use crate::ParseInput;
use core::convert::{TryFrom, TryInto};
use nom::combinator::map_res;
use nom::IResult;

#[derive(Debug, PartialEq, Copy, Clone)]
pub enum ToolChange {
    /// M6
    ToolChange,
}

impl TryFrom<u8> for ToolChange {
    type Error = ();

    fn try_from(number: u8) -> Result<Self, Self::Error> {
        match number {
            6 => Ok(ToolChange::ToolChange),
            _ => Err(()),
        }
    }
}

pub fn tool_change(i: ParseInput) -> IResult<ParseInput, ToolChange> {
    map_res(word::<u8, _>('M'), |word| word.value.try_into())(i)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::rem;

    #[test]
    fn change_tool() {
        assert_eq!(
            tool_change(ParseInput::new("M6")),
            Ok((rem!("", 2), ToolChange::ToolChange))
        );
        assert_eq!(
            tool_change(ParseInput::new("M06")),
            Ok((rem!("", 3), ToolChange::ToolChange))
        );
    }
}
