use crate::{
    mode::Mode,
    msg::{Cmd, Msg},
    state::State,
};
use crossbeam_channel::Sender;

pub fn handle_command(state: &mut State, cmd: Cmd, msg_sender: Sender<Msg>) -> bool {
    match (state.mode, cmd) {
        (_, Cmd::ChangeMode(mode)) => {
            state.mode = dbg!(mode);
            true
        }
        (_, Cmd::Quit) => {
            unreachable!();
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
            buffer.step(direction);
            true
        }
    }
}
