use crate::{
    mode::Mode,
    msg::{Cmd, DeleteDirection, Direction, Msg},
    render::RenderFrame,
};
use crossbeam_channel::Sender;

use winit::dpi::PhysicalSize;
use wgpu_glyph::{Scale, Section};

#[derive(Debug, PartialEq, Clone, Default)]
pub struct CommandBuffer {
    buffer: String,
    position: usize,
}

impl CommandBuffer {
    fn run_command(&mut self, msg_sender: Sender<Msg>) {
        msg_sender
            .send(Msg::Cmd(Cmd::ChangeMode(Mode::Normal)))
            .expect("Changing to normal mode");
        match self.buffer.as_ref() {
            "q" => {
                msg_sender
                    .send(Msg::Cmd(Cmd::Quit))
                    .expect("Sending quit message");
            }
            cmd if cmd.starts_with("edit") => {
                let maybe_file = cmd.split(' ').skip(1).next();
                if let Some(file) = maybe_file {
                    msg_sender
                        .send(Msg::Cmd(Cmd::LoadFile(std::path::PathBuf::from(file))))
                        .expect("sending load file command");
                } else {
                    eprintln!("Missing file");
                }
            }
            buffer => {
                eprintln!("Unknown command: {}", buffer);
            }
        }
        self.buffer.clear();
        self.position = 0;
    }
    pub fn handle_command(&mut self, cmd: Cmd, msg_sender: Sender<Msg>) -> bool {
        match cmd {
            Cmd::InsertChar(c) => match c {
                '\n' => {
                    self.run_command(msg_sender);
                    true
                }
                c => {
                    self.buffer.insert(self.position, c);
                    self.position += 1;
                    true
                }
            },
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
        }
    }
    pub fn render(&self, render_frame: &mut RenderFrame, window_size: PhysicalSize) {
        render_frame.queue_text(Section {
            text: &format!(":{}", self.buffer),
            screen_position: (10., window_size.height as f32 - 30.),
            color: [0.514, 0.58, 0.588, 1. ],
            scale: Scale { x: 30., y: 30. },
            ..Section::default()
        });
    }
}
