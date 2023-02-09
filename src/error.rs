use std::{fmt, str::from_utf8};

#[derive(Debug, Clone, Copy)]
pub enum AtoiSimdError<'a> {
    Empty,
    Size(usize, &'a [u8]),
    Overflow64(u64, &'a [u8]),
    Overflow128(u128, &'a [u8]),
    Invalid64(u64, usize),
    Invalid128(u128, usize),
    I64Min,
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
                    "atoi_simd max: {:?} overflow: {}",
                    max,
                    from_utf8(val).unwrap_or("not string")
                )
            }
            Self::Overflow128(max, val) => {
                write!(
                    f,
                    "atoi_simd max: {:?} overflow: {}",
                    max,
                    from_utf8(val).unwrap_or("not string")
                )
            }
            Self::Invalid64(_, index) | Self::Invalid128(_, index) => {
                write!(
                    f,
                    "atoi_simd invalid at index: {} it must contain only digits",
                    index,
                )
            }
            Self::I64Min => write!(f, "atoi_simd i64::min"), // internal error
        }
    }
}

impl std::error::Error for AtoiSimdError<'_> {}
