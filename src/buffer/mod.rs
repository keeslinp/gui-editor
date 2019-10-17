use crate::{
    cursor::Cursor,
    msg::{DeleteDirection, Direction},
    render::RenderFrame,
};
use ropey::Rope;
use slotmap::DefaultKey;
use wgpu_glyph::{Scale, Section};
use winit::dpi::PhysicalSize;

pub type BufferKey = DefaultKey;

pub struct Buffer {
    rope: Rope,
    cursor: Cursor,
    offset: usize,
}

impl Buffer {
    pub fn new() -> Buffer {
        Buffer {
            rope: Rope::new(),
            cursor: Cursor::new(),
            offset: 0,
        }
    }

    pub fn load_file(file_path: &std::path::Path) -> Buffer {
        Buffer {
            rope: Rope::from_reader(std::fs::File::open(file_path).expect("loading file"))
                .expect("building rope"),
            cursor: Cursor::new(),
            offset: 0,
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
            }
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
            }
            DeleteDirection::After => {
                let char_index = self.cursor.index(&self.rope);
                self.rope.remove(char_index..char_index + 1);
            }
        };
    }

    pub fn render(&self, render_frame: &mut RenderFrame, window_size: PhysicalSize) {
        let visible_lines = get_visible_lines(window_size);
        for (line_index, line) in self.rope.lines().skip(self.offset).enumerate().take(visible_lines) {
            if let Some(text) = line.as_str() {
                render_frame.queue_text(Section {
                    text,
                    screen_position: (10., 10. + line_index as f32 * 25.),
                    color: [0.514, 0.58, 0.588, 1.],
                    scale: Scale { x: 30., y: 30. },
                    ..Section::default()
                });
            }
        }
        self.cursor.render(render_frame, self.offset);
    }

    pub fn step(&mut self, direction: Direction, window_size: PhysicalSize) {
        self.cursor.step(direction, &self.rope);
        if self.cursor.row() < self.offset {
            self.offset = self.cursor.row();
        } else {
            let visible_lines = get_visible_lines(window_size);
            if self.cursor.row() >= visible_lines + self.offset {
                self.offset = self.cursor.row() - visible_lines + 1;
            }
        }
    }
}

fn get_visible_lines(window_size: PhysicalSize) -> usize {
    ((window_size.height - 10.) / 25.).floor() as usize - 1
}
