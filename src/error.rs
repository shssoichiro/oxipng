use std::error::Error;
use std::fmt;

#[derive(Debug, Clone)]
pub enum PngError {
    DeflatedDataTooLong(usize),
    Other(Box<str>),
}

impl Error for PngError {
    // deprecated
    fn description(&self) -> &str {
        ""
    }
}

impl fmt::Display for PngError {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            PngError::DeflatedDataTooLong(_) => f.write_str("deflated data too long"),
            PngError::Other(ref s) => f.write_str(s),
        }
    }
}

impl PngError {
    #[inline]
    pub fn new(description: &str) -> PngError {
        PngError::Other(description.into())
    }
}
