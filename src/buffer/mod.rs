use pathfinder_canvas::{CanvasRenderingContext2D};
use pathfinder_geometry::vector::{Vector2F};
use slotmap::{DefaultKey};
use ropey::Rope;
use crate::{
    cursor::Cursor,
    msg::{
        DeleteDirection,
        Direction,
    },
};

pub type BufferKey = DefaultKey;

pub struct Buffer {
    rope: Rope,
    cursor: Cursor,
}

impl Buffer {
    pub fn new() -> Buffer {
        Buffer {
            rope: Rope::new(),
            cursor: Cursor::new(),
        }
    }

    pub fn insert_char(&mut self, c: char) {
        match c {
            '\t' => {
                self.rope.insert(self.cursor.index(&self.rope), "    ");

                self.cursor.step(Direction::Right, &self.rope);
                self.cursor.step(Direction::Right, &self.rope);
                self.cursor.step(Direction::Right, &self.rope);
                self.cursor.step(Direction::Right, &self.rope);
            },
            c => {
                self.rope.insert_char(self.cursor.index(&self.rope), c);
                self.cursor.step(Direction::Right, &self.rope);
            }
        }
    }

    pub fn delete_char(&mut self, direction: DeleteDirection) {
        match direction {
            DeleteDirection::Before => {
                let char_index = self.cursor.index(&self.rope);
                if char_index > 0 {
                    self.rope.remove(char_index - 1..char_index);
                    self.cursor.step(Direction::Left, &self.rope);
                }
            },
            DeleteDirection::After => {
                let char_index = self.cursor.index(&self.rope);
                self.rope.remove(char_index..char_index + 1);
            }
        };
    }

    pub fn render(&self, canvas: &mut CanvasRenderingContext2D) {
        for (line_index, line) in self.rope.lines().enumerate() {
            canvas.fill_text(line.as_str().unwrap_or("").trim_end(), Vector2F::new(10.0, 10.0 + (line_index as f32 * 20.0)));
        }
        self.cursor.render(canvas);
    }

    pub fn step(&mut self, direction: Direction) {
        self.cursor.step(direction, &self.rope);
    }
}
