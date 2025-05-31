use std::collections::VecDeque;
use crate::direction::Direction;

pub struct InputBuffer {
    pub commands: VecDeque<Direction>,
    max_size: usize,
}

impl InputBuffer {
    pub fn new(max_size: usize) -> Self {
        Self {
            commands: VecDeque::with_capacity(max_size),
            max_size,
        }
    }

    pub fn add_command(&mut self, direction: Direction) {
        if self.commands.len() >= self.max_size {
            self.commands.pop_front();
        }
        if self.commands.back() != Some(&direction) {
            self.commands.push_back(direction);
        }
    }

    pub fn get_next_command(&mut self) -> Option<Direction> {
        self.commands.pop_front()
    }

    pub fn clear(&mut self) {
        self.commands.clear();
    }
}