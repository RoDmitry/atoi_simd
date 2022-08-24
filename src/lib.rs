//! # Rust fast `&str` to integer parser (x86_64 SIMD, SSE4.1, AVX2)
//!
//! Must be used when you are certain that the string contains only digits.
//! If you pass not only digits, it will give you the wrong output (not error).
//!
//! The 64 bit functions use SSE4.1, max string length is 16 numbers (17 with sign).
//!
//! The 128 bit functions use AVX2, max string length is 32 numbers (33 with sign).
//!
//! # Examples
//!
//! ```
//! assert_eq!(atoi_simd::parse("0").unwrap(), 0_u64);
//! assert_eq!(atoi_simd::parse("1234").unwrap(), 1234_u64);
//!
//! assert_eq!(atoi_simd::parse_i64("2345").unwrap(), 2345_i64);
//! assert_eq!(atoi_simd::parse_i64("-2345").unwrap(), -2345_i64);
//!
//! assert_eq!(atoi_simd::parse_u128("1234").unwrap(), 1234_u128);
//!
//! assert_eq!(atoi_simd::parse_i128("2345").unwrap(), 2345_i128);
//! assert_eq!(atoi_simd::parse_i128("-1234").unwrap(), -1234_i128);
//! ```

use core::arch::x86_64::{
    __m128i, __m256i, _mm256_add_epi64, _mm256_and_si256, _mm256_bslli_epi128, _mm256_bsrli_epi128,
    _mm256_lddqu_si256, _mm256_madd_epi16, _mm256_maddubs_epi16, _mm256_mul_epu32, _mm256_or_si256,
    _mm256_packus_epi32, _mm256_permute2x128_si256, _mm256_set1_epi8, _mm256_set_epi16,
    _mm256_set_epi32, _mm256_set_epi8, _mm256_srli_epi64, _mm_and_si128, _mm_bslli_si128,
    _mm_cvtsi128_si64, _mm_lddqu_si128, _mm_madd_epi16, _mm_maddubs_epi16, _mm_packus_epi32,
    _mm_set1_epi8, _mm_set_epi16, _mm_set_epi64x, _mm_set_epi8,
};
use std::fmt;

#[derive(Debug, Clone)]
pub struct AtoiSimdError(usize);

impl fmt::Display for AtoiSimdError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "atoi_simd string wrong size {}", self.0)
    }
}

/// s = "1234567890123456"
unsafe fn read(s: &str) -> __m128i {
    _mm_lddqu_si128(std::mem::transmute_copy(&s))
}

unsafe fn read_avx(s: &str) -> __m256i {
    _mm256_lddqu_si256(std::mem::transmute_copy(&s))
}

/// converts chars  [ 0x36353433323130393837363534333231 ]
/// to numbers      [ 0x06050403020100090807060504030201 ]
unsafe fn to_numbers(chunk: __m128i) -> __m128i {
    // _mm_unpacklo_epi8 for hex decoding?
    _mm_and_si128(chunk, _mm_set1_epi8(0xf))
}

unsafe fn to_numbers_val(chunk: __m128i, lval: i64) -> __m128i {
    _mm_and_si128(chunk, _mm_set_epi64x(0, lval))
}

/// combine numbers [ 0x0038 | 0x0022 | 0x000c | 0x005a | 0x004e | 0x0038 | 0x0022 | 0x000c ( 56 | 34 | 12 | 90 | 78 | 56 | 34 | 12 ) ]
unsafe fn mult_10(chunk: __m128i) -> __m128i {
    _mm_maddubs_epi16(
        chunk,
        _mm_set_epi8(1, 10, 1, 10, 1, 10, 1, 10, 1, 10, 1, 10, 1, 10, 1, 10),
    )
}

/// combine again   [ 0x0000 | 0x0d80 | 0x0000 | 0x2334 | 0x0000 | 0x162e | 0x0000 | 0x04d2 ( 0 | 3456 | 0 | 9012 | 0 | 5678 | 0 | 1234) ]
unsafe fn mult_100(chunk: __m128i) -> __m128i {
    _mm_madd_epi16(chunk, _mm_set_epi16(1, 100, 1, 100, 1, 100, 1, 100))
}

unsafe fn to_u64(chunk: __m128i) -> u64 {
    _mm_cvtsi128_si64(chunk) as u64
}

