use crate::format::Format;

use crate::ryu::d2s::{DOUBLE_EXPONENT_BITS, DOUBLE_MANTISSA_BITS};
use crate::ryu::f2s::{FLOAT_EXPONENT_BITS, FLOAT_MANTISSA_BITS};
use crate::ryu::raw;

use core::str;
#[cfg(feature = "no-panic")]
use no_panic::no_panic;

const NAN: &str = "NaN";
const INFINITY: &str = "inf";
const NEG_INFINITY: &str = "-inf";

#[derive(Debug, Copy, Clone)]
pub struct FloatIeeeData32 {
    pub is_negative: bool,
    pub mantissa: u32, // 32 bit mantissa
    pub exponent: u32,
}

#[derive(Debug, Copy, Clone)]
pub struct FloatIeeeData64 {
    pub is_negative: bool,
    pub mantissa: u64, // 64 bit mantissa
    pub exponent: u32,
}

/// A floating point number, f32 or f64, that can be written into a buffer.
///
pub trait Float: private::Sealed {
    /// *mut u8 bytes with pos
    type Buffer: 'static;
    /// Ieee about
    type FloatIeeeData: 'static;

    ///
    fn parse_ieee_data(self) -> Self::FloatIeeeData;

    ///
    fn is_nonfinite(self) -> bool;

    ///
    fn format_nonfinite(self) -> &'static str;

    ///
    fn format_finite<'a, Fmt: Format>(self, buf: &mut Self::Buffer, format: &Fmt) -> &'a str;

    ///
    fn write<'a>(self, buf: &mut Self::Buffer) -> &'a str;
}

// Seal to prevent downstream implementations of the Float trait.
mod private {
    pub trait Sealed: Copy {}
}

impl private::Sealed for f32 {}

impl Float for f32 {
    type Buffer = (*mut u8, usize); // ptr, pos
    type FloatIeeeData = FloatIeeeData32;

    #[inline(never)]
    fn parse_ieee_data(self) -> Self::FloatIeeeData {
        let bits = self.to_bits();
        let sign = ((bits >> (FLOAT_MANTISSA_BITS + FLOAT_EXPONENT_BITS)) & 1) != 0;
        let ieee_mantissa = bits & ((1u32 << FLOAT_MANTISSA_BITS) - 1);
        let ieee_exponent = (bits >> FLOAT_MANTISSA_BITS) & ((1u32 << FLOAT_EXPONENT_BITS) - 1);

        FloatIeeeData32 {
            is_negative: sign,
            mantissa: ieee_mantissa,
            exponent: ieee_exponent,
        }
    }

    #[inline(never)]
    fn is_nonfinite(self) -> bool {
        const EXP_MASK: u32 = 0x7f800000;
        let bits = self.to_bits();
        bits & EXP_MASK == EXP_MASK
    }

    #[cold]
    #[cfg_attr(feature = "no-panic", inline)]
    fn format_nonfinite(self) -> &'static str {
        const MANTISSA_MASK: u32 = 0x007fffff;
        const SIGN_MASK: u32 = 0x80000000;
        let bits = self.to_bits();
        if bits & MANTISSA_MASK != 0 {
            NAN
        } else if bits & SIGN_MASK != 0 {
            NEG_INFINITY
        } else {
            INFINITY
        }
    }

    #[inline(never)]
    fn format_finite<'a, Fmt: Format>(self, buf: &mut Self::Buffer, format: &Fmt) -> &'a str {
        let buf_ptr = buf.0;
        let pos1 = buf.1;
        let pos0 = raw::format32(self, buf_ptr, pos1, format);
        let len = pos1 - pos0;

        unsafe {
            let slice = std::slice::from_raw_parts(buf_ptr.offset(pos0 as isize), len);
            str::from_utf8_unchecked(slice)
        }
    }

    #[inline(never)]
    fn write<'a>(self, buf: &mut Self::Buffer) -> &'a str {
        format_float(self, buf.0, buf.1, &crate::Locale::en_US_POSIX)
    }
}

impl private::Sealed for f64 {}

impl Float for f64 {
    type Buffer = (*mut u8, usize); // ptr, pos
    type FloatIeeeData = FloatIeeeData64;

    #[inline(never)]
    fn parse_ieee_data(self) -> Self::FloatIeeeData {
        let bits = self.to_bits();
        let sign = ((bits >> (DOUBLE_MANTISSA_BITS + DOUBLE_EXPONENT_BITS)) & 1) != 0;
        let ieee_mantissa = bits & ((1u64 << DOUBLE_MANTISSA_BITS) - 1);
        let ieee_exponent =
            (bits >> DOUBLE_MANTISSA_BITS) as u32 & ((1u32 << DOUBLE_EXPONENT_BITS) - 1);

        FloatIeeeData64 {
            is_negative: sign,
            mantissa: ieee_mantissa,
            exponent: ieee_exponent,
        }
    }

    #[inline(never)]
    fn is_nonfinite(self) -> bool {
        const EXP_MASK: u64 = 0x7ff0000000000000;
        let bits = self.to_bits();
        bits & EXP_MASK == EXP_MASK
    }

    #[cold]
    #[cfg_attr(feature = "no-panic", inline)]
    fn format_nonfinite(self) -> &'static str {
        const MANTISSA_MASK: u64 = 0x000fffffffffffff;
        const SIGN_MASK: u64 = 0x8000000000000000;
        let bits = self.to_bits();
        if bits & MANTISSA_MASK != 0 {
            NAN
        } else if bits & SIGN_MASK != 0 {
            NEG_INFINITY
        } else {
            INFINITY
        }
    }

    #[inline(never)]
    fn format_finite<'a, Fmt: Format>(self, buf: &mut Self::Buffer, format: &Fmt) -> &'a str {
        let buf_ptr = buf.0;
        let pos1 = buf.1;
        let pos0 = raw::format64(self, buf_ptr, pos1, format);
        let len = pos1 - pos0;

        unsafe {
            let slice = std::slice::from_raw_parts(buf_ptr.offset(pos0 as isize), len);
            str::from_utf8_unchecked(slice)
        }
    }

    #[inline(never)]
    fn write<'a>(self, buf: &mut Self::Buffer) -> &'a str {
        format_float(self, buf.0, buf.1, &crate::Locale::en_US_POSIX)
    }
}

#[inline(never)]
#[cfg_attr(feature = "no-panic", no_panic)]
pub fn format_float<'a, Fl, Fmt>(f: Fl, buf: *mut u8, pos: usize, format: &Fmt) -> &'a str
where
    Fl: crate::ryu::Float<Buffer = (*mut u8, usize)>,
    Fmt: Format,
{
    if f.is_nonfinite() {
        let s = f.format_nonfinite();
        let len = s.len();

        unsafe {
            let offset = pos - len;
            let dst = std::slice::from_raw_parts_mut(buf.offset(offset as isize), len);
            dst.copy_from_slice(s.as_bytes());
            str::from_utf8_unchecked(dst)
        }
    } else {
        f.format_finite(&mut (buf, pos), format)
    }
}
