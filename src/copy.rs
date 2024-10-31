use super::{BorrowedBuf, Read, Result, Write, DEFAULT_BUF_SIZE};
use core::mem::MaybeUninit;
pub fn copy<R: ?Sized + Read, W: ?Sized + Write>(reader: &mut R, writer: &mut W) -> Result<u64> {
    stack_buffer_copy(reader, writer)
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
