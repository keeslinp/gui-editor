use crate::{
    cursor::Cursor,
    error::Error,
    msg::{DeleteDirection, Direction, JumpType},
};
use std::borrow::Cow;

use anyhow::Result;
use ropey::Rope;
use slotmap::DefaultKey;
use syntect::{parsing::{SyntaxSet, SyntaxReference}, highlighting::Theme};

pub type BufferKey = DefaultKey;

pub struct Buffer {
    rope: Rope,
    cursor: Cursor,
    file: Option<std::path::PathBuf>,
    syntax: Option<SyntaxReference>,
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
            file: None,
            syntax: None,
        })
    }

    pub fn load_file(file_path: std::path::PathBuf, syntax_set: &SyntaxSet) -> Result<Buffer> {
        let rope = Rope::from_reader(std::fs::File::open(file_path.as_path())?)?;
        let syntax = match file_path.extension().and_then(|os_str| os_str.to_str()).and_then(|ext| syntax_set.find_syntax_by_extension(ext)) {
            Some(syntax) => Some(syntax),
            None => {
                syntax_set.find_syntax_by_first_line(rope.chunk_at_char(0).0)
            }
        }.cloned();
        Ok(Buffer {
            rope,
            cursor: Cursor::new(),
            file: Some(file_path),
            syntax,
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

    pub fn insert_char(&mut self, c: char, should_step: bool) {
        let index = self.cursor.index(&self.rope.slice(..));
        match c {
            '\t' => {
                self.rope.insert(index, "    ");

                if should_step {
                    self.cursor.step(Direction::Right, &self.rope.slice(..));
                    self.cursor.step(Direction::Right, &self.rope.slice(..));
                    self.cursor.step(Direction::Right, &self.rope.slice(..));
                    self.cursor.step(Direction::Right, &self.rope.slice(..));
                }
            }
            c => {
                self.rope.insert_char(index, c);
                if should_step {
                    self.cursor.step(Direction::Right, &self.rope.slice(..));
                }
            }
        }
    }

    pub fn delete_char(&mut self, direction: DeleteDirection) {
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
    }

    pub fn render(&self, ui: &imgui::Ui, theme: &Theme, ps: &SyntaxSet) {
        let _visible_lines = get_visible_lines(ui);
        let line_len = self.rope.len_lines();
        let line_offset = log10(line_len);
        let line_offset_px = 5. + line_offset as f32 * 10.;

        ui.group(|| {
            ui.set_cursor_pos([0., 0.]);
            ui.new_line();
            ui.indent_by(line_offset_px);
            if let Some(ref syntax) = self.syntax {
                use syntect::{easy::HighlightLines, util:: LinesWithEndings};
                let mut h = HighlightLines::new(syntax, theme);
                for line in self.rope.lines() {
                    let text: Cow<str> = line.into();
                    // TODO: for obvious reasons, don't just parse the whole file each render
                    for (style, val) in h.highlight(&text, &ps) {
                        let syntect::highlighting::Color { r, g, b, a } = style.foreground;
                        ui.text_colored([r as f32 / 255., g as f32 / 255., b as f32 / 255., a as f32 / 255.], val);
                        ui.same_line_with_spacing(0., 0.);
                    }
                    ui.new_line();
                }
            } else {
                for line in self.rope.lines() {
                    let text: Cow<str> = line.into();
                    ui.text(text);
                }
            }
        });
        ui.group(|| {
            ui.set_cursor_pos([0., 0.]);
            ui.new_line();
            ui.indent_by(5.);
            for line in 0..line_len {
                ui.text(&format!("{}", line + 1));
            }
        });
        self.cursor.render(ui, line_offset_px);
    }

    pub fn step(&mut self, direction: Direction) {
        self.cursor.step(direction, &self.rope.slice(..));
    }

    pub fn jump(&mut self, jump_type: JumpType) {
        self.cursor.jump(jump_type, &self.rope.slice(..));
    }
}

pub fn get_visible_lines(ui: &imgui::Ui) -> usize {
    let window_height = ui.window_content_region_max()[1];
    let line_height = ui.text_line_height_with_spacing();
    ((window_height) / line_height) as usize - 2
}
