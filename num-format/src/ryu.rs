mod common;
mod d2s;
#[cfg(not(feature = "small"))]
mod d2s_full_table;
mod d2s_intrinsics;
#[cfg(feature = "small")]
mod d2s_small_table;
mod f2s;
mod f2s_intrinsics;
pub(crate) mod float;
mod pretty;

pub use crate::ryu::float::Float;

/// Unsafe functions that mirror the API of the C implementation of Ryū.
pub mod raw {
    pub use crate::ryu::pretty::{format32, format64};
}

/// Format a float to buffer
pub fn format<'a, Fl>(f: Fl, buf: *mut u8, pos: usize) -> &'a str
where
    Fl: Float<Buffer = (*mut u8, usize)>,
{
    f.write(&mut (buf, pos))
}
