use gcode_parser::coord::Coord;
use gcode_parser::motion::Motion;
use gcode_parser::Axes;
use gcode_parser::Block;
use gcode_parser::GcodeProgram;
use gcode_parser::TokenType;
use std::ops::Add;
use std::slice::Iter;

pub struct GcodeInterpreter<'a> {
    program: &'a GcodeProgram<'a>,

    blocks: Iter<'a, Block>,

    state: State,
}

/// Interpreter state
#[derive(Debug, PartialEq, Copy, Clone)]
pub struct State {
    /// G group 0
    pub motion: Motion,

    pub next_position: Axes,
}

impl Default for State {
    fn default() -> Self {
        Self {
            motion: Motion::Feed,
            next_position: Axes::zeros(),
        }
    }
}

#[derive(Default)]
struct BlockCommands<'a> {
    motion: Option<Motion>,

    coord: Option<&'a Coord>,
}

impl<'a> GcodeInterpreter<'a> {
    pub fn new(program: &'a GcodeProgram) -> Self {
        Self {
            program,
            blocks: program.blocks.iter(),
            state: State::default(),
        }
    }

    pub fn block_iter(&self) -> BlockIterator {
        BlockIterator {
            state: self.state,
            blocks: self.program.blocks.iter(),
        }
    }
}

pub struct BlockIterator<'a> {
    state: State,

    blocks: Iter<'a, Block>,
}

impl<'a> BlockIterator<'a> {
    fn new_state_from_block(&mut self, block: &Block) -> Result<State, String> {
        let mut block_actions = BlockCommands::default();

        for token in block.tokens.iter() {
            match &token.token {
                TokenType::Motion(m) => {
                    if let None = block_actions.motion {
                        block_actions.motion = Some(*m);
                    } else {
                        return Err("Cannot set multiple motion modes on same line".into());
                    }
                }
                TokenType::Coord(c) => {
                    if let None = block_actions.coord {
                        block_actions.coord = Some(c);
                    } else {
                        return Err("Cannot set multiple coordinates on same line".into());
                    }
                }
                t => unimplemented!("{:?}", t),
            }
        }

        Ok(State {
            motion: block_actions.motion.unwrap_or(self.state.motion),
            next_position: block_actions
                .coord
                .map(|c| merge_vector9_and_coord(&self.state.next_position, c))
                .unwrap_or(self.state.next_position),
        })
    }
}

impl<'a> Iterator for BlockIterator<'a> {
    type Item = Result<State, String>;

    fn next(&mut self) -> Option<Result<State, String>> {
        self.blocks.next().map(move |block| {
            self.state = self.new_state_from_block(block)?;

            Ok(self.state)
        })
    }
}

pub fn merge_vector9_and_coord(current: &Axes, coord: &Coord) -> Axes {
    let mut new = current.clone();
    let coord_c = coord.clone();

    new[0] = coord_c.x.unwrap_or_else(|| current[0]);
    new[1] = coord_c.y.unwrap_or_else(|| current[1]);
    new[2] = coord_c.z.unwrap_or_else(|| current[2]);
    new[3] = coord_c.u.unwrap_or_else(|| current[3]);
    new[4] = coord_c.v.unwrap_or_else(|| current[4]);
    new[5] = coord_c.w.unwrap_or_else(|| current[5]);
    new[6] = coord_c.a.unwrap_or_else(|| current[6]);
    new[7] = coord_c.b.unwrap_or_else(|| current[7]);
    new[8] = coord_c.c.unwrap_or_else(|| current[8]);

    new
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rapid() {
        let program = GcodeProgram::from_str("G0").unwrap();

        let interp = GcodeInterpreter::new(&program);

        let mut blocks = interp.block_iter();

        assert_eq!(
            blocks.next(),
            Some(Ok(State {
                motion: Motion::Rapid,
                ..State::default()
            }))
        );
        assert_eq!(blocks.next(), None);
    }

    #[test]
    fn do_some_moves() {
        let program = GcodeProgram::from_str(
            r#"G0 X0 Y0 Z0
            X10 Y20 Z30
            Z10
            G1 Z-2
            X0 Y0
            Z5
            G0 Z30"#,
        )
        .unwrap();

        let interp = GcodeInterpreter::new(&program);

        let mut blocks = interp.block_iter();

        assert_eq!(
            blocks.next(),
            Some(Ok(State {
                motion: Motion::Rapid,
                next_position: Axes::zeros()
            }))
        );

        assert_eq!(
            blocks.next(),
            Some(Ok(State {
                motion: Motion::Rapid,
                next_position: Axes::from([10.0, 20.0, 30.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0])
            }))
        );

        assert_eq!(
            blocks.next(),
            Some(Ok(State {
                motion: Motion::Rapid,
                next_position: Axes::from([10.0, 20.0, 10.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0])
            }))
        );

        assert_eq!(
            blocks.next(),
            Some(Ok(State {
                motion: Motion::Feed,
                next_position: Axes::from([10.0, 20.0, -2.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0])
            }))
        );

        assert_eq!(
            blocks.next(),
            Some(Ok(State {
                motion: Motion::Feed,
                next_position: Axes::from([0.0, 0.0, -2.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0])
            }))
        );

        assert_eq!(
            blocks.next(),
            Some(Ok(State {
                motion: Motion::Feed,
                next_position: Axes::from([0.0, 0.0, 5.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0])
            }))
        );

        assert_eq!(
            blocks.next(),
            Some(Ok(State {
                motion: Motion::Rapid,
                next_position: Axes::from([0.0, 0.0, 30.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0])
            }))
        );

        assert_eq!(blocks.next(), None);
    }
}
