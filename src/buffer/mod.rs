use crate::{
    cursor::Cursor,
    error::Error,
    msg::{DeleteDirection, Direction, JumpType},
    render::RenderFrame,
};

use anyhow::Result;
use ropey::Rope;
use slotmap::DefaultKey;
use wgpu_glyph::{Scale, Section};
use winit::dpi::PhysicalSize;

pub type BufferKey = DefaultKey;

mod highlighter;

use highlighter::Highlighter;

pub struct Buffer {
    rope: Rope,
    cursor: Cursor,
    offset: usize,
    file: Option<std::path::PathBuf>,
    highlighter: Highlighter,
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
            highlighter: Highlighter::new()?,
        })
    }

    pub fn load_file(file_path: std::path::PathBuf) -> Result<Buffer> {
        let mut highlighter = Highlighter::new()?;
        let rope = Rope::from_reader(std::fs::File::open(file_path.as_path())?)?;
        highlighter.parse(rope.slice(..));
        Ok(Buffer {
            rope,
            cursor: Cursor::new(),
            offset: 0,
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

    pub fn insert_char(&mut self, c: char, should_step: bool, window_size: PhysicalSize<u32>) {
        match c {
            '\t' => {
                self.rope
                    .insert(self.cursor.index(&self.rope.slice(..)), "    ");

                if should_step {
                    self.cursor.step(Direction::Right, &self.rope.slice(..));
                    self.cursor.step(Direction::Right, &self.rope.slice(..));
                    self.cursor.step(Direction::Right, &self.rope.slice(..));
                    self.cursor.step(Direction::Right, &self.rope.slice(..));
                }
            }
            c => {
                self.rope
                    .insert_char(self.cursor.index(&self.rope.slice(..)), c);
                if should_step {
                    self.cursor.step(Direction::Right, &self.rope.slice(..));
                }
            }
        }
        self.adjust_viewport(window_size);
        self.highlighter.parse(self.rope.slice(..));
    }

    fn adjust_viewport(&mut self, window_size: PhysicalSize<u32>) {
        if self.cursor.row() < self.offset {
            self.offset = self.cursor.row();
            self.highlighter.parse(self.rope.slice(..));
        } else {
            let visible_lines = get_visible_lines(window_size);
            if self.cursor.row() >= visible_lines + self.offset {
                self.offset = self.cursor.row() - visible_lines + 1;
                self.highlighter.parse(self.rope.slice(..));
            }
        }
    }

    pub fn delete_char(&mut self, direction: DeleteDirection, window_size: PhysicalSize<u32>) {
        match direction {
            DeleteDirection::Before => {
                let char_index = self.cursor.index(&self.rope.slice(..));
                if char_index > 0 {
                    self.rope.remove(char_index - 1..char_index);
                    self.cursor.step(Direction::Left, &self.rope.slice(..));
                }
            }
            DeleteDirection::After => {
                let char_index = self.cursor.index(&self.rope.slice(..));
                if char_index < self.rope.len_chars() {
                    self.rope.remove(char_index..=char_index);
                }
            }
        };
        self.adjust_viewport(window_size);
        self.highlighter.parse(self.rope.slice(..));
    }

    pub fn render(&self, render_frame: &mut RenderFrame, window_size: PhysicalSize<u32>) {
        let visible_lines = get_visible_lines(window_size);
        let line_len = self.rope.len_lines();
        let line_offset = log10(line_len);
        let line_offset_px = line_offset as f32 * 15.;
        use wgpu_glyph::{HorizontalAlign, Layout};
        let char_offset = self.rope.line_to_char(self.offset);
        let char_end = self.rope.len_chars();
        self.highlighter.render(
            render_frame,
            (char_offset..char_end),
            self.rope.slice(..),
            30. + line_offset_px,
            self.offset as f32 * 25. - 10.,
        );
        // for visible_line in 0..visible_lines {
        //     let real_line = self.offset + visible_line;
        //     let line_in_buffer: bool = real_line < line_len;
        //     if line_in_buffer {
        //         render_frame.queue_text(Section {
        //             text: &format!("{}", real_line + 1),
        //             screen_position: (10. + line_offset_px, 10. + visible_line as f32 * 25.),
        //             color: [0.514, 0.58, 0.588, 1.],
        //             scale: Scale { x: 30., y: 30. },
        //             layout: Layout::default().h_align(HorizontalAlign::Right),
        //             ..Section::default()
        //         });
        //         let line = self.rope.line(real_line);
        //         if let Some(text) = line.as_str() {
        //             render_frame.queue_text(Section {
        //                 text,
        //                 screen_position: (30. + line_offset_px, 10. + visible_line as f32 * 25.),
        //                 color: [0.514, 0.58, 0.588, 1.],
        //                 scale: Scale { x: 30., y: 30. },
        //                 ..Section::default()
        //             });
        //         }
        //     } else {
        //         render_frame.queue_text(Section {
        //             text: "~",
        //             screen_position: (10., 10. + visible_line as f32 * 25.),
        //             color: [0., 1., 0., 1.],
        //             scale: Scale { x: 30., y: 30. },
        //             ..Section::default()
        //         });
        //     }
        // }
        self.cursor.render(render_frame, line_offset, self.offset);
    }

    pub fn step(&mut self, direction: Direction, window_size: PhysicalSize<u32>) {
        self.cursor.step(direction, &self.rope.slice(..));
        self.adjust_viewport(window_size);
    }

    pub fn jump(&mut self, jump_type: JumpType, window_size: PhysicalSize<u32>) {
        self.cursor.jump(jump_type, &self.rope.slice(..));
        self.adjust_viewport(window_size);
    }
}

pub fn get_visible_lines(window_size: PhysicalSize<u32>) -> usize {
    ((window_size.height - 10) / 25) as usize - 1
}
