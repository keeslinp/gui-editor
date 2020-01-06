use crate::{
    msg::{Cmd, DeleteDirection, Direction},
    render::RenderFrame,
};

use anyhow::Result;

use wgpu_glyph::{Scale, Section};
use winit::dpi::PhysicalSize;

#[derive(Debug, PartialEq, Clone, Default)]
pub struct TextBuffer {
    buffer: String,
    position: usize,
}

impl TextBuffer {
    pub fn clear(&mut self) {
        self.buffer.clear();
        self.position = 0;
    }
    pub fn as_str<'a>(&'a self) -> &'a str {
        self.buffer.as_ref()
    }

    pub fn handle_command(&mut self, cmd: Cmd) -> Result<bool> {
        Ok(match cmd {
            Cmd::InsertChar(c, should_step) => {
                self.buffer.insert(self.position, c);
                if should_step {
                    self.position += 1;
                }
                true
            }
            Cmd::MoveCursor(Direction::Left) => {
                if self.position > 0 {
                    self.position -= 1;
                }
                true
            }
            Cmd::MoveCursor(Direction::Right) => {
                if self.position < self.buffer.len() {
                    self.position += 1;
                }
                true
            }
            Cmd::MoveCursor(Direction::Up) | Cmd::MoveCursor(Direction::Down) => {
                false // Con't care
            }
            Cmd::DeleteChar(DeleteDirection::Before) => {
                if self.position > 0 {
                    self.buffer.remove(self.position - 1);
                    self.position -= 1;
                    true
                } else {
                    false
                }
            }
            _ => false,
        })
    }
    pub fn render(&self, render_frame: &mut RenderFrame, window_size: PhysicalSize<u32>) {
        render_frame.queue_text(Section {
            text: &format!(":{}", self.buffer.as_str()),
            screen_position: (10., window_size.height as f32 - 30.),
            color: [0.514, 0.58, 0.588, 1.],
            scale: Scale { x: 30., y: 30. },
            ..Section::default()
        });
    }
}
