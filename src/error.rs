use std::{fmt, str::from_utf8};

#[derive(Debug, Clone, Copy)]
pub enum AtoiSimdError<'a> {
    Empty,
    Size(usize, &'a [u8]),
    Overflow64(u64, &'a [u8]),
    Overflow64Neg(i64, &'a [u8]),
    Overflow128(u128, &'a [u8]),
    Overflow128Neg(i128, &'a [u8]),
    Invalid64(u64, usize),
    Invalid128(u128, usize),
}

impl fmt::Display for AtoiSimdError<'_> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::Empty => write!(f, "atoi_simd string is empty"),
            Self::Size(len, input) => write!(
                f,
                "atoi_simd wrong size: {} input: {}",
                len,
                from_utf8(input).unwrap_or("not string")
            ),
            Self::Overflow64(max, input) => {
                write!(
                    f,
                    "atoi_simd overflow 64, max value: {} input: {}",
                    max,
                    from_utf8(input).unwrap_or("not string")
                )
            }
            Self::Overflow64Neg(min, input) => {
                write!(
                    f,
                    "atoi_simd overflow 64, min value: {} input: {}",
                    min,
                    from_utf8(input).unwrap_or("not string")
                )
            }
            Self::Overflow128(max, input) => {
                write!(
                    f,
                    "atoi_simd overflow 128, max value: {} input: {}",
                    max,
                    from_utf8(input).unwrap_or("not string")
                )
            }
            Self::Overflow128Neg(min, input) => {
                write!(
                    f,
                    "atoi_simd overflow 128, min value: {} input: {}",
                    min,
                    from_utf8(input).unwrap_or("not string")
                )
            }
            Self::Invalid64(res, index) => {
                write!(
                    f,
                    "atoi_simd invalid at index: {} it must contain only digits, starting with: {}",
                    index, res,
                )
            }
            Self::Invalid128(res, index) => {
                write!(
                    f,
                    "atoi_simd invalid at index: {} it must contain only digits, starting with: {}",
                    index, res,
                )
            }
        }
    }
}

impl std::error::Error for AtoiSimdError<'_> {}
