use std::{error, fmt, io};

#[derive(Debug)]
pub enum Error {
    StdIoError(io::Error),
    TransformError(String),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::StdIoError(e) => write!(f, "{:?}", e),
            Self::TransformError(e) => write!(f, "{:?}", e),
        }
    }
}

impl error::Error for Error {
    fn source(&self) -> Option<&(dyn error::Error + 'static)> {
        match self {
            Self::StdIoError(e) => Some(e),
            Self::TransformError(_) => None,
        }
    }
}

impl From<io::Error> for Error {
    fn from(io_err: io::Error) -> Self {
        Self::StdIoError(io_err)
    }
}
