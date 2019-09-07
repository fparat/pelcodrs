/// Possible types of error.
#[derive(Debug)]
pub enum ErrorKind {
    /// Invalid parameter.
    InvalidValue,

    /// IO error
    Io(std::io::Error),
}

/// Error type for the crate.
#[derive(Debug)]
pub struct Error {
    kind: ErrorKind,
    description: String,
}

impl Error {
    pub fn new(kind: ErrorKind, description: &str) -> Error {
        Error {
            kind,
            description: String::from(description),
        }
    }
}

impl std::error::Error for Error {
    fn description(&self) -> &str {
        &self.description
    }

    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        None
    }
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}", &self.description)
    }
}

impl From<std::io::Error> for Error {
    fn from(e: std::io::Error) -> Self {
        Error::new(ErrorKind::Io(e), "IO error")
    }
}

/// Result type used in the crate.
pub type Result<T> = std::result::Result<T, Error>;
