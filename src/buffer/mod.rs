use crate::{
    cursor::Cursor,
    error::Result,
    msg::{DeleteDirection, Direction, JumpType},
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

fn log10(num: usize) -> usize {
    match num {
        n if n < 10 => 1,
        n if n < 100 => 2,
        n if n < 1000 => 3,
        n if n < 10000 => 4,
        n if n < 100000 => 5,
        n if n < 1000000 => 6,
        _ => 7, // Cross that bridge when we get there
    }
}

impl Buffer {
    pub fn new() -> Buffer {
        Buffer {
            rope: Rope::new(),
            cursor: Cursor::new(),
            offset: 0,
        }
    }

    pub fn load_file(file_path: &std::path::Path) -> Result<Buffer> {
        Ok(Buffer {
            rope: Rope::from_reader(std::fs::File::open(file_path)?)?,
            cursor: Cursor::new(),
            offset: 0,
        })
    }

    pub fn insert_char(&mut self, c: char, should_step: bool, window_size: PhysicalSize) {
        match c {
            '\t' => {
                self.rope.insert(self.cursor.index(&self.rope), "    ");

                if should_step {
                    self.cursor.step(Direction::Right, &self.rope);
                    self.cursor.step(Direction::Right, &self.rope);
                    self.cursor.step(Direction::Right, &self.rope);
                    self.cursor.step(Direction::Right, &self.rope);
                }
            }
            c => {
                self.rope.insert_char(self.cursor.index(&self.rope), c);
                if should_step {
                    self.cursor.step(Direction::Right, &self.rope);
                }
            }
        }
        self.adjust_viewport(window_size);
    }

    fn adjust_viewport(&mut self, window_size: PhysicalSize) {
        if self.cursor.row() < self.offset {
            self.offset = self.cursor.row();
        } else {
            let visible_lines = get_visible_lines(window_size);
            if self.cursor.row() >= visible_lines + self.offset {
                self.offset = self.cursor.row() - visible_lines + 1;
            }
        }
    }

    pub fn delete_char(&mut self, direction: DeleteDirection, window_size: PhysicalSize) {
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
                if char_index < self.rope.len_chars() {
                    self.rope.remove(char_index..=char_index);
                }
            }
        };
        self.adjust_viewport(window_size);
    }

    pub fn render(&self, render_frame: &mut RenderFrame, window_size: PhysicalSize) {
        let visible_lines = get_visible_lines(window_size);
        let line_len = self.rope.len_lines();
        let line_offset = log10(line_len);
        let line_offset_px = line_offset as f32 * 15.;
        use wgpu_glyph::{Layout, HorizontalAlign};
        for visible_line in 0..visible_lines {
            let real_line = self.offset + visible_line;
            let line_in_buffer: bool = real_line < line_len;
            if line_in_buffer {
                render_frame.queue_text(Section {
                    text: &format!("{}", real_line + 1),
                    screen_position: (10. + line_offset_px, 10. + visible_line as f32 * 25.),
                    color: [0.514, 0.58, 0.588, 1.],
                    scale: Scale { x: 30., y: 30. },
                    layout: Layout::default().h_align(HorizontalAlign::Right),
                    ..Section::default()
                });
                let line = self.rope.line(real_line);
                if let Some(text) = line.as_str() {
                    render_frame.queue_text(Section {
                        text,
                        screen_position: (30. + line_offset_px, 10. + visible_line as f32 * 25.),
                        color: [0.514, 0.58, 0.588, 1.],
                        scale: Scale { x: 30., y: 30. },
                        ..Section::default()
                    });
                }
            } else {
                render_frame.queue_text(Section {
                    text: "~",
                    screen_position: (10., 10. + visible_line as f32 * 25.),
                    color: [0., 1., 0., 1.],
                    scale: Scale { x: 30., y: 30. },
                    ..Section::default()
                });
            }
        }
        self.cursor.render(render_frame, line_offset, self.offset);
    }

    pub fn step(&mut self, direction: Direction, window_size: PhysicalSize) {
        self.cursor.step(direction, &self.rope);
        self.adjust_viewport(window_size);
    }

    pub fn jump(&mut self, jump_type: JumpType, window_size: PhysicalSize) {
        self.cursor.jump(jump_type, &self.rope);
        self.adjust_viewport(window_size);
    }
}

fn get_visible_lines(window_size: PhysicalSize) -> usize {
    ((window_size.height - 10.) / 25.).floor() as usize - 1
}
