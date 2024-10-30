use crate::{self, BufWriter, IoSlice, Write};
use core::slice::memchr;
#[derive(Debug)]
pub struct LineWriterShim<'a, W: ?Sized + Write> {
    buffer: &'a mut BufWriter<W>,
}
impl<'a, W: ?Sized + Write> LineWriterShim<'a, W> {
    pub fn new(buffer: &'a mut BufWriter<W>) -> Self {
        Self { buffer }
    }
    fn inner(&self) -> &W {
        self.buffer.get_ref()
    }
    fn inner_mut(&mut self) -> &mut W {
        self.buffer.get_mut()
    }
    fn buffered(&self) -> &[u8] {
        self.buffer.buffer()
    }
    fn flush_if_completed_line(&mut self) -> io::Result<()> {
        match self.buffered().last().copied() {
            Some(b'\n') => self.buffer.flush_buf(),
            _ => Ok(()),
        }
    }
}
impl<'a, W: ?Sized + Write> Write for LineWriterShim<'a, W> {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        let newline_idx = match memchr::memrchr(b'\n', buf) {
            None => {
                self.flush_if_completed_line()?;
                return self.buffer.write(buf);
            }
            Some(newline_idx) => newline_idx + 1,
        };
        self.buffer.flush_buf()?;
        let lines = &buf[..newline_idx];
        let flushed = self.inner_mut().write(lines)?;
        if flushed == 0 {
            return Ok(0);
        }
        let tail = if flushed >= newline_idx {
            &buf[flushed..]
        } else if newline_idx - flushed <= self.buffer.capacity() {
            &buf[flushed..newline_idx]
        } else {
            let scan_area = &buf[flushed..];
            let scan_area = &scan_area[..self.buffer.capacity()];
            match memchr::memrchr(b'\n', scan_area) {
                Some(newline_idx) => &scan_area[..newline_idx + 1],
                None => scan_area,
            }
        };
        let buffered = self.buffer.write_to_buf(tail);
        Ok(flushed + buffered)
    }
    fn flush(&mut self) -> io::Result<()> {
        self.buffer.flush()
    }
    fn write_vectored(&mut self, bufs: &[IoSlice<'_>]) -> io::Result<usize> {
        if !self.is_write_vectored() {
            return match bufs.iter().find(|buf| !buf.is_empty()) {
                Some(buf) => self.write(buf),
                None => Ok(0),
            };
        }
        let last_newline_buf_idx = bufs
            .iter()
            .enumerate()
            .rev()
            .find_map(|(i, buf)| memchr::memchr(b'\n', buf).map(|_| i));
        let last_newline_buf_idx = match last_newline_buf_idx {
            None => {
                self.flush_if_completed_line()?;
                return self.buffer.write_vectored(bufs);
            }
            Some(i) => i,
        };
        self.buffer.flush_buf()?;
        let (lines, tail) = bufs.split_at(last_newline_buf_idx + 1);
        let flushed = self.inner_mut().write_vectored(lines)?;
        if flushed == 0 {
            return Ok(0);
        }
        let mut lines_len: usize = 0;
        for buf in lines {
            lines_len = lines_len.saturating_add(buf.len());
            if flushed < lines_len {
                return Ok(flushed);
            }
        }
        let buffered: usize = tail
            .iter()
            .filter(|buf| !buf.is_empty())
            .map(|buf| self.buffer.write_to_buf(buf))
            .take_while(|&n| n > 0)
            .sum();
        Ok(flushed + buffered)
    }
    fn is_write_vectored(&self) -> bool {
        self.inner().is_write_vectored()
    }
    fn write_all(&mut self, buf: &[u8]) -> io::Result<()> {
        match memchr::memrchr(b'\n', buf) {
            None => {
                self.flush_if_completed_line()?;
                self.buffer.write_all(buf)
            }
            Some(newline_idx) => {
                let (lines, tail) = buf.split_at(newline_idx + 1);
                if self.buffered().is_empty() {
                    self.inner_mut().write_all(lines)?;
                } else {
                    self.buffer.write_all(lines)?;
                    self.buffer.flush_buf()?;
                }
                self.buffer.write_all(tail)
            }
        }
    }
}
