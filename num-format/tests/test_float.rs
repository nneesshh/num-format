mod common;

use num_format::{Buffer, CustomFormat};
#[cfg(feature = "std")]
use num_format::{ToFormattedString, WriteFormatted};

use crate::common::POLICIES;

#[test]
fn test_f64() {
    use std::{mem::MaybeUninit, slice, str};

    let f = 1.234_f64;

    let mut buffer = [MaybeUninit::<u8>::uninit(); 24];
    let pos = num_format::ryu::raw::format64(
        f,
        buffer.as_mut_ptr() as *mut u8,
        24,
        &num_format::Locale::en,
    );

    unsafe {
        let dst = slice::from_raw_parts(
            (buffer.as_ptr() as *const u8).offset(pos as isize),
            24 - pos,
        );
        let printed = str::from_utf8_unchecked(dst);
        assert_eq!(printed, "1.234");
    }
    println!("aaa");

    let test_cases: &[(&str, f64, &CustomFormat)] = &[
        ("0.0", 0f64, &POLICIES[0]),
        ("0.0", 0f64, &POLICIES[1]),
        ("0.0", 0f64, &POLICIES[2]),
        ("0.0", 0f64, &POLICIES[3]),
        ("0.0", 0f64, &POLICIES[4]),
        ("18,446,744,073.709", 18446744073.709_f64, &POLICIES[0]),
        ("18𠜱446𠜱744𠜱073.709", 18446744073.709_f64, &POLICIES[1]),
        ("18𠜱44𠜱67𠜱44𠜱073.709", 18446744073.709_f64, &POLICIES[2]),
        ("18446744073.709", 18446744073.709_f64, &POLICIES[3]),
        ("18446744073.709", 18446744073.709_f64, &POLICIES[4]),
    ];

    for (expected, input, format) in test_cases {
        // Buffer
        let mut buf = Buffer::default();
        buf.write_formatted(input, *format);
        assert_eq!(*expected, buf.as_str());

        #[cfg(feature = "std")]
        {
            // ToFormattedString
            assert_eq!(expected.to_string(), input.to_formatted_string(*format));

            // WriteFormatted
            let mut s = String::new();
            s.write_formatted(input, *format).unwrap();
            assert_eq!(expected.to_string(), s);
        }
    }
}
