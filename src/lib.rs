//! # Rust fast `&[u8]` to integer parser
//!
//! Faster on x86_64 (uses SIMD, SSE4.1, AVX2), but can be used even if you don't have x86_64 SIMD capable cpu.
//!
//! Supports negative values and validates the input.
//!
//! Supported output types: u8, i8, u16, i16, u32, i32, u64, i64, u128, i128, usize, isize.
//! The 128 bit max slice length is 32 numbers (33 with '-' sign), because it's limited by AVX2.
//!
//! Has good test coverage, and can be considered safe.
//!
//! To enable SIMD it needs the `target-feature` or `target-cpu` flags set, or it will fallback to non-SIMD functions.
//! To do it you can copy the `./.cargo/config.toml` in your project, or you can use one of the following environment variables:
//!
//! -   `RUSTFLAGS="-C target-feature=+sse2,+sse3,+sse4.1,+ssse3,+avx,+avx2"`
//!
//! -   `RUSTFLAGS="-C target-cpu=native"`
//!
//! If you have `&str` then use `.as_bytes()`
//!
//! # Examples
//!
//! ```
//! let val: u64 = atoi_simd::parse("1234".as_bytes()).unwrap();
//! assert_eq!(val, 1234_u64);
//!
//! assert_eq!(atoi_simd::parse::<i64>("-2345".as_bytes()).unwrap(), -2345_i64);
//! ```
#![allow(clippy::comparison_chain)]
#![allow(clippy::collapsible_else_if)]

/// Not for public use.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum ParseType {
    I64,
    I64Neg,
    I32,
    I32Neg,
    I16,
    I16Neg,
    I8,
    I8Neg,
    None,
}

mod error;
mod fallback;
mod simd;
#[cfg(test)]
mod test;

pub use crate::error::AtoiSimdError;
use crate::fallback::*;
use crate::simd::*;

#[inline]
fn parse_i8_neg(s: &[u8]) -> Result<i8, AtoiSimdError> {
    if cfg!(target_feature = "sse2")
        && cfg!(target_feature = "sse3")
        && cfg!(target_feature = "sse4.1")
        && cfg!(target_feature = "ssse3")
        && cfg!(target_feature = "avx")
        && cfg!(target_feature = "avx2")
    {
        parse_simd_i8_neg(s)
    } else {
        parse_fb_neg::<{ i8::MIN as i64 }>(s).map(|(v, _)| v as i8)
    }
}

#[inline]
fn parse_i8(s: &[u8]) -> Result<i8, AtoiSimdError> {
    if *s.first().ok_or(AtoiSimdError::Empty)? == b'-' {
        parse_i8_neg(&s[1..])
    } else {
        if cfg!(target_feature = "sse2")
            && cfg!(target_feature = "sse3")
            && cfg!(target_feature = "sse4.1")
            && cfg!(target_feature = "ssse3")
            && cfg!(target_feature = "avx")
            && cfg!(target_feature = "avx2")
        {
            parse_simd_u8(s, ParseType::I8).map(|v| v as i8)
        } else {
            parse_fb_pos::<{ i8::MAX as u64 }>(s).map(|(v, _)| v as i8)
        }
    }
}

#[inline]
fn parse_i16_neg(s: &[u8]) -> Result<i16, AtoiSimdError> {
    if cfg!(target_feature = "sse2")
        && cfg!(target_feature = "sse3")
        && cfg!(target_feature = "sse4.1")
        && cfg!(target_feature = "ssse3")
        && cfg!(target_feature = "avx")
        && cfg!(target_feature = "avx2")
    {
        parse_simd_i16_neg(s)
    } else {
        parse_fb_neg::<{ i16::MIN as i64 }>(s).map(|(v, _)| v as i16)
    }
}

#[inline]
fn parse_i16(s: &[u8]) -> Result<i16, AtoiSimdError> {
    if *s.first().ok_or(AtoiSimdError::Empty)? == b'-' {
        parse_i16_neg(&s[1..])
    } else {
        if cfg!(target_feature = "sse2")
            && cfg!(target_feature = "sse3")
            && cfg!(target_feature = "sse4.1")
            && cfg!(target_feature = "ssse3")
            && cfg!(target_feature = "avx")
            && cfg!(target_feature = "avx2")
        {
            parse_simd_u16(s, ParseType::I16).map(|v| v as i16)
        } else {
            parse_fb_pos::<{ i16::MAX as u64 }>(s).map(|(v, _)| v as i16)
        }
    }
}

