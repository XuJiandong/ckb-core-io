use alloc::{boxed::Box, fmt};
use core::result;

pub mod core_error {
    #[cfg(feature = "rust_before_181")]
    pub use crate::cherry_picking::error::Error;
    #[cfg(not(feature = "rust_before_181"))]
    pub use core::error::Error;
}

pub type Result<T> = result::Result<T, Error>;

pub enum Error {
    Os(i64),
    Simple(ErrorKind),
    SimpleMessage(&'static SimpleMessage),
    Custom(Box<Custom>),
}

impl Error {
    pub fn new_simple(kind: ErrorKind) -> Self {
        Self::Simple(kind)
    }
    pub fn new_custom(custom: Box<Custom>) -> Self {
        Self::Custom(custom)
    }
    pub const fn new_simple_message(msg: &'static SimpleMessage) -> Self {
        Self::SimpleMessage(msg)
    }
    pub fn new_os(code: i64) -> Self {
        Self::Os(code)
    }
}

impl fmt::Debug for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Debug::fmt(&self, f)
    }
}

#[macro_export]
macro_rules! const_io_error {
    ($kind:expr, $message:expr $(,)?) => {
        $crate::error::Error::from_static_message({
            const MESSAGE_DATA: $crate::error::SimpleMessage =
                $crate::error::SimpleMessage::new($kind, $message);
            &MESSAGE_DATA
        })
    };
}

#[allow(dead_code)]
impl Error {
    pub(crate) const INVALID_UTF8: Self =
        const_io_error!(ErrorKind::InvalidData, "stream did not contain valid UTF-8");

    pub(crate) const READ_EXACT_EOF: Self =
        const_io_error!(ErrorKind::UnexpectedEof, "failed to fill whole buffer");

    pub(crate) const UNKNOWN_THREAD_COUNT: Self = const_io_error!(
        ErrorKind::NotFound,
        "The number of hardware threads is not known for the target platform"
    );

    pub(crate) const UNSUPPORTED_PLATFORM: Self = const_io_error!(
        ErrorKind::Unsupported,
        "operation not supported on this platform"
    );

    pub(crate) const WRITE_ALL_EOF: Self =
        const_io_error!(ErrorKind::WriteZero, "failed to write whole buffer");

    pub(crate) const ZERO_TIMEOUT: Self =
        const_io_error!(ErrorKind::InvalidInput, "cannot set a 0 duration timeout");
}

impl From<alloc::ffi::NulError> for Error {
    fn from(_: alloc::ffi::NulError) -> Error {
        const_io_error!(ErrorKind::InvalidInput, "data provided contains a nul byte")
    }
}

impl From<alloc::collections::TryReserveError> for Error {
    fn from(_: alloc::collections::TryReserveError) -> Error {
        ErrorKind::OutOfMemory.into()
    }
}

#[repr(align(4))]
#[derive(Debug)]
pub struct SimpleMessage {
    kind: ErrorKind,
    message: &'static str,
}

impl SimpleMessage {
    pub(crate) const fn new(kind: ErrorKind, message: &'static str) -> Self {
        Self { kind, message }
    }
}

#[derive(Debug)]
#[repr(align(4))]
pub struct Custom {
    kind: ErrorKind,
    error: Box<dyn core_error::Error + Send + Sync>,
}
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[allow(deprecated)]
#[non_exhaustive]
pub enum ErrorKind {
    NotFound,
    PermissionDenied,
    ConnectionRefused,
    ConnectionReset,
    HostUnreachable,
    NetworkUnreachable,
    ConnectionAborted,
    NotConnected,
    AddrInUse,
    AddrNotAvailable,
    NetworkDown,
    BrokenPipe,
    AlreadyExists,
    WouldBlock,
    NotADirectory,
    IsADirectory,
    DirectoryNotEmpty,
    ReadOnlyFilesystem,
    FilesystemLoop,
    StaleNetworkFileHandle,
    InvalidInput,
    InvalidData,
    TimedOut,
    WriteZero,
    StorageFull,
    NotSeekable,
    FilesystemQuotaExceeded,
    FileTooLarge,
    ResourceBusy,
    ExecutableFileBusy,
    Deadlock,
    CrossesDevices,
    TooManyLinks,
    InvalidFilename,
    ArgumentListTooLong,
    Interrupted,
    Unsupported,
    UnexpectedEof,
    OutOfMemory,
    Other,
    Uncategorized,
}

impl ErrorKind {
    pub(crate) fn as_str(&self) -> &'static str {
        use ErrorKind::*;
        match *self {
            AddrInUse => "address in use",
            AddrNotAvailable => "address not available",
            AlreadyExists => "entity already exists",
            ArgumentListTooLong => "argument list too long",
            BrokenPipe => "broken pipe",
            ConnectionAborted => "connection aborted",
            ConnectionRefused => "connection refused",
            ConnectionReset => "connection reset",
            CrossesDevices => "cross-device link or rename",
            Deadlock => "deadlock",
            DirectoryNotEmpty => "directory not empty",
            ExecutableFileBusy => "executable file busy",
            FileTooLarge => "file too large",
            FilesystemLoop => "filesystem loop or indirection limit (e.g. symlink loop)",
            FilesystemQuotaExceeded => "filesystem quota exceeded",
            HostUnreachable => "host unreachable",
            Interrupted => "operation interrupted",
            InvalidData => "invalid data",
            InvalidFilename => "invalid filename",
            InvalidInput => "invalid input parameter",
            IsADirectory => "is a directory",
            NetworkDown => "network down",
            NetworkUnreachable => "network unreachable",
            NotADirectory => "not a directory",
            NotConnected => "not connected",
            NotFound => "entity not found",
            NotSeekable => "seek on unseekable file",
            Other => "other error",
            OutOfMemory => "out of memory",
            PermissionDenied => "permission denied",
            ReadOnlyFilesystem => "read-only filesystem or storage medium",
            ResourceBusy => "resource busy",
            StaleNetworkFileHandle => "stale network file handle",
            StorageFull => "no storage space",
            TimedOut => "timed out",
            TooManyLinks => "too many links",
            Uncategorized => "uncategorized error",
            UnexpectedEof => "unexpected end of file",
            Unsupported => "unsupported",
            WouldBlock => "operation would block",
            WriteZero => "write zero",
        }
    }
}

