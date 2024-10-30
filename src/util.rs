use crate::{BorrowedCursor, BufRead, Read, Seek, SeekFrom, SizeHint, Write};
use alloc::fmt;
#[non_exhaustive]
#[derive(Copy, Clone, Debug, Default)]
pub struct Empty;
#[must_use]
pub const fn empty() -> Empty {
    Empty
}
impl Read for Empty {
    #[inline]
    fn read(&mut self, _buf: &mut [u8]) -> crate::Result<usize> {
        Ok(0)
    }
    #[inline]
    fn read_buf(&mut self, _cursor: BorrowedCursor<'_>) -> crate::Result<()> {
        Ok(())
    }
}
impl BufRead for Empty {
    #[inline]
    fn fill_buf(&mut self) -> crate::Result<&[u8]> {
        Ok(&[])
    }
    #[inline]
    fn consume(&mut self, _n: usize) {}
}
impl Seek for Empty {
    fn seek(&mut self, _pos: SeekFrom) -> crate::Result<u64> {
        Ok(0)
    }
    fn stream_len(&mut self) -> crate::Result<u64> {
        Ok(0)
    }
    fn stream_position(&mut self) -> crate::Result<u64> {
        Ok(0)
    }
}
impl SizeHint for Empty {
    #[inline]
    fn upper_bound(&self) -> Option<usize> {
        Some(0)
    }
}
impl Write for Empty {
    #[inline]
    fn write(&mut self, buf: &[u8]) -> crate::Result<usize> {
        Ok(buf.len())
    }
    #[inline]
    fn is_write_vectored(&self) -> bool {
        true
    }
    #[inline]
    fn flush(&mut self) -> crate::Result<()> {
        Ok(())
    }
}
impl Write for &Empty {
    #[inline]
    fn write(&mut self, buf: &[u8]) -> crate::Result<usize> {
        Ok(buf.len())
    }
    #[inline]
    fn is_write_vectored(&self) -> bool {
        true
    }
    #[inline]
    fn flush(&mut self) -> crate::Result<()> {
        Ok(())
    }
}
pub struct Repeat {
    byte: u8,
}
#[must_use]
pub const fn repeat(byte: u8) -> Repeat {
    Repeat { byte }
}
impl Read for Repeat {
    #[inline]
    fn read(&mut self, buf: &mut [u8]) -> crate::Result<usize> {
        for slot in &mut *buf {
            *slot = self.byte;
        }
        Ok(buf.len())
    }
    fn read_buf(&mut self, mut buf: BorrowedCursor<'_>) -> crate::Result<()> {
        for slot in unsafe { buf.as_mut() } {
            slot.write(self.byte);
        }
        let remaining = buf.capacity();
        unsafe {
            buf.advance_unchecked(remaining);
        }
        Ok(())
    }
    fn read_to_end(&mut self, _: &mut Vec<u8>) -> crate::Result<usize> {
        Err(crate::Error::from(crate::Error::OutOfMemory))
    }
    fn read_to_string(&mut self, _: &mut String) -> crate::Result<usize> {
        Err(crate::Error::from(crate::Error::OutOfMemory))
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
#[non_exhaustive]
#[derive(Copy, Clone, Debug, Default)]
pub struct Sink;
#[must_use]
pub const fn sink() -> Sink {
    Sink
}
impl Write for Sink {
    #[inline]
    fn write(&mut self, buf: &[u8]) -> crate::Result<usize> {
        Ok(buf.len())
    }
    #[inline]
    fn is_write_vectored(&self) -> bool {
        true
    }
    #[inline]
    fn flush(&mut self) -> crate::Result<()> {
        Ok(())
    }
}
impl Write for &Sink {
    #[inline]
    fn write(&mut self, buf: &[u8]) -> crate::Result<usize> {
        Ok(buf.len())
    }
    #[inline]
    fn is_write_vectored(&self) -> bool {
        true
    }
    #[inline]
    fn flush(&mut self) -> crate::Result<()> {
        Ok(())
    }
}
