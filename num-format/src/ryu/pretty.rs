mod exponent;
mod mantissa;

use self::mantissa::*;

use crate::format::{write_one_byte_with_sep, write_two_bytes_with_sep, Format};

use crate::ryu::common;
use crate::ryu::d2s::*;
use crate::ryu::f2s::*;
use crate::ryu::float::{FloatIeeeData32, FloatIeeeData64};
use core::ptr;

#[cfg(feature = "no-panic")]
use no_panic::no_panic;

/// Print f64 to the given buffer and return number of bytes written.
///
/// At most 24 bytes will be written.
///
/// ## Special cases
///
/// This function **does not** check for NaN or infinity. If the input
/// number is not a finite float, the printed representation will be some
/// correctly formatted but unspecified numerical value.
///
/// Please check [`is_finite`] yourself before calling this function, or
/// check [`is_nan`] and [`is_infinite`] and handle those cases yourself.
///
/// [`is_finite`]: https://doc.rust-lang.org/std/primitive.f64.html#method.is_finite
/// [`is_nan`]: https://doc.rust-lang.org/std/primitive.f64.html#method.is_nan
/// [`is_infinite`]: https://doc.rust-lang.org/std/primitive.f64.html#method.is_infinite
///
/// ## Safety
///
/// The `result` pointer argument must point to sufficiently many writable bytes
/// to hold Ryū's representation of `f`.
///
/// ## Example
///
/// ```
/// use std::{mem::MaybeUninit, slice, str};
///
/// let f = 1.234_f64;
/// let mut buffer = [MaybeUninit::<u8>::uninit(); 24];
///
/// let pos = num_format::ryu::raw::format64( f, buffer.as_mut_ptr() as *mut u8, 24, &num_format::Locale::en );
/// unsafe {
///     let dst = slice::from_raw_parts( (buffer.as_ptr() as *const u8).offset(pos as isize), 24 - pos );
///     let printed = str::from_utf8_unchecked(dst);
///     assert_eq!(printed, "1.234");
/// }
/// ```
#[must_use]
#[cfg_attr(feature = "no-panic", no_panic)]
pub fn format64<Fl, Fmt>(f: Fl, result: *mut u8, pos: usize, format: &Fmt) -> usize
where
    Fl: crate::ryu::Float<FloatIeeeData = FloatIeeeData64>,
    Fmt: Format,
{
    let ieee = f.parse_ieee_data();

    // Bail out early if we can just use ryu
    // (i.e. if we don't have a separator)
    let separator = format.separator().into_str();
    let grouping = format.grouping();
    let minus_sign = format.minus_sign().into_str();

    if separator.is_empty() || grouping == crate::Grouping::Posix {
        format64_posix(result, pos, &ieee)
    } else {
        format64_custom(result, pos, &ieee, separator, grouping, minus_sign)
    }
}

/// Print f32 to the given buffer and return number of bytes written.
///
/// At most 16 bytes will be written.
///
/// ## Special cases
///
/// This function **does not** check for NaN or infinity. If the input
/// number is not a finite float, the printed representation will be some
/// correctly formatted but unspecified numerical value.
///
/// Please check [`is_finite`] yourself before calling this function, or
/// check [`is_nan`] and [`is_infinite`] and handle those cases yourself.
///
/// [`is_finite`]: https://doc.rust-lang.org/std/primitive.f32.html#method.is_finite
/// [`is_nan`]: https://doc.rust-lang.org/std/primitive.f32.html#method.is_nan
/// [`is_infinite`]: https://doc.rust-lang.org/std/primitive.f32.html#method.is_infinite
///
/// ## Safety
///
/// The `result` pointer argument must point to sufficiently many writable bytes
/// to hold Ryū's representation of `f`.
///
/// ## Example
///
/// ```
/// use std::{mem::MaybeUninit, slice, str};
///
/// let f = 1.234_f32;
/// let mut buffer = [MaybeUninit::<u8>::uninit(); 24];
///
/// let pos = num_format::ryu::raw::format32( f, buffer.as_mut_ptr() as *mut u8, 24, &num_format::Locale::en );
/// unsafe {
///     let dst = slice::from_raw_parts( (buffer.as_ptr() as *const u8).offset(pos as isize), 24 - pos );
///     let printed = str::from_utf8_unchecked(dst);
///     assert_eq!(printed, "1.234");
/// }
/// ```
#[must_use]
#[cfg_attr(feature = "no-panic", no_panic)]
pub fn format32<Fl, Fmt>(f: Fl, result: *mut u8, pos: usize, format: &Fmt) -> usize
where
    Fl: crate::ryu::Float<FloatIeeeData = FloatIeeeData32>,
    Fmt: Format,
{
    let ieee = f.parse_ieee_data();

    // Bail out early if we can just use ryu
    // (i.e. if we don't have a separator)
    let separator = format.separator().into_str();
    let grouping = format.grouping();
    let minus_sign = format.minus_sign().into_str();

    if separator.is_empty() || grouping == crate::Grouping::Posix {
        format32_posix(result, pos, &ieee)
    } else {
        format32_custom(result, pos, &ieee, separator, grouping, minus_sign)
    }
}

