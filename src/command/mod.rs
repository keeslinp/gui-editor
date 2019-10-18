use crate::{
    error::{CommandError, Result},
    mode::Mode,
    msg::{Cmd, Msg},
    render::RenderFrame,
    text_buffer::TextBuffer,
};

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
        let result = match self.buffer.as_str().to_lowercase().as_str() {
            "q" => {
                msg_sender
                    .send_event(Msg::Cmd(Cmd::Quit))
                    .expect("Sending quit message");
                Ok(())
            }
            cmd if cmd.starts_with("edit") => {
                let maybe_file = cmd.split(' ').nth(1);
                if let Some(file) = maybe_file {
                    msg_sender
                        .send_event(Msg::Cmd(Cmd::LoadFile(std::path::PathBuf::from(file))))
                        .expect("sending load file command");
                    Ok(())
                } else {
                    Err(CommandError::MissingArg)
                }
            }
            buffer => Err(CommandError::UnknownCommand(buffer.to_owned())),
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
    pub fn render(&self, render_frame: &mut RenderFrame, window_size: PhysicalSize) {
        self.buffer.render(render_frame, window_size);
    }
}
