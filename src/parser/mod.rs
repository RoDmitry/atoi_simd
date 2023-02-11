#[cfg(all(
    target_feature = "sse2",
    target_feature = "sse3",
    target_feature = "sse4.1",
    target_feature = "ssse3",
    target_feature = "avx",
    target_feature = "avx2"
))]
mod simd_avx;
#[cfg(all(
    target_feature = "sse2",
    target_feature = "sse3",
    target_feature = "sse4.1",
    target_feature = "ssse3"
))]
mod simd_sse;

#[cfg(not(all(
    target_feature = "sse2",
    target_feature = "sse3",
    target_feature = "sse4.1",
    target_feature = "ssse3"
)))]
mod fb_32;
#[cfg(not(all(
    target_feature = "sse2",
    target_feature = "sse3",
    target_feature = "sse4.1",
    target_feature = "ssse3",
    target_feature = "avx",
    target_feature = "avx2"
)))]
mod fb_64;

use crate::AtoiSimdError;

pub trait ParserPos<T>: Sized {
    fn atoi_simd_parse_pos(s: &[u8]) -> Result<T, AtoiSimdError>;
    fn atoi_simd_parse_until_invalid_pos(s: &[u8]) -> Result<(T, usize), AtoiSimdError>;
}

pub trait ParserNeg<T>: Sized {
    fn atoi_simd_parse_neg(s: &[u8]) -> Result<T, AtoiSimdError>;
    fn atoi_simd_parse_until_invalid_neg(s: &[u8]) -> Result<(T, usize), AtoiSimdError>;
}

pub trait Parser<T: ParserPos<T>>: Sized {
    #[inline(always)]
    fn atoi_simd_parse(s: &[u8]) -> Result<T, AtoiSimdError> {
        T::atoi_simd_parse_pos(s)
    }

    #[inline(always)]
    fn atoi_simd_parse_until_invalid(s: &[u8]) -> Result<(T, usize), AtoiSimdError> {
        T::atoi_simd_parse_until_invalid_pos(s)
    }
}

#[inline(always)]
pub(crate) fn atoi_simd_parse_signed<T: ParserPos<T> + ParserNeg<T>>(
    s: &[u8],
) -> Result<T, AtoiSimdError> {
    if *s.first().ok_or(AtoiSimdError::Empty)? == b'-' {
        T::atoi_simd_parse_neg(&s[1..])
    } else {
        T::atoi_simd_parse_pos(s)
    }
}

#[inline(always)]
pub(crate) fn atoi_simd_parse_until_invalid_signed<T: ParserPos<T> + ParserNeg<T>>(
    s: &[u8],
) -> Result<(T, usize), AtoiSimdError> {
    if *s.first().ok_or(AtoiSimdError::Empty)? == b'-' {
        T::atoi_simd_parse_until_invalid_neg(&s[1..]).map(|(v, i)| (v, i + 1))
    } else {
        T::atoi_simd_parse_until_invalid_pos(s)
    }
}
