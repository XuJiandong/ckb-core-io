use crate::{IntoInnerError, Seek, SeekFrom, Write, DEFAULT_BUF_SIZE};
use alloc::fmt;
use alloc::vec::Vec;
use core::error;
use core::mem::{self, ManuallyDrop};
use core::ptr;
pub struct BufWriter<W: ?Sized + Write> {
    buf: Vec<u8>,
    panicked: bool,
    inner: W,
}
impl<W: Write> BufWriter<W> {
    pub fn new(inner: W) -> BufWriter<W> {
        BufWriter::with_capacity(DEFAULT_BUF_SIZE, inner)
    }
    pub fn with_capacity(capacity: usize, inner: W) -> BufWriter<W> {
        BufWriter {
            inner,
            buf: Vec::with_capacity(capacity),
            panicked: false,
        }
    }
    pub fn into_inner(mut self) -> Result<W, IntoInnerError<BufWriter<W>>> {
        match self.flush_buf() {
            Err(e) => Err(IntoInnerError::new(self, e)),
            Ok(()) => Ok(self.into_parts().0),
        }
    }
    pub fn into_parts(self) -> (W, Result<Vec<u8>, WriterPanicked>) {
        let mut this = ManuallyDrop::new(self);
        let buf = mem::take(&mut this.buf);
        let buf = if !this.panicked {
            Ok(buf)
        } else {
            Err(WriterPanicked { buf })
        };
        let inner = unsafe { ptr::read(&this.inner) };
        (inner, buf)
    }
}
impl<W: ?Sized + Write> BufWriter<W> {
    pub(crate) fn flush_buf(&mut self) -> crate::Result<()> {
        struct BufGuard<'a> {
            buffer: &'a mut Vec<u8>,
            written: usize,
        }
        impl<'a> BufGuard<'a> {
            fn new(buffer: &'a mut Vec<u8>) -> Self {
                Self { buffer, written: 0 }
            }
            fn remaining(&self) -> &[u8] {
                &self.buffer[self.written..]
            }
            fn consume(&mut self, amt: usize) {
                self.written += amt;
            }
            fn done(&self) -> bool {
                self.written >= self.buffer.len()
            }
        }
        impl Drop for BufGuard<'_> {
            fn drop(&mut self) {
                if self.written > 0 {
                    self.buffer.drain(..self.written);
                }
            }
        }
        let mut guard = BufGuard::new(&mut self.buf);
        while !guard.done() {
            self.panicked = true;
            let r = self.inner.write(guard.remaining());
            self.panicked = false;
            match r {
                Ok(0) => {
                    return Err(crate::const_io_error!(
                        crate::error::ErrorKind::WriteZero,
                        "failed to write the buffered data",
                    ));
                }
                Ok(n) => guard.consume(n),
                Err(ref e) if e.is_interrupted() => {}
                Err(e) => return Err(e),
            }
        }
        Ok(())
    }
    pub(super) fn write_to_buf(&mut self, buf: &[u8]) -> usize {
        let available = self.spare_capacity();
        let amt_to_buffer = available.min(buf.len());
        unsafe {
            self.write_to_buffer_unchecked(&buf[..amt_to_buffer]);
        }
        amt_to_buffer
    }
    pub fn get_ref(&self) -> &W {
        &self.inner
    }
    pub fn get_mut(&mut self) -> &mut W {
        &mut self.inner
    }
    pub fn buffer(&self) -> &[u8] {
        &self.buf
    }
    #[allow(dead_code)]
    pub(crate) fn buffer_mut(&mut self) -> &mut Vec<u8> {
        &mut self.buf
    }
    pub fn capacity(&self) -> usize {
        self.buf.capacity()
    }
    #[cold]
    #[inline(never)]
    fn write_cold(&mut self, buf: &[u8]) -> crate::Result<usize> {
        if buf.len() > self.spare_capacity() {
            self.flush_buf()?;
        }
        if buf.len() >= self.buf.capacity() {
            self.panicked = true;
            let r = self.get_mut().write(buf);
            self.panicked = false;
            r
        } else {
            unsafe {
                self.write_to_buffer_unchecked(buf);
            }
            Ok(buf.len())
        }
    }
    #[cold]
    #[inline(never)]
    fn write_all_cold(&mut self, buf: &[u8]) -> crate::Result<()> {
        if buf.len() > self.spare_capacity() {
            self.flush_buf()?;
        }
        if buf.len() >= self.buf.capacity() {
            self.panicked = true;
            let r = self.get_mut().write_all(buf);
            self.panicked = false;
            r
        } else {
            unsafe {
                self.write_to_buffer_unchecked(buf);
            }
            Ok(())
        }
    }
    #[inline]
    unsafe fn write_to_buffer_unchecked(&mut self, buf: &[u8]) {
        debug_assert!(buf.len() <= self.spare_capacity());
        let old_len = self.buf.len();
        let buf_len = buf.len();
        let src = buf.as_ptr();
        unsafe {
            let dst = self.buf.as_mut_ptr().add(old_len);
            ptr::copy_nonoverlapping(src, dst, buf_len);
            self.buf.set_len(old_len + buf_len);
        }
    }
    #[inline]
    fn spare_capacity(&self) -> usize {
        self.buf.capacity() - self.buf.len()
    }
}
pub struct WriterPanicked {
    buf: Vec<u8>,
}
impl WriterPanicked {
    pub fn into_inner(self) -> Vec<u8> {
        self.buf
    }
    const DESCRIPTION: &'static str =
        "BufWriter inner writer panicked, what data remains unwritten is not known";
}
impl error::Error for WriterPanicked {
    #[allow(deprecated, deprecated_in_future)]
    fn description(&self) -> &str {
        Self::DESCRIPTION
    }
}
impl fmt::Display for WriterPanicked {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", Self::DESCRIPTION)
    }
}
impl fmt::Debug for WriterPanicked {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("WriterPanicked")
            .field(
                "buffer",
                &format_args!("{}/{}", self.buf.len(), self.buf.capacity()),
            )
            .finish()
    }
}
impl<W: ?Sized + Write> Write for BufWriter<W> {
    #[inline]
    fn write(&mut self, buf: &[u8]) -> crate::Result<usize> {
        if buf.len() < self.spare_capacity() {
            unsafe {
                self.write_to_buffer_unchecked(buf);
            }
            Ok(buf.len())
        } else {
            self.write_cold(buf)
        }
    }
    #[inline]
    fn write_all(&mut self, buf: &[u8]) -> crate::Result<()> {
        if buf.len() < self.spare_capacity() {
            unsafe {
                self.write_to_buffer_unchecked(buf);
            }
            Ok(())
        } else {
            self.write_all_cold(buf)
        }
    }
    fn is_write_vectored(&self) -> bool {
        true
    }
    fn flush(&mut self) -> crate::Result<()> {
        self.flush_buf().and_then(|()| self.get_mut().flush())
    }
}
impl<W: ?Sized + Write> fmt::Debug for BufWriter<W>
where
    W: fmt::Debug,
{
    fn fmt(&self, fmt: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt.debug_struct("BufWriter")
            .field("writer", &&self.inner)
            .field(
                "buffer",
                &format_args!("{}/{}", self.buf.len(), self.buf.capacity()),
            )
            .finish()
    }
}
impl<W: ?Sized + Write + Seek> Seek for BufWriter<W> {
    fn seek(&mut self, pos: SeekFrom) -> crate::Result<u64> {
        self.flush_buf()?;
        self.get_mut().seek(pos)
    }
}
impl<W: ?Sized + Write> Drop for BufWriter<W> {
    fn drop(&mut self) {
        if !self.panicked {
            let _r = self.flush_buf();
        }
    }
}