unsafe fn to_u32x4(chunk: __m128i) -> [u32; 4] {
    std::mem::transmute(chunk)
}

unsafe fn process_internal(mut chunk: __m128i) -> __m128i {
    chunk = mult_10(chunk);
    chunk = mult_100(chunk);

    // remove extra bytes [ (64 bits, same as the right ) | 0x0d80 | 0x2334 | 0x162e | 0x04d2 ( 3456 | 9012 | 5678 | 1234) ]
    chunk = _mm_packus_epi32(chunk, chunk);

    // _mm_set_epi16(1, 10000, 0, 0, 0, 0, 1, 10000);

    // combine again [ (64 bits, zeroes) | 0x055f2cc0 | 0x00bc614e ( 90123456 | 12345678 ) ]
    _mm_madd_epi16(chunk, _mm_set_epi16(0, 0, 0, 0, 1, 10000, 1, 10000))
}

unsafe fn process_small(mut chunk: __m128i) -> u64 {
    chunk = to_numbers_val(chunk, 0xF0F0F0F);
    chunk = mult_10(chunk);
    chunk = mult_100(chunk);

    to_u64(chunk)
    // std::mem::transmute::<__m128i, [u32; 4]>(chunk)[3] as u64 // same performance
}

unsafe fn process_medium(mut chunk: __m128i) -> u64 {
    chunk = to_numbers_val(chunk, 0xF0F0F0F0F0F0F0F);
    chunk = process_internal(chunk);

    to_u64(chunk)
}

unsafe fn process_big(mut chunk: __m128i) -> u64 {
    chunk = to_numbers(chunk);
    chunk = process_internal(chunk);

    // let chunk = to_u64(chunk);
    // ((chunk & 0xFFFF_FFFF) * 100_000_000) + (chunk >> 32)

    let arr = to_u32x4(chunk);
    (arr[0] as u64 * 100_000_000) + (arr[1] as u64)
}

/// Parses string of *only* digits. String length must be 1..=16.
/// Uses SSE4.1 intrinsics
pub fn parse(s: &str) -> Result<u64, AtoiSimdError> {
    unsafe {
        Ok(match s.len() {
            1 => (*s.as_bytes().first().unwrap() & 0xf) as u64,
            2 => {
                let mut chunk = read(s);
                chunk = to_numbers_val(chunk, 0xF0F);
                chunk = mult_10(chunk);
                to_u64(chunk)
                // std::mem::transmute::<__m128i, [u16; 8]>(chunk)[7] as u64 // same performance
            }
            3 => {
                let mut chunk = read(s);
                chunk = _mm_bslli_si128(chunk, 1);
                process_small(chunk)
            }
            4 => process_small(read(s)),
            5 => {
                let mut chunk = read(s);
                chunk = _mm_bslli_si128(chunk, 3);
                process_medium(chunk)
            }
            6 => {
                let mut chunk = read(s);
                chunk = _mm_bslli_si128(chunk, 2);
                process_medium(chunk)
            }
            7 => {
                let mut chunk = read(s);
                chunk = _mm_bslli_si128(chunk, 1);
                process_medium(chunk)
            }
            8 => process_medium(read(s)),
            9 => {
                let mut chunk = read(s);
                chunk = _mm_bslli_si128(chunk, 7);
                process_big(chunk)
            }
            10 => {
                let mut chunk = read(s);
                chunk = _mm_bslli_si128(chunk, 6);
                process_big(chunk)
            }
            11 => {
                let mut chunk = read(s);
                chunk = _mm_bslli_si128(chunk, 5);
                process_big(chunk)
            }
            12 => {
                let mut chunk = read(s);
                chunk = _mm_bslli_si128(chunk, 4);
                process_big(chunk)
            }
            13 => {
                let mut chunk = read(s);
                chunk = _mm_bslli_si128(chunk, 3);
                process_big(chunk)
            }
            14 => {
                let mut chunk = read(s);
                chunk = _mm_bslli_si128(chunk, 2);
                process_big(chunk)
            }
            15 => {
                let mut chunk = read(s);
                chunk = _mm_bslli_si128(chunk, 1);
                process_big(chunk)
            }
            16 => process_big(read(s)),
            s_len => return Err(AtoiSimdError(s_len)),
        })
    }
}

