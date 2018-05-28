use std::error;
use std::fmt;

/// List of all errors.
#[derive(Debug)]
pub enum Error {
    /// Only `svg` and `svgz` suffixes are supported.
    InvalidFileSuffix,

    /// Failed to open the provided file.
    FileOpenFailed,

    /// Only UTF-8 content are supported.
    NotAnUtf8Str,

    /// Compressed SVG must use the GZip algorithm.
    MalformedGZip,
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Error::InvalidFileSuffix => {
                write!(f, "invalid file suffix")
            }
            Error::FileOpenFailed => {
                write!(f, "failed to open the provided file")
            }
            Error::NotAnUtf8Str => {
                write!(f, "provided data has not an UTF-8 encoding")
            }
            Error::MalformedGZip => {
                write!(f, "provided data has a malformed GZip content")
            }
        }
    }
}

impl error::Error for Error {
    fn description(&self) -> &str {
        "an SVG simplification error"
    }
}
