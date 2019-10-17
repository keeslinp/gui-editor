use crate::{
    buffer::Buffer,
    mode::Mode,
    msg::{Cmd, Msg},
    state::State,
};
use crossbeam_channel::Sender;
use winit::dpi::PhysicalSize;

pub fn handle_command(state: &mut State, cmd: Cmd, msg_sender: Sender<Msg>, window_size: PhysicalSize) -> bool {
    match (state.mode, cmd) {
        (_, Cmd::ChangeMode(mode)) => {
            state.mode = dbg!(mode);
            true
        }
        (_, Cmd::Quit) => {
            unreachable!();
        }
        (_, Cmd::LoadFile(file)) => {
            let buffer = Buffer::load_file(file.as_path());
            let new_buffer_key = state.buffer_keys.insert(());
            state.buffers.insert(new_buffer_key, buffer);
            state.current_buffer = new_buffer_key;
            true
        }
        (Mode::Command, cmd) => state.command_buffer.handle_command(cmd, msg_sender),
        // All other modes just work on the buffer
        (_, Cmd::InsertChar(c)) => {
            let buffer = &mut state.buffers[state.current_buffer];
            buffer.insert_char(c);
            true
        }
        (_, Cmd::DeleteChar(direction)) => {
            let buffer = &mut state.buffers[state.current_buffer];
            buffer.delete_char(direction);
            true
        }
        (_, Cmd::MoveCursor(direction)) => {
            let buffer = &mut state.buffers[state.current_buffer];
            buffer.step(direction, window_size);
            true
        }
    }
}
