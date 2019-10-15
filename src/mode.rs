use winit::dpi::PhysicalSize;
use wgpu_glyph::{Scale, Section};

use crate::renderer::RenderFrame;

#[derive(Debug, PartialEq, Copy, Clone)]
pub enum Mode {
    Normal,
    Insert,
    Command,
    // Select,
}

impl Mode {
    fn as_str(&self) -> &'static str {
        match self {
            Mode::Normal => "Normal",
            Mode::Insert => "Insert",
            Mode::Command => "Command",
        }
    }
    pub fn render(&self, render_frame: &mut RenderFrame, window_size: PhysicalSize) {
        let value = self.as_str();
        render_frame.queue_text(Section {
            text: value,
            screen_position: (window_size.width as f32 - (value.len() as f32 * 20.), window_size.height as f32 - 30.),
            color: [0.514, 0.58, 0.588, 1. ],
            scale: Scale { x: 30., y: 30. },
            ..Section::default()
        });
    }
}
