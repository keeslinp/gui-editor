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
            cmd_sender(Cmd::InsertChar(c, true))
        }
        (Mode::Insert, InputMsg::KeyPressed(key)) => match key {
            VirtualKeyCode::Back => cmd_sender(Cmd::DeleteChar(DeleteDirection::Before)),
            VirtualKeyCode::Return => cmd_sender(Cmd::InsertChar('\n', true)),
            VirtualKeyCode::Escape => cmd_sender(Cmd::ChangeMode(Mode::Normal)),
            _ => {}
        },

        // Command
        (Mode::Command, InputMsg::CharPressed(c)) if !c.is_control() => {
            cmd_sender(Cmd::InsertChar(c, true))
        }
        (Mode::Command, InputMsg::KeyPressed(key)) => match key {
            VirtualKeyCode::Back => cmd_sender(Cmd::DeleteChar(DeleteDirection::Before)),
            VirtualKeyCode::Return => cmd_sender(Cmd::InsertChar('\n', true)),
            VirtualKeyCode::Escape => cmd_sender(Cmd::ChangeMode(Mode::Normal)),
            _ => {}
        },

        (Mode::Jump, InputMsg::KeyPressed(key)) => match key {
            VirtualKeyCode::Escape => cmd_sender(Cmd::ChangeMode(Mode::Normal)),
            _ => {}
        }
        (Mode::Jump, InputMsg::CharPressed(c)) => {
            match c {
                'l' => cmd_sender(Cmd::Jump(JumpType::EndOfLine)),
                'h' => cmd_sender(Cmd::Jump(JumpType::StartOfLine)),
                'k' => cmd_sender(Cmd::Jump(JumpType::StartOfFile)),
                'j' => cmd_sender(Cmd::Jump(JumpType::EndOfFile)),
                _ => {}
            }
            cmd_sender(Cmd::ChangeMode(Mode::Normal));
        }

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
                cmd_sender(Cmd::InsertChar('\n', false));
                cmd_sender(Cmd::MoveCursor(Direction::Down));
                // Reset the saved x value
                cmd_sender(Cmd::MoveCursor(Direction::Left));
                cmd_sender(Cmd::MoveCursor(Direction::Right));

                cmd_sender(Cmd::ChangeMode(Mode::Insert));
            }
            'O' => {
                cmd_sender(Cmd::Jump(JumpType::StartOfLine));
                cmd_sender(Cmd::InsertChar('\n', false));
                cmd_sender(Cmd::ChangeMode(Mode::Insert));
            }
            'g' => {
                cmd_sender(Cmd::ChangeMode(Mode::Jump));
            }
            _ => {}
        },
        _ => {}
    }
}
