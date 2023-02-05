use crate::ParseType;
use std::{fmt, str::from_utf8};

#[derive(Debug, Clone, Copy)]
pub enum AtoiSimdError<'a> {
    Empty,
    Size(usize, &'a [u8]),
    Overflow(ParseType, &'a [u8]),
    Invalid(&'a [u8]),
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
            Self::Overflow(t, val) => {
                write!(
                    f,
                    "atoi_simd {:?} overflow: {}",
                    t,
                    from_utf8(val).unwrap_or("not string")
                )
            }
            Self::Invalid(val) => {
                write!(
                    f,
                    "atoi_simd invalid, it must contain only digits: {}",
                    from_utf8(val).unwrap_or("not string")
                )
            }
            Self::I64Min => write!(f, "atoi_simd i64::min"), // internal error
        }
    }
}

impl std::error::Error for AtoiSimdError<'_> {}
