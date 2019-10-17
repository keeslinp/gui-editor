#[derive(Debug, PartialEq, Clone)]
pub enum CommandError {
    MissingArg,
    UnknownCommand(String),
}

#[derive(Debug, PartialEq, Clone)]
pub enum Error {
    IOError(String),
    Command(CommandError),
}

impl CommandError {
    pub fn as_string(&self) -> String {
        use CommandError::*;
        match self {
            MissingArg => {
                "Missing an argument".to_owned()
            },
            UnknownCommand(cmd) => {
                format!("Unknown command: {}", cmd)
            },
        }
    }
}

impl Error {
    pub fn as_string(&self) -> String {
        use Error::*;
        match self {
            IOError(err) => {
                format!("IOError: {:#}", err)
            },
            Command(err) => {
                err.as_string()
            },
        }
    }
}

impl From<std::io::Error> for Error {
    fn from(err: std::io::Error) -> Error{
        Error::IOError(err.to_string())
    }
}

impl From<CommandError> for Error {
    fn from(err: CommandError) -> Error {
        Error::Command(err)
    }
}

pub type Result<T> = core::result::Result<T, Error>;
