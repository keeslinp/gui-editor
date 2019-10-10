use crate::msg::Direction;
use ropey::Rope;

#[derive(Debug, PartialEq, Default)]
pub struct Point {
    pub x: u16,
    pub y: u16,
}

impl Point {
    pub fn index(&self, rope: &Rope) -> usize {
        rope.line_to_char(self.y as usize) + self.x as usize
    }
    pub fn step_to_index(&mut self, index: usize, rope: &Rope) {
        self.y = rope.char_to_line(index) as u16;
        self.x = (index - rope.line_to_char(self.y as usize)) as u16;
    }
    pub fn prevent_runoff(&mut self, rope: &Rope) {
        let line_len = rope.line(self.y as usize).len_chars() as u16;
        if line_len <= self.x {
            self.x = line_len - 1;
        }
    }
    pub fn step(&mut self, direction: Direction, rope: &Rope) {
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
}
