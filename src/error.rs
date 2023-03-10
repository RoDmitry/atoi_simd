use core::fmt;

#[derive(Debug, Clone, Copy)]
pub enum AtoiSimdError<'a> {
    Empty,
    Size(usize, &'a [u8]),
    Overflow(u128, &'a [u8]),
    Invalid64(u64, usize, &'a [u8]),
    Invalid128(u128, usize, &'a [u8]),
}

impl fmt::Display for AtoiSimdError<'_> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::Empty => write!(f, "atoi_simd string is empty"),
            Self::Size(len, input) => {
                write!(f, "atoi_simd wrong size: {} input: {:X?}", len, input)
            }
            Self::Overflow(max, input) => {
                write!(
                    f,
                    "atoi_simd overflow, max value: {} input: {:X?}",
                    max, input
                )
            }
            Self::Invalid64(res, index, input) => {
                write!(
                    f,
                    "atoi_simd invalid at index: {} it must contain only digits, starting with: {}  input: {:X?}",
                    index, res, input
                )
            }
            Self::Invalid128(res, index, input) => {
                write!(
                    f,
                    "atoi_simd invalid at index: {} it must contain only digits, starting with: {} input: {:X?}",
                    index, res, input
                )
            }
        }
    }
}

#[cfg(feature = "std")]
impl std::error::Error for AtoiSimdError<'_> {}
