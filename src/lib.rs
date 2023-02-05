//! # Rust fast `&[u8]` to integer parser
//!
//! Faster on x86_64 (uses SIMD, SSE4.1, AVX2), but can be used even if you don't have x86_64 SIMD capable cpu.
//!
//! To enable SIMD it needs the `target-feature` or `target-cpu` flags set, or it will fallback to non-SIMD functions.
//!
//! If you have `&str` then use `.as_bytes()`
//!
//! Supported output types: u8, i8, u16, i16, u32, i32, u64, i64, u128, i128, usize, isize.
//! The 128 bit max slice length is 32 numbers (33 with '-' sign), because it's limited by AVX2.
//!
//! Has good test coverage, and can be considered safe.
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
use crate::fallback::{parse_fb_128_neg, parse_fb_128_pos, parse_fb_neg, parse_fb_pos};
use crate::simd::{parse_simd_u128, parse_simd_u64};

#[inline]
fn parse_simd_u8(s: &[u8], parse_type: ParseType) -> Result<u8, AtoiSimdError> {
    let val = unsafe { parse_simd_u64(s, parse_type)? };
    match parse_type {
        ParseType::I8 => {
            if val > i8::MAX as u64 {
                Err(AtoiSimdError::Overflow(parse_type, s))
            } else {
                Ok(val as u8)
            }
        }
        _ => {
            if val > u8::MAX as u64 {
                Err(AtoiSimdError::Overflow(parse_type, s))
            } else {
                Ok(val as u8)
            }
        }
    }
}

#[inline]
fn parse_simd_i8_neg(s: &[u8]) -> Result<i8, AtoiSimdError> {
    let val = unsafe { parse_simd_u64(s, ParseType::None)? };
    if val > i8::MAX as u64 + 1 {
        Err(AtoiSimdError::Overflow(ParseType::I8Neg, s))
    } else if val == i8::MAX as u64 + 1 {
        Ok(i8::MIN)
    } else {
        Ok(-(val as i8))
    }
}

#[inline]
fn parse_i8(s: &[u8]) -> Result<i8, AtoiSimdError> {
    if *s.first().ok_or(AtoiSimdError::Empty)? == b'-' {
        if cfg!(target_feature = "sse2")
            && cfg!(target_feature = "sse3")
            && cfg!(target_feature = "sse4.1")
            && cfg!(target_feature = "ssse3")
            && cfg!(target_feature = "avx")
            && cfg!(target_feature = "avx2")
        {
            parse_simd_i8_neg(&s[1..])
        } else {
            parse_fb_neg::<{ i8::MIN as i64 }>(&s[1..]).map(|v| v as i8)
        }
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
            parse_fb_pos::<{ i8::MAX as u64 }>(s).map(|v| v as i8)
        }
    }
}

#[inline]
fn parse_simd_u16(s: &[u8], parse_type: ParseType) -> Result<u16, AtoiSimdError> {
    let val = unsafe { parse_simd_u64(s, parse_type)? };
    match parse_type {
        ParseType::I16 => {
            if val > i16::MAX as u64 {
                Err(AtoiSimdError::Overflow(parse_type, s))
            } else {
                Ok(val as u16)
            }
        }
        _ => {
            if val > u16::MAX as u64 {
                Err(AtoiSimdError::Overflow(parse_type, s))
            } else {
                Ok(val as u16)
            }
        }
    }
}

#[inline]
fn parse_simd_i16_neg(s: &[u8]) -> Result<i16, AtoiSimdError> {
    let val = unsafe { parse_simd_u64(s, ParseType::None)? };
    if val > i16::MAX as u64 + 1 {
        Err(AtoiSimdError::Overflow(ParseType::I16Neg, s))
    } else if val == i16::MAX as u64 + 1 {
        Ok(i16::MIN)
    } else {
        Ok(-(val as i16))
    }
}

