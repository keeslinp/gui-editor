use crate::msg::{Direction, JumpType};
use ropey::{RopeSlice, iter::Chars};

use flamer::flame;

#[derive(Debug, PartialEq, Default)]
pub struct Point {
    pub x: u16,
    pub y: u16,
}

struct RevChars<'a> {
    chars: Chars<'a>,
}

impl<'a> Iterator for RevChars<'a> {
    type Item = char;
    fn next(&mut self) -> Option<char> {
        self.chars.prev()
    }
}

impl<'a> From<Chars<'a>> for RevChars<'a> {
    fn from(chars: Chars<'a>) -> RevChars<'a> {
        RevChars {
            chars
        }
    }
}

impl Point {
    pub fn index(&self, rope: &RopeSlice) -> usize {
        rope.line_to_char(self.y as usize) + self.x as usize
    }

    pub fn get_char(&self, rope: &RopeSlice) -> char {
        rope.char(self.index(rope))
    }

    pub fn is_start(&self) -> bool {
        self.x == 0 && self.y == 0
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

    fn search<T: Iterator<Item = char>>(&mut self, t: T, rope: &RopeSlice, direction: Direction, while_fn: fn(&char) -> bool) {
        t.take_while(while_fn).for_each(|_| self.step(direction, rope));
    }
    fn step_while(&mut self, rope: &RopeSlice, direction: Direction, while_fn: fn(&char) -> bool) {
        let iter = rope.chars_at(self.index(rope));
        match direction {
            Direction::Right => self.search(iter, rope, direction, while_fn),
            Direction::Left => self.search(RevChars::from(iter), rope, direction, while_fn),
            Direction::Up | Direction::Down => unimplemented!(),
        }
    }
    pub fn jump(&mut self, jump_type: JumpType, rope: &RopeSlice, line_count: usize) {
        match jump_type {
            JumpType::EndOfLine => {
                let line = rope.line(self.y as usize);
                self.x = line.len_chars() as u16
                    - if self.y as usize == rope.len_lines() - 1 {
                        0
                    } else {
                        1
                    };
            }
            JumpType::StartOfLine => {
                use std::borrow::Cow;
                let line: Cow<str> = rope.line(self.y as usize).into();
                self.x = (line.len() - line.trim_start().len()) as u16;
            }
            JumpType::StartOfFile => {
                self.y = 0;
                self.x = 0;
            }
            JumpType::EndOfFile => {
                self.y = rope.len_lines() as u16 - 1;
                self.x = 0;
            }
            JumpType::NextWord => {
                self.step_while(rope, Direction::Right, |c| c.is_alphanumeric());
                self.step_while(rope, Direction::Right, |c| !c.is_alphanumeric());
            }
            JumpType::EndOfWord => {
                if rope.char(self.index(rope)).is_alphanumeric() && !rope.char(self.index(rope) + 1).is_alphanumeric() { // TODO: Handle edges of buffer
                    self.step(Direction::Right, rope);
                }
                self.step_while(rope, Direction::Right, |c| !c.is_alphanumeric());
                self.step_while(rope, Direction::Right, |c| c.is_alphanumeric());
                self.step(Direction::Left, rope);
            }
            JumpType::PrevWord => {
                if rope.char(self.index(rope)).is_alphanumeric() && !rope.char(self.index(rope) - 1).is_alphanumeric() { // TODO: Handle edges of buffer
                    self.step(Direction::Left, rope);
                    self.step_while(rope, Direction::Left, |c| !c.is_alphanumeric())
                }
                self.step_while(rope, Direction::Left, |c| c.is_alphanumeric())
            }
            JumpType::PageForward => {
                for _ in 0..line_count {
                    self.step(Direction::Down, rope);
                }
                self.prevent_runoff(rope);
            }
            JumpType::PageBackward => {
                for _ in 0..line_count {
                    self.step(Direction::Up, rope);
                }
                self.prevent_runoff(rope);
            }
        }
    }
}
