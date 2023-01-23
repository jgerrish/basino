//! Error results that can occur working with basino functions
#![warn(missing_docs)]
#![warn(unsafe_code)]

use core::fmt::{Debug, Display, Formatter, Result};
use ufmt::{uDebug, uWrite};

/// The kinds of errors that can occur working with stacks
#[derive(Eq, PartialEq)]
pub enum ErrorKind {
    /// A stack overflow would occur if an item is pushed
    StackOverflow,
    /// A stack underflow would occur if an item is popped
    StackUnderflow,
    /// An unknown error type
    Unknown,
}

impl uDebug for ErrorKind {
    fn fmt<T>(&self, f: &mut ufmt::Formatter<'_, T>) -> core::result::Result<(), T::Error>
    where
        T: uWrite + ?Sized,
    {
        match self {
            ErrorKind::StackOverflow => f.write_str("Stack overflow error"),
            ErrorKind::StackUnderflow => f.write_str("Stack underflow error"),
            ErrorKind::Unknown => f.write_str("Unknown error"),
        }
    }
}

impl Display for ErrorKind {
    fn fmt(&self, f: &mut Formatter) -> Result {
        match self {
            ErrorKind::StackOverflow => write!(f, "A stack overflow occurred"),
            ErrorKind::StackUnderflow => write!(f, "A stack underflow occurred"),
            ErrorKind::Unknown => write!(f, "An unknown error occurred"),
        }
    }
}

/// An error that can occur when working with a temperature server
pub struct Error {
    kind: ErrorKind,
}

impl Error {
    /// Create a new Error with a given ErrorKind variant
    pub fn new(kind: ErrorKind) -> Error {
        Error { kind }
    }
}

impl Debug for Error {
    fn fmt(&self, f: &mut Formatter) -> Result {
        write!(f, "{}", self.kind)
    }
}

impl Display for Error {
    fn fmt(&self, f: &mut Formatter) -> Result {
        write!(f, "{}", self.kind)
    }
}
