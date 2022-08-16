//! # Rust fast `&str` to `i64` parser (x86_64 SIMD, SSE4.1)
//!
//! Must be used when you are certain that the string contains only digits.
//! If you pass not only digits, it will give you the wrong output (not error).
//!
//! Max string length is 16 numbers (17 with sign).
//!
//! # Examples
//!
//! ```
//! assert_eq!(atoi_simd::parse("0").unwrap(), 0_u64);
//! assert_eq!(atoi_simd::parse("1234").unwrap(), 1234_u64);
//!
//! assert_eq!(atoi_simd::parse_i64("2345").unwrap(), 2345_i64);
//! assert_eq!(atoi_simd::parse_i64("-2345").unwrap(), -2345_i64);
//! ```

use core::arch::x86_64::{
    __m128i, _mm_and_si128, _mm_bslli_si128, _mm_cvtsi128_si64, _mm_lddqu_si128, _mm_madd_epi16,
    _mm_maddubs_epi16, _mm_packus_epi32, _mm_set1_epi8, _mm_set_epi16, _mm_set_epi8,
};
use std::fmt;

#[derive(Debug, Clone)]
pub struct AtoiSimdError(usize);

impl fmt::Display for AtoiSimdError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "atoi_simd string wrong size {}", self.0)
    }
}

unsafe fn bslli_si128(chunk: __m128i, s_len: usize) -> Result<__m128i, AtoiSimdError> {
    Ok(match s_len {
        1 => _mm_bslli_si128(chunk, 15),
        2 => _mm_bslli_si128(chunk, 14),
        3 => _mm_bslli_si128(chunk, 13),
        4 => _mm_bslli_si128(chunk, 12),
        5 => _mm_bslli_si128(chunk, 11),
        6 => _mm_bslli_si128(chunk, 10),
        7 => _mm_bslli_si128(chunk, 9),
        8 => _mm_bslli_si128(chunk, 8),
        9 => _mm_bslli_si128(chunk, 7),
        10 => _mm_bslli_si128(chunk, 6),
        11 => _mm_bslli_si128(chunk, 5),
        12 => _mm_bslli_si128(chunk, 4),
        13 => _mm_bslli_si128(chunk, 3),
        14 => _mm_bslli_si128(chunk, 2),
        15 => _mm_bslli_si128(chunk, 1),
        16 => chunk,
        val => return Err(AtoiSimdError(val)),
    })
}

/// Parses string of *only* digits. String length must be 1..=16.
pub fn parse(s: &str) -> Result<u64, AtoiSimdError> {
    unsafe {
        let mut chunk = _mm_lddqu_si128(std::mem::transmute_copy(&s));
        let mut mult = _mm_set1_epi8(0xf);
        chunk = _mm_and_si128(chunk, mult);

        chunk = bslli_si128(chunk, s.len())?;

        mult = _mm_set_epi8(1, 10, 1, 10, 1, 10, 1, 10, 1, 10, 1, 10, 1, 10, 1, 10);
        chunk = _mm_maddubs_epi16(chunk, mult);

        mult = _mm_set_epi16(1, 100, 1, 100, 1, 100, 1, 100);
        chunk = _mm_madd_epi16(chunk, mult);

        chunk = _mm_packus_epi32(chunk, chunk);
        mult = _mm_set_epi16(0, 0, 0, 0, 1, 10000, 1, 10000);
        chunk = _mm_madd_epi16(chunk, mult);

        let chunk = _mm_cvtsi128_si64(chunk) as u64;
        Ok(((chunk & 0xffffffff) * 100000000) + (chunk >> 32))
    }
}

/// Parses string of *only* digits and first '-' char.
/// String length (except '-' char) must be 1..=16.
/// This function is slower than `parse()`, because it checks for '-' sign.
pub fn parse_i64(s: &str) -> Result<i64, AtoiSimdError> {
    if let Some(strip) = s.strip_prefix('-') {
        parse(strip).map(|v| -(v as i64))
    } else {
        parse(s).map(|v| v as i64)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse() {
        if parse("").is_ok() {
            panic!("error");
        }

        if parse("12345678901234567").is_ok() {
            panic!("error");
        }

        assert_eq!(parse("0").unwrap(), 0_u64);

        assert_eq!(parse("12345").unwrap(), 12345_u64);

        assert_eq!(parse("1234567890123456").unwrap(), 1234567890123456_u64);

        assert_eq!(
            parse("9999999999999999").unwrap(),
            9_999_999_999_999_999_u64
        );
    }

    #[test]
    fn test_parse_i64() {
        if parse_i64("").is_ok() {
            panic!("error");
        }

        if parse_i64("12345678901234567").is_ok() {
            panic!("error");
        }

        assert_eq!(parse_i64("0").unwrap(), 0_i64);

        assert_eq!(parse_i64("-0").unwrap(), 0_i64);

        assert_eq!(parse_i64("12345").unwrap(), 12345_i64);

        assert_eq!(parse_i64("-12345").unwrap(), -12345_i64);

        assert_eq!(parse_i64("1234567890123456").unwrap(), 1234567890123456_i64);

        assert_eq!(
            parse_i64("-1234567890123456").unwrap(),
            -1234567890123456_i64
        );

        assert_eq!(
            parse_i64("-9999999999999999").unwrap(),
            -9_999_999_999_999_999_i64
        );

        assert_eq!(
            parse_i64("9999999999999999").unwrap(),
            9_999_999_999_999_999_i64
        );
    }
}