/// Uses AVX/AVX2 intrinsics
unsafe fn process_avx(mut chunk: __m256i) -> u128 {
    // to numbers
    chunk = _mm256_and_si256(chunk, _mm256_set1_epi8(0xf));

    let mut mult = _mm256_set_epi8(
        1, 10, 1, 10, 1, 10, 1, 10, 1, 10, 1, 10, 1, 10, 1, 10, 1, 10, 1, 10, 1, 10, 1, 10, 1, 10,
        1, 10, 1, 10, 1, 10,
    );
    // mult 10
    chunk = _mm256_maddubs_epi16(chunk, mult);

    mult = _mm256_set_epi16(
        1, 100, 1, 100, 1, 100, 1, 100, 1, 100, 1, 100, 1, 100, 1, 100,
    );
    // mult 100
    chunk = _mm256_madd_epi16(chunk, mult);

    // remove extra bytes
    chunk = _mm256_packus_epi32(chunk, chunk);

    // it can be used to move to the SSE intrinsics
    // move by 64 bits ( unused | unused | third [191:128] | first [63:0] )
    // chunk = _mm256_permute4x64_epi64(chunk, 8);

    mult = _mm256_set_epi16(
        0, 0, 0, 0, 1, 10000, 1, 10000, 0, 0, 0, 0, 1, 10000, 1, 10000,
    );
    // mult 10000
    chunk = _mm256_madd_epi16(chunk, mult);

    mult = _mm256_set_epi32(0, 0, 0, 100_000_000, 0, 0, 0, 100_000_000);
    // mult 100_000_000
    mult = _mm256_mul_epu32(chunk, mult);

    let sum_chunk = _mm256_srli_epi64(chunk, 32);
    chunk = _mm256_add_epi64(sum_chunk, mult);

    let arr = std::mem::transmute::<__m256i, [u128; 2]>(chunk);

    arr[0] * 10_000_000_000_000_000 + arr[1]
}

/*
/// SSE intrinsics for the previous function `process_avx()`
unsafe fn process_u128(s: &str) -> u128 {
    let mut chunk = process_m256i_to_m128i(s);

    let mut mult = _mm_set_epi16(1, 10000, 1, 10000, 1, 10000, 1, 10000);
    // mult_10000
    chunk = _mm_madd_epi16(chunk, mult);

    mult = _mm_set_epi32(0, 100_000_000, 0, 100_000_000);
    // mult 100_000_000
    mult = _mm_mul_epu32(chunk, mult);

    let sum_chunk = _mm_srli_epi64(chunk, 32);
    chunk = _mm_add_epi64(sum_chunk, mult);

    std::mem::transmute::<__m128i, u128>(chunk)
} */

unsafe fn process_avx_permute2x128(chunk: __m256i) -> __m256i {
    _mm256_permute2x128_si256(chunk, chunk, 8)
}

unsafe fn process_avx_or(chunk: __m256i, mult: __m256i) -> __m256i {
    _mm256_or_si256(chunk, mult)
}