#[inline(never)]
fn write_mantissa_with_sep(
    result: *mut u8,
    index: isize,
    sep: &mut crate::format::Sep<'_>,
    mantissa: u64,
) -> isize {
    let mut index = index;
    let mut n = mantissa;
    while n >= 10_000 {
        let remainder = n % 10_000;
        let table_index = ((remainder % 100) << 1) as isize;
        index = write_two_bytes_with_sep(result, index, sep, table_index);
        let table_index = ((remainder / 100) << 1) as isize;
        index = write_two_bytes_with_sep(result, index, sep, table_index);
        n /= 10_000;
    }

    while n >= 100 {
        let table_index = ((n % 100) << 1) as isize;
        index = write_two_bytes_with_sep(result, index, sep, table_index);
        n /= 100;
    }

    if n >= 10 {
        let table_index = (n << 1) as isize;
        index = write_two_bytes_with_sep(result, index, sep, table_index);
    } else {
        let table_index = (n << 1) as isize;
        index = write_one_byte_with_sep(result, index, sep, table_index + 1);
    }

    index
}

fn format64_posix(result: *mut u8, pos: usize, ieee: &FloatIeeeData64) -> usize {
    // Ieee
    let sign = ieee.is_negative;
    let ieee_mantissa = ieee.mantissa;
    let ieee_exponent = ieee.exponent;

    // Walk buf from tail to head
    let mut index = pos as isize;

    // Parse exponent and mantissa
    unsafe {
        if ieee_exponent == 0 && ieee_mantissa == 0 {
            index -= 3;
            ptr::copy_nonoverlapping(b"0.0".as_ptr(), result.offset(index), 3);
        } else {
            let v = d2d(ieee_mantissa, ieee_exponent);

            let length = decimal_length17(v.mantissa) as isize;
            let k = v.exponent as isize;
            let kk = length + k; // 10^(kk-1) <= v < 10^kk
            debug_assert!(k >= -324);

            if 0 <= k && kk <= 16 {
                // 1234e7 -> 12340000000.0
                index -= 1;
                *result.offset(index) = b'0';
                index -= 1;
                *result.offset(index) = b'.';

                for _ in length..kk {
                    index -= 1;
                    *result.offset(index) = b'0';
                }

                write_mantissa_long(v.mantissa, result.offset(index));
                index -= length;

                assert_eq!(pos - index as usize, kk as usize + 2);
            } else if 0 < kk && kk <= 16 {
                // 1234e-2 -> 12.34
                write_mantissa_long(v.mantissa, result.offset(index));
                index -= length;

                index -= 1;
                ptr::copy(result.offset(index + 1), result.offset(index), kk as usize);

                *result.offset(index + kk) = b'.';
                assert_eq!(pos - index as usize, length as usize + 1);
            } else if -5 < kk && kk <= 0 {
                // 1234e-6 -> 0.001234
                let offset = 2 - kk;

                write_mantissa_long(v.mantissa, result.offset(index));
                index -= length;

                for _ in 2..offset {
                    index -= 1;
                    *result.offset(index) = b'0';
                }

                index -= 1;
                *result.offset(index) = b'.';
                index -= 1;
                *result.offset(index) = b'0';
                assert_eq!(pos - index as usize, length as usize + offset as usize);
            } else if length == 1 {
                // same as 1e7 -> 10000000.0
                // 1e30 ->
                index -= 1;
                *result.offset(index) = b'0';
                index -= 1;
                *result.offset(index) = b'.';

                for _ in 1..kk {
                    index -= 1;
                    *result.offset(index) = b'0';
                }

                index -= 1;
                *result.offset(index) = b'0' + v.mantissa as u8;
                assert_eq!(pos - index as usize, kk as usize + 2);
            } else {
                // same as 1234e7 -> 12340000000.0
                // 1234e30 ->
                index -= 1;
                *result.offset(index) = b'0';
                index -= 1;
                *result.offset(index) = b'.';

                for _ in length..kk {
                    index -= 1;
                    *result.offset(index) = b'0';
                }

                write_mantissa_long(v.mantissa, result.offset(index));
                index -= length;

                assert_eq!(pos - index as usize, kk as usize + 2);
            }
        }

        if sign {
            index -= 1;
            *result.offset(index) = b'-';
        }
    } // end of unsafe

    index as usize
}

