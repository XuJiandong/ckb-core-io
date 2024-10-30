#[derive(Debug)]
pub enum Error {
    OutOfMemory,
    WriteZero,
}

pub type Result<T> = core::result::Result<T, Error>;
