use crate::{
    msg::{Cmd, Msg},
    state::State,
};
use crossbeam_channel::Sender;

pub fn handle_command(state: &mut State, cmd: Cmd, _msg_sender: Sender<Msg>) -> bool {
    match cmd {
        Cmd::InsertChar(c) => {
            let buffer = &mut state.buffers[state.current_buffer];
            buffer.insert_char(c);
            true
        }
        Cmd::DeleteChar(direction) => {
            let buffer = &mut state.buffers[state.current_buffer];
            buffer.delete_char(direction);
            true
        }
        Cmd::MoveCursor(direction) => {
            let buffer = &mut state.buffers[state.current_buffer];
            buffer.step(direction);
            true
        }
        Cmd::ChangeMode(mode) => {
            state.mode = mode;
            true
        }
    }
}
