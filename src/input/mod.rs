use winit::event::VirtualKeyCode;

use crate::{
    mode::Mode,
    msg::{Cmd, DeleteDirection, Direction, InputMsg},
};

pub fn build_cmd_from_input(input_msg: InputMsg, mode: Mode) -> Option<Cmd> {
    match (mode, input_msg) {
        // Shared
        (_, InputMsg::KeyPressed(VirtualKeyCode::Left)) => Some(Cmd::MoveCursor(Direction::Left)),
        (_, InputMsg::KeyPressed(VirtualKeyCode::Right)) => Some(Cmd::MoveCursor(Direction::Right)),
        (_, InputMsg::KeyPressed(VirtualKeyCode::Down)) => Some(Cmd::MoveCursor(Direction::Down)),
        (_, InputMsg::KeyPressed(VirtualKeyCode::Up)) => Some(Cmd::MoveCursor(Direction::Up)),

        // Insert
        (Mode::Insert, InputMsg::CharPressed(c)) if c.is_alphanumeric() => Some(Cmd::InsertChar(c)),
        (Mode::Insert, InputMsg::KeyPressed(key)) => match key {
            VirtualKeyCode::Back => Some(Cmd::DeleteChar(DeleteDirection::Before)),
            VirtualKeyCode::Return => Some(Cmd::InsertChar('\n')),
            VirtualKeyCode::Escape => Some(Cmd::ChangeMode(Mode::Normal)),
            _ => None,
        },

        // Command
        (Mode::Command, InputMsg::CharPressed(c)) if c.is_alphanumeric() => {
            Some(Cmd::InsertChar(c))
        }
        (Mode::Command, InputMsg::KeyPressed(key)) => match key {
            VirtualKeyCode::Back => Some(Cmd::DeleteChar(DeleteDirection::Before)),
            VirtualKeyCode::Return => Some(Cmd::InsertChar('\n')),
            VirtualKeyCode::Escape => Some(Cmd::ChangeMode(Mode::Normal)),
            _ => None,
        },

        // Normal
        (Mode::Normal, InputMsg::CharPressed(c)) => match c {
            'h' => Some(Cmd::MoveCursor(Direction::Left)),
            'l' => Some(Cmd::MoveCursor(Direction::Right)),
            'k' => Some(Cmd::MoveCursor(Direction::Up)),
            'j' => Some(Cmd::MoveCursor(Direction::Down)),
            'i' => Some(Cmd::ChangeMode(Mode::Insert)),
            ':' => Some(Cmd::ChangeMode(Mode::Command)),
            _ => None,
        },
        _ => None,
    }
}
