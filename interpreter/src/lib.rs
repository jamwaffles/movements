#![no_std]

extern crate alloc;

use alloc::collections::VecDeque;
use common::{Command, Motion};

pub struct Interpreter {
    queue: VecDeque<Command>,
    modal_groups: ModalGroupState,
}

impl Interpreter {
    pub fn new() -> Self {
        Self {
            queue: VecDeque::new(),
            modal_groups: ModalGroupState::default(),
        }
    }

    pub fn queue_command(&mut self, command: Command) {
        self.queue.push_front(command);
    }

    // Pop command at beginning of queue and update interpreter state
    pub fn pop_command(&mut self) {
        if let Some(command) = self.queue.pop_back() {
            match command {
                Command::Position { axis, value } => todo!(),
                Command::Motion(motion) => self.modal_groups.motion = Some(motion),
            }
        }
    }
}

pub struct ModalGroupState {
    motion: Option<Motion>,
}

impl Default for ModalGroupState {
    fn default() -> Self {
        Self { motion: None }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // #[test]
    // fn it_works() {
    //     let ass: VecDeque<i32> = VecDeque::new();
    // }
}
