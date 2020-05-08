use crate::{
    color_scheme::ColorScheme,
    cursor::Cursor,
    error::Error,
    msg::{DeleteDirection, Direction, JumpType},
};

use anyhow::Result;
use ropey::Rope;
use slotmap::DefaultKey;

use winit::dpi::PhysicalSize;

pub type BufferKey = DefaultKey;

pub mod highlighter;

use highlighter::Highlighter;

pub struct Buffer {
    rope: Rope,
    cursor: Cursor,
    offset: usize,
    file: Option<std::path::PathBuf>,
    highlighter: Option<Highlighter>, // The option is temporary
}

fn log10(num: usize) -> usize {
    match num {
        n if n < 1 => panic!("log10 doesn't work for n < 1"),
        n if n < 10 => 1,
        n if n < 100 => 2,
        n if n < 1000 => 3,
        n if n < 10000 => 4,
        n if n < 100_000 => 5,
        n if n < 1_000_000 => 6,
        _ => unimplemented!(), // Cross that bridge when we get there
    }
}

impl Buffer {
    pub fn new() -> Result<Buffer> {
        Ok(Buffer {
            rope: Rope::new(),
            cursor: Cursor::new(),
            offset: 0,
            file: None,
            highlighter: None,
        })
    }

    pub fn load_file(file_path: std::path::PathBuf) -> Result<Buffer> {
        let mut highlighter = Highlighter::new(file_path.extension().and_then(|s| s.to_str()).unwrap_or("rs"))?;
        let rope = Rope::from_reader(std::fs::File::open(file_path.as_path())?)?;
        highlighter.parse(rope.slice(..));
        Ok(Buffer {
            rope,
            cursor: Cursor::new(),
            offset: 0,
            file: Some(file_path),
            highlighter: Some(highlighter),
        })
    }

    pub fn write(&mut self, file_path: Option<std::path::PathBuf>) -> Result<()> {
        if let Some(path) = file_path.or(self.file.take()) {
            self.rope
                .write_to(std::io::BufWriter::new(std::fs::File::create(
                    path.as_path(),
                )?))?;
            self.file = Some(path);
            Ok(())
        } else {
            Err(anyhow::Error::new(Error::NeedFilePath))
        }
    }

    fn mark_dirty(&mut self) {
        if let Some(ref mut highlighter) = self.highlighter {
            highlighter.mark_dirty(self.rope.line_to_char(std::cmp::max(1, self.cursor.row()) - 1));
        }
    }

    pub fn insert_char(&mut self, c: char, should_step: bool) {
        self.mark_dirty();
        let index = self.cursor.index(&self.rope.slice(..));
        match c {
            '\t' => {
                self.rope
                    .insert(index, "    ");

                if should_step {
                    self.cursor.step(Direction::Right, &self.rope.slice(..));
                    self.cursor.step(Direction::Right, &self.rope.slice(..));
                    self.cursor.step(Direction::Right, &self.rope.slice(..));
                    self.cursor.step(Direction::Right, &self.rope.slice(..));
                }
            }
            c => {
                self.rope
                    .insert_char(index, c);
                if should_step {
                    self.cursor.step(Direction::Right, &self.rope.slice(..));
                }
            }
        }
        // self.adjust_viewport(window_size);
    }

    // fn adjust_viewport(&mut self, height: f32) {
    //     let visible_lines = get_visible_lines(height);
    //     if self.cursor.row() < self.offset {
    //         self.offset = self.cursor.row();
    //     } else {
    //         if self.cursor.row() >= visible_lines + self.offset {
    //             self.offset = self.cursor.row() - visible_lines + 1;
    //         }
    //     }
    //     let last_char_index = self.rope.line_to_char(std::cmp::min(self.offset + visible_lines, self.rope.len_lines()));
    //     if let Some(ref mut highlighter) = self.highlighter {
    //         highlighter.parse(self.rope.slice(..last_char_index));
    //     }
    // }

    pub fn delete_char(&mut self, direction: DeleteDirection) {
        self.mark_dirty();
        let char_index = self.cursor.index(&self.rope.slice(..));
        match direction {
            DeleteDirection::Before => {
                if char_index > 0 {
                    self.rope.remove(char_index - 1..char_index);
                    self.cursor.step(Direction::Left, &self.rope.slice(..));
                }
            }
            DeleteDirection::After => {
                if char_index < self.rope.len_chars() {
                    self.rope.remove(char_index..=char_index);
                }
            }
        };
        // self.adjust_viewport(window_size);
    }

    pub fn render(
        &self,
        ui: &imgui::Ui,
        color_scheme: &ColorScheme,
    ) {
        let visible_lines = get_visible_lines(ui);
        let line_len = self.rope.len_lines();
        let line_offset = log10(line_len);
        let line_offset_px = line_offset as f32 * 10.;

        let char_offset = self.rope.line_to_char(self.offset);
        let char_end = self.rope.len_chars();
        // if let Some(ref highlighter) = self.highlighter {
        //     highlighter.render(
        //         ui,
        //         char_offset..char_end,
        //         self.rope.slice(..),
        //         30. + line_offset_px,
        //         self.offset as f32 * 25. - 10.,
        //         color_scheme,
        //     );
        // }
        ui.group(|| {
            ui.set_cursor_pos([0., 0.]);
            ui.new_line();
            ui.indent_by(5. + line_offset_px);
            for line in self.rope.lines().skip(self.offset).take(visible_lines) {
                use std::borrow::Cow;
                let text: Cow<str> = line.into();
                ui.text(text);
            }
        });
        ui.group(|| {
            ui.set_cursor_pos([0., 0.]);
            ui.new_line();
            ui.indent_by(5.);
            for visible_line in 0..visible_lines {
                let real_line = self.offset + visible_line;
                let line_in_buffer: bool = real_line < line_len;
                if line_in_buffer {
                    ui.text(&format!("{}", real_line + 1));
                } else {
                    ui.text("~");
                };
            }
        })
        // self.cursor.render(render_frame, line_offset, self.offset);
    }

    pub fn step(&mut self, direction: Direction) {
        self.cursor.step(direction, &self.rope.slice(..));
        // self.adjust_viewport(window_size);
    }

    pub fn jump(&mut self, jump_type: JumpType) {
        self.cursor.jump(jump_type, &self.rope.slice(..));
        // self.adjust_viewport(window_size);
    }
}

pub fn get_visible_lines(ui: &imgui::Ui) -> usize {
    let window_height = ui.window_content_region_max()[1];
    let line_height = ui.text_line_height_with_spacing();
    ((window_height) / line_height) as usize - 2
}
