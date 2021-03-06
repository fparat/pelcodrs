/// Crate error type
#[derive(Debug)]
pub enum Error {
    /// Invalid parameter
    InvalidValue(String),
    /// IO error
    Io(std::io::Error),
}

impl Error {
    pub(crate) fn invalid_value<T: ToString>(description: T) -> Error {
        Error::InvalidValue(description.to_string())
    }
}

impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Error::InvalidValue(_) => None,
            Error::Io(e) => Some(e),
        }
    }
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::InvalidValue(s) => write!(f, "Invalid value: {}", s),
            Error::Io(e) => write!(f, "IO error: {}", e),
        }
    }
}

impl From<std::io::Error> for Error {
    fn from(e: std::io::Error) -> Self {
        Error::Io(e)
    }
}

/// Result type used in the crate.
pub type Result<T> = std::result::Result<T, Error>;
