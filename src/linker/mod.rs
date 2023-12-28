#[cfg(any(
    all(target_arch = "aarch64", target_feature = "neon"),
    all(
        target_feature = "sse2",
        target_feature = "sse3",
        target_feature = "sse4.1",
        target_feature = "ssse3"
    ),
))]
mod simd_32;
#[cfg(any(
    all(target_arch = "aarch64", target_feature = "neon"),
    all(
        target_feature = "sse2",
        target_feature = "sse3",
        target_feature = "sse4.1",
        target_feature = "ssse3",
        target_feature = "avx",
        target_feature = "avx2"
    ),
))]
mod simd_64;

#[cfg(not(any(
    all(target_arch = "aarch64", target_feature = "neon"),
    all(
        target_feature = "sse2",
        target_feature = "sse3",
        target_feature = "sse4.1",
        target_feature = "ssse3"
    ),
)))]
mod fb_32;
#[cfg(not(any(
    all(target_arch = "aarch64", target_feature = "neon"),
    all(
        target_feature = "sse2",
        target_feature = "sse3",
        target_feature = "sse4.1",
        target_feature = "ssse3",
        target_feature = "avx",
        target_feature = "avx2"
    ),
)))]
mod fb_64;

use crate::{safe_unchecked::SliceGetter, AtoiSimdError};

pub trait ParserPos: Sized {
    fn atoi_simd_parse_pos(s: &[u8]) -> Result<Self, AtoiSimdError>;
    fn atoi_simd_parse_until_invalid_pos(s: &[u8]) -> Result<(Self, usize), AtoiSimdError>;
}

pub trait ParserNeg: Sized {
    fn atoi_simd_parse_neg(s: &[u8]) -> Result<Self, AtoiSimdError>;
    fn atoi_simd_parse_until_invalid_neg(s: &[u8]) -> Result<(Self, usize), AtoiSimdError>;
}

pub trait Parser: ParserPos {
    #[inline(always)]
    fn atoi_simd_parse(s: &[u8]) -> Result<Self, AtoiSimdError> {
        Self::atoi_simd_parse_pos(s)
    }

    #[inline(always)]
    fn atoi_simd_parse_until_invalid(s: &[u8]) -> Result<(Self, usize), AtoiSimdError> {
        Self::atoi_simd_parse_until_invalid_pos(s)
    }

    #[inline(always)]
    fn atoi_simd_parse_skipped(s: &[u8]) -> Result<Self, AtoiSimdError> {
        let mut i = 0;
        if *s.first().ok_or(AtoiSimdError::Empty)? == b'+' {
            i = 1;
        }
        let extra_len = s.len().saturating_sub(16);
        while i < extra_len && *s.get_safe_unchecked(i) == b'0' {
            i += 1;
        }

        Self::atoi_simd_parse_pos(s.get_safe_unchecked(i..))
    }
}

#[inline(always)]
fn atoi_simd_parse_signed<T: ParserPos + ParserNeg>(s: &[u8]) -> Result<T, AtoiSimdError> {
    if *s.first().ok_or(AtoiSimdError::Empty)? == b'-' {
        T::atoi_simd_parse_neg(s.get_safe_unchecked(1..))
    } else {
        T::atoi_simd_parse_pos(s)
    }
}

#[inline(always)]
fn atoi_simd_parse_until_invalid_signed<T: ParserPos + ParserNeg>(
    s: &[u8],
) -> Result<(T, usize), AtoiSimdError> {
    if *s.first().ok_or(AtoiSimdError::Empty)? == b'-' {
        T::atoi_simd_parse_until_invalid_neg(s.get_safe_unchecked(1..)).map(|(v, i)| (v, i + 1))
    } else {
        T::atoi_simd_parse_until_invalid_pos(s)
    }
}

#[inline(always)]
fn atoi_simd_parse_skipped_signed<T: ParserPos + ParserNeg>(
    s: &[u8],
) -> Result<T, AtoiSimdError> {
    let mut neg = false;
    let mut i = match *s.first().ok_or(AtoiSimdError::Empty)? {
        b'+' => 1,
        b'-' => {
            neg = true;
            1
        }
        _ => 0,
    };
    let extra_len = s.len().saturating_sub(16);
    while i < extra_len && *s.get_safe_unchecked(i) == b'0' {
        i += 1;
    }

    let input = s.get_safe_unchecked(i..);
    if neg {
        T::atoi_simd_parse_neg(input)
    } else {
        T::atoi_simd_parse_pos(input)
    }
}

impl Parser for u8 {}
impl Parser for u16 {}
impl Parser for u32 {}
impl Parser for usize {}
impl Parser for u64 {}
impl Parser for u128 {}

macro_rules! impl_signed {
    () => {
        #[inline(always)]
        fn atoi_simd_parse(s: &[u8]) -> Result<Self, AtoiSimdError> {
            atoi_simd_parse_signed(s)
        }

        #[inline(always)]
        fn atoi_simd_parse_until_invalid(s: &[u8]) -> Result<(Self, usize), AtoiSimdError> {
            atoi_simd_parse_until_invalid_signed(s)
        }

        #[inline(always)]
        fn atoi_simd_parse_skipped(s: &[u8]) -> Result<Self, AtoiSimdError> {
            atoi_simd_parse_skipped_signed(s)
        }
    };
}

impl Parser for i8 {
    impl_signed!();
}
impl Parser for i16 {
    impl_signed!();
}
impl Parser for i32 {
    impl_signed!();
}
impl Parser for isize {
    impl_signed!();
}
impl Parser for i64 {
    impl_signed!();
}
impl Parser for i128 {
    impl_signed!();
}