#[inline]
fn parse_i32_neg(s: &[u8]) -> Result<i32, AtoiSimdError> {
    if cfg!(target_feature = "sse2")
        && cfg!(target_feature = "sse3")
        && cfg!(target_feature = "sse4.1")
        && cfg!(target_feature = "ssse3")
        && cfg!(target_feature = "avx")
        && cfg!(target_feature = "avx2")
    {
        parse_simd_i32_neg(s)
    } else {
        parse_fb_neg::<{ i32::MIN as i64 }>(s).map(|(v, _)| v as i32)
    }
}

#[inline]
fn parse_i32(s: &[u8]) -> Result<i32, AtoiSimdError> {
    if *s.first().ok_or(AtoiSimdError::Empty)? == b'-' {
        parse_i32_neg(&s[1..])
    } else {
        if cfg!(target_feature = "sse2")
            && cfg!(target_feature = "sse3")
            && cfg!(target_feature = "sse4.1")
            && cfg!(target_feature = "ssse3")
            && cfg!(target_feature = "avx")
            && cfg!(target_feature = "avx2")
        {
            parse_simd_u32(s, ParseType::I32).map(|v| v as i32)
        } else {
            parse_fb_pos::<{ i32::MAX as u64 }>(s).map(|(v, _)| v as i32)
        }
    }
}

#[inline]
fn parse_i64_neg(s: &[u8]) -> Result<i64, AtoiSimdError> {
    if cfg!(target_feature = "sse2")
        && cfg!(target_feature = "sse3")
        && cfg!(target_feature = "sse4.1")
        && cfg!(target_feature = "ssse3")
        && cfg!(target_feature = "avx")
        && cfg!(target_feature = "avx2")
    {
        parse_simd_i64_neg(s)
    } else {
        parse_fb_neg::<{ i64::MIN }>(s).map(|(v, _)| v)
    }
}

/// Parses slice of digits and first '-' char.
/// Slice length (except '-' char) must be 1..=20.
/// This function is slower than `parse_simd_u64()`, because it checks for '-' sign.
/// Uses SSE4.1 intrinsics
#[inline]
fn parse_i64(s: &[u8]) -> Result<i64, AtoiSimdError> {
    if *s.first().ok_or(AtoiSimdError::Empty)? == b'-' {
        parse_i64_neg(&s[1..])
    } else {
        if cfg!(target_feature = "sse2")
            && cfg!(target_feature = "sse3")
            && cfg!(target_feature = "sse4.1")
            && cfg!(target_feature = "ssse3")
            && cfg!(target_feature = "avx")
            && cfg!(target_feature = "avx2")
        {
            unsafe { parse_simd_u64(s, ParseType::I64).map(|v| v as i64) }
        } else {
            parse_fb_pos::<{ i64::MAX as u64 }>(s).map(|(v, _)| v as i64)
        }
    }
}

#[inline]
fn parse_i128_neg(s: &[u8]) -> Result<i128, AtoiSimdError> {
    if cfg!(target_feature = "sse2")
        && cfg!(target_feature = "sse3")
        && cfg!(target_feature = "sse4.1")
        && cfg!(target_feature = "ssse3")
        && cfg!(target_feature = "avx")
        && cfg!(target_feature = "avx2")
    {
        unsafe { parse_simd_u128(s).map(|v| -(v as i128)) }
    } else {
        parse_fb_128_neg(s).map(|(v, _)| v)
    }
}

/// Parses slice of digits and first '-' char.
/// Slice length (except '-' char) must be 1..=32.
/// This function is slower than `parse_simd_u128()`, because it checks for '-' sign.
/// Uses AVX/AVX2 intrinsics
#[inline]
fn parse_i128(s: &[u8]) -> Result<i128, AtoiSimdError> {
    if *s.first().ok_or(AtoiSimdError::Empty)? == b'-' {
        parse_i128_neg(&s[1..])
    } else {
        if cfg!(target_feature = "sse2")
            && cfg!(target_feature = "sse3")
            && cfg!(target_feature = "sse4.1")
            && cfg!(target_feature = "ssse3")
            && cfg!(target_feature = "avx")
            && cfg!(target_feature = "avx2")
        {
            unsafe { parse_simd_u128(s).map(|v| v as i128) }
        } else {
            parse_fb_128_pos(s).map(|(v, _)| v as i128)
        }
    }
}

