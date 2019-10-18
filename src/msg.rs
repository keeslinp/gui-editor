#![allow(dead_code)]

use crate::{error::Error, mode::Mode};
use winit::event::VirtualKeyCode;

#[derive(PartialEq, Debug)]
pub enum InputMsg {
    CharPressed(char),
    KeyPressed(VirtualKeyCode),
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum Direction {
    Left,
    Right,
    Up,
    Down,
}

#[derive(Debug, PartialEq)]
pub enum DeleteDirection {
    Before,
    After,
}

#[derive(Debug, PartialEq)]
pub enum JumpType {
    EndOfLine,
    StartOfLine,
    StartOfFile,
    EndOfFile,
}

#[derive(Debug, PartialEq)]
pub enum Cmd {
    MoveCursor(Direction),
    Quit,
    ChangeMode(Mode),
    InsertChar(char, bool),
    SetError(Error),
    Submit,
    // InsertCharAtPoint(char, Point),
    // InsertStringAtPoint(String, Point),
    // DeleteCharRange(Point, Point),
    DeleteChar(DeleteDirection),
    Jump(JumpType),
    // RunCommand,
    // WriteBuffer(std::path::PathBuf),
    LoadFile(std::path::PathBuf),
    // BufferLoaded,
    // BufferModified,
    // SearchFiles,
    // CleanRender,
    // Yank,
    // YankValue(String),
    // Paste,
    // PasteAtPoint(Point),
}

#[derive(PartialEq, Debug)]
pub enum Msg {
    Input(InputMsg),
    Cmd(Cmd),
}
