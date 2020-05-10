use crate::msg::{Cmd, DeleteDirection, Direction};

use anyhow::Result;

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
    pub fn render(&self, ui: &imgui::Ui) {
        let im_string = imgui::ImString::new(format!(":{}", self.buffer.as_str()));
        ui.set_cursor_pos([10., 0.]);
        ui.text(im_string);
    }
}