pub trait Parser<T> {
    fn atoi_simd_parse(s: &[u8]) -> Result<T, AtoiSimdError>;
    fn atoi_simd_parse_until_invalid(s: &[u8]) -> Result<(T, usize), AtoiSimdError>;
}

pub trait ParserNeg<T> {
    fn atoi_simd_parse_neg(s: &[u8]) -> Result<T, AtoiSimdError>;
}

#[cfg(all(
    target_feature = "sse2",
    target_feature = "sse3",
    target_feature = "sse4.1",
    target_feature = "ssse3",
    target_feature = "avx",
    target_feature = "avx2"
))]
impl Parser<u8> for u8 {
    #[inline]
    fn atoi_simd_parse(s: &[u8]) -> Result<u8, AtoiSimdError> {
        parse_simd_u8(s, ParseType::None)
    }

    #[inline]
    fn atoi_simd_parse_until_invalid(s: &[u8]) -> Result<(u8, usize), AtoiSimdError> {
        parse_fb_until_invalid_pos::<{ u8::MAX as u64 }>(s).map(|(v, i)| (v as u8, i))
    }
}

#[cfg(not(all(
    target_feature = "sse2",
    target_feature = "sse3",
    target_feature = "sse4.1",
    target_feature = "ssse3",
    target_feature = "avx",
    target_feature = "avx2"
)))]
impl Parser<u8> for u8 {
    #[inline]
    fn atoi_simd_parse(s: &[u8]) -> Result<u8, AtoiSimdError> {
        parse_fb_pos::<{ u8::MAX as u64 }>(s).map(|(v, _)| v as u8)
    }

    #[inline]
    fn atoi_simd_parse_until_invalid(s: &[u8]) -> Result<(u8, usize), AtoiSimdError> {
        parse_fb_until_invalid_pos::<{ u8::MAX as u64 }>(s).map(|(v, i)| (v as u8, i))
    }
}

impl Parser<i8> for i8 {
    #[inline]
    fn atoi_simd_parse(s: &[u8]) -> Result<i8, AtoiSimdError> {
        parse_i8(s)
    }

    #[inline]
    fn atoi_simd_parse_until_invalid(s: &[u8]) -> Result<(i8, usize), AtoiSimdError> {
        if *s.first().ok_or(AtoiSimdError::Empty)? == b'-' {
            parse_fb_until_invalid_neg::<{ i8::MIN as i64 }>(s).map(|(v, i)| (v as i8, i))
        } else {
            parse_fb_until_invalid_pos::<{ i8::MAX as u64 }>(s).map(|(v, i)| (v as i8, i))
        }
    }
}

impl ParserNeg<i8> for i8 {
    #[inline]
    fn atoi_simd_parse_neg(s: &[u8]) -> Result<i8, AtoiSimdError> {
        parse_i8_neg(s)
    }
}

#[cfg(all(
    target_feature = "sse2",
    target_feature = "sse3",
    target_feature = "sse4.1",
    target_feature = "ssse3",
    target_feature = "avx",
    target_feature = "avx2"
))]
impl Parser<u16> for u16 {
    #[inline]
    fn atoi_simd_parse(s: &[u8]) -> Result<u16, AtoiSimdError> {
        parse_simd_u16(s, ParseType::None)
    }

    #[inline]
    fn atoi_simd_parse_until_invalid(s: &[u8]) -> Result<(u16, usize), AtoiSimdError> {
        parse_fb_until_invalid_pos::<{ u16::MAX as u64 }>(s).map(|(v, i)| (v as u16, i))
    }
}

#[cfg(not(all(
    target_feature = "sse2",
    target_feature = "sse3",
    target_feature = "sse4.1",
    target_feature = "ssse3",
    target_feature = "avx",
    target_feature = "avx2"
)))]
impl Parser<u16> for u16 {
    #[inline]
    fn atoi_simd_parse(s: &[u8]) -> Result<u16, AtoiSimdError> {
        parse_fb_pos::<{ u16::MAX as u64 }>(s).map(|(v, _)| v as u16)
    }

