#![allow(trivial_numeric_casts)]

use crate::format::Format;

use crate::to_formatted_str::ToFormattedStr;

// helper functions

#[inline(never)]
fn run_core_algorithm_ryu<Fl, Fmt>(f: Fl, buf: &mut crate::Buffer, format: &Fmt) -> usize
where
    Fl: crate::ryu::Float<Buffer = (*mut u8, usize)>,
    Fmt: Format,
{
    let s = crate::ryu::float::format_float(f, buf.inner.as_mut_ptr(), buf.pos, format);
    let s_len = s.len();
    buf.pos -= s_len;
    s_len
}

// float 32bit

impl ToFormattedStr for f32 {
    #[doc(hidden)]
    #[inline(never)]
    fn read_to_buffer<'a, Fmt>(&self, buf: &'a mut crate::Buffer, format: &Fmt) -> usize
    where
        Fmt: Format,
    {
        run_core_algorithm_ryu(*self, buf, format)
    }
}

impl crate::private::Sealed for f32 {}

impl ToFormattedStr for f64 {
    #[doc(hidden)]
    #[inline(never)]
    fn read_to_buffer<'a, Fmt>(&self, buf: &'a mut crate::Buffer, format: &Fmt) -> usize
    where
        Fmt: Format,
    {
        run_core_algorithm_ryu(*self, buf, format)
    }
}

impl crate::private::Sealed for f64 {}
