use crate::{
    cursor::Cursor,
    error::Error,
    msg::{DeleteDirection, Direction, JumpType},
    state::Config,
};
use std::borrow::Cow;

use anyhow::Result;
use ropey::Rope;
use slotmap::DefaultKey;

use syntect::{highlighting::Theme, parsing::SyntaxSet};

mod highlighter;
use highlighter::HighlightContainer;

pub type BufferKey = DefaultKey;

pub struct Buffer {
    rope: Rope,
    cursor: Cursor,
    file: Option<std::path::PathBuf>,
    highlighter: Option<HighlightContainer>,
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
            highlighter: None,
        })
    }

    pub fn load_file(file_path: std::path::PathBuf, config: &Config) -> Result<Buffer> {
        let rope = Rope::from_reader(std::fs::File::open(file_path.as_path())?)?;
        let highlighter = match file_path
            .extension()
            .and_then(|os_str| os_str.to_str())
            .and_then(|ext| config.syntax_set.find_syntax_by_extension(ext))
        {
            Some(syntax) => Some(syntax),
            None => config
                .syntax_set
                .find_syntax_by_first_line(rope.chunk_at_char(0).0),
        }
        .cloned()
        .map(|syntax| {
            let mut val = HighlightContainer::new(syntax);
            val.highlight(&rope.slice(..), config);
            val
        });
        Ok(Buffer {
            rope,
            cursor: Cursor::new(),
            file: Some(file_path),
            highlighter,
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

    pub fn insert_char(&mut self, config: &Config, c: char, should_step: bool) {
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
            '\n' => {
                let line: Cow<str> = self.rope.line(self.cursor.row()).into();
                let space_count = line.len() - line.trim_start().len();
                self.rope.insert_char(index, '\n');
                for _ in 0..space_count {
                    self.rope.insert_char(index + 1, ' ');
                }
                if should_step {
                    let slice = self.rope.slice(..);
                    self.cursor.step(Direction::Right, &slice);
                    for _ in 0..space_count {
                        self.cursor.step(Direction::Right, &slice);
                    }
                }
            }
            c => {
                self.rope.insert_char(index, c);
                if should_step {
                    self.cursor.step(Direction::Right, &self.rope.slice(..));
                }
            }
        }
        if let Some(ref mut highlighter) = self.highlighter {
            highlighter.highlight(&self.rope.slice(..), config);
        }
    }

    pub fn delete_char(&mut self, config: &Config, direction: DeleteDirection) {
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
        if let Some(ref mut highlighter) = self.highlighter {
            highlighter.highlight(&self.rope.slice(..), config);
        }
    }

    pub fn render(&self, ui: &imgui::Ui) {
        let _visible_lines = get_visible_lines(ui);
        let line_len = self.rope.len_lines();
        let line_offset = log10(line_len);
        let line_offset_px = 5. + line_offset as f32 * 10.;

        ui.group(|| {
            ui.set_cursor_pos([0., 0.]);
            ui.new_line();
            ui.indent_by(line_offset_px);
            if let Some(ref highlighter) = self.highlighter {
                highlighter.render(&self.rope.slice(..), ui);
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
        self.cursor.render(ui, line_offset_px, &self.rope.slice(..));
    }

    pub fn step(&mut self, direction: Direction) {
        self.cursor.step(direction, &self.rope.slice(..));
    }

    pub fn jump(&mut self, jump_type: JumpType, line_count: usize) {
        self.cursor
            .jump(jump_type, &self.rope.slice(..), line_count);
    }
}

pub fn get_visible_lines(ui: &imgui::Ui) -> usize {
    let window_height = ui.window_size()[1];
    let line_height = ui.text_line_height_with_spacing();
    ((window_height) / line_height) as usize - 2
}
