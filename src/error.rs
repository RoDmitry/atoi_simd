use std::{fmt, str::from_utf8};

#[derive(Debug, Clone, Copy)]
pub enum AtoiSimdError<'a> {
    Empty,
    Size(usize, &'a [u8]),
    Overflow64(u64, &'a [u8]),
    Overflow128(u128, &'a [u8]),
    Invalid64(u64, usize),
    Invalid128(u128, usize),
}

impl fmt::Display for AtoiSimdError<'_> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::Empty => write!(f, "atoi_simd string is empty"),
            Self::Size(len, val) => write!(
                f,
                "atoi_simd wrong size: {} input: {}",
                len,
                from_utf8(val).unwrap_or("not string")
            ),
            Self::Overflow64(max, val) => {
                write!(
                    f,
                    "atoi_simd overflow, max value: {} input: {}",
                    max,
                    from_utf8(val).unwrap_or("not string")
                )
            }
            Self::Overflow128(max, val) => {
                write!(
                    f,
                    "atoi_simd overflow, max value: {} input: {}",
                    max,
                    from_utf8(val).unwrap_or("not string")
                )
            }
            Self::Invalid64(val, index) => {
                write!(
                    f,
                    "atoi_simd invalid at index: {} it must contain only digits, starting with: {}",
                    index, val,
                )
            }
            Self::Invalid128(val, index) => {
                write!(
                    f,
                    "atoi_simd invalid at index: {} it must contain only digits, starting with: {}",
                    index, val,
                )
            }
        }
    }
}

impl std::error::Error for AtoiSimdError<'_> {}