#[inline]
fn parse_i16(s: &[u8]) -> Result<i16, AtoiSimdError> {
    if *s.first().ok_or(AtoiSimdError::Empty)? == b'-' {
        if cfg!(target_feature = "sse2")
            && cfg!(target_feature = "sse3")
            && cfg!(target_feature = "sse4.1")
            && cfg!(target_feature = "ssse3")
            && cfg!(target_feature = "avx")
            && cfg!(target_feature = "avx2")
        {
            parse_simd_i16_neg(&s[1..])
        } else {
            parse_fb_neg::<{ i16::MIN as i64 }>(&s[1..]).map(|v| v as i16)
        }
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
            parse_fb_pos::<{ i16::MAX as u64 }>(s).map(|v| v as i16)
        }
    }
}

#[inline]
fn parse_simd_u32(s: &[u8], parse_type: ParseType) -> Result<u32, AtoiSimdError> {
    let val = unsafe { parse_simd_u64(s, parse_type)? };
    match parse_type {
        ParseType::I32 => {
            if val > i32::MAX as u64 {
                Err(AtoiSimdError::Overflow(parse_type, s))
            } else {
                Ok(val as u32)
            }
        }
        _ => {
            if val > u32::MAX as u64 {
                Err(AtoiSimdError::Overflow(parse_type, s))
            } else {
                Ok(val as u32)
            }
        }
    }
}

#[inline]
fn parse_simd_i32_neg(s: &[u8]) -> Result<i32, AtoiSimdError> {
    let val = unsafe { parse_simd_u64(s, ParseType::None)? };
    if val > i32::MAX as u64 + 1 {
        Err(AtoiSimdError::Overflow(ParseType::I32Neg, s))
    } else if val == i32::MAX as u64 + 1 {
        Ok(i32::MIN)
    } else {
        Ok(-(val as i32))
    }
}

#[inline]
fn parse_i32(s: &[u8]) -> Result<i32, AtoiSimdError> {
    if *s.first().ok_or(AtoiSimdError::Empty)? == b'-' {
        if cfg!(target_feature = "sse2")
            && cfg!(target_feature = "sse3")
            && cfg!(target_feature = "sse4.1")
            && cfg!(target_feature = "ssse3")
            && cfg!(target_feature = "avx")
            && cfg!(target_feature = "avx2")
        {
            parse_simd_i32_neg(&s[1..])
        } else {
            parse_fb_neg::<{ i32::MIN as i64 }>(&s[1..]).map(|v| v as i32)
        }
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
            parse_fb_pos::<{ i32::MAX as u64 }>(s).map(|v| v as i32)
        }
    }
}

#[inline]
fn parse_simd_i64_neg(s: &[u8]) -> Result<i64, AtoiSimdError> {
    let res = unsafe { parse_simd_u64(s, ParseType::I64Neg).map(|v| -(v as i64)) };

    if let Err(AtoiSimdError::I64Min) = res {
        return Ok(i64::MIN);
    }

    res
}

/// Parses string of digits and first '-' char.
/// String length (except '-' char) must be 1..=20.
/// This function is slower than `parse_u64()`, because it checks for '-' sign.
/// Uses SSE4.1 intrinsics
#[inline]
fn parse_i64(s: &[u8]) -> Result<i64, AtoiSimdError> {
    if *s.first().ok_or(AtoiSimdError::Empty)? == b'-' {
        if cfg!(target_feature = "sse2")
            && cfg!(target_feature = "sse3")
            && cfg!(target_feature = "sse4.1")
            && cfg!(target_feature = "ssse3")
            && cfg!(target_feature = "avx")
            && cfg!(target_feature = "avx2")
        {
            parse_simd_i64_neg(&s[1..])
        } else {
            parse_fb_neg::<{ i64::MIN }>(&s[1..])
        }
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
            parse_fb_pos::<{ i64::MAX as u64 }>(s).map(|v| v as i64)
        }
    }
}

