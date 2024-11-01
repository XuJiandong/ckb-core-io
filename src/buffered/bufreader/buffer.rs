use alloc::boxed::Box;
use alloc::vec;

use crate::{BorrowedBuf, Read};
use core::cmp;
use core::mem::MaybeUninit;
pub struct Buffer {
    buf: Box<[MaybeUninit<u8>]>,
    pos: usize,
    filled: usize,
    initialized: usize,
}
impl Buffer {
    #[inline]
    pub fn with_capacity(capacity: usize) -> Self {
        let buf = vec![MaybeUninit::uninit(); capacity].into_boxed_slice();
        Self {
            buf,
            pos: 0,
            filled: 0,
            initialized: 0,
        }
    }
    #[inline]
    pub fn buffer(&self) -> &[u8] {
        unsafe {
            core::slice::from_raw_parts(
                self.buf.as_ptr().add(self.pos) as *const u8,
                self.filled - self.pos,
            )
        }
    }
    #[inline]
    pub fn capacity(&self) -> usize {
        self.buf.len()
    }
    #[inline]
    pub fn filled(&self) -> usize {
        self.filled
    }
    #[inline]
    pub fn pos(&self) -> usize {
        self.pos
    }
    #[cfg(test)]
    pub fn initialized(&self) -> usize {
        self.initialized
    }
    #[inline]
    pub fn discard_buffer(&mut self) {
        self.pos = 0;
        self.filled = 0;
    }
    #[inline]
    pub fn consume(&mut self, amt: usize) {
        self.pos = cmp::min(self.pos + amt, self.filled);
    }
    #[inline]
    pub fn consume_with<V>(&mut self, amt: usize, mut visitor: V) -> bool
    where
        V: FnMut(&[u8]),
    {
        if let Some(claimed) = self.buffer().get(..amt) {
            visitor(claimed);
            self.pos += amt;
            true
        } else {
            false
        }
    }
    #[inline]
    pub fn unconsume(&mut self, amt: usize) {
        self.pos = self.pos.saturating_sub(amt);
    }
    #[inline]
    pub fn fill_buf(&mut self, mut reader: impl Read) -> crate::Result<&[u8]> {
        if self.pos >= self.filled {
            debug_assert!(self.pos == self.filled);
            let mut buf = BorrowedBuf::from(&mut *self.buf);
            unsafe {
                buf.set_init(self.initialized);
            }
            reader.read_buf(buf.unfilled())?;
            self.pos = 0;
            self.filled = buf.len();
            self.initialized = buf.init_len();
        }
        Ok(self.buffer())
    }
}
