use alloc::boxed::Box;
use alloc::string::String;
use alloc::vec::Vec;

use crate::{prelude::*, Error};
use crate::{BorrowedCursor, SeekFrom};
use core::cmp;
#[derive(Debug, Default, Eq, PartialEq)]
pub struct Cursor<T> {
    inner: T,
    pos: u64,
}
impl<T> Cursor<T> {
    pub const fn new(inner: T) -> Cursor<T> {
        Cursor { pos: 0, inner }
    }
    pub fn into_inner(self) -> T {
        self.inner
    }
    pub const fn get_ref(&self) -> &T {
        &self.inner
    }
    pub fn get_mut(&mut self) -> &mut T {
        &mut self.inner
    }
    pub const fn position(&self) -> u64 {
        self.pos
    }
    pub fn set_position(&mut self, pos: u64) {
        self.pos = pos;
    }
}
impl<T> Cursor<T>
where
    T: AsRef<[u8]>,
{
    pub fn remaining_slice(&self) -> &[u8] {
        let len = self.pos.min(self.inner.as_ref().len() as u64);
        &self.inner.as_ref()[(len as usize)..]
    }
    pub fn is_empty(&self) -> bool {
        self.pos >= self.inner.as_ref().len() as u64
    }
}
impl<T> Clone for Cursor<T>
where
    T: Clone,
{
    #[inline]
    fn clone(&self) -> Self {
        Cursor {
            inner: self.inner.clone(),
            pos: self.pos,
        }
    }
    #[inline]
    fn clone_from(&mut self, other: &Self) {
        self.inner.clone_from(&other.inner);
        self.pos = other.pos;
    }
}
impl<T> crate::Seek for Cursor<T>
where
    T: AsRef<[u8]>,
{
    fn seek(&mut self, style: SeekFrom) -> crate::Result<u64> {
        let (base_pos, offset) = match style {
            SeekFrom::Start(n) => {
                self.pos = n;
                return Ok(n);
            }
            SeekFrom::End(n) => (self.inner.as_ref().len() as u64, n),
            SeekFrom::Current(n) => (self.pos, n),
        };
        match base_pos.checked_add_signed(offset) {
            Some(n) => {
                self.pos = n;
                Ok(self.pos)
            }
            None => Err(Error::InvalidInput),
        }
    }
    fn stream_len(&mut self) -> crate::Result<u64> {
        Ok(self.inner.as_ref().len() as u64)
    }
    fn stream_position(&mut self) -> crate::Result<u64> {
        Ok(self.pos)
    }
}
impl<T> Read for Cursor<T>
where
    T: AsRef<[u8]>,
{
    fn read(&mut self, buf: &mut [u8]) -> crate::Result<usize> {
        let n = Read::read(&mut self.remaining_slice(), buf)?;
        self.pos += n as u64;
        Ok(n)
    }
    fn read_buf(&mut self, mut cursor: BorrowedCursor<'_>) -> crate::Result<()> {
        let prev_written = cursor.written();
        Read::read_buf(&mut self.remaining_slice(), cursor.reborrow())?;
        self.pos += (cursor.written() - prev_written) as u64;
        Ok(())
    }
    fn is_read_vectored(&self) -> bool {
        true
    }
    fn read_exact(&mut self, buf: &mut [u8]) -> crate::Result<()> {
        let result = Read::read_exact(&mut self.remaining_slice(), buf);
        match result {
            Ok(_) => self.pos += buf.len() as u64,
            Err(_) => self.pos = self.inner.as_ref().len() as u64,
        }
        result
    }
    fn read_buf_exact(&mut self, mut cursor: BorrowedCursor<'_>) -> crate::Result<()> {
        let prev_written = cursor.written();
        let result = Read::read_buf_exact(&mut self.remaining_slice(), cursor.reborrow());
        self.pos += (cursor.written() - prev_written) as u64;
        result
    }
    fn read_to_end(&mut self, buf: &mut Vec<u8>) -> crate::Result<usize> {
        let content = self.remaining_slice();
        let len = content.len();
        buf.try_reserve(len)?;
        buf.extend_from_slice(content);
        self.pos += len as u64;
        Ok(len)
    }
    fn read_to_string(&mut self, buf: &mut String) -> crate::Result<usize> {
        let content =
            alloc::str::from_utf8(self.remaining_slice()).map_err(|_| crate::Error::InvalidUtf8)?;
        let len = content.len();
        buf.try_reserve(len)?;
        buf.push_str(content);
        self.pos += len as u64;
        Ok(len)
    }
}
impl<T> BufRead for Cursor<T>
where
    T: AsRef<[u8]>,
{
    fn fill_buf(&mut self) -> crate::Result<&[u8]> {
        Ok(self.remaining_slice())
    }
    fn consume(&mut self, amt: usize) {
        self.pos += amt as u64;
    }
}
#[inline]
fn slice_write(pos_mut: &mut u64, slice: &mut [u8], buf: &[u8]) -> crate::Result<usize> {
    let pos = cmp::min(*pos_mut, slice.len() as u64);
    let amt = (&mut slice[(pos as usize)..]).write(buf)?;
    *pos_mut += amt as u64;
    Ok(amt)
}
fn reserve_and_pad(pos_mut: &mut u64, vec: &mut Vec<u8>, buf_len: usize) -> crate::Result<usize> {
    let pos: usize = (*pos_mut).try_into().map_err(|_| Error::InvalidInput)?;
    let desired_cap = pos.saturating_add(buf_len);
    if desired_cap > vec.capacity() {
        vec.reserve(desired_cap - vec.len());
    }
    if pos > vec.len() {
        let diff = pos - vec.len();
        let spare = vec.spare_capacity_mut();
        debug_assert!(spare.len() >= diff);
        unsafe {
            spare
                .get_unchecked_mut(..diff)
                .fill(core::mem::MaybeUninit::new(0));
            vec.set_len(pos);
        }
    }
    Ok(pos)
}
unsafe fn vec_write_unchecked(pos: usize, vec: &mut Vec<u8>, buf: &[u8]) -> usize {
    debug_assert!(vec.capacity() >= pos + buf.len());
    unsafe { vec.as_mut_ptr().add(pos).copy_from(buf.as_ptr(), buf.len()) };
    pos + buf.len()
}

