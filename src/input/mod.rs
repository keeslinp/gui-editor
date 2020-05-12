use winit::event::VirtualKeyCode;

use crate::{
    mode::Mode,
    msg::{Cmd, DeleteDirection, Direction, InputMsg, JumpType},
};

fn is_valid_key(c: char) -> bool {
    c != '\r' && (c.is_alphanumeric() || c.is_whitespace() || c.is_ascii_punctuation())
}

pub fn process_input(input_msg: InputMsg, mode: Mode, cmd_sender: impl Fn(Cmd) -> ()) {
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
        (Mode::Insert, InputMsg::CharPressed(c)) if is_valid_key(c) => {
            cmd_sender(Cmd::InsertChar(c, true))
        }
        (Mode::Insert, InputMsg::KeyPressed(key)) => match key {
            VirtualKeyCode::Back => cmd_sender(Cmd::DeleteChar(DeleteDirection::Before)),
            VirtualKeyCode::Return => cmd_sender(Cmd::InsertChar('\n', true)),
            VirtualKeyCode::Escape => cmd_sender(Cmd::ChangeMode(Mode::Normal)),
            _ => {}
        },

        // Command
        (Mode::Command, InputMsg::CharPressed(c)) if is_valid_key(c) => {
            cmd_sender(Cmd::InsertChar(c, true))
        }
        (Mode::Command, InputMsg::KeyPressed(key)) => match key {
            VirtualKeyCode::Back => cmd_sender(Cmd::DeleteChar(DeleteDirection::Before)),
            VirtualKeyCode::Return => cmd_sender(Cmd::Submit),
            VirtualKeyCode::Escape => cmd_sender(Cmd::ChangeMode(Mode::Normal)),
            _ => {}
        },

        (Mode::Jump, InputMsg::KeyPressed(key)) => {
            if key == VirtualKeyCode::Escape {
                cmd_sender(Cmd::ChangeMode(Mode::Normal));
            }
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

        (Mode::Skim, InputMsg::CharPressed('\n')) => cmd_sender(Cmd::MoveCursor(Direction::Down)),
        (Mode::Skim, InputMsg::CharPressed('\u{b}')) => cmd_sender(Cmd::MoveCursor(Direction::Up)),

        (Mode::Skim, InputMsg::CharPressed(c)) if is_valid_key(c) => {
            cmd_sender(Cmd::InsertChar(c, true))
        }
        (Mode::Skim, InputMsg::KeyPressed(key)) => match key {
            VirtualKeyCode::Back => cmd_sender(Cmd::DeleteChar(DeleteDirection::Before)),
            VirtualKeyCode::Return => cmd_sender(Cmd::Submit),
            VirtualKeyCode::Escape => cmd_sender(Cmd::ChangeMode(Mode::Normal)),
            _ => {}
        },

        // Normal
        (Mode::Normal, InputMsg::CharPressed(c)) => match c {
            '\u{10}' => cmd_sender(Cmd::ChangeMode(Mode::Skim)),
            'h' => cmd_sender(Cmd::MoveCursor(Direction::Left)),
            'l' => cmd_sender(Cmd::MoveCursor(Direction::Right)),
            'k' => cmd_sender(Cmd::MoveCursor(Direction::Up)),
            'j' => cmd_sender(Cmd::MoveCursor(Direction::Down)),
            'i' => cmd_sender(Cmd::ChangeMode(Mode::Insert)),
            'I' => {
                cmd_sender(Cmd::Jump(JumpType::StartOfLine));
                cmd_sender(Cmd::ChangeMode(Mode::Insert));
            },
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
            'o' | 'O' => {
                if c == 'O' {
                    cmd_sender(Cmd::MoveCursor(Direction::Up));
                }
                cmd_sender(Cmd::Jump(JumpType::EndOfLine));
                cmd_sender(Cmd::InsertChar('\n', false));
                cmd_sender(Cmd::MoveCursor(Direction::Down));
                // Reset the saved x value
                cmd_sender(Cmd::MoveCursor(Direction::Left));
                cmd_sender(Cmd::MoveCursor(Direction::Right));

                cmd_sender(Cmd::ChangeMode(Mode::Insert));
            }
            'g' => {
                cmd_sender(Cmd::ChangeMode(Mode::Jump));
            }
            '>' => {
                cmd_sender(Cmd::Jump(JumpType::StartOfLine));
                cmd_sender(Cmd::InsertChar('\t', true));
            }
            _ => {}
        },
        _ => {}
    }
}
