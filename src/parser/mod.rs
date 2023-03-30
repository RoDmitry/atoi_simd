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

use crate::{safe_unchecked::SafeUnchecked, AtoiSimdError};

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
        T::atoi_simd_parse_neg(s.get_safe_unchecked(1..))
    } else {
        T::atoi_simd_parse_pos(s)
    }
}

#[inline(always)]
pub(crate) fn atoi_simd_parse_until_invalid_signed<T: ParserPos<T> + ParserNeg<T>>(
    s: &[u8],
) -> Result<(T, usize), AtoiSimdError> {
    if *s.first().ok_or(AtoiSimdError::Empty)? == b'-' {
        T::atoi_simd_parse_until_invalid_neg(s.get_safe_unchecked(1..)).map(|(v, i)| (v, i + 1))
    } else {
        T::atoi_simd_parse_until_invalid_pos(s)
    }
}

impl Parser<u8> for u8 {}
impl Parser<u16> for u16 {}
impl Parser<u32> for u32 {}
impl Parser<usize> for usize {}
impl Parser<u64> for u64 {}
impl Parser<u128> for u128 {}

impl Parser<i8> for i8 {
    #[inline(always)]
    fn atoi_simd_parse(s: &[u8]) -> Result<i8, AtoiSimdError> {
        atoi_simd_parse_signed(s)
    }

    #[inline(always)]
    fn atoi_simd_parse_until_invalid(s: &[u8]) -> Result<(i8, usize), AtoiSimdError> {
        atoi_simd_parse_until_invalid_signed(s)
    }
}

impl Parser<i16> for i16 {
    #[inline(always)]
    fn atoi_simd_parse(s: &[u8]) -> Result<i16, AtoiSimdError> {
        atoi_simd_parse_signed(s)
    }

    #[inline(always)]
    fn atoi_simd_parse_until_invalid(s: &[u8]) -> Result<(i16, usize), AtoiSimdError> {
        atoi_simd_parse_until_invalid_signed(s)
    }
}

impl Parser<i32> for i32 {
    #[inline(always)]
    fn atoi_simd_parse(s: &[u8]) -> Result<i32, AtoiSimdError> {
        atoi_simd_parse_signed(s)
    }

    #[inline(always)]
    fn atoi_simd_parse_until_invalid(s: &[u8]) -> Result<(i32, usize), AtoiSimdError> {
        atoi_simd_parse_until_invalid_signed(s)
    }
}

impl Parser<isize> for isize {
    #[inline(always)]
    fn atoi_simd_parse(s: &[u8]) -> Result<isize, AtoiSimdError> {
        atoi_simd_parse_signed(s)
    }

    #[inline(always)]
    fn atoi_simd_parse_until_invalid(s: &[u8]) -> Result<(isize, usize), AtoiSimdError> {
        atoi_simd_parse_until_invalid_signed(s)
    }
}

impl Parser<i64> for i64 {
    #[inline(always)]
    fn atoi_simd_parse(s: &[u8]) -> Result<i64, AtoiSimdError> {
        atoi_simd_parse_signed(s)
    }

    #[inline(always)]
    fn atoi_simd_parse_until_invalid(s: &[u8]) -> Result<(i64, usize), AtoiSimdError> {
        atoi_simd_parse_until_invalid_signed(s)
    }
}

impl Parser<i128> for i128 {
    #[inline(always)]
    fn atoi_simd_parse(s: &[u8]) -> Result<i128, AtoiSimdError> {
        atoi_simd_parse_signed(s)
    }

    #[inline(always)]
    fn atoi_simd_parse_until_invalid(s: &[u8]) -> Result<(i128, usize), AtoiSimdError> {
        atoi_simd_parse_until_invalid_signed(s)
    }
}