/// Parses string of *only* digits. String length must be 1..=32.
/// Uses AVX/AVX2 intrinsics
pub fn parse_u128(s: &str) -> Result<u128, AtoiSimdError> {
    unsafe {
        let mut chunk = read_avx(s);

        match s.len() {
            17 => {
                let mut mult = process_avx_permute2x128(chunk);
                mult = _mm256_bsrli_epi128(mult, 1);
                chunk = _mm256_bslli_epi128(chunk, 15);
                chunk = process_avx_or(chunk, mult);
            }
            18 => {
                let mut mult = process_avx_permute2x128(chunk);
                mult = _mm256_bsrli_epi128(mult, 2);
                chunk = _mm256_bslli_epi128(chunk, 14);
                chunk = process_avx_or(chunk, mult);
            }
            19 => {
                let mut mult = process_avx_permute2x128(chunk);
                mult = _mm256_bsrli_epi128(mult, 3);
                chunk = _mm256_bslli_epi128(chunk, 13);
                chunk = process_avx_or(chunk, mult);
            }
            20 => {
                // maybe can be optimized even further with _mm256_permutevar8x32_epi32
                let mut mult = process_avx_permute2x128(chunk);
                mult = _mm256_bsrli_epi128(mult, 4);
                chunk = _mm256_bslli_epi128(chunk, 12);
                chunk = process_avx_or(chunk, mult);
            }
            21 => {
                let mut mult = process_avx_permute2x128(chunk);
                mult = _mm256_bsrli_epi128(mult, 5);
                chunk = _mm256_bslli_epi128(chunk, 11);
                chunk = process_avx_or(chunk, mult);
            }
            22 => {
                let mut mult = process_avx_permute2x128(chunk);
                mult = _mm256_bsrli_epi128(mult, 6);
                chunk = _mm256_bslli_epi128(chunk, 10);
                chunk = process_avx_or(chunk, mult);
            }
            23 => {
                let mut mult = process_avx_permute2x128(chunk);
                mult = _mm256_bsrli_epi128(mult, 7);
                chunk = _mm256_bslli_epi128(chunk, 9);
                chunk = process_avx_or(chunk, mult);
            }
            24 => {
                // maybe can be optimized even further with _mm256_permute4x64_epi64
                let mut mult = process_avx_permute2x128(chunk);
                mult = _mm256_bsrli_epi128(mult, 8);
                chunk = _mm256_bslli_epi128(chunk, 8);
                chunk = process_avx_or(chunk, mult);
            }
            25 => {
                let mut mult = process_avx_permute2x128(chunk);
                mult = _mm256_bsrli_epi128(mult, 9);
                chunk = _mm256_bslli_epi128(chunk, 7);
                chunk = process_avx_or(chunk, mult);
            }
            26 => {
                let mut mult = process_avx_permute2x128(chunk);
                mult = _mm256_bsrli_epi128(mult, 10);
                chunk = _mm256_bslli_epi128(chunk, 6);
                chunk = process_avx_or(chunk, mult);
            }
            27 => {
                let mut mult = process_avx_permute2x128(chunk);
                mult = _mm256_bsrli_epi128(mult, 11);
                chunk = _mm256_bslli_epi128(chunk, 5);
                chunk = process_avx_or(chunk, mult);
            }
            28 => {
                // maybe can be optimized even further with _mm256_permutevar8x32_epi32
                // let mult = _mm256_set_epi32(6, 5, 4, 3, 2, 1, 0, 0);
                // chunk = _mm256_permutevar8x32_epi32(chunk, mult);
                let mut mult = process_avx_permute2x128(chunk);
                mult = _mm256_bsrli_epi128(mult, 12);
                chunk = _mm256_bslli_epi128(chunk, 4);
                chunk = process_avx_or(chunk, mult);
            }
            29 => {
                let mut mult = process_avx_permute2x128(chunk);
                mult = _mm256_bsrli_epi128(mult, 13);
                chunk = _mm256_bslli_epi128(chunk, 3);
                chunk = process_avx_or(chunk, mult);
            }
            30 => {
                let mut mult = process_avx_permute2x128(chunk);
                mult = _mm256_bsrli_epi128(mult, 14);
                chunk = _mm256_bslli_epi128(chunk, 2);
                chunk = process_avx_or(chunk, mult);
            }
            31 => {
                let mut mult = process_avx_permute2x128(chunk);
                // somehow it'll be bugged, if you move permute after it
                mult = _mm256_bsrli_epi128(mult, 15);
                chunk = _mm256_bslli_epi128(chunk, 1);
                chunk = process_avx_or(chunk, mult);
            }
            32 => (),
            _ => return parse(s).map(|v| v as u128),
        }

        Ok(process_avx(chunk))
    }
}

/// Parses string of *only* digits and first '-' char.
/// String length (except '-' char) must be 1..=16.
/// This function is slower than `parse()`, because it checks for '-' sign.
/// Uses SSE4.1 intrinsics
pub fn parse_i64(s: &str) -> Result<i64, AtoiSimdError> {
    if let Some(strip) = s.strip_prefix('-') {
        parse(strip).map(|v| -(v as i64))
    } else {
        parse(s).map(|v| v as i64)
    }
}

