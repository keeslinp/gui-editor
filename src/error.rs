use thiserror::Error;

#[derive(Error, Debug, PartialEq, Clone)]
pub enum Error {
    #[error("Need a file path to save a new buffer")]
    NeedFilePath,
    #[error("Missing and argument")]
    MissingArg,
    #[error("Unknown command: {0}")]
    UnknownCommand(String),
}
