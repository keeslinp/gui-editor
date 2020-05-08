use crate::{
    error::Error,
    mode::Mode,
    msg::{Cmd, Msg},
    text_buffer::TextBuffer,
};

use anyhow::Result;

use winit::{dpi::PhysicalSize, event_loop::EventLoopProxy};

#[derive(Debug, PartialEq, Clone, Default)]
pub struct CommandBuffer {
    buffer: TextBuffer,
}

impl CommandBuffer {
    pub fn clear(&mut self) {
        self.buffer.clear();
    }
    fn run_command(&mut self, msg_sender: EventLoopProxy<Msg>) -> Result<()> {
        msg_sender
            .send_event(Msg::Cmd(Cmd::ChangeMode(Mode::Normal)))
            .expect("Changing to normal mode");
        // Case insensitive cause I have always hated my life when I accidentally
        // hold down shift while trying to save files
        let command = self.buffer.as_str().to_lowercase();
        let mut words = command.split_whitespace();
        let result = match words.next() {
            Some("q") => {
                msg_sender
                    .send_event(Msg::Cmd(Cmd::Quit))
                    .expect("Sending quit message");
                Ok(())
            }
            Some("w") => {
                let maybe_path = {
                    let rest = words.fold(String::new(), |mut acc, word| {
                        acc.push_str(&word);
                        acc
                    });
                    if rest.len() == 0 {
                        None
                    } else {
                        Some(std::path::PathBuf::from(rest))
                    }
                };
                msg_sender
                    .send_event(Msg::Cmd(Cmd::WriteBuffer(maybe_path)))
                    .expect("Sending write message");
                Ok(())
            }
            Some("edit") => {
                let maybe_file = words.next();
                if let Some(file) = maybe_file {
                    msg_sender
                        .send_event(Msg::Cmd(Cmd::LoadFile(std::path::PathBuf::from(file))))
                        .expect("sending load file command");
                    Ok(())
                } else {
                    Err(Error::MissingArg)
                }
            }
            Some(buffer) => Err(Error::UnknownCommand(buffer.to_owned())),
            None => Ok(()),
        };
        self.buffer.clear();
        result.map_err(|cmd_err| cmd_err.into())
    }
    pub fn handle_command(&mut self, cmd: Cmd, msg_sender: EventLoopProxy<Msg>) -> Result<bool> {
        Ok(match cmd {
            Cmd::Submit => {
                self.run_command(msg_sender)?;
                true
            }
            cmd => self.buffer.handle_command(cmd)?,
        })
    }
    // pub fn render(&self, render_frame: &mut RenderFrame, window_size: PhysicalSize<u32>) {
    //     self.buffer.render(render_frame, window_size);
    // }
}