fn vec_write(pos_mut: &mut u64, vec: &mut Vec<u8>, buf: &[u8]) -> crate::Result<usize> {
    let buf_len = buf.len();
    let mut pos = reserve_and_pad(pos_mut, vec, buf_len)?;
    unsafe {
        pos = vec_write_unchecked(pos, vec, buf);
        if pos > vec.len() {
            vec.set_len(pos);
        }
    };
    *pos_mut += buf_len as u64;
    Ok(buf_len)
}

impl Write for Cursor<&mut [u8]> {
    #[inline]
    fn write(&mut self, buf: &[u8]) -> crate::Result<usize> {
        slice_write(&mut self.pos, self.inner, buf)
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
impl Write for Cursor<&mut Vec<u8>> {
    fn write(&mut self, buf: &[u8]) -> crate::Result<usize> {
        vec_write(&mut self.pos, self.inner, buf)
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
impl Write for Cursor<Vec<u8>> {
    fn write(&mut self, buf: &[u8]) -> crate::Result<usize> {
        vec_write(&mut self.pos, &mut self.inner, buf)
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
impl Write for Cursor<Box<[u8]>> {
    #[inline]
    fn write(&mut self, buf: &[u8]) -> crate::Result<usize> {
        slice_write(&mut self.pos, &mut self.inner, buf)
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
impl<const N: usize> Write for Cursor<[u8; N]> {
    #[inline]
    fn write(&mut self, buf: &[u8]) -> crate::Result<usize> {
        slice_write(&mut self.pos, &mut self.inner, buf)
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
