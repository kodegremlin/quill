use std::{fmt, io, time::SystemTimeError};

pub type Result<T> = std::result::Result<T, Error>;

/// Errors the terminal can encounter.
pub enum Error {
    IoError(io::Error),
    SystemTimeError(SystemTimeError),
    TooSmallWindow(usize, usize),
    UnknownWindowSize,
    NotUtf8Input(Vec<u8>),
    ControlCharInText(char),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use Error::*;
        match self {
            SystemTimeError(err) => write!(f, "{}", err),

            IoError(err) => write!(f, "{}", err),

            TooSmallWindow(width, height) => write!(
                f,
                "Screen {}x{} is too small. At least 1x3 is necessary in width x height",
                width, height
            ),
            UnknownWindowSize => write!(f, "Could not detect terminal window size"),

            NotUtf8Input(items) => {
                write!(f, "Cannot handle non-UTF8 multi-byte input sequence: ")?;
                for byte in items.iter() {
                    write!(f, "\\x{:x}", byte)?;
                }
                Ok(())
            }
            ControlCharInText(ch) => write!(f, "Invalid character for text is included: {:?}", ch),
        }
    }
}

impl From<io::Error> for Error {
    fn from(value: io::Error) -> Self {
        Error::IoError(value)
    }
}

impl From<SystemTimeError> for Error {
    fn from(value: SystemTimeError) -> Self {
        Error::SystemTimeError(value)
    }
}
