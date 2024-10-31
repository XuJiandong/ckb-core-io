pub const fn memchr(x: u8, text: &[u8]) -> Option<usize> {
    let mut i = 0;
    while i < text.len() {
        if text[i] == x {
            return Some(i);
        }

        i += 1;
    }

    None
}

/// Returns the last index matching the byte `x` in `text`.
pub fn memrchr(x: u8, text: &[u8]) -> Option<usize> {
    let mut i = text.len();
    while i > 0 {
        i -= 1;
        if text[i] == x {
            return Some(i);
        }
    }
    None
}