fn format64_custom(
    result: *mut u8,
    pos: usize,
    ieee: &FloatIeeeData64,
    separator: &str,
    grouping: crate::Grouping,
    minus_sign: &str,
) -> usize {
    // Ieee
    let sign = ieee.is_negative;
    let ieee_mantissa = ieee.mantissa;
    let ieee_exponent = ieee.exponent;

    // Walk buf from tail to head
    let mut index = pos as isize;

    // Collect separator information
    let mut sep = crate::format::Sep {
        ptr: separator.as_bytes().as_ptr(),
        len: separator.len(),
        pos: index,
        step: match grouping {
            crate::Grouping::Standard => 4isize,
            crate::Grouping::Indian => 3isize,
            crate::Grouping::Posix => unreachable!(),
        },
        phantom: std::marker::PhantomData,
    };

    // Parse exponent and mantissa
    unsafe {
        if ieee_exponent == 0 && ieee_mantissa == 0 {
            index -= 3;
            ptr::copy_nonoverlapping(b"0.0".as_ptr(), result.offset(index), 3);
        } else {
            let v = d2d(ieee_mantissa, ieee_exponent);

            let length = decimal_length17(v.mantissa) as isize;
            let k = v.exponent as isize;
            let kk = length + k; // 10^(kk-1) <= v < 10^kk
            debug_assert!(k >= -324);

            if 0 <= k && kk <= 16 {
                // 1234e7 -> 12340000000.0
                index -= 1;
                *result.offset(index) = b'0';
                index -= 1;
                *result.offset(index) = b'.';

                sep.pos = index - 4; // start sep
                for _ in length..kk {
                    let n = 0isize;
                    let table_index = n << 1;
                    index = write_one_byte_with_sep(result, index, &mut sep, table_index + 1);
                }

                index = write_mantissa_with_sep(result, index, &mut sep, v.mantissa);
            } else if 0 < kk && kk <= 16 {
                // 1234e-2 -> 12.34
                let d = 10_u64.pow(-k as u32);
                let q = v.mantissa / d;

                write_mantissa_long(v.mantissa, result.offset(index));
                index -= -k;

                index -= 1;
                *result.offset(index) = b'.';

                sep.pos = index - 4; // start sep
                index = write_mantissa_with_sep(result, index, &mut sep, q);
            } else if -5 < kk && kk <= 0 {
                // 1234e-6 -> 0.001234
                let offset = 2 - kk;

                write_mantissa_long(v.mantissa, result.offset(index));
                index -= length;

                for _ in 2..offset {
                    index -= 1;
                    *result.offset(index) = b'0';
                }

                index -= 1;
                *result.offset(index) = b'.';
                index -= 1;
                *result.offset(index) = b'0';
                assert_eq!(pos - index as usize, length as usize + offset as usize);
            } else if length == 1 {
                // same as 1e7 -> 10000000.0
                // 1e30 ->
                index -= 1;
                *result.offset(index) = b'0';
                index -= 1;
                *result.offset(index) = b'.';

                sep.pos = index - 4; // start sep
                for _ in 1..kk {
                    let n = 0isize;
                    let table_index = n << 1;
                    index = write_one_byte_with_sep(result, index, &mut sep, table_index + 1);
                }

                // One byte only mantissa
                let n = v.mantissa as isize;
                let table_index = n << 1;
                index = write_one_byte_with_sep(result, index, &mut sep, table_index + 1);
            } else {
                // same as 1234e7 -> 12340000000.0
                // 1234e30 ->
                index -= 1;
                *result.offset(index) = b'0';
                index -= 1;
                *result.offset(index) = b'.';

                sep.pos = index - 4; // start sep
                for _ in length..kk {
                    let n = 0isize;
                    let table_index = n << 1;
                    index = write_one_byte_with_sep(result, index, &mut sep, table_index + 1);
                }

                index = write_mantissa_with_sep(result, index, &mut sep, v.mantissa);
            }
        }

        if sign {
            let minus_len = minus_sign.len();
            index -= minus_len as isize;
            for (i, byte) in minus_sign.as_bytes().iter().enumerate() {
                *result.offset(index + i as isize) = *byte;
            }
        }
    } // end of unsafe

    index as usize
}

