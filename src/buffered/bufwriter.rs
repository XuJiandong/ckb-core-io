use crate::error;
use crate::ptr;
use crate::{self, ErrorKind, IntoInnerError, IoSlice, Seek, SeekFrom, Write, DEFAULT_BUF_SIZE};
use alloc::fmt;
use core::mem::{self, ManuallyDrop};
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
    pub(crate) fn flush_buf(&mut self) -> io::Result<()> {
        #[doc = " Helper struct to ensure the buffer is updated after all the writes"]
        #[doc = " are complete. It tracks the number of written bytes and drains them"]
        #[doc = " all from the front of the buffer when dropped."]
        struct BufGuard<'a> {
            buffer: &'a mut Vec<u8>,
            written: usize,
        }
        impl<'a> BufGuard<'a> {
            fn new(buffer: &'a mut Vec<u8>) -> Self {
                Self { buffer, written: 0 }
            }
            #[doc = " The unwritten part of the buffer"]
            fn remaining(&self) -> &[u8] {
                &self.buffer[self.written..]
            }
            #[doc = " Flag some bytes as removed from the front of the buffer"]
            fn consume(&mut self, amt: usize) {
                self.written += amt;
            }
            #[doc = " true if all of the bytes have been written"]
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
                    return Err(io::const_io_error!(
                        ErrorKind::WriteZero,
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
    pub(crate) fn buffer_mut(&mut self) -> &mut Vec<u8> {
        &mut self.buf
    }
    pub fn capacity(&self) -> usize {
        self.buf.capacity()
    }
    #[cold]
    #[inline(never)]
    fn write_cold(&mut self, buf: &[u8]) -> io::Result<usize> {
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
    fn write_all_cold(&mut self, buf: &[u8]) -> io::Result<()> {
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
    #[must_use = "`self` will be dropped if the result is not used"]
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
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
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
    fn write_all(&mut self, buf: &[u8]) -> io::Result<()> {
        if buf.len() < self.spare_capacity() {
            unsafe {
                self.write_to_buffer_unchecked(buf);
            }
            Ok(())
        } else {
            self.write_all_cold(buf)
        }
    }
    fn write_vectored(&mut self, bufs: &[IoSlice<'_>]) -> io::Result<usize> {
        if self.get_ref().is_write_vectored() {
            let mut saturated_total_len: usize = 0;
            for buf in bufs {
                saturated_total_len = saturated_total_len.saturating_add(buf.len());
                if saturated_total_len > self.spare_capacity() && !self.buf.is_empty() {
                    self.flush_buf()?;
                }
                if saturated_total_len >= self.buf.capacity() {
                    self.panicked = true;
                    let r = self.get_mut().write_vectored(bufs);
                    self.panicked = false;
                    return r;
                }
            }
            unsafe {
                bufs.iter().for_each(|b| self.write_to_buffer_unchecked(b));
            };
            Ok(saturated_total_len)
        } else {
            let mut iter = bufs.iter();
            let mut total_written = if let Some(buf) = iter.by_ref().find(|&buf| !buf.is_empty()) {
                if buf.len() > self.spare_capacity() {
                    self.flush_buf()?;
                }
                if buf.len() >= self.buf.capacity() {
                    self.panicked = true;
                    let r = self.get_mut().write(buf);
                    self.panicked = false;
                    return r;
                } else {
                    unsafe {
                        self.write_to_buffer_unchecked(buf);
                    }
                    buf.len()
                }
            } else {
                return Ok(0);
            };
            debug_assert!(total_written != 0);
            for buf in iter {
                if buf.len() <= self.spare_capacity() {
                    unsafe {
                        self.write_to_buffer_unchecked(buf);
                    }
                    total_written += buf.len();
                } else {
                    break;
                }
            }
            Ok(total_written)
        }
    }
    fn is_write_vectored(&self) -> bool {
        true
    }
    fn flush(&mut self) -> io::Result<()> {
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
    fn seek(&mut self, pos: SeekFrom) -> io::Result<u64> {
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
