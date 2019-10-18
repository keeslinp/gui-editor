use crate::{
    buffer::Buffer,
    error::Result,
    mode::Mode,
    msg::{Cmd, Msg},
    state::State,
};
use winit::{dpi::PhysicalSize, event_loop::EventLoopProxy};

pub fn handle_command(
    state: &mut State,
    cmd: Cmd,
    msg_sender: EventLoopProxy<Msg>,
    window_size: PhysicalSize,
) -> Result<bool> {
    Ok(match (state.mode, cmd) {
        (_, Cmd::SetError(_err)) => {
            unreachable!();
        }
        (_, Cmd::ChangeMode(mode)) => {
            state.mode = mode;
            match mode {
                Mode::Skim => state.skim_buffer.refresh_files(),
                _ => {} // the rest don't need setup
            }
            true
        }
        (_, Cmd::Quit) => {
            unreachable!();
        }
        (_, Cmd::LoadFile(file)) => {
            let buffer = Buffer::load_file(file.as_path())?;
            let new_buffer_key = state.buffer_keys.insert(());
            state.buffers.insert(new_buffer_key, buffer);
            state.current_buffer = new_buffer_key;
            true
        }
        (Mode::Skim, cmd) => state.skim_buffer.handle_command(cmd, msg_sender)?,
        (Mode::Command, cmd) => state.command_buffer.handle_command(cmd, msg_sender)?,
        // All other modes just work on the buffer
        (_, Cmd::Jump(jump_type)) => {
            let buffer = &mut state.buffers[state.current_buffer];
            buffer.jump(jump_type, window_size);
            true
        }
        (_, Cmd::InsertChar(c, should_step)) => {
            let buffer = &mut state.buffers[state.current_buffer];
            buffer.insert_char(c, should_step, window_size);
            true
        }
        (_, Cmd::DeleteChar(direction)) => {
            let buffer = &mut state.buffers[state.current_buffer];
            buffer.delete_char(direction, window_size);
            true
        }
        (_, Cmd::MoveCursor(direction)) => {
            let buffer = &mut state.buffers[state.current_buffer];
            buffer.step(direction, window_size);
            true
        }
    })
}
