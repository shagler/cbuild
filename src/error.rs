
use thiserror::Error;

/// Custom error type
#[derive(Error, Debug)]
pub enum Error {
    #[error("IO Error: {0}")]
    IO(#[from] std::io::Error),

    #[error("No configuration file found")]
    NoConfig(),

    #[error("Configuration error: {0}")]
    Config(String),

    #[error("Argument error: {0}")]
    Arguments(String),

    #[error("Project creation error: {0}")]
    ProjectCreation(String),

    #[error("Library error: {0}")]
    Library(String),

    #[error("Build failed")]
    BuildFailed(),
}

/// Custom Result type alias
pub type Result<T> = std::result::Result<T, Error>;
