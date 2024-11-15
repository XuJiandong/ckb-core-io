use super::{BorrowedBuf, Read, Result, Write, DEFAULT_BUF_SIZE};
use core::mem::MaybeUninit;
/// Copies the entire contents of a reader into a writer.
///
/// This function will continuously read data from `reader` and then
/// write it into `writer` in a streaming fashion until `reader`
/// returns EOF.
///
/// On success, the total number of bytes that were copied from
/// `reader` to `writer` is returned.
///
/// If you want to copy the contents of one file to another and you’re
/// working with filesystem paths, see the [`fs::copy`] function.
///
/// [`fs::copy`]: crate::fs::copy
///
/// # Errors
///
/// This function will return an error immediately if any call to [`read`] or
/// [`write`] returns an error. All instances of [`ErrorKind::Interrupted`] are
/// handled by this function and the underlying operation is retried.
///
/// [`read`]: Read::read
/// [`write`]: Write::write
/// [`ErrorKind::Interrupted`]: crate::io::ErrorKind::Interrupted
///
/// # Examples
///
/// ```
/// use std::io;
///
/// fn main() -> io::Result<()> {
///     let mut reader: &[u8] = b"hello";
///     let mut writer: Vec<u8> = vec![];
///
///     io::copy(&mut reader, &mut writer)?;
///
///     assert_eq!(&b"hello"[..], &writer[..]);
///     Ok(())
/// }
/// ```
///
/// # Platform-specific behavior
///
/// On Linux (including Android), this function uses `copy_file_range(2)`,
/// `sendfile(2)` or `splice(2)` syscalls to move data directly between file
/// descriptors if possible.
///
/// Note that platform-specific behavior [may change in the future][changes].
///
/// [changes]: crate::io#platform-specific-behavior
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