/// Parses string of *only* digits and first '-' char.
/// String length (except '-' char) must be 1..=32.
/// This function is slower than `parse_u128()`, because it checks for '-' sign.
/// Uses AVX/AVX2 intrinsics
pub fn parse_i128(s: &str) -> Result<i128, AtoiSimdError> {
    if let Some(strip) = s.strip_prefix('-') {
        parse_u128(strip).map(|v| -(v as i128))
    } else {
        parse_u128(s).map(|v| v as i128)
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

        let mut str = String::with_capacity(16);
        for i in '1'..='9' {
            str.push(i);
            assert_eq!(parse(&str).unwrap(), str.parse::<u64>().unwrap());
        }
        for i in '0'..='6' {
            str.push(i);
            assert_eq!(parse(&str).unwrap(), str.parse::<u64>().unwrap());
        }

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

        let mut str = String::with_capacity(16);
        let mut str_neg = String::with_capacity(17);
        str_neg.push('-');
        for i in '1'..='9' {
            str.push(i);
            str_neg.push(i);
            assert_eq!(parse_i64(&str).unwrap(), str.parse::<i64>().unwrap());
            assert_eq!(
                parse_i64(&str_neg).unwrap(),
                str_neg.parse::<i64>().unwrap()
            );
        }
        for i in '0'..='6' {
            str.push(i);
            str_neg.push(i);
            assert_eq!(parse_i64(&str).unwrap(), str.parse::<i64>().unwrap());
            assert_eq!(
                parse_i64(&str_neg).unwrap(),
                str_neg.parse::<i64>().unwrap()
            );
        }

        assert_eq!(
            parse_i64("-9999999999999999").unwrap(),
            -9_999_999_999_999_999_i64
        );

        assert_eq!(
            parse_i64("9999999999999999").unwrap(),
            9_999_999_999_999_999_i64
        );
    }

    #[test]
    fn test_parse_u128() {
        if parse_u128("").is_ok() {
            panic!("error");
        }

        if parse_u128("123456789012345678901234567890123").is_ok() {
            panic!("error");
        }

        assert_eq!(parse_u128("0").unwrap(), 0_u128);

        let mut str = String::with_capacity(32);
        for i in '1'..='9' {
            str.push(i);
            assert_eq!(parse_u128(&str).unwrap(), str.parse::<u128>().unwrap());
        }
        for _ in 0..2 {
            for i in '0'..='9' {
                str.push(i);
                assert_eq!(parse_u128(&str).unwrap(), str.parse::<u128>().unwrap());
            }
        }
        for i in '0'..='2' {
            str.push(i);
            assert_eq!(parse_u128(&str).unwrap(), str.parse::<u128>().unwrap());
        }

        assert_eq!(
            parse_u128("9999999999999999").unwrap(),
            9_999_999_999_999_999_u128
        );
    }

    #[test]
    fn test_parse_i128() {
        if parse_i128("").is_ok() {
            panic!("error");
        }

        if parse_i128("123456789012345678901234567890123").is_ok() {
            panic!("error");
        }

        assert_eq!(parse_i128("0").unwrap(), 0_i128);

        assert_eq!(parse_i128("-0").unwrap(), 0_i128);

        let mut str = String::with_capacity(32);
        let mut str_neg = String::with_capacity(33);
        str_neg.push('-');
        for i in '1'..='9' {
            str.push(i);
            str_neg.push(i);
            assert_eq!(parse_i128(&str).unwrap(), str.parse::<i128>().unwrap());
            assert_eq!(
                parse_i128(&str_neg).unwrap(),
                str_neg.parse::<i128>().unwrap()
            );
        }
        for _ in 0..2 {
            for i in '0'..='9' {
                str.push(i);
                str_neg.push(i);
                assert_eq!(parse_i128(&str).unwrap(), str.parse::<i128>().unwrap());
                assert_eq!(
                    parse_i128(&str_neg).unwrap(),
                    str_neg.parse::<i128>().unwrap()
                );
            }
        }
        for i in '0'..='2' {
            str.push(i);
            str_neg.push(i);
            assert_eq!(parse_i128(&str).unwrap(), str.parse::<i128>().unwrap());
            assert_eq!(
                parse_i128(&str_neg).unwrap(),
                str_neg.parse::<i128>().unwrap()
            );
        }

        assert_eq!(
            parse_i128("-9999999999999999").unwrap(),
            -9_999_999_999_999_999_i128
        );

        assert_eq!(
            parse_i128("9999999999999999").unwrap(),
            9_999_999_999_999_999_i128
        );

        assert_eq!(
            parse_i128("-99999999999999999999999999999999").unwrap(),
            -99_999_999_999_999_999_999_999_999_999_999_i128
        );

        assert_eq!(
            parse_i128("99999999999999999999999999999999").unwrap(),
            99_999_999_999_999_999_999_999_999_999_999_i128
        );
    }
}