    #[inline]
    fn atoi_simd_parse_until_invalid(s: &[u8]) -> Result<(u16, usize), AtoiSimdError> {
        parse_fb_until_invalid_pos::<{ u16::MAX as u64 }>(s).map(|(v, i)| (v as u16, i))
    }
}

impl Parser<i16> for i16 {
    #[inline]
    fn atoi_simd_parse(s: &[u8]) -> Result<i16, AtoiSimdError> {
        parse_i16(s)
    }

    #[inline]
    fn atoi_simd_parse_until_invalid(s: &[u8]) -> Result<(i16, usize), AtoiSimdError> {
        if *s.first().ok_or(AtoiSimdError::Empty)? == b'-' {
            parse_fb_until_invalid_neg::<{ i16::MIN as i64 }>(s).map(|(v, i)| (v as i16, i))
        } else {
            parse_fb_until_invalid_pos::<{ i16::MAX as u64 }>(s).map(|(v, i)| (v as i16, i))
        }
    }
}

impl ParserNeg<i16> for i16 {
    #[inline]
    fn atoi_simd_parse_neg(s: &[u8]) -> Result<i16, AtoiSimdError> {
        parse_i16_neg(s)
    }
}

#[cfg(all(
    target_feature = "sse2",
    target_feature = "sse3",
    target_feature = "sse4.1",
    target_feature = "ssse3",
    target_feature = "avx",
    target_feature = "avx2"
))]
impl Parser<u32> for u32 {
    #[inline]
    fn atoi_simd_parse(s: &[u8]) -> Result<u32, AtoiSimdError> {
        parse_simd_u32(s, ParseType::None)
    }

    #[inline]
    fn atoi_simd_parse_until_invalid(s: &[u8]) -> Result<(u32, usize), AtoiSimdError> {
        parse_fb_until_invalid_pos::<{ u32::MAX as u64 }>(s).map(|(v, i)| (v as u32, i))
    }
}

#[cfg(not(all(
    target_feature = "sse2",
    target_feature = "sse3",
    target_feature = "sse4.1",
    target_feature = "ssse3",
    target_feature = "avx",
    target_feature = "avx2"
)))]
impl Parser<u32> for u32 {
    #[inline]
    fn atoi_simd_parse(s: &[u8]) -> Result<u32, AtoiSimdError> {
        parse_fb_pos::<{ u32::MAX as u64 }>(s).map(|(v, _)| v as u32)
    }

    #[inline]
    fn atoi_simd_parse_until_invalid(s: &[u8]) -> Result<(u32, usize), AtoiSimdError> {
        parse_fb_until_invalid_pos::<{ u32::MAX as u64 }>(s).map(|(v, i)| (v as u32, i))
    }
}

impl Parser<i32> for i32 {
    #[inline]
    fn atoi_simd_parse(s: &[u8]) -> Result<i32, AtoiSimdError> {
        parse_i32(s)
    }

    #[inline]
    fn atoi_simd_parse_until_invalid(s: &[u8]) -> Result<(i32, usize), AtoiSimdError> {
        if *s.first().ok_or(AtoiSimdError::Empty)? == b'-' {
            parse_fb_until_invalid_neg::<{ i32::MIN as i64 }>(s).map(|(v, i)| (v as i32, i))
        } else {
            parse_fb_until_invalid_pos::<{ i32::MAX as u64 }>(s).map(|(v, i)| (v as i32, i))
        }
    }
}

impl ParserNeg<i32> for i32 {
    #[inline]
    fn atoi_simd_parse_neg(s: &[u8]) -> Result<i32, AtoiSimdError> {
        parse_i32_neg(s)
    }
}

#[cfg(all(
    target_feature = "sse2",
    target_feature = "sse3",
    target_feature = "sse4.1",
    target_feature = "ssse3",
    target_feature = "avx",
    target_feature = "avx2"
))]
impl Parser<usize> for usize {
    #[cfg(target_pointer_width = "32")]
    #[inline]
    fn atoi_simd_parse(s: &[u8]) -> Result<usize, AtoiSimdError> {
        parse_simd_u32(s, ParseType::None).map(|v| v as usize)
    }

