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

    #[inline(always)]
    fn atoi_simd_parse_skipped(s: &[u8]) -> Result<T, AtoiSimdError> {
        let mut i = 0;
        if *s.first().ok_or(AtoiSimdError::Empty)? == b'+' {
            i = 1;
        }
        while s.len() > i {
            if *s.get_safe_unchecked(i) != b'0' {
                break;
            }
            i += 1;
        }

        T::atoi_simd_parse_pos(s.get_safe_unchecked(i..))
    }
}

#[inline(always)]
fn atoi_simd_parse_signed<T: ParserPos<T> + ParserNeg<T>>(s: &[u8]) -> Result<T, AtoiSimdError> {
    if *s.first().ok_or(AtoiSimdError::Empty)? == b'-' {
        T::atoi_simd_parse_neg(s.get_safe_unchecked(1..))
    } else {
        T::atoi_simd_parse_pos(s)
    }
}

#[inline(always)]
fn atoi_simd_parse_until_invalid_signed<T: ParserPos<T> + ParserNeg<T>>(
    s: &[u8],
) -> Result<(T, usize), AtoiSimdError> {
    if *s.first().ok_or(AtoiSimdError::Empty)? == b'-' {
        T::atoi_simd_parse_until_invalid_neg(s.get_safe_unchecked(1..)).map(|(v, i)| (v, i + 1))
    } else {
        T::atoi_simd_parse_until_invalid_pos(s)
    }
}

#[inline(always)]
fn atoi_simd_parse_skipped_signed<T: ParserPos<T> + ParserNeg<T>>(
    s: &[u8],
) -> Result<T, AtoiSimdError> {
    let mut i = 0;
    let mut neg = false;
    match *s.first().ok_or(AtoiSimdError::Empty)? {
        b'+' => {
            i = 1;
        }
        b'-' => {
            i = 1;
            neg = true;
        }
        _ => {}
    };
    while s.len() > i {
        if *s.get_safe_unchecked(i) != b'0' {
            break;
        }
        i += 1;
    }

    let input = s.get_safe_unchecked(i..);
    if neg {
        T::atoi_simd_parse_neg(input)
    } else {
        T::atoi_simd_parse_pos(input)
    }
}

impl Parser<u8> for u8 {}
impl Parser<u16> for u16 {}
impl Parser<u32> for u32 {}
impl Parser<usize> for usize {}
impl Parser<u64> for u64 {}
impl Parser<u128> for u128 {}

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

impl Parser<i8> for i8 {
    impl_signed!();
}
impl Parser<i16> for i16 {
    impl_signed!();
}
impl Parser<i32> for i32 {
    impl_signed!();
}
impl Parser<isize> for isize {
    impl_signed!();
}
impl Parser<i64> for i64 {
    impl_signed!();
}
impl Parser<i128> for i128 {
    impl_signed!();
}