/// Parses string of digits and first '-' char.
/// String length (except '-' char) must be 1..=32.
/// This function is slower than `parse_u128()`, because it checks for '-' sign.
/// Uses AVX/AVX2 intrinsics
#[inline]
fn parse_i128(s: &[u8]) -> Result<i128, AtoiSimdError> {
    if *s.first().ok_or(AtoiSimdError::Empty)? == b'-' {
        if cfg!(target_feature = "sse2")
            && cfg!(target_feature = "sse3")
            && cfg!(target_feature = "sse4.1")
            && cfg!(target_feature = "ssse3")
            && cfg!(target_feature = "avx")
            && cfg!(target_feature = "avx2")
        {
            unsafe { parse_simd_u128(&s[1..]).map(|v| -(v as i128)) }
        } else {
            parse_fb_128_neg(&s[1..])
        }
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
            parse_fb_128_pos(s).map(|v| v as i128)
        }
    }
}

pub trait Parser<T> {
    fn atoi_simd_parse(s: &[u8]) -> Result<T, AtoiSimdError>;
}

impl Parser<u8> for u8 {
    #[cfg(all(
        target_feature = "sse2",
        target_feature = "sse3",
        target_feature = "sse4.1",
        target_feature = "ssse3",
        target_feature = "avx",
        target_feature = "avx2"
    ))]
    #[inline]
    fn atoi_simd_parse(s: &[u8]) -> Result<u8, AtoiSimdError> {
        parse_simd_u8(s, ParseType::None)
    }

    #[cfg(not(all(
        target_feature = "sse2",
        target_feature = "sse3",
        target_feature = "sse4.1",
        target_feature = "ssse3",
        target_feature = "avx",
        target_feature = "avx2"
    )))]
    #[inline]
    fn atoi_simd_parse(s: &[u8]) -> Result<u8, AtoiSimdError> {
        parse_fb_pos::<{ u8::MAX as u64 }>(s).map(|v| v as u8)
    }
}

impl Parser<i8> for i8 {
    #[inline]
    fn atoi_simd_parse(s: &[u8]) -> Result<i8, AtoiSimdError> {
        parse_i8(s)
    }
}

impl Parser<u16> for u16 {
    #[cfg(all(
        target_feature = "sse2",
        target_feature = "sse3",
        target_feature = "sse4.1",
        target_feature = "ssse3",
        target_feature = "avx",
        target_feature = "avx2"
    ))]
    #[inline]
    fn atoi_simd_parse(s: &[u8]) -> Result<u16, AtoiSimdError> {
        parse_simd_u16(s, ParseType::None)
    }

    #[cfg(not(all(
        target_feature = "sse2",
        target_feature = "sse3",
        target_feature = "sse4.1",
        target_feature = "ssse3",
        target_feature = "avx",
        target_feature = "avx2"
    )))]
    #[inline]
    fn atoi_simd_parse(s: &[u8]) -> Result<u16, AtoiSimdError> {
        parse_fb_pos::<{ u16::MAX as u64 }>(s).map(|v| v as u16)
    }
}

impl Parser<i16> for i16 {
    #[inline]
    fn atoi_simd_parse(s: &[u8]) -> Result<i16, AtoiSimdError> {
        parse_i16(s)
    }
}

impl Parser<u32> for u32 {
    #[cfg(all(
        target_feature = "sse2",
        target_feature = "sse3",
        target_feature = "sse4.1",
        target_feature = "ssse3",
        target_feature = "avx",
        target_feature = "avx2"
    ))]
    #[inline]
    fn atoi_simd_parse(s: &[u8]) -> Result<u32, AtoiSimdError> {
        parse_simd_u32(s, ParseType::None)
    }

    #[cfg(not(all(
        target_feature = "sse2",
        target_feature = "sse3",
        target_feature = "sse4.1",
        target_feature = "ssse3",
        target_feature = "avx",
        target_feature = "avx2"
    )))]
    #[inline]
    fn atoi_simd_parse(s: &[u8]) -> Result<u32, AtoiSimdError> {
        parse_fb_pos::<{ u32::MAX as u64 }>(s).map(|v| v as u32)
    }
}

impl Parser<i32> for i32 {
    #[inline]
    fn atoi_simd_parse(s: &[u8]) -> Result<i32, AtoiSimdError> {
        parse_i32(s)
    }
}