fn format32_posix(result: *mut u8, pos: usize, ieee: &FloatIeeeData32) -> usize {
    // Ieee
    let sign = ieee.is_negative;
    let ieee_mantissa = ieee.mantissa;
    let ieee_exponent = ieee.exponent;

    // Walk buf from tail to head
    let mut index = pos as isize;

    // Parse exponent and mantissa
    unsafe {
        if ieee_exponent == 0 && ieee_mantissa == 0 {
            index -= 3;
            ptr::copy_nonoverlapping(b"0.0".as_ptr(), result.offset(index), 3);
        } else {
            let v = f2d(ieee_mantissa, ieee_exponent);

            let length = common::decimal_length9(v.mantissa) as isize;
            let k = v.exponent as isize;
            let kk = length + k; // 10^(kk-1) <= v < 10^kk
            debug_assert!(k >= -45);

            if 0 <= k && kk <= 13 {
                // 1234e7 -> 12340000000.0
                index -= 1;
                *result.offset(index) = b'0';
                index -= 1;
                *result.offset(index) = b'.';

                for _ in length..kk {
                    index -= 1;
                    *result.offset(index) = b'0';
                }

                write_mantissa(v.mantissa, result.offset(index));
                index -= length;

                assert_eq!(pos - index as usize, kk as usize + 2);
            } else if 0 < kk && kk <= 13 {
                // 1234e-2 -> 12.34
                write_mantissa(v.mantissa, result.offset(index));
                index -= length;

                index -= 1;
                ptr::copy(result.offset(index + 1), result.offset(index), kk as usize);

                *result.offset(index + kk) = b'.';
                assert_eq!(pos - index as usize, length as usize + 1);
            } else if -6 < kk && kk <= 0 {
                // 1234e-6 -> 0.001234
                let offset = 2 - kk;

                write_mantissa(v.mantissa, result.offset(index));
                index -= length;

                for _ in 2..offset {
                    index -= 1;
                    *result.offset(index) = b'0';
                }

                index -= 1;
                *result.offset(index) = b'.';
                index -= 1;
                *result.offset(index) = b'0';
                assert_eq!(pos - index as usize, length as usize + offset as usize);
            } else if length == 1 {
                // same as 1e7 -> 10000000.0
                // 1e30 ->
                index -= 1;
                *result.offset(index) = b'0';
                index -= 1;
                *result.offset(index) = b'.';

                for _ in 1..kk {
                    index -= 1;
                    *result.offset(index) = b'0';
                }

                index -= 1;
                *result.offset(index) = b'0' + v.mantissa as u8;
                assert_eq!(pos - index as usize, kk as usize + 2);
            } else {
                // same as 1234e7 -> 12340000000.0
                // 1234e30 ->
                index -= 1;
                *result.offset(index) = b'0';
                index -= 1;
                *result.offset(index) = b'.';

                for _ in length..kk {
                    index -= 1;
                    *result.offset(index) = b'0';
                }

                write_mantissa(v.mantissa, result.offset(index));
                index -= length;

                assert_eq!(pos - index as usize, kk as usize + 2);
            }
        }

        if sign {
            index -= 1;
            *result.offset(index) = b'-';
        }
    } // end of unsafe
    index as usize
}