    #[cfg(target_pointer_width = "64")]
    #[inline]
    fn atoi_simd_parse(s: &[u8]) -> Result<usize, AtoiSimdError> {
        unsafe { parse_simd_u64(s, ParseType::None).map(|v| v as usize) }
    }

    #[inline]
    fn atoi_simd_parse_until_invalid(s: &[u8]) -> Result<(usize, usize), AtoiSimdError> {
        parse_fb_until_invalid_pos::<{ usize::MAX as u64 }>(s).map(|(v, i)| (v as usize, i))
    }
}

#[cfg(not(all(
    target_feature = "sse2",
    target_feature = "sse3",
    target_feature = "sse4.1",
    target_feature = "ssse3",
    target_feature = "avx",
    target_feature = "avx2"
)))]
impl Parser<usize> for usize {
    #[cfg(target_pointer_width = "32")]
    #[inline]
    fn atoi_simd_parse(s: &[u8]) -> Result<usize, AtoiSimdError> {
        parse_fb_pos::<{ u32::MAX as u64 }>(s).map(|(v, _)| v as usize)
    }

    #[cfg(target_pointer_width = "64")]
    #[inline]
    fn atoi_simd_parse(s: &[u8]) -> Result<usize, AtoiSimdError> {
        parse_fb_pos::<{ u64::MAX }>(s).map(|(v, _)| v as usize)
    }

    #[inline]
    fn atoi_simd_parse_until_invalid(s: &[u8]) -> Result<(usize, usize), AtoiSimdError> {
        parse_fb_until_invalid_pos::<{ usize::MAX as u64 }>(s).map(|(v, i)| (v as usize, i))
    }
}

impl Parser<isize> for isize {
    #[cfg(target_pointer_width = "32")]
    #[inline]
    fn atoi_simd_parse(s: &[u8]) -> Result<isize, AtoiSimdError> {
        parse_i32(s).map(|v| v as isize)
    }

    #[cfg(target_pointer_width = "64")]
    #[inline]
    fn atoi_simd_parse(s: &[u8]) -> Result<isize, AtoiSimdError> {
        parse_i64(s).map(|v| v as isize)
    }

    #[inline]
    fn atoi_simd_parse_until_invalid(s: &[u8]) -> Result<(isize, usize), AtoiSimdError> {
        if *s.first().ok_or(AtoiSimdError::Empty)? == b'-' {
            parse_fb_until_invalid_neg::<{ isize::MIN as i64 }>(s).map(|(v, i)| (v as isize, i))
        } else {
            parse_fb_until_invalid_pos::<{ isize::MAX as u64 }>(s).map(|(v, i)| (v as isize, i))
        }
    }
}

impl ParserNeg<isize> for isize {
    #[cfg(target_pointer_width = "32")]
    #[inline]
    fn atoi_simd_parse_neg(s: &[u8]) -> Result<isize, AtoiSimdError> {
        parse_i32_neg(s).map(|v| v as isize)
    }

    #[cfg(target_pointer_width = "64")]
    #[inline]
    fn atoi_simd_parse_neg(s: &[u8]) -> Result<isize, AtoiSimdError> {
        parse_i64_neg(s).map(|v| v as isize)
    }
}

#[cfg(all(
    target_feature = "sse2",
    target_feature = "sse3",
    target_feature = "sse4.1",
    target_feature = "ssse3",
    target_feature = "avx",
    target_feature = "avx2"
))]
impl Parser<u64> for u64 {
    #[inline]
    fn atoi_simd_parse(s: &[u8]) -> Result<u64, AtoiSimdError> {
        unsafe { parse_simd_u64(s, ParseType::None) }
    }

    #[inline]
    fn atoi_simd_parse_until_invalid(s: &[u8]) -> Result<(u64, usize), AtoiSimdError> {
        parse_fb_until_invalid_pos::<{ u64::MAX }>(s)
    }
}

#[cfg(not(all(
    target_feature = "sse2",
    target_feature = "sse3",
    target_feature = "sse4.1",
    target_feature = "ssse3",
    target_feature = "avx",
    target_feature = "avx2"
)))]
impl Parser<u64> for u64 {
    #[inline]
    fn atoi_simd_parse(s: &[u8]) -> Result<u64, AtoiSimdError> {
        parse_fb_pos::<{ u64::MAX }>(s).map(|(v, _)| v)
    }

