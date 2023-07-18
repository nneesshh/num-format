mod udiv128;

use core::str;

#[cfg(feature = "no-panic")]
use no_panic::no_panic;

/// An integer that can be written into an ['Buffer'].
///
/// This trait is sealed and cannot be implemented for types outside of itoa.
pub trait Integer: crate::private::Sealed {
    /// *mut u8 bytes with pos
    type Buffer: 'static;

    /// Write integer to buffer
    fn write<'a>(self, buf: &mut Self::Buffer) -> &'a str;
}

/// Format an integer to buffer
pub fn format<'a, N>(n: N, buf: *mut u8, pos: usize) -> &'a str
where
    N: Integer<Buffer = (*mut u8, usize)>,
{
    n.write(&mut (buf, pos))
}

mod integer128;
mod integer32;
mod integer64;
mod integers;
