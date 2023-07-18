use crate::DIGIT_TABLE;

use core::mem::{self};
use core::{ptr, slice, str};
#[cfg(feature = "no-panic")]
use no_panic::no_panic;

// Adaptation of the original implementation at
// https://github.com/rust-lang/rust/blob/b8214dc6c6fc20d0a660fb5700dca9ebf51ebe89/src/libcore/fmt/num.rs#L188-L266

impl super::Integer for i64 {
    type Buffer = (*mut u8, usize); // ptr, pos

    #[allow(unused_comparisons)]
    #[inline(never)]
    #[cfg_attr(feature = "no-panic", no_panic)]
    fn write<'a>(self, buf: &mut Self::Buffer) -> &'a str {
        let is_nonnegative = self >= 0;
        let n = if is_nonnegative {
            self as u64
        } else {
            // convert the negative num to positive by summing 1 to it's 2 complement
            (!(self as u64)).wrapping_add(1)
        };

        let buf_ptr = buf.0;
        let buf_len = buf.1;
        let mut curr = write_u64(n, buf_ptr, buf_len) as isize;

        if !is_nonnegative {
            curr -= 1;
            unsafe {
                *buf_ptr.offset(curr) = b'-';
            }
        }

        let len = buf_len - curr as usize;
        unsafe {
            let bytes = slice::from_raw_parts(buf_ptr.offset(curr), len);
            str::from_utf8_unchecked(bytes)
        }
    }
}

impl super::Integer for u64 {
    type Buffer = (*mut u8, usize); // ptr, pos

    #[allow(unused_comparisons)]
    #[inline(never)]
    #[cfg_attr(feature = "no-panic", no_panic)]
    fn write<'a>(self, buf: &mut Self::Buffer) -> &'a str {
        let buf_ptr = buf.0;
        let buf_len = buf.1;
        let curr = write_u64(self, buf_ptr, buf_len) as isize;

        let len = buf_len - curr as usize;
        unsafe {
            let bytes = slice::from_raw_parts(buf_ptr.offset(curr), len);
            str::from_utf8_unchecked(bytes)
        }
    }
}

#[inline(never)]
fn write_u64(mut n: u64, buf: *mut u8, pos: usize) -> usize {
    let mut curr = pos as isize;
    let lut_ptr = DIGIT_TABLE.as_ptr();

    unsafe {
        // need at least 16 bits for the 4-characters-at-a-time to work.
        if mem::size_of::<u64>() >= 2 {
            // eagerly decode 4 characters at a time
            while n >= 10000 {
                let rem = (n % 10000) as isize;
                n /= 10000;

                let d1 = (rem / 100) << 1;
                let d2 = (rem % 100) << 1;
                curr -= 4;
                ptr::copy_nonoverlapping(lut_ptr.offset(d1), buf.offset(curr), 2);
                ptr::copy_nonoverlapping(lut_ptr.offset(d2), buf.offset(curr + 2), 2);
            }
        }

        // if we reach here numbers are <= 9999, so at most 4 chars long
        let mut n = n as isize; // possibly reduce 64bit math

        // decode 2 more chars, if > 2 chars
        if n >= 100 {
            let d1 = (n % 100) << 1;
            n /= 100;
            curr -= 2;
            ptr::copy_nonoverlapping(lut_ptr.offset(d1), buf.offset(curr), 2);
        }

        // decode last 1 or 2 chars
        if n < 10 {
            curr -= 1;
            *buf.offset(curr) = (n as u8) + b'0';
        } else {
            let d1 = n << 1;
            curr -= 2;
            ptr::copy_nonoverlapping(lut_ptr.offset(d1), buf.offset(curr), 2);
        }
    }

    curr as usize
}
