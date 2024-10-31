use alloc::collections::TryReserveError;

#[derive(Debug)]
pub enum Error {
    OutOfMemory,
    WriteZero,
    WriteAllEof,
    Interrupted,
    InvalidInput,
    ReadExactEof,
    InvalidUtf8,
}

pub type Result<T> = core::result::Result<T, Error>;

impl Error {
    pub fn is_interrupted(&self) -> bool {
        matches!(self, Error::Interrupted)
    }
}

impl From<TryReserveError> for Error {
    fn from(_value: TryReserveError) -> Self {
        Error::OutOfMemory
    }
}
