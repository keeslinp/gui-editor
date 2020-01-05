use crate::msg::Msg;
#[derive(Debug, PartialEq, Clone)]
pub enum CommandError {
    MissingArg,
    UnknownCommand(String),
}

#[derive(Debug, PartialEq, Clone)]
pub enum Error {
    IOError(String),
    Command(CommandError),
    WalkDir(String),
    NeedFilePath,
    YAML(String),
    BuildingSyntax,
    Highlighting,
    MsgFailed,
}

impl CommandError {
    pub fn as_string(&self) -> String {
        use CommandError::*;
        match self {
            MissingArg => "Missing an argument".to_owned(),
            UnknownCommand(cmd) => format!("Unknown command: \"{}\"", cmd),
        }
    }
}

impl Error {
    pub fn as_string(&self) -> String {
        use Error::*;
        match self {
            IOError(err) => format!("IOError: {}", err),
            Command(err) => err.as_string(),
            WalkDir(err) => format!("WalkDir: {}", err),
            NeedFilePath => "Need a file path to save a new buffer".to_owned(),
            YAML(err) => format!("YAML error: {}", err),
            BuildingSyntax => "Failed to build syntax".to_owned(),
            Highlighting => "Failed to highlight".to_owned(),
            MsgFailed => "Failed to pass message to winit".to_owned(),
        }
    }
}

impl From<walkdir::Error> for Error {
    fn from(err: walkdir::Error) -> Error {
        Error::WalkDir(err.to_string())
    }
}

impl From<std::io::Error> for Error {
    fn from(err: std::io::Error) -> Error {
        Error::IOError(err.to_string())
    }
}

impl From<CommandError> for Error {
    fn from(err: CommandError) -> Error {
        Error::Command(err)
    }
}

impl From<serde_yaml::Error> for Error {
    fn from(err: serde_yaml::Error) -> Error {
        Error::YAML(err.to_string())
    }
}

impl From<winit::event_loop::EventLoopClosed<Msg>> for Error {
    fn from(err: winit::event_loop::EventLoopClosed<Msg>) -> Error {
        Error::MsgFailed // TODO: Fix Cycle so Msg can live here
    }
}

pub type Result<T> = core::result::Result<T, Error>;
