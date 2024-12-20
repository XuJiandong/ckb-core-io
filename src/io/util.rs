#[cfg(test)]
mod tests;

use crate::io;
use crate::io::{BorrowedCursor, BufRead, Read, Seek, SeekFrom, SizeHint, Write};
use alloc::fmt;
use alloc::string::String;
use alloc::vec::Vec;

/// `Empty` ignores any data written via [`Write`], and will always be empty
/// (returning zero bytes) when read via [`Read`].
///
/// This struct is generally created by calling [`empty()`]. Please
/// see the documentation of [`empty()`] for more details.
#[non_exhaustive]
#[derive(Copy, Clone, Debug, Default)]
pub struct Empty;

/// Creates a value that is always at EOF for reads, and ignores all data written.
///
/// All calls to [`write`] on the returned instance will return [`Ok(buf.len())`]
/// and the contents of the buffer will not be inspected.
///
/// All calls to [`read`] from the returned reader will return [`Ok(0)`].
///
/// [`Ok(buf.len())`]: Ok
/// [`Ok(0)`]: Ok
///
/// [`write`]: Write::write
/// [`read`]: Read::read
///
/// # Examples
///
/// ```rust
/// use std::io::{self, Write};
///
/// let buffer = vec![1, 2, 3, 5, 8];
/// let num_bytes = io::empty().write(&buffer).unwrap();
/// assert_eq!(num_bytes, 5);
/// ```
///
///
/// ```rust
/// use std::io::{self, Read};
///
/// let mut buffer = String::new();
/// io::empty().read_to_string(&mut buffer).unwrap();
/// assert!(buffer.is_empty());
/// ```
#[must_use]
pub const fn empty() -> Empty {
    Empty
}
impl Read for Empty {
    #[inline]
    fn read(&mut self, _buf: &mut [u8]) -> io::Result<usize> {
        Ok(0)
    }

    #[inline]
    fn read_buf(&mut self, _cursor: BorrowedCursor<'_>) -> io::Result<()> {
        Ok(())
    }
}
impl BufRead for Empty {
    #[inline]
    fn fill_buf(&mut self) -> io::Result<&[u8]> {
        Ok(&[])
    }
    #[inline]
    fn consume(&mut self, _n: usize) {}
}
impl Seek for Empty {
    fn seek(&mut self, _pos: SeekFrom) -> io::Result<u64> {
        Ok(0)
    }

    fn stream_len(&mut self) -> io::Result<u64> {
        Ok(0)
    }

    fn stream_position(&mut self) -> io::Result<u64> {
        Ok(0)
    }
}

impl SizeHint for Empty {
    #[inline]
    fn lower_bound(&self) -> usize {
        0
    }
    #[inline]
    fn upper_bound(&self) -> Option<usize> {
        Some(0)
    }
}
impl Write for Empty {
    #[inline]
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        Ok(buf.len())
    }
    #[inline]
    fn is_write_vectored(&self) -> bool {
        true
    }

    #[inline]
    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}
impl Write for &Empty {
    #[inline]
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        Ok(buf.len())
    }
    #[inline]
    fn is_write_vectored(&self) -> bool {
        true
    }

    #[inline]
    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

/// A reader which yields one byte over and over and over and over and over and...
///
/// This struct is generally created by calling [`repeat()`]. Please
/// see the documentation of [`repeat()`] for more details.
pub struct Repeat {
    byte: u8,
}

/// Creates an instance of a reader that infinitely repeats one byte.
///
/// All reads from this reader will succeed by filling the specified buffer with
/// the given byte.
///
/// # Examples
///
/// ```
/// use std::io::{self, Read};
///
/// let mut buffer = [0; 3];
/// io::repeat(0b101).read_exact(&mut buffer).unwrap();
/// assert_eq!(buffer, [0b101, 0b101, 0b101]);
/// ```
#[must_use]
pub const fn repeat(byte: u8) -> Repeat {
    Repeat { byte }
}
impl Read for Repeat {
    #[inline]
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        for slot in &mut *buf {
            *slot = self.byte;
        }
        Ok(buf.len())
    }

    fn read_buf(&mut self, mut buf: BorrowedCursor<'_>) -> io::Result<()> {
        // SAFETY: No uninit bytes are being written
        for slot in unsafe { buf.as_mut() } {
            slot.write(self.byte);
        }

        let remaining = buf.capacity();

        // SAFETY: the entire unfilled portion of buf has been initialized
        unsafe {
            buf.advance_unchecked(remaining);
        }

        Ok(())
    }

    /// This function is not supported by `io::Repeat`, because there's no end of its data
    fn read_to_end(&mut self, _: &mut Vec<u8>) -> io::Result<usize> {
        Err(io::Error::from(io::ErrorKind::OutOfMemory))
    }

    /// This function is not supported by `io::Repeat`, because there's no end of its data
    fn read_to_string(&mut self, _: &mut String) -> io::Result<usize> {
        Err(io::Error::from(io::ErrorKind::OutOfMemory))
    }
}

impl SizeHint for Repeat {
    #[inline]
    fn lower_bound(&self) -> usize {
        usize::MAX
    }

    #[inline]
    fn upper_bound(&self) -> Option<usize> {
        None
    }
}
impl fmt::Debug for Repeat {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Repeat").finish_non_exhaustive()
    }
}

/// A writer which will move data into the void.
///
/// This struct is generally created by calling [`sink()`]. Please
/// see the documentation of [`sink()`] for more details.
#[non_exhaustive]
#[derive(Copy, Clone, Debug, Default)]
pub struct Sink;

/// Creates an instance of a writer which will successfully consume all data.
///
/// All calls to [`write`] on the returned instance will return [`Ok(buf.len())`]
/// and the contents of the buffer will not be inspected.
///
/// [`write`]: Write::write
/// [`Ok(buf.len())`]: Ok
///
/// # Examples
///
/// ```rust
/// use std::io::{self, Write};
///
/// let buffer = vec![1, 2, 3, 5, 8];
/// let num_bytes = io::sink().write(&buffer).unwrap();
/// assert_eq!(num_bytes, 5);
/// ```
#[must_use]
pub const fn sink() -> Sink {
    Sink
}
impl Write for Sink {
    #[inline]
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        Ok(buf.len())
    }

    #[inline]
    fn is_write_vectored(&self) -> bool {
        true
    }

    #[inline]
    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}
impl Write for &Sink {
    #[inline]
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        Ok(buf.len())
    }
    #[inline]
    fn is_write_vectored(&self) -> bool {
        true
    }

    #[inline]
    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}
