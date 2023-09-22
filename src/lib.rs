//! # Fast `&[u8]` to integer parser
//!
//! SIMD (fast parsing) is supported on x86_64 (SSE4.1, AVX2) and on Arm64 (aarch64, Neon), but this library works even if you don't have SIMD supported cpu (and it will be still faster than str::parse).
//!
//! Supports negative values and validates the input.
//!
//! Supported output types: u8, i8, u16, i16, u32, i32, u64, i64, u128, i128, usize, isize.
//!
//! Has good test coverage, and can be considered safe.
//!
//! To enable SIMD it needs the `target-feature` or `target-cpu` flags set, or it will fallback to non-SIMD functions.
//! You can copy the `./.cargo/config.toml` to your project, or use one of the following environment variables:
//!
//! -   `RUSTFLAGS="-C target-feature=+sse2,+sse3,+sse4.1,+ssse3,+avx,+avx2"` for x86_64
//!
//! -   `RUSTFLAGS="-C target-feature=+neon"` for Arm64
//!
//! -   `RUSTFLAGS="-C target-cpu=native"` will optimize for your current cpu
//!
//! If you have `&str` then use `.as_bytes()`
//!
//! Supports `no_std` with `--no-default-features`
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
#![cfg_attr(not(feature = "std"), no_std)]
// #![feature(stdsimd)]

mod error;
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
mod fallback;
mod linker;
mod safe_unchecked;
mod short;
#[cfg(any(
    all(target_arch = "aarch64", target_feature = "neon"),
    all(
        target_feature = "sse2",
        target_feature = "sse",
        target_feature = "sse4.1",
        target_feature = "ssse3"
    ),
))]
mod simd;
#[cfg(test)]
mod test;

pub use crate::error::AtoiSimdError;
use crate::linker::{Parser, ParserNeg, ParserPos};

/// Parses slice of digits, and checks first '-' char for signed integers.
#[inline]
pub fn parse<T: Parser<T> + ParserPos<T>>(s: &[u8]) -> Result<T, AtoiSimdError> {
    T::atoi_simd_parse(s)
}

/// Parses positive integer.
#[inline]
pub fn parse_pos<T: ParserPos<T>>(s: &[u8]) -> Result<T, AtoiSimdError> {
    T::atoi_simd_parse_pos(s)
}

/// Parses negative integer. Slice must not contain '-' sign.
#[inline]
pub fn parse_neg<T: ParserNeg<T>>(s: &[u8]) -> Result<T, AtoiSimdError> {
    T::atoi_simd_parse_neg(s)
}

/// Parses slice of digits until it reaches invalid character, and checks first '-' char for signed integers.
/// Returns parsed value and parsed size of the slice.
#[inline]
pub fn parse_until_invalid<T: Parser<T> + ParserPos<T>>(
    s: &[u8],
) -> Result<(T, usize), AtoiSimdError> {
    T::atoi_simd_parse_until_invalid(s)
}

/// Parses positive integer until it reaches invalid character.
/// Returns parsed value and parsed size of the slice.
#[inline]
pub fn parse_until_invalid_pos<T: ParserPos<T>>(s: &[u8]) -> Result<(T, usize), AtoiSimdError> {
    T::atoi_simd_parse_until_invalid_pos(s)
}

/// Parses negative integer until it reaches invalid character. Slice must not contain '-' sign.
/// Returns parsed value and parsed size of the slice.
#[inline]
pub fn parse_until_invalid_neg<T: ParserNeg<T>>(s: &[u8]) -> Result<(T, usize), AtoiSimdError> {
    T::atoi_simd_parse_until_invalid_neg(s)
}
