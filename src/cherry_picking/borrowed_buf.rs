use alloc::fmt::{self, Debug, Formatter};
use core::mem::{self, MaybeUninit};
use core::{cmp, ptr};

pub struct BorrowedBuf<'data> {
    buf: &'data mut [MaybeUninit<u8>],
    filled: usize,
    init: usize,
}

impl Debug for BorrowedBuf<'_> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.debug_struct("BorrowedBuf")
            .field("init", &self.init)
            .field("filled", &self.filled)
            .field("capacity", &self.capacity())
            .finish()
    }
}

impl<'data> From<&'data mut [u8]> for BorrowedBuf<'data> {
    #[inline]
    fn from(slice: &'data mut [u8]) -> BorrowedBuf<'data> {
        let len = slice.len();
        BorrowedBuf {
            buf: unsafe { &mut *(slice as *mut [u8] as *mut [MaybeUninit<u8>]) },
            filled: 0,
            init: len,
        }
    }
}

impl<'data> From<&'data mut [MaybeUninit<u8>]> for BorrowedBuf<'data> {
    #[inline]
    fn from(buf: &'data mut [MaybeUninit<u8>]) -> BorrowedBuf<'data> {
        BorrowedBuf {
            buf,
            filled: 0,
            init: 0,
        }
    }
}

impl<'data> BorrowedBuf<'data> {
    #[inline]
    pub fn capacity(&self) -> usize {
        self.buf.len()
    }

    #[inline]
    pub fn len(&self) -> usize {
        self.filled
    }

    #[inline]
    pub fn is_empty(&self) -> bool {
        self.filled == 0
    }

    #[inline]
    pub fn init_len(&self) -> usize {
        self.init
    }

    #[inline]
    pub fn filled(&self) -> &[u8] {
        unsafe {
            let buf = self.buf.get_unchecked(..self.filled);
            &*(buf as *const [MaybeUninit<u8>] as *const [u8])
        }
    }
    #[inline]
    pub fn filled_mut(&mut self) -> &mut [u8] {
        unsafe {
            let buf = self.buf.get_unchecked_mut(..self.filled);
            &mut *(buf as *mut [MaybeUninit<u8>] as *mut [u8])
        }
    }

    #[inline]
    pub fn unfilled<'this>(&'this mut self) -> BorrowedCursor<'this> {
        BorrowedCursor {
            start: self.filled,
            buf: unsafe {
                mem::transmute::<&'this mut BorrowedBuf<'data>, &'this mut BorrowedBuf<'this>>(self)
            },
        }
    }
    #[inline]
    pub fn clear(&mut self) -> &mut Self {
        self.filled = 0;
        self
    }

    #[inline]
    pub unsafe fn set_init(&mut self, n: usize) -> &mut Self {
        self.init = cmp::max(self.init, n);
        self
    }
}

#[derive(Debug)]
pub struct BorrowedCursor<'a> {
    buf: &'a mut BorrowedBuf<'a>,
    start: usize,
}

impl<'a> BorrowedCursor<'a> {
    #[inline]
    pub fn reborrow<'this>(&'this mut self) -> BorrowedCursor<'this> {
        BorrowedCursor {
            buf: unsafe {
                mem::transmute::<&'this mut BorrowedBuf<'a>, &'this mut BorrowedBuf<'this>>(
                    self.buf,
                )
            },
            start: self.start,
        }
    }
    #[inline]
    pub fn capacity(&self) -> usize {
        self.buf.capacity() - self.buf.filled
    }

    #[inline]
    pub fn written(&self) -> usize {
        self.buf.filled - self.start
    }

    #[inline]
    pub fn init_ref(&self) -> &[u8] {
        unsafe {
            let buf = self.buf.buf.get_unchecked(self.buf.filled..self.buf.init);
            &*(buf as *const [MaybeUninit<u8>] as *const [u8])
        }
    }
    #[inline]
    pub fn init_mut(&mut self) -> &mut [u8] {
        unsafe {
            let buf = self
                .buf
                .buf
                .get_unchecked_mut(self.buf.filled..self.buf.init);
            &mut *(buf as *mut [MaybeUninit<u8>] as *mut [u8])
        }
    }
    #[inline]
    pub fn uninit_mut(&mut self) -> &mut [MaybeUninit<u8>] {
        unsafe { self.buf.buf.get_unchecked_mut(self.buf.init..) }
    }
    #[inline]
    pub unsafe fn as_mut(&mut self) -> &mut [MaybeUninit<u8>] {
        unsafe { self.buf.buf.get_unchecked_mut(self.buf.filled..) }
    }
    #[inline]
    pub fn advance(&mut self, n: usize) -> &mut Self {
        let filled = self.buf.filled.checked_add(n).unwrap();
        assert!(filled <= self.buf.init);

        self.buf.filled = filled;
        self
    }
    #[inline]
    pub unsafe fn advance_unchecked(&mut self, n: usize) -> &mut Self {
        self.buf.filled += n;
        self.buf.init = cmp::max(self.buf.init, self.buf.filled);
        self
    }

    #[inline]
    pub fn ensure_init(&mut self) -> &mut Self {
        let uninit = self.uninit_mut();
        unsafe {
            ptr::write_bytes(uninit.as_mut_ptr(), 0, uninit.len());
        }
        self.buf.init = self.buf.capacity();

        self
    }
    #[inline]
    pub unsafe fn set_init(&mut self, n: usize) -> &mut Self {
        self.buf.init = cmp::max(self.buf.init, self.buf.filled + n);
        self
    }
    #[inline]
    pub fn append(&mut self, buf: &[u8]) {
        assert!(self.capacity() >= buf.len());
        unsafe {
            ptr::copy(
                buf.as_ptr(),
                self.as_mut().as_mut_ptr() as *mut u8,
                buf.len(),
            );
        }
        unsafe {
            self.set_init(buf.len());
        }
        self.buf.filled += buf.len();
    }
}