    #[inline]
    fn atoi_simd_parse_until_invalid(s: &[u8]) -> Result<(u64, usize), AtoiSimdError> {
        parse_fb_until_invalid_pos::<{ u64::MAX }>(s)
    }
}

impl Parser<i64> for i64 {
    #[inline]
    fn atoi_simd_parse(s: &[u8]) -> Result<i64, AtoiSimdError> {
        parse_i64(s)
    }

    #[inline]
    fn atoi_simd_parse_until_invalid(s: &[u8]) -> Result<(i64, usize), AtoiSimdError> {
        if *s.first().ok_or(AtoiSimdError::Empty)? == b'-' {
            parse_fb_until_invalid_neg::<{ i64::MIN }>(s)
        } else {
            parse_fb_until_invalid_pos::<{ i64::MAX as u64 }>(s).map(|(v, i)| (v as i64, i))
        }
    }
}

impl ParserNeg<i64> for i64 {
    #[inline]
    fn atoi_simd_parse_neg(s: &[u8]) -> Result<i64, AtoiSimdError> {
        parse_i64_neg(s)
    }
}

#[cfg(all(
    target_feature = "sse2",
    target_feature = "sse3",
    target_feature = "sse4.1",
    target_feature = "ssse3",
    target_feature = "avx",
    target_feature = "avx2"
))]
impl Parser<u128> for u128 {
    #[inline]
    fn atoi_simd_parse(s: &[u8]) -> Result<u128, AtoiSimdError> {
        unsafe { parse_simd_u128(s) }
    }

    #[inline]
    fn atoi_simd_parse_until_invalid(s: &[u8]) -> Result<(u128, usize), AtoiSimdError> {
        parse_fb_until_invalid_128_pos(s)
    }
}

#[cfg(not(all(
    target_feature = "sse2",
    target_feature = "sse3",
    target_feature = "sse4.1",
    target_feature = "ssse3",
    target_feature = "avx",
    target_feature = "avx2"
)))]
impl Parser<u128> for u128 {
    #[inline]
    fn atoi_simd_parse(s: &[u8]) -> Result<u128, AtoiSimdError> {
        parse_fb_128_pos(s).map(|(v, _)| v)
    }

    #[inline]
    fn atoi_simd_parse_until_invalid(s: &[u8]) -> Result<(u128, usize), AtoiSimdError> {
        parse_fb_until_invalid_128_pos(s)
    }
}

impl Parser<i128> for i128 {
    #[inline]
    fn atoi_simd_parse(s: &[u8]) -> Result<i128, AtoiSimdError> {
        parse_i128(s)
    }

    #[inline]
    fn atoi_simd_parse_until_invalid(s: &[u8]) -> Result<(i128, usize), AtoiSimdError> {
        if *s.first().ok_or(AtoiSimdError::Empty)? == b'-' {
            parse_fb_until_invalid_128_neg(s)
        } else {
            parse_fb_until_invalid_128_pos(s).map(|(v, i)| (v as i128, i))
        }
    }
}

impl ParserNeg<i128> for i128 {
    #[inline]
    fn atoi_simd_parse_neg(s: &[u8]) -> Result<i128, AtoiSimdError> {
        parse_i128_neg(s)
    }
}

/// Parses slice of digits, and checks first '-' char for signed integers.
#[inline]
pub fn parse<T: Parser<T>>(s: &[u8]) -> Result<T, AtoiSimdError> {
    T::atoi_simd_parse(s)
}

/// Parses negative integer. Slice must not contain '-' sign.
#[inline]
pub fn parse_neg<T: ParserNeg<T>>(s: &[u8]) -> Result<T, AtoiSimdError> {
    T::atoi_simd_parse_neg(s)
}

/// Parses integer until it reaches invalid character.
/// Returns parsed value and parsed size of the slice.
/// It does not use SIMD.
#[inline]
pub fn parse_until_invalid<T: Parser<T>>(s: &[u8]) -> Result<(T, usize), AtoiSimdError> {
    T::atoi_simd_parse_until_invalid(s)
}
