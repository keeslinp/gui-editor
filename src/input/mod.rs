use winit::event::VirtualKeyCode;

use crate::{
    mode::Mode,
    msg::{Cmd, DeleteDirection, Direction, InputMsg, JumpType},
};

pub fn build_cmd_from_input(input_msg: InputMsg, mode: Mode, cmd_sender: impl Fn(Cmd) -> ()) {
    match (mode, input_msg) {
        // Shared
        (_, InputMsg::KeyPressed(VirtualKeyCode::Left)) => {
            cmd_sender(Cmd::MoveCursor(Direction::Left))
        }
        (_, InputMsg::KeyPressed(VirtualKeyCode::Right)) => {
            cmd_sender(Cmd::MoveCursor(Direction::Right))
        }
        (_, InputMsg::KeyPressed(VirtualKeyCode::Down)) => {
            cmd_sender(Cmd::MoveCursor(Direction::Down))
        }
        (_, InputMsg::KeyPressed(VirtualKeyCode::Up)) => cmd_sender(Cmd::MoveCursor(Direction::Up)),

        // Insert
        (Mode::Insert, InputMsg::CharPressed(c)) if !c.is_control() => {
            cmd_sender(Cmd::InsertChar(c))
        }
        (Mode::Insert, InputMsg::KeyPressed(key)) => match key {
            VirtualKeyCode::Back => cmd_sender(Cmd::DeleteChar(DeleteDirection::Before)),
            VirtualKeyCode::Return => cmd_sender(Cmd::InsertChar('\n')),
            VirtualKeyCode::Escape => cmd_sender(Cmd::ChangeMode(Mode::Normal)),
            _ => {}
        },

        // Command
        (Mode::Command, InputMsg::CharPressed(c)) if !c.is_control() => {
            cmd_sender(Cmd::InsertChar(c))
        }
        (Mode::Command, InputMsg::KeyPressed(key)) => match key {
            VirtualKeyCode::Back => cmd_sender(Cmd::DeleteChar(DeleteDirection::Before)),
            VirtualKeyCode::Return => cmd_sender(Cmd::InsertChar('\n')),
            VirtualKeyCode::Escape => cmd_sender(Cmd::ChangeMode(Mode::Normal)),
            _ => {}
        },

        // Normal
        (Mode::Normal, InputMsg::CharPressed(c)) => match c {
            'h' => cmd_sender(Cmd::MoveCursor(Direction::Left)),
            'l' => cmd_sender(Cmd::MoveCursor(Direction::Right)),
            'k' => cmd_sender(Cmd::MoveCursor(Direction::Up)),
            'j' => cmd_sender(Cmd::MoveCursor(Direction::Down)),
            'i' => cmd_sender(Cmd::ChangeMode(Mode::Insert)),
            ':' => cmd_sender(Cmd::ChangeMode(Mode::Command)),
            'd' => cmd_sender(Cmd::DeleteChar(DeleteDirection::After)),
            'a' => {
                cmd_sender(Cmd::MoveCursor(Direction::Right));
                cmd_sender(Cmd::ChangeMode(Mode::Insert));
            }
            'A' => {
                cmd_sender(Cmd::Jump(JumpType::EndOfLine));
                cmd_sender(Cmd::ChangeMode(Mode::Insert));
            }
            'o' => {
                cmd_sender(Cmd::Jump(JumpType::EndOfLine));
                cmd_sender(Cmd::InsertChar('\n'));
                cmd_sender(Cmd::ChangeMode(Mode::Insert));
            }
            'O' => {
                cmd_sender(Cmd::Jump(JumpType::StartOfLine));
                cmd_sender(Cmd::InsertChar('\n'));
                cmd_sender(Cmd::ChangeMode(Mode::Insert));
                cmd_sender(Cmd::MoveCursor(Direction::Left));
            }
            _ => {}
        },
        _ => {}
    }
}