fn format32_custom(
    result: *mut u8,
    pos: usize,
    ieee: &FloatIeeeData32,
    separator: &str,
    grouping: crate::Grouping,
    minus_sign: &str,
) -> usize {
    // Ieee
    let sign = ieee.is_negative;
    let ieee_mantissa = ieee.mantissa;
    let ieee_exponent = ieee.exponent;

    // Walk buf from tail to head
    let mut index = pos as isize;

    // Collect separator information
    unsafe {
        let mut sep = crate::format::Sep {
            ptr: separator.as_bytes().as_ptr(),
            len: separator.len(),
            pos: index,
            step: match grouping {
                crate::Grouping::Standard => 4isize,
                crate::Grouping::Indian => 3isize,
                crate::Grouping::Posix => unreachable!(),
            },
            phantom: std::marker::PhantomData,
        };

        // Parse exponent and mantissa
        if ieee_exponent == 0 && ieee_mantissa == 0 {
            index -= 3;
            ptr::copy_nonoverlapping(b"0.0".as_ptr(), result.offset(index), 3);
        } else {
            let v = f2d(ieee_mantissa, ieee_exponent);

            let length = common::decimal_length9(v.mantissa) as isize;
            let k = v.exponent as isize;
            let kk = length + k; // 10^(kk-1) <= v < 10^kk
            debug_assert!(k >= -45);

            if 0 <= k && kk <= 13 {
                // 1234e7 -> 12340000000.0
                index -= 1;
                *result.offset(index) = b'0';
                index -= 1;
                *result.offset(index) = b'.';

                sep.pos = index - 4; // start sep
                for _ in length..kk {
                    let n = 0isize;
                    let table_index = n << 1;
                    index = write_one_byte_with_sep(result, index, &mut sep, table_index + 1);
                }

                index = write_mantissa_with_sep(result, index, &mut sep, v.mantissa as u64);
            } else if 0 < kk && kk <= 16 {
                // 1234e-2 -> 12.34
                let d = 10_u32.pow(-k as u32);
                let q = v.mantissa / d;

                write_mantissa(v.mantissa, result.offset(index));
                index -= -k;

                index -= 1;
                *result.offset(index) = b'.';

                sep.pos = index - 4; // start sep
                index = write_mantissa_with_sep(result, index, &mut sep, q as u64);
            } else if -6 < kk && kk <= 0 {
                // 1234e-6 -> 0.001234
                let offset = 2 - kk;

                write_mantissa(v.mantissa, result.offset(index));
                index -= length;

                for _ in 2..offset {
                    index -= 1;
                    *result.offset(index) = b'0';
                }

                index -= 1;
                *result.offset(index) = b'.';
                index -= 1;
                *result.offset(index) = b'0';
                assert_eq!(pos - index as usize, length as usize + offset as usize);
            } else if length == 1 {
                // same as 1e7 -> 10000000.0
                // 1e30 ->
                index -= 1;
                *result.offset(index) = b'0';
                index -= 1;
                *result.offset(index) = b'.';

                sep.pos = index - 4; // start sep
                for _ in 1..kk {
                    let n = 0isize;
                    let table_index = n << 1;
                    index = write_one_byte_with_sep(result, index, &mut sep, table_index + 1);
                }

                // One byte only mantissa
                let n = v.mantissa as isize;
                let table_index = n << 1;
                index = write_one_byte_with_sep(result, index, &mut sep, table_index + 1);
            } else {
                // same as 1234e7 -> 12340000000.0
                // 1234e30 ->
                index -= 1;
                *result.offset(index) = b'0';
                index -= 1;
                *result.offset(index) = b'.';

                sep.pos = index - 4; // start sep
                for _ in length..kk {
                    let n = 0isize;
                    let table_index = n << 1;
                    index = write_one_byte_with_sep(result, index, &mut sep, table_index + 1);
                }

                index = write_mantissa_with_sep(result, index, &mut sep, v.mantissa as u64);
            }
        }

        if sign {
            let minus_len = minus_sign.len();
            index -= minus_len as isize;
            for (i, byte) in minus_sign.as_bytes().iter().enumerate() {
                *result.offset(index + i as isize) = *byte;
            }
        }
    } // end of unsafe

    index as usize
}
