use crate::msg::{Direction, JumpType};
use ropey::RopeSlice;

use flamer::flame;

#[derive(Debug, PartialEq, Default)]
pub struct Point {
    pub x: u16,
    pub y: u16,
}

impl Point {
    pub fn index(&self, rope: &RopeSlice) -> usize {
        rope.line_to_char(self.y as usize) + self.x as usize
    }

    #[flame("Point")]
    pub fn from_index(index: usize, rope: &RopeSlice) -> Self {
        let y = rope.char_to_line(index) as u16;
        let x = (index - rope.line_to_char(y as usize)) as u16;
        Point { x, y }
    }
    pub fn step_to_index(&mut self, index: usize, rope: &RopeSlice) {
        *self = Point::from_index(index, rope);
    }
    pub fn prevent_runoff(&mut self, rope: &RopeSlice) {
        let line_len = rope.line(self.y as usize).len_chars() as u16;
        if line_len <= self.x {
            if line_len == 0 {
                self.x = 0;
            } else {
                self.x = line_len - 1;
            }
        }
    }
    pub fn step(&mut self, direction: Direction, rope: &RopeSlice) {
        match direction {
            Direction::Left => {
                let index = self.index(rope);
                if index > 0 {
                    self.step_to_index(index - 1, rope);
                }
            }
            Direction::Right => {
                let index = self.index(rope);
                if index < rope.len_chars() {
                    self.step_to_index(index + 1, rope);
                }
            }
            Direction::Down => {
                if self.y + 1 < rope.len_lines() as u16 {
                    self.y += 1;
                }
            }
            Direction::Up => {
                if self.y > 0 {
                    self.y -= 1;
                }
            }
        }
    }
    pub fn jump(&mut self, jump_type: JumpType, rope: &RopeSlice) {
        match jump_type {
            JumpType::EndOfLine => {
                let line = rope.line(self.y as usize);
                self.x = line.len_chars() as u16 - 1;
            }
            JumpType::StartOfLine => {
                self.x = 0;
            }
            JumpType::StartOfFile => {
                self.y = 0;
                self.x = 0;
            }
            JumpType::EndOfFile => {
                self.y = rope.len_lines() as u16 - 1;
                self.x = 0;
            }
        }
    }
}
