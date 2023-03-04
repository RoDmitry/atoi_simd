//! # Fast `&[u8]` to integer parser
//!
//! Faster on x86_64 (uses SIMD, SSE4.1, AVX2), but can be used even if you don't have x86_64 SIMD capable cpu.
//!
//! Supports negative values and validates the input.
//!
//! Supported output types: u8, i8, u16, i16, u32, i32, u64, i64, u128, i128, usize, isize.
//!
//! Has good test coverage, and can be considered safe.
//!
//! The code will be inlined to the place where you call the function.
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

mod error;
mod fallback;
mod parser;
#[cfg(all(
    target_feature = "sse2",
    target_feature = "sse3",
    target_feature = "sse4.1",
    target_feature = "ssse3"
))]
mod simd_sse_avx;
#[cfg(test)]
mod test;

pub use crate::error::AtoiSimdError;
use crate::parser::{Parser, ParserNeg, ParserPos};

/// Parses slice of digits, and checks first '-' char for signed integers.
#[inline(always)]
pub fn parse<T: Parser<T> + ParserPos<T>>(s: &[u8]) -> Result<T, AtoiSimdError> {
    T::atoi_simd_parse(s)
}

/// Parses positive integer.
#[inline(always)]
pub fn parse_pos<T: ParserPos<T>>(s: &[u8]) -> Result<T, AtoiSimdError> {
    T::atoi_simd_parse_pos(s)
}

/// Parses negative integer. Slice must not contain '-' sign.
#[inline(always)]
pub fn parse_neg<T: ParserNeg<T>>(s: &[u8]) -> Result<T, AtoiSimdError> {
    T::atoi_simd_parse_neg(s)
}

/// Parses slice of digits until it reaches invalid character, and checks first '-' char for signed integers.
/// Returns parsed value and parsed size of the slice.
#[inline(always)]
pub fn parse_until_invalid<T: Parser<T> + ParserPos<T>>(
    s: &[u8],
) -> Result<(T, usize), AtoiSimdError> {
    T::atoi_simd_parse_until_invalid(s)
}

/// Parses positive integer until it reaches invalid character.
/// Returns parsed value and parsed size of the slice.
#[inline(always)]
pub fn parse_until_invalid_pos<T: ParserPos<T>>(s: &[u8]) -> Result<(T, usize), AtoiSimdError> {
    T::atoi_simd_parse_until_invalid_pos(s)
}

/// Parses negative integer until it reaches invalid character. Slice must not contain '-' sign.
/// Returns parsed value and parsed size of the slice.
#[inline(always)]
pub fn parse_until_invalid_neg<T: ParserNeg<T>>(s: &[u8]) -> Result<(T, usize), AtoiSimdError> {
    T::atoi_simd_parse_until_invalid_neg(s)
}
