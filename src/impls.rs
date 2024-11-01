use crate::{BorrowedCursor, BufRead, Read, Seek, SeekFrom, Write};
use alloc::boxed::Box;
use alloc::collections::VecDeque;
use alloc::fmt;
use alloc::str;
use alloc::string::String;
use alloc::vec::Vec;
use core::cmp;
use core::mem;
impl<R: Read + ?Sized> Read for &mut R {
    #[inline]
    fn read(&mut self, buf: &mut [u8]) -> crate::Result<usize> {
        (**self).read(buf)
    }
    #[inline]
    fn read_buf(&mut self, cursor: BorrowedCursor<'_>) -> crate::Result<()> {
        (**self).read_buf(cursor)
    }
    #[inline]
    fn is_read_vectored(&self) -> bool {
        (**self).is_read_vectored()
    }
    #[inline]
    fn read_to_end(&mut self, buf: &mut Vec<u8>) -> crate::Result<usize> {
        (**self).read_to_end(buf)
    }
    #[inline]
    fn read_to_string(&mut self, buf: &mut String) -> crate::Result<usize> {
        (**self).read_to_string(buf)
    }
    #[inline]
    fn read_exact(&mut self, buf: &mut [u8]) -> crate::Result<()> {
        (**self).read_exact(buf)
    }
    #[inline]
    fn read_buf_exact(&mut self, cursor: BorrowedCursor<'_>) -> crate::Result<()> {
        (**self).read_buf_exact(cursor)
    }
}
impl<W: Write + ?Sized> Write for &mut W {
    #[inline]
    fn write(&mut self, buf: &[u8]) -> crate::Result<usize> {
        (**self).write(buf)
    }
    #[inline]
    fn flush(&mut self) -> crate::Result<()> {
        (**self).flush()
    }
    #[inline]
    fn write_all(&mut self, buf: &[u8]) -> crate::Result<()> {
        (**self).write_all(buf)
    }
    #[inline]
    fn write_fmt(&mut self, fmt: fmt::Arguments<'_>) -> crate::Result<()> {
        (**self).write_fmt(fmt)
    }
}
impl<S: Seek + ?Sized> Seek for &mut S {
    #[inline]
    fn seek(&mut self, pos: SeekFrom) -> crate::Result<u64> {
        (**self).seek(pos)
    }
    #[inline]
    fn stream_position(&mut self) -> crate::Result<u64> {
        (**self).stream_position()
    }
}
impl<B: BufRead + ?Sized> BufRead for &mut B {
    #[inline]
    fn fill_buf(&mut self) -> crate::Result<&[u8]> {
        (**self).fill_buf()
    }
    #[inline]
    fn consume(&mut self, amt: usize) {
        (**self).consume(amt)
    }
    #[inline]
    fn read_until(&mut self, byte: u8, buf: &mut Vec<u8>) -> crate::Result<usize> {
        (**self).read_until(byte, buf)
    }
    #[inline]
    fn read_line(&mut self, buf: &mut String) -> crate::Result<usize> {
        (**self).read_line(buf)
    }
}
impl<R: Read + ?Sized> Read for Box<R> {
    #[inline]
    fn read(&mut self, buf: &mut [u8]) -> crate::Result<usize> {
        (**self).read(buf)
    }
    #[inline]
    fn read_buf(&mut self, cursor: BorrowedCursor<'_>) -> crate::Result<()> {
        (**self).read_buf(cursor)
    }
    #[inline]
    fn is_read_vectored(&self) -> bool {
        (**self).is_read_vectored()
    }
    #[inline]
    fn read_to_end(&mut self, buf: &mut Vec<u8>) -> crate::Result<usize> {
        (**self).read_to_end(buf)
    }
    #[inline]
    fn read_to_string(&mut self, buf: &mut String) -> crate::Result<usize> {
        (**self).read_to_string(buf)
    }
    #[inline]
    fn read_exact(&mut self, buf: &mut [u8]) -> crate::Result<()> {
        (**self).read_exact(buf)
    }
    #[inline]
    fn read_buf_exact(&mut self, cursor: BorrowedCursor<'_>) -> crate::Result<()> {
        (**self).read_buf_exact(cursor)
    }
}
impl<W: Write + ?Sized> Write for Box<W> {
    #[inline]
    fn write(&mut self, buf: &[u8]) -> crate::Result<usize> {
        (**self).write(buf)
    }
    #[inline]
    fn flush(&mut self) -> crate::Result<()> {
        (**self).flush()
    }
    #[inline]
    fn write_all(&mut self, buf: &[u8]) -> crate::Result<()> {
        (**self).write_all(buf)
    }
    #[inline]
    fn write_fmt(&mut self, fmt: fmt::Arguments<'_>) -> crate::Result<()> {
        (**self).write_fmt(fmt)
    }
}
impl<S: Seek + ?Sized> Seek for Box<S> {
    #[inline]
    fn seek(&mut self, pos: SeekFrom) -> crate::Result<u64> {
        (**self).seek(pos)
    }
    #[inline]
    fn stream_position(&mut self) -> crate::Result<u64> {
        (**self).stream_position()
    }
}
impl<B: BufRead + ?Sized> BufRead for Box<B> {
    #[inline]
    fn fill_buf(&mut self) -> crate::Result<&[u8]> {
        (**self).fill_buf()
    }
    #[inline]
    fn consume(&mut self, amt: usize) {
        (**self).consume(amt)
    }
    #[inline]
    fn read_until(&mut self, byte: u8, buf: &mut Vec<u8>) -> crate::Result<usize> {
        (**self).read_until(byte, buf)
    }
    #[inline]
    fn read_line(&mut self, buf: &mut String) -> crate::Result<usize> {
        (**self).read_line(buf)
    }
}
impl Read for &[u8] {
    #[inline]
    fn read(&mut self, buf: &mut [u8]) -> crate::Result<usize> {
        let amt = cmp::min(buf.len(), self.len());
        let (a, b) = self.split_at(amt);
        if amt == 1 {
            buf[0] = a[0];
        } else {
            buf[..amt].copy_from_slice(a);
        }
        *self = b;
        Ok(amt)
    }
    #[inline]
    fn read_buf(&mut self, mut cursor: BorrowedCursor<'_>) -> crate::Result<()> {
        let amt = cmp::min(cursor.capacity(), self.len());
        let (a, b) = self.split_at(amt);
        cursor.append(a);
        *self = b;
        Ok(())
    }
    #[inline]
    fn read_exact(&mut self, buf: &mut [u8]) -> crate::Result<()> {
        if buf.len() > self.len() {
            *self = &self[self.len()..];
            return Err(crate::Error::READ_EXACT_EOF);
        }
        let (a, b) = self.split_at(buf.len());
        if buf.len() == 1 {
            buf[0] = a[0];
        } else {
            buf.copy_from_slice(a);
        }
        *self = b;
        Ok(())
    }
    #[inline]
    fn read_buf_exact(&mut self, mut cursor: BorrowedCursor<'_>) -> crate::Result<()> {
        if cursor.capacity() > self.len() {
            cursor.append(self);
            *self = &self[self.len()..];
            return Err(crate::Error::READ_EXACT_EOF);
        }
        let (a, b) = self.split_at(cursor.capacity());
        cursor.append(a);
        *self = b;
        Ok(())
    }
    #[inline]
    fn read_to_end(&mut self, buf: &mut Vec<u8>) -> crate::Result<usize> {
        let len = self.len();
        buf.try_reserve(len)?;
        buf.extend_from_slice(self);
        *self = &self[len..];
        Ok(len)
    }
    #[inline]
    fn read_to_string(&mut self, buf: &mut String) -> crate::Result<usize> {
        let content = str::from_utf8(self).map_err(|_| crate::Error::INVALID_UTF8)?;
        let len = self.len();
        buf.try_reserve(len)?;
        buf.push_str(content);
        *self = &self[len..];
        Ok(len)
    }
}
impl BufRead for &[u8] {
    #[inline]
    fn fill_buf(&mut self) -> crate::Result<&[u8]> {
        Ok(*self)
    }
    #[inline]
    fn consume(&mut self, amt: usize) {
        *self = &self[amt..];
    }
}
impl Write for &mut [u8] {
    #[inline]
    fn write(&mut self, data: &[u8]) -> crate::Result<usize> {
        let amt = cmp::min(data.len(), self.len());
        let (a, b) = mem::take(self).split_at_mut(amt);
        a.copy_from_slice(&data[..amt]);
        *self = b;
        Ok(amt)
    }
    #[inline]
    fn is_write_vectored(&self) -> bool {
        true
    }
    #[inline]
    fn write_all(&mut self, data: &[u8]) -> crate::Result<()> {
        if self.write(data)? == data.len() {
            Ok(())
        } else {
            Err(crate::Error::WRITE_ALL_EOF)
        }
    }
    #[inline]
    fn flush(&mut self) -> crate::Result<()> {
        Ok(())
    }
}
impl Write for Vec<u8> {
    #[inline]
    fn write(&mut self, buf: &[u8]) -> crate::Result<usize> {
        self.extend_from_slice(buf);
        Ok(buf.len())
    }
    #[inline]
    fn is_write_vectored(&self) -> bool {
        true
    }
    #[inline]
    fn write_all(&mut self, buf: &[u8]) -> crate::Result<()> {
        self.extend_from_slice(buf);
        Ok(())
    }
    #[inline]
    fn flush(&mut self) -> crate::Result<()> {
        Ok(())
    }
}
impl Read for VecDeque<u8> {
    #[inline]
    fn read(&mut self, buf: &mut [u8]) -> crate::Result<usize> {
        let (ref mut front, _) = self.as_slices();
        let n = Read::read(front, buf)?;
        self.drain(..n);
        Ok(n)
    }
    #[inline]
    fn read_buf(&mut self, cursor: BorrowedCursor<'_>) -> crate::Result<()> {
        let (ref mut front, _) = self.as_slices();
        let n = cmp::min(cursor.capacity(), front.len());
        Read::read_buf(front, cursor)?;
        self.drain(..n);
        Ok(())
    }
    #[inline]
    fn read_to_end(&mut self, buf: &mut Vec<u8>) -> crate::Result<usize> {
        let len = self.len();
        buf.try_reserve(len)?;
        let (front, back) = self.as_slices();
        buf.extend_from_slice(front);
        buf.extend_from_slice(back);
        self.clear();
        Ok(len)
    }
    #[inline]
    fn read_to_string(&mut self, buf: &mut String) -> crate::Result<usize> {
        unsafe { crate::append_to_string(buf, |buf| self.read_to_end(buf)) }
    }
}
impl BufRead for VecDeque<u8> {
    #[inline]
    fn fill_buf(&mut self) -> crate::Result<&[u8]> {
        let (front, _) = self.as_slices();
        Ok(front)
    }
    #[inline]
    fn consume(&mut self, amt: usize) {
        self.drain(..amt);
    }
}
impl Write for VecDeque<u8> {
    #[inline]
    fn write(&mut self, buf: &[u8]) -> crate::Result<usize> {
        self.extend(buf);
        Ok(buf.len())
    }
    #[inline]
    fn is_write_vectored(&self) -> bool {
        true
    }
    #[inline]
    fn write_all(&mut self, buf: &[u8]) -> crate::Result<()> {
        self.extend(buf);
        Ok(())
    }
    #[inline]
    fn flush(&mut self) -> crate::Result<()> {
        Ok(())
    }
}
impl<'a> crate::Write for crate::BorrowedCursor<'a> {
    fn write(&mut self, buf: &[u8]) -> crate::Result<usize> {
        let amt = cmp::min(buf.len(), self.capacity());
        self.append(&buf[..amt]);
        Ok(amt)
    }
    #[inline]
    fn flush(&mut self) -> crate::Result<()> {
        Ok(())
    }
}