impl Parser<usize> for usize {
    #[cfg(all(
        target_feature = "sse2",
        target_feature = "sse3",
        target_feature = "sse4.1",
        target_feature = "ssse3",
        target_feature = "avx",
        target_feature = "avx2"
    ))]
    #[cfg(target_pointer_width = "32")]
    #[inline]
    fn atoi_simd_parse(s: &[u8]) -> Result<usize, AtoiSimdError> {
        parse_simd_u32(s, ParseType::None).map(|v| v as usize)
    }

    #[cfg(all(
        target_feature = "sse2",
        target_feature = "sse3",
        target_feature = "sse4.1",
        target_feature = "ssse3",
        target_feature = "avx",
        target_feature = "avx2"
    ))]
    #[cfg(target_pointer_width = "64")]
    #[inline]
    fn atoi_simd_parse(s: &[u8]) -> Result<usize, AtoiSimdError> {
        unsafe { parse_simd_u64(s, ParseType::None).map(|v| v as usize) }
    }

    #[cfg(not(all(
        target_feature = "sse2",
        target_feature = "sse3",
        target_feature = "sse4.1",
        target_feature = "ssse3",
        target_feature = "avx",
        target_feature = "avx2"
    )))]
    #[cfg(target_pointer_width = "32")]
    #[inline]
    fn atoi_simd_parse(s: &[u8]) -> Result<usize, AtoiSimdError> {
        parse_fb_pos::<{ u32::MAX as u64 }>(s).map(|v| v as usize)
    }

    #[cfg(not(all(
        target_feature = "sse2",
        target_feature = "sse3",
        target_feature = "sse4.1",
        target_feature = "ssse3",
        target_feature = "avx",
        target_feature = "avx2"
    )))]
    #[cfg(target_pointer_width = "64")]
    #[inline]
    fn atoi_simd_parse(s: &[u8]) -> Result<usize, AtoiSimdError> {
        parse_fb_pos::<{ u64::MAX }>(s).map(|v| v as usize)
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
}

impl Parser<u64> for u64 {
    #[cfg(all(
        target_feature = "sse2",
        target_feature = "sse3",
        target_feature = "sse4.1",
        target_feature = "ssse3",
        target_feature = "avx",
        target_feature = "avx2"
    ))]
    #[inline]
    fn atoi_simd_parse(s: &[u8]) -> Result<u64, AtoiSimdError> {
        unsafe { parse_simd_u64(s, ParseType::None) }
    }

    #[cfg(not(all(
        target_feature = "sse2",
        target_feature = "sse3",
        target_feature = "sse4.1",
        target_feature = "ssse3",
        target_feature = "avx",
        target_feature = "avx2"
    )))]
    #[inline]
    fn atoi_simd_parse(s: &[u8]) -> Result<u64, AtoiSimdError> {
        parse_fb_pos::<{ u64::MAX }>(s)
    }
}

impl Parser<i64> for i64 {
    #[inline]
    fn atoi_simd_parse(s: &[u8]) -> Result<i64, AtoiSimdError> {
        parse_i64(s)
    }
}

impl Parser<u128> for u128 {
    #[cfg(all(
        target_feature = "sse2",
        target_feature = "sse3",
        target_feature = "sse4.1",
        target_feature = "ssse3",
        target_feature = "avx",
        target_feature = "avx2"
    ))]
    #[inline]
    fn atoi_simd_parse(s: &[u8]) -> Result<u128, AtoiSimdError> {
        unsafe { parse_simd_u128(s) }
    }

    #[cfg(not(all(
        target_feature = "sse2",
        target_feature = "sse3",
        target_feature = "sse4.1",
        target_feature = "ssse3",
        target_feature = "avx",
        target_feature = "avx2"
    )))]
    #[inline]
    fn atoi_simd_parse(s: &[u8]) -> Result<u128, AtoiSimdError> {
        parse_fb_128_pos(s)
    }
}

impl Parser<i128> for i128 {
    #[inline]
    fn atoi_simd_parse(s: &[u8]) -> Result<i128, AtoiSimdError> {
        parse_i128(s)
    }
}

#[inline]
pub fn parse<T: Parser<T>>(s: &[u8]) -> Result<T, AtoiSimdError> {
    T::atoi_simd_parse(s)
}
