#[derive(Debug)]
pub enum ShiError {
    Io(String),
    Db(String),
    ParseInt,
    IntoInner(String),
    FromUtf8(String),
    Copy,
    Input,
}

impl std::error::Error for ShiError {}

impl std::fmt::Display for ShiError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let printable = match self {
            ShiError::Io(s) => s.as_ref(),
            ShiError::Db(s) => s.as_ref(),
            ShiError::ParseInt => "Cannot parse input as link.",
            ShiError::IntoInner(s) => s.as_ref(),
            ShiError::FromUtf8(s) => s.as_ref(),
            ShiError::Copy => "Cannot connect to the clipboard.",
            ShiError::Input => "Exit.",
        };
        write!(f, "{}", printable)
    }
}

impl From<std::io::Error> for ShiError {
    fn from(err: std::io::Error) -> Self {
        ShiError::Io(err.to_string())
    }
}

impl From<sqlite::Error> for ShiError {
    fn from(err: sqlite::Error) -> Self {
        ShiError::Db(err.to_string())
    }
}

impl From<std::num::ParseIntError> for ShiError {
    fn from(_err: std::num::ParseIntError) -> Self {
        ShiError::ParseInt
    }
}

impl<T> From<csv::IntoInnerError<T>> for ShiError {
    fn from(err: csv::IntoInnerError<T>) -> Self {
        ShiError::IntoInner(err.to_string())
    }
}

impl From<std::string::FromUtf8Error> for ShiError {
    fn from(err: std::string::FromUtf8Error) -> Self {
        ShiError::FromUtf8(err.to_string())
    }
}

impl From<Box<dyn std::error::Error + std::marker::Send + std::marker::Sync>> for ShiError {
    fn from(_err: Box<dyn std::error::Error + std::marker::Send + std::marker::Sync>) -> Self {
        ShiError::Copy
    }
}
