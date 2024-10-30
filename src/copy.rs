use super::{BorrowedBuf, BufReader, BufWriter, Read, Result, Write, DEFAULT_BUF_SIZE};
use crate::IoSlice;
use alloc::collections::VecDeque;
use core::alloc::Allocator;
use core::cmp;
use core::mem::MaybeUninit;
pub fn copy<R: ?Sized, W: ?Sized>(reader: &mut R, writer: &mut W) -> Result<u64>
where
    R: Read,
    W: Write,
{
    cfg_if::cfg_if! { if # [cfg (any (target_os = "linux" , target_os = "android"))] { crate :: sys :: kernel_copy :: copy_spec (reader , writer) } else { generic_copy (reader , writer) } }
}
pub(crate) fn generic_copy<R: ?Sized, W: ?Sized>(reader: &mut R, writer: &mut W) -> Result<u64>
where
    R: Read,
    W: Write,
{
    let read_buf = BufferedReaderSpec::buffer_size(reader);
    let write_buf = BufferedWriterSpec::buffer_size(writer);
    if read_buf >= DEFAULT_BUF_SIZE && read_buf >= write_buf {
        return BufferedReaderSpec::copy_to(reader, writer);
    }
    BufferedWriterSpec::copy_from(writer, reader)
}
trait BufferedReaderSpec {
    fn buffer_size(&self) -> usize;
    fn copy_to(&mut self, to: &mut (impl Write + ?Sized)) -> Result<u64>;
}
impl<T> BufferedReaderSpec for T
where
    Self: Read,
    T: ?Sized,
{
    #[inline]
    fn buffer_size(&self) -> usize {
        0
    }
    fn copy_to(&mut self, _to: &mut (impl Write + ?Sized)) -> Result<u64> {
        unreachable!("only called from specializations")
    }
}
impl BufferedReaderSpec for &[u8] {
    fn buffer_size(&self) -> usize {
        usize::MAX
    }
    fn copy_to(&mut self, to: &mut (impl Write + ?Sized)) -> Result<u64> {
        let len = self.len();
        to.write_all(self)?;
        *self = &self[len..];
        Ok(len as u64)
    }
}
impl<A: Allocator> BufferedReaderSpec for VecDeque<u8, A> {
    fn buffer_size(&self) -> usize {
        usize::MAX
    }
    fn copy_to(&mut self, to: &mut (impl Write + ?Sized)) -> Result<u64> {
        let len = self.len();
        let (front, back) = self.as_slices();
        let bufs = &mut [IoSlice::new(front), IoSlice::new(back)];
        to.write_all_vectored(bufs)?;
        self.clear();
        Ok(len as u64)
    }
}
impl<I> BufferedReaderSpec for BufReader<I>
where
    Self: Read,
    I: ?Sized,
{
    fn buffer_size(&self) -> usize {
        self.capacity()
    }
    fn copy_to(&mut self, to: &mut (impl Write + ?Sized)) -> Result<u64> {
        let mut len = 0;
        loop {
            match self.read(&mut []) {
                Ok(_) => {}
                Err(e) if e.is_interrupted() => continue,
                Err(e) => return Err(e),
            }
            let buf = self.buffer();
            if self.buffer().len() == 0 {
                return Ok(len);
            }
            to.write_all(buf)?;
            len += buf.len() as u64;
            self.discard_buffer();
        }
    }
}
trait BufferedWriterSpec: Write {
    fn buffer_size(&self) -> usize;
    fn copy_from<R: Read + ?Sized>(&mut self, reader: &mut R) -> Result<u64>;
}
impl<W: Write + ?Sized> BufferedWriterSpec for W {
    #[inline]
    fn buffer_size(&self) -> usize {
        0
    }
    fn copy_from<R: Read + ?Sized>(&mut self, reader: &mut R) -> Result<u64> {
        stack_buffer_copy(reader, self)
    }
}
impl<I: Write + ?Sized> BufferedWriterSpec for BufWriter<I> {
    fn buffer_size(&self) -> usize {
        self.capacity()
    }
    fn copy_from<R: Read + ?Sized>(&mut self, reader: &mut R) -> Result<u64> {
        if self.capacity() < DEFAULT_BUF_SIZE {
            return stack_buffer_copy(reader, self);
        }
        let mut len = 0;
        let mut init = 0;
        loop {
            let buf = self.buffer_mut();
            let mut read_buf: BorrowedBuf<'_> = buf.spare_capacity_mut().into();
            unsafe {
                read_buf.set_init(init);
            }
            if read_buf.capacity() >= DEFAULT_BUF_SIZE {
                let mut cursor = read_buf.unfilled();
                match reader.read_buf(cursor.reborrow()) {
                    Ok(()) => {
                        let bytes_read = cursor.written();
                        if bytes_read == 0 {
                            return Ok(len);
                        }
                        init = read_buf.init_len() - bytes_read;
                        len += bytes_read as u64;
                        unsafe { buf.set_len(buf.len() + bytes_read) };
                    }
                    Err(ref e) if e.is_interrupted() => {}
                    Err(e) => return Err(e),
                }
            } else {
                self.flush_buf()?;
                init = 0;
            }
        }
    }
}
impl BufferedWriterSpec for Vec<u8> {
    fn buffer_size(&self) -> usize {
        cmp::max(DEFAULT_BUF_SIZE, self.capacity() - self.len())
    }
    fn copy_from<R: Read + ?Sized>(&mut self, reader: &mut R) -> Result<u64> {
        reader
            .read_to_end(self)
            .map(|bytes| u64::try_from(bytes).expect("usize overflowed u64"))
    }
}
pub fn stack_buffer_copy<R: Read + ?Sized, W: Write + ?Sized>(
    reader: &mut R,
    writer: &mut W,
) -> Result<u64> {
    let buf: &mut [_] = &mut [MaybeUninit::uninit(); DEFAULT_BUF_SIZE];
    let mut buf: BorrowedBuf<'_> = buf.into();
    let mut len = 0;
    loop {
        match reader.read_buf(buf.unfilled()) {
            Ok(()) => {}
            Err(e) if e.is_interrupted() => continue,
            Err(e) => return Err(e),
        };
        if buf.filled().is_empty() {
            break;
        }
        len += buf.filled().len() as u64;
        writer.write_all(buf.filled())?;
        buf.clear();
    }
    Ok(len)
}
