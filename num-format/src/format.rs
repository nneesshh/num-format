use crate::strings::{DecimalStr, InfinityStr, MinusSignStr, NanStr, PlusSignStr, SeparatorStr};
use crate::Grouping;
use crate::DIGIT_TABLE;

use std::ptr;

pub struct Sep<'a> {
    pub ptr: *const u8,
    pub len: usize,
    pub pos: isize,
    pub step: isize,
    pub phantom: std::marker::PhantomData<&'a ()>,
}

#[inline(never)]
pub fn write_one_byte_with_sep(
    result: *mut u8,
    index: isize,
    sep: &mut Sep<'_>,
    table_index: isize,
) -> isize {
    let mut index = index - 1;
    if sep.pos == index {
        index -= sep.len as isize - 1;
        unsafe { ptr::copy_nonoverlapping(sep.ptr, result.offset(index), sep.len) }
        sep.pos -= sep.step + (sep.len as isize - 1);
        index -= 1;
    }
    unsafe {
        ptr::copy_nonoverlapping(
            DIGIT_TABLE.as_ptr().offset(table_index),
            result.offset(index),
            1,
        );
    }

    index
}

#[inline(never)]
pub fn write_two_bytes_with_sep(
    result: *mut u8,
    index: isize,
    sep: &mut Sep<'_>,
    table_index: isize,
) -> isize {
    let index1 = write_one_byte_with_sep(result, index, sep, table_index + 1);
    let index0 = write_one_byte_with_sep(result, index1, sep, table_index);
    index0
}

/// Trait that abstracts over [`CustomFormat`], [`Locale`], and `SystemLocale`.
///
/// [`CustomFormat`]: struct.CustomFormat.html
/// [`Locale`]: enum.Locale.html
pub trait Format {
    /// Returns the string representation of a decimal point.
    fn decimal(&self) -> DecimalStr<'_>;
    /// Returns the [`Grouping`] to use for separating digits. (see [`Grouping`])
    ///
    /// [`Grouping`]: enum.Grouping.html
    fn grouping(&self) -> Grouping;
    /// Returns the string representation of an infinity symbol.
    fn infinity(&self) -> InfinityStr<'_>;
    /// Returns the string representation of a minus sign.
    fn minus_sign(&self) -> MinusSignStr<'_>;
    /// Returns the string representation of NaN.
    fn nan(&self) -> NanStr<'_>;
    /// Returns the string representation of a plus sign.
    fn plus_sign(&self) -> PlusSignStr<'_>;
    /// Returns the string representation of a thousands separator.
    fn separator(&self) -> SeparatorStr<'_>;
}
