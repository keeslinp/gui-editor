use crate::{
    buffer::Buffer,
    mode::Mode,
    msg::{Cmd, Msg},
    state::State,
};
use anyhow::Result;
use winit::event_loop::EventLoopProxy;

pub fn handle_command(
    state: &mut State,
    cmd: Cmd,
    msg_sender: EventLoopProxy<Msg>,
) -> Result<bool> {
    Ok(match (state.mode, cmd) {
        (_, Cmd::SetStatusText(_text)) => {
            unreachable!();
        }
        (_, Cmd::ChangeMode(mode)) => {
            flame::start("change_mode");
            state.mode = mode;
            match mode {
                Mode::Skim => state.skim_buffer.refresh_files(),
                Mode::Command => state.command_buffer.clear(),
                _ => {} // the rest don't need setup
            }
            flame::end("change_mode");
            true
        }
        (_, Cmd::Quit) => {
            unreachable!();
        }
        (_, Cmd::LoadFile(file)) => {
            flame::start("load_file");
            let buffer = Buffer::load_file(file, &state.syntax_set)?;
            let new_buffer_key = state.buffer_keys.insert(());
            state.buffers.insert(new_buffer_key, buffer);
            state.current_buffer = new_buffer_key;
            flame::end("load_file");
            true
        }
        (_, Cmd::WriteBuffer(maybe_path)) => {
            flame::start("write_buffer");
            let buffer = &mut state.buffers[state.current_buffer];
            buffer.write(maybe_path)?;
            flame::end("write_buffer");
            true
        }

        (Mode::Skim, cmd) => state.skim_buffer.handle_command(cmd, msg_sender)?,
        (Mode::Command, cmd) => state.command_buffer.handle_command(cmd, msg_sender)?,
        (_, Cmd::Submit) => false, // None of the other modes care
        // All other modes just work on the buffer
        (_, Cmd::Jump(jump_type)) => {
            flame::start("jump");
            let buffer = &mut state.buffers[state.current_buffer];
            buffer.jump(jump_type);
            flame::end("jump");
            true
        }
        (_, Cmd::InsertChar(c, should_step)) => {
            flame::start("insert");
            let buffer = &mut state.buffers[state.current_buffer];
            buffer.insert_char(c, should_step);
            flame::end("insert");
            true
        }
        (_, Cmd::DeleteChar(direction)) => {
            flame::start("delete");
            let buffer = &mut state.buffers[state.current_buffer];
            buffer.delete_char(direction);
            flame::end("delete");
            true
        }
        (_, Cmd::MoveCursor(direction)) => {
            flame::start("move");
            let buffer = &mut state.buffers[state.current_buffer];
            buffer.step(direction);
            flame::end("move");
            true
        }
    })
}
