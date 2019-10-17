use crate::{
    mode::Mode,
    msg::{Cmd, DeleteDirection, Direction, Msg},
    render::RenderFrame,
    error::{CommandError, Result},
};

use wgpu_glyph::{Scale, Section};
use winit::{
    dpi::PhysicalSize,
    event_loop::EventLoopProxy,
};

#[derive(Debug, PartialEq, Clone, Default)]
pub struct CommandBuffer {
    buffer: String,
    position: usize,
}

impl CommandBuffer {
    fn run_command(&mut self, msg_sender: EventLoopProxy<Msg>) -> Result<()> {
        msg_sender
            .send_event(Msg::Cmd(Cmd::ChangeMode(Mode::Normal)))
            .expect("Changing to normal mode");
        let result = match self.buffer.as_ref() {
            "q" => {
                msg_sender
                    .send_event(Msg::Cmd(Cmd::Quit))
                    .expect("Sending quit message");
                Ok(())
            }
            cmd if cmd.starts_with("edit") => {
                let maybe_file = cmd.split(' ').skip(1).next();
                if let Some(file) = maybe_file {
                    msg_sender
                        .send_event(Msg::Cmd(Cmd::LoadFile(std::path::PathBuf::from(file))))
                        .expect("sending load file command");
                    Ok(())
                } else {
                    Err(CommandError::MissingArg)
                }
            }
            buffer => {
                Err(CommandError::UnknownCommand(buffer.to_owned()))
            }
        };
        self.buffer.clear();
        self.position = 0;
        result.map_err(|cmd_err| cmd_err.into())
    }
    pub fn handle_command(&mut self, cmd: Cmd, msg_sender: EventLoopProxy<Msg>) -> Result<bool> {
        Ok(match cmd {
            Cmd::InsertChar(c) => match c {
                '\n' => {
                    self.run_command(msg_sender)?;
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
        })
    }
    pub fn render(&self, render_frame: &mut RenderFrame, window_size: PhysicalSize) {
        render_frame.queue_text(Section {
            text: &format!(":{}", self.buffer),
            screen_position: (10., window_size.height as f32 - 30.),
            color: [0.514, 0.58, 0.588, 1.],
            scale: Scale { x: 30., y: 30. },
            ..Section::default()
        });
    }
}
