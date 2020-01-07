#[derive(thiserror::Error, Debug, PartialEq, Clone)]
pub enum Error {
    #[error("Need a file path to save a new buffer")]
    NeedFilePath,
    #[error("Missing and argument")]
    MissingArg,
    #[error("Unknown command: {0}")]
    UnknownCommand(String),
    #[error("Failed to parse syntax")]
    BuildingSyntax,
    // #[error("Something went wrong highlighting")]
    // Highlighting,
}

impl Error {
    pub fn anyhow(self) -> anyhow::Error {
        anyhow::Error::new(self)
    }
}
