mod bufreader;
mod bufwriter;
mod linewriter;
mod linewritershim;
pub use self::{bufreader::BufReader, bufwriter::BufWriter, linewriter::LineWriter};
use crate::error;
use crate::Error;
use alloc::fmt;
pub use bufwriter::WriterPanicked;
use linewritershim::LineWriterShim;
#[derive(Debug)]
pub struct IntoInnerError<W>(W, Error);
impl<W> IntoInnerError<W> {
    fn new(writer: W, error: Error) -> Self {
        Self(writer, error)
    }
    fn new_wrapped<W2>(self, f: impl FnOnce(W) -> W2) -> IntoInnerError<W2> {
        let Self(writer, error) = self;
        IntoInnerError::new(f(writer), error)
    }
    pub fn error(&self) -> &Error {
        &self.1
    }
    pub fn into_inner(self) -> W {
        self.0
    }
    pub fn into_error(self) -> Error {
        self.1
    }
    pub fn into_parts(self) -> (Error, W) {
        (self.1, self.0)
    }
}
impl<W> From<IntoInnerError<W>> for Error {
    fn from(iie: IntoInnerError<W>) -> Error {
        iie.1
    }
}
impl<W: Send + fmt::Debug> core::error::Error for IntoInnerError<W> {
    #[allow(deprecated, deprecated_in_future)]
    fn description(&self) -> &str {
        error::Error::description(self.error())
    }
}
impl<W> fmt::Display for IntoInnerError<W> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.error().fmt(f)
    }
}
