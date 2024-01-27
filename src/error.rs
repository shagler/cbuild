
/// Custom error type
#[derive(Debug)]
pub enum Error {
    /// IO Error
    IOError(std::io::Error),
}

/// Implement the formatter for our custom error type
impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Error::IOError(err) =>
                writeln!(f, "IO Error: {}", err);
        }
    }
}

/// Implement standard error trait and conversion from other error types
impl std::error::Error for Error {}
impl From<std::io::Error> for Error {
    fn from(err: std::io::Error) -> Self {
        Error::IOError(self)
    }
}

/// Custom Result type alias
pub type Result<T> = std::result::Result<T, Error>;
