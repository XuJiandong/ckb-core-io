mod buffer;
use crate::{
    uninlined_slow_read_byte, BorrowedCursor, BufRead, Read, Seek, SeekFrom, SizeHint,
    SpecReadByte, DEFAULT_BUF_SIZE,
};
use alloc::{fmt, string::String, vec::Vec};
use buffer::Buffer;
pub struct BufReader<R: ?Sized> {
    buf: Buffer,
    inner: R,
}
impl<R: Read> BufReader<R> {
    pub fn new(inner: R) -> BufReader<R> {
        BufReader::with_capacity(DEFAULT_BUF_SIZE, inner)
    }
    pub fn with_capacity(capacity: usize, inner: R) -> BufReader<R> {
        BufReader {
            inner,
            buf: Buffer::with_capacity(capacity),
        }
    }
}
impl<R: ?Sized> BufReader<R> {
    pub fn get_ref(&self) -> &R {
        &self.inner
    }
    pub fn get_mut(&mut self) -> &mut R {
        &mut self.inner
    }
    pub fn buffer(&self) -> &[u8] {
        self.buf.buffer()
    }
    pub fn capacity(&self) -> usize {
        self.buf.capacity()
    }
    pub fn into_inner(self) -> R
    where
        R: Sized,
    {
        self.inner
    }
    #[inline]
    pub(crate) fn discard_buffer(&mut self) {
        self.buf.discard_buffer()
    }
}
#[cfg(test)]
impl<R: ?Sized> BufReader<R> {
    pub fn initialized(&self) -> usize {
        self.buf.initialized()
    }
}
impl<R: ?Sized + Seek> BufReader<R> {
    pub fn seek_relative(&mut self, offset: i64) -> crate::Result<()> {
        let pos = self.buf.pos() as u64;
        if offset < 0 {
            if pos.checked_sub((-offset) as u64).is_some() {
                self.buf.unconsume((-offset) as usize);
                return Ok(());
            }
        } else if let Some(new_pos) = pos.checked_add(offset as u64) {
            if new_pos <= self.buf.filled() as u64 {
                self.buf.consume(offset as usize);
                return Ok(());
            }
        }
        self.seek(SeekFrom::Current(offset)).map(drop)
    }
}
impl<R> SpecReadByte for BufReader<R>
where
    Self: Read,
{
    #[inline]
    fn spec_read_byte(&mut self) -> Option<crate::Result<u8>> {
        let mut byte = 0;
        if self.buf.consume_with(1, |claimed| byte = claimed[0]) {
            return Some(Ok(byte));
        }
        uninlined_slow_read_byte(self)
    }
}
impl<R: ?Sized + Read> Read for BufReader<R> {
    fn read(&mut self, buf: &mut [u8]) -> crate::Result<usize> {
        if self.buf.pos() == self.buf.filled() && buf.len() >= self.capacity() {
            self.discard_buffer();
            return self.inner.read(buf);
        }
        let mut rem = self.fill_buf()?;
        let nread = rem.read(buf)?;
        self.consume(nread);
        Ok(nread)
    }
    fn read_buf(&mut self, mut cursor: BorrowedCursor<'_>) -> crate::Result<()> {
        if self.buf.pos() == self.buf.filled() && cursor.capacity() >= self.capacity() {
            self.discard_buffer();
            return self.inner.read_buf(cursor);
        }
        let prev = cursor.written();
        let mut rem = self.fill_buf()?;
        rem.read_buf(cursor.reborrow())?;
        self.consume(cursor.written() - prev);
        Ok(())
    }
    fn read_exact(&mut self, buf: &mut [u8]) -> crate::Result<()> {
        if self
            .buf
            .consume_with(buf.len(), |claimed| buf.copy_from_slice(claimed))
        {
            return Ok(());
        }
        crate::default_read_exact(self, buf)
    }
    fn read_buf_exact(&mut self, mut cursor: BorrowedCursor<'_>) -> crate::Result<()> {
        if self
            .buf
            .consume_with(cursor.capacity(), |claimed| cursor.append(claimed))
        {
            return Ok(());
        }
        crate::default_read_buf_exact(self, cursor)
    }
    fn read_to_end(&mut self, buf: &mut Vec<u8>) -> crate::Result<usize> {
        let inner_buf = self.buffer();
        buf.try_reserve(inner_buf.len())?;
        buf.extend_from_slice(inner_buf);
        let nread = inner_buf.len();
        self.discard_buffer();
        Ok(nread + self.inner.read_to_end(buf)?)
    }
    fn read_to_string(&mut self, buf: &mut String) -> crate::Result<usize> {
        if buf.is_empty() {
            unsafe { crate::append_to_string(buf, |b| self.read_to_end(b)) }
        } else {
            let mut bytes = Vec::new();
            self.read_to_end(&mut bytes)?;
            let string = alloc::str::from_utf8(&bytes).map_err(|_| crate::Error::InvalidUtf8)?;
            *buf += string;
            Ok(string.len())
        }
    }
}
impl<R: ?Sized + Read> BufRead for BufReader<R> {
    fn fill_buf(&mut self) -> crate::Result<&[u8]> {
        self.buf.fill_buf(&mut self.inner)
    }
    fn consume(&mut self, amt: usize) {
        self.buf.consume(amt)
    }
}
impl<R> fmt::Debug for BufReader<R>
where
    R: ?Sized + fmt::Debug,
{
    fn fmt(&self, fmt: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt.debug_struct("BufReader")
            .field("reader", &&self.inner)
            .field(
                "buffer",
                &format_args!("{}/{}", self.buf.filled() - self.buf.pos(), self.capacity()),
            )
            .finish()
    }
}
impl<R: ?Sized + Seek> Seek for BufReader<R> {
    fn seek(&mut self, pos: SeekFrom) -> crate::Result<u64> {
        let result: u64;
        if let SeekFrom::Current(n) = pos {
            let remainder = (self.buf.filled() - self.buf.pos()) as i64;
            if let Some(offset) = n.checked_sub(remainder) {
                result = self.inner.seek(SeekFrom::Current(offset))?;
            } else {
                self.inner.seek(SeekFrom::Current(-remainder))?;
                self.discard_buffer();
                result = self.inner.seek(SeekFrom::Current(n))?;
            }
        } else {
            result = self.inner.seek(pos)?;
        }
        self.discard_buffer();
        Ok(result)
    }
    fn stream_position(&mut self) -> crate::Result<u64> {
        let remainder = (self.buf.filled() - self.buf.pos()) as u64;
        self.inner.stream_position().map(|pos| {
            pos.checked_sub(remainder).expect(
                "overflow when subtracting remaining buffer size from inner stream position",
            )
        })
    }
    fn seek_relative(&mut self, offset: i64) -> crate::Result<()> {
        self.seek_relative(offset)
    }
}
impl<T: ?Sized + SizeHint> SizeHint for BufReader<T> {
    #[inline]
    fn lower_bound(&self) -> usize {
        SizeHint::lower_bound(self.get_ref()) + self.buffer().len()
    }
    #[inline]
    fn upper_bound(&self) -> Option<usize> {
        SizeHint::upper_bound(self.get_ref()).and_then(|up| self.buffer().len().checked_add(up))
    }
}