impl fmt::Display for ErrorKind {
    fn fmt(&self, fmt: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt.write_str(self.as_str())
    }
}
impl From<ErrorKind> for Error {
    #[inline]
    fn from(kind: ErrorKind) -> Error {
        Error::new_simple(kind)
    }
}

impl Error {
    #[inline(never)]
    pub fn new<E>(kind: ErrorKind, error: E) -> Error
    where
        E: Into<Box<dyn core_error::Error + Send + Sync>>,
    {
        Self::_new(kind, error.into())
    }
    pub fn other<E>(error: E) -> Error
    where
        E: Into<Box<dyn core_error::Error + Send + Sync>>,
    {
        Self::_new(ErrorKind::Other, error.into())
    }

    fn _new(kind: ErrorKind, error: Box<dyn core_error::Error + Send + Sync>) -> Error {
        Error::new_custom(Box::new(Custom { kind, error }))
    }
    #[inline]
    pub(crate) const fn from_static_message(msg: &'static SimpleMessage) -> Error {
        Error::new_simple_message(msg)
    }

    #[inline]
    pub fn from_raw_os_error(code: i64) -> Error {
        Error::new_os(code)
    }

    #[inline]
    pub fn raw_os_error(&self) -> Option<i64> {
        match self {
            Error::Os(i) => Some(*i),
            Error::Custom(..) => None,
            Error::Simple(..) => None,
            Error::SimpleMessage(..) => None,
        }
    }
    #[inline]
    pub fn get_ref(&self) -> Option<&(dyn core_error::Error + Send + Sync + 'static)> {
        match self {
            Error::Os(..) => None,
            Error::Simple(..) => None,
            Error::SimpleMessage(..) => None,
            Error::Custom(c) => Some(&*c.error),
        }
    }

    #[must_use]
    #[inline]
    pub fn get_mut(&mut self) -> Option<&mut (dyn core_error::Error + Send + Sync + 'static)> {
        match self {
            Error::Os(..) => None,
            Error::Simple(..) => None,
            Error::SimpleMessage(..) => None,
            Error::Custom(c) => Some(&mut *c.error),
        }
    }
    #[inline]
    pub fn into_inner(self) -> Option<Box<dyn core_error::Error + Send + Sync>> {
        match self {
            Error::Os(..) => None,
            Error::Simple(..) => None,
            Error::SimpleMessage(..) => None,
            Error::Custom(c) => Some(c.error),
        }
    }
    #[cfg(not(feature = "rust_before_181"))]
    pub fn downcast<E>(self) -> result::Result<E, Self>
    where
        E: core_error::Error + Send + Sync + 'static,
    {
        match self {
            Error::Custom(b) if b.error.is::<E>() => {
                let res = b.error.downcast::<E>();
                Ok(*res.unwrap())
            }
            err => Err(err),
        }
    }

    #[must_use]
    #[inline]
    pub fn kind(&self) -> ErrorKind {
        match self {
            Error::Os(..) => ErrorKind::Other,
            Error::Custom(c) => c.kind,
            Error::Simple(kind) => *kind,
            Error::SimpleMessage(m) => m.kind,
        }
    }

    #[inline]
    pub(crate) fn is_interrupted(&self) -> bool {
        match self {
            Error::Os(..) => false,
            Error::Custom(c) => c.kind == ErrorKind::Interrupted,
            Error::Simple(kind) => *kind == ErrorKind::Interrupted,
            Error::SimpleMessage(m) => m.kind == ErrorKind::Interrupted,
        }
    }
}

impl fmt::Display for Error {
    fn fmt(&self, fmt: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::Os(code) => {
                write!(fmt, "os error {code}")
            }
            Error::Custom(ref c) => c.error.fmt(fmt),
            Error::Simple(kind) => write!(fmt, "{}", kind.as_str()),
            Error::SimpleMessage(msg) => msg.message.fmt(fmt),
        }
    }
}

impl core_error::Error for Error {
    #[allow(deprecated, deprecated_in_future)]
    fn description(&self) -> &str {
        match self {
            Error::Os(..) | Error::Simple(..) => self.kind().as_str(),
            Error::SimpleMessage(msg) => msg.message,
            Error::Custom(c) => c.error.description(),
        }
    }

    #[allow(deprecated)]
    fn cause(&self) -> Option<&dyn core_error::Error> {
        match self {
            Error::Os(..) => None,
            Error::Simple(..) => None,
            Error::SimpleMessage(..) => None,
            Error::Custom(c) => c.error.cause(),
        }
    }

    fn source(&self) -> Option<&(dyn core_error::Error + 'static)> {
        match self {
            Error::Os(..) => None,
            Error::Simple(..) => None,
            Error::SimpleMessage(..) => None,
            Error::Custom(c) => c.error.source(),
        }
    }
}

fn _assert_error_is_sync_send() {
    fn _is_sync_send<T: Sync + Send>() {}
    _is_sync_send::<Error>();
}
