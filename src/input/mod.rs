use winit::event::VirtualKeyCode;

use crate::msg::{Cmd, DeleteDirection, InputMsg, Direction};

pub fn build_cmd_from_input(input_msg: InputMsg) -> Option<Cmd> {
    match input_msg {
        InputMsg::CharPressed(c) => Some(Cmd::InsertChar(c)),
        InputMsg::KeyPressed(key) => match key {
            VirtualKeyCode::Back => Some(Cmd::DeleteChar(DeleteDirection::Before)),
            VirtualKeyCode::Return => Some(Cmd::InsertChar('\n')),
            VirtualKeyCode::Left => Some(Cmd::MoveCursor(Direction::Left)),
            VirtualKeyCode::Right => Some(Cmd::MoveCursor(Direction::Right)),
            VirtualKeyCode::Up => Some(Cmd::MoveCursor(Direction::Up)),
            VirtualKeyCode::Down => Some(Cmd::MoveCursor(Direction::Down)),
            _ => None,
        },
    }
}
