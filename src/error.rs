
/// Custom error type
#[derive(Debug)]
pub enum Error {
    /// Default Error
    Error(&'static str),

    /// IO Error
    IOError(std::io::Error),
}

/// Implement the formatter for our custom error type
impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Error::Error(err) =>
                writeln!(f, "Error: {}", err),
            Error::IOError(err) =>
                writeln!(f, "IO Error: {}", err),
        }
    }
}

/// Implement standard error trait and conversion from other error types
impl std::error::Error for Error {}
impl From<std::io::Error> for Error {
    fn from(err: std::io::Error) -> Self {
        Error::IOError(err)
    }
}

/// Custom Result type alias
pub type Result<T> = std::result::Result<T, Error>;
