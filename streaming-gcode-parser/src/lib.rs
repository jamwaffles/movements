mod coord;
mod expression;
mod modal_groups;
mod parameter;
mod value;
mod word;

use coord::Coord;
use modal_groups::Motion;
use parameter::Parameter;
use value::Value;
use word::parse_word;

pub struct Block {
    block_delete: bool,
    line_number: Option<u32>,
    words: Vec<Statement>,
}

pub enum Statement {
    LineNumber(u32),
    Comment { comment: String },
    SetParameter { parameter: Parameter, value: Value },
    Motion(Motion),
    Coord(Coord),
    Dynamic { letter: char, number: Value },
}
