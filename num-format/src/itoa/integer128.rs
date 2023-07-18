use super::udiv128;
use super::Integer;

use core::{ptr, slice, str};

#[cfg(feature = "no-panic")]
use no_panic::no_panic;

/// Integer for i128
impl super::Integer for i128 {
    type Buffer = (*mut u8, usize); // ptr, pos

    #[allow(unused_comparisons)]
    #[inline(never)]
    #[cfg_attr(feature = "no-panic", no_panic)]
    fn write<'a>(self, buf: &mut Self::Buffer) -> &'a str {
        let is_nonnegative = self >= 0;
        let n = if !is_nonnegative {
            self as u128
        } else {
            // convert the negative num to positive by summing 1 to it's 2 complement
            (!(self as u128)).wrapping_add(1)
        };

        let buf_ptr = buf.0;
        let buf_len = buf.1;
        let mut curr = write_u128(n, buf_ptr, buf_len) as isize;

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

/// Integer for u128
impl super::Integer for u128 {
    type Buffer = (*mut u8, usize); // ptr, pos

    #[allow(unused_comparisons)]
    #[inline(never)]
    #[cfg_attr(feature = "no-panic", no_panic)]
    fn write<'a>(self, buf: &mut Self::Buffer) -> &'a str {
        let buf_ptr = buf.0;
        let buf_len = buf.1;
        let curr = write_u128(self, buf_ptr, buf_len) as isize;

        let len = buf_len - curr as usize;
        unsafe {
            let bytes = slice::from_raw_parts(buf_ptr.offset(curr), len);
            str::from_utf8_unchecked(bytes)
        }
    }
}

#[inline(never)]
fn write_u128(n: u128, buf: *mut u8, pos: usize) -> usize {
    let mut curr = pos as isize;

    unsafe {
        // Divide by 10^19 which is the highest power less than 2^64.
        let (n, rem) = udiv128::udivmod_1e19(n);
        curr -= rem.write(&mut (buf, curr as usize)).len() as isize;

        if n != 0 {
            // Memset the base10 leading zeros of rem.
            let target = pos as isize - 19;
            ptr::write_bytes(buf.offset(target), b'0', (curr - target) as usize);
            curr = target;

            // Divide by 10^19 again.
            let (n, rem) = udiv128::udivmod_1e19(n);
            curr -= rem.write(&mut (buf, curr as usize)).len() as isize;

            if n != 0 {
                // Memset the leading zeros.
                let target = pos as isize - 38;
                ptr::write_bytes(buf.offset(target), b'0', (curr - target) as usize);
                curr = target;

                // There is at most one digit left
                // because u128::max / 10^19 / 10^19 is 3.
                curr -= 1;
                *buf.offset(curr) = (n as u8) + b'0';
            }
        }
    }

    curr as usize
}
