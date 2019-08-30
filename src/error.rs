#[derive(Debug, PartialEq, Clone, Copy)]
pub enum ErrorKind {
    InvalidValue,
}

#[derive(Debug, PartialEq, Clone)]
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

pub type Result<T> = std::result::Result<T, Error>;
