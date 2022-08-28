//! # Rust fast `&str` to integer parser (x86_64 SIMD, SSE4.1, AVX2)
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
    _mm256_cmpgt_epi8, _mm256_lddqu_si256, _mm256_madd_epi16, _mm256_maddubs_epi16,
    _mm256_mul_epu32, _mm256_or_si256, _mm256_packus_epi32, _mm256_permute2x128_si256,
    _mm256_set1_epi8, _mm256_set_epi16, _mm256_set_epi32, _mm256_set_epi64x, _mm256_set_epi8,
    _mm256_srli_epi64, _mm256_testz_si256, _mm_and_si128, _mm_andnot_si128, _mm_bslli_si128,
    _mm_cmpgt_epi8, _mm_cvtsi128_si64, _mm_lddqu_si128, _mm_madd_epi16, _mm_maddubs_epi16,
    _mm_or_si128, _mm_packus_epi32, _mm_set1_epi8, _mm_set_epi16, _mm_set_epi64x, _mm_set_epi8,
    _mm_test_all_ones,
};
use std::fmt;

#[derive(Debug, Clone)]
pub enum AtoiSimdError<'a> {
    Size(usize),
    Invalid(&'a str),
}

impl fmt::Display for AtoiSimdError<'_> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            AtoiSimdError::Size(len) => write!(f, "atoi_simd string wrong size: {}", len),
            AtoiSimdError::Invalid(val) => {
                write!(
                    f,
                    "atoi_simd invalid string, it must contain only digits: {}",
                    val
                )
            }
        }
    }
}

const HIGH_U64: u64 = 0xFFFF_FFFF_FFFF_FFFF;
const HIGH: i8 = 0x7F;
const LOW: i8 = -0x80;
const CHAR_MAX: i8 = 0x39;
const CHAR_MIN: i8 = 0x30;

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
    let mult = _mm_set1_epi8(0xF);

    _mm_and_si128(chunk, mult)
}

unsafe fn process_and(chunk: __m128i, lval: i64) -> __m128i {
    let mult = _mm_set_epi64x(0, lval);

    _mm_and_si128(chunk, mult)
}

unsafe fn process_gt(cmp_left: __m128i, cmp_right: __m128i) -> __m128i {
    _mm_cmpgt_epi8(cmp_left, cmp_right)
}

unsafe fn process_avx_gt(cmp_left: __m256i, cmp_right: __m256i) -> __m256i {
    _mm256_cmpgt_epi8(cmp_left, cmp_right)
}

/// combine numbers [ 0x0038 | 0x0022 | 0x000c | 0x005a | 0x004e | 0x0038 | 0x0022 | 0x000c ( 56 | 34 | 12 | 90 | 78 | 56 | 34 | 12 ) ]
unsafe fn mult_10(chunk: __m128i) -> __m128i {
    let mult = _mm_set_epi8(1, 10, 1, 10, 1, 10, 1, 10, 1, 10, 1, 10, 1, 10, 1, 10);

    _mm_maddubs_epi16(chunk, mult)
}

/// combine again   [ 0x0000 | 0x0d80 | 0x0000 | 0x2334 | 0x0000 | 0x162e | 0x0000 | 0x04d2 ( 0 | 3456 | 0 | 9012 | 0 | 5678 | 0 | 1234) ]
unsafe fn mult_100(chunk: __m128i) -> __m128i {
    let mult = _mm_set_epi16(1, 100, 1, 100, 1, 100, 1, 100);

    _mm_madd_epi16(chunk, mult)
}

unsafe fn to_u64(chunk: __m128i) -> u64 {
    _mm_cvtsi128_si64(chunk) as u64
}

unsafe fn to_u32x4(chunk: __m128i) -> [u32; 4] {
    std::mem::transmute(chunk)
}

unsafe fn process_internal(mut chunk: __m128i) -> __m128i {
    chunk = mult_100(chunk);

    // remove extra bytes [ (64 bits, same as the right ) | 0x0d80 | 0x2334 | 0x162e | 0x04d2 ( 3456 | 9012 | 5678 | 1234) ]
    chunk = _mm_packus_epi32(chunk, chunk);

    // _mm_set_epi16(1, 10000, 0, 0, 0, 0, 1, 10000);

    let mult = _mm_set_epi16(0, 0, 0, 0, 1, 10000, 1, 10000);
    // combine again [ (64 bits, zeroes) | 0x055f2cc0 | 0x00bc614e ( 90123456 | 12345678 ) ]
    _mm_madd_epi16(chunk, mult)
}

unsafe fn checker(check: __m128i, check2: __m128i, s: &str) -> Result<(), AtoiSimdError> {
    let mut chunk = _mm_or_si128(check, check2);

    let mult = _mm_set_epi64x(HIGH_U64 as i64, HIGH_U64 as i64);
    // invert all bits
    chunk = _mm_andnot_si128(chunk, mult);

    let res = _mm_test_all_ones(chunk);
    if res == 0 {
        return Err(AtoiSimdError::Invalid(s));
    }

    Ok(())
}

unsafe fn checker_avx(check: __m256i, check2: __m256i, s: &str) -> Result<(), AtoiSimdError> {
    let chunk = _mm256_or_si256(check, check2);

    let mult = _mm256_set_epi64x(
        HIGH_U64 as i64,
        HIGH_U64 as i64,
        HIGH_U64 as i64,
        HIGH_U64 as i64,
    );
    // test all zeroes
    let res = _mm256_testz_si256(chunk, mult);
    if res == 0 {
        return Err(AtoiSimdError::Invalid(s));
    }

    Ok(())
}

unsafe fn process_small(
    mut chunk: __m128i,
    check: __m128i,
    check2: __m128i,
    s: &str,
) -> Result<u64, AtoiSimdError> {
    chunk = process_and(chunk, 0xF0F0F0F);
    chunk = mult_10(chunk);

    checker(check, check2, s)?;

    chunk = mult_100(chunk);

    Ok(to_u64(chunk))
    // std::mem::transmute::<__m128i, [u32; 4]>(chunk)[3] as u64 // same performance
}

unsafe fn process_medium(
    mut chunk: __m128i,
    check: __m128i,
    check2: __m128i,
    s: &str,
) -> Result<u64, AtoiSimdError> {
    chunk = process_and(chunk, 0xF0F0F0F0F0F0F0F);
    chunk = mult_10(chunk);

    checker(check, check2, s)?;

    chunk = process_internal(chunk);

    Ok(to_u64(chunk))
}

unsafe fn process_big(
    mut chunk: __m128i,
    check: __m128i,
    check2: __m128i,
    s: &str,
) -> Result<u64, AtoiSimdError> {
    chunk = to_numbers(chunk);
    chunk = mult_10(chunk);

    checker(check, check2, s)?;

    chunk = process_internal(chunk);

    // let chunk = to_u64(chunk);
    // ((chunk & 0xFFFF_FFFF) * 100_000_000) + (chunk >> 32)

    let arr = to_u32x4(chunk);
    Ok((arr[0] as u64 * 100_000_000) + (arr[1] as u64))
}

/// Parses string of *only* digits. String length must be 1..=16.
/// Uses SSE4.1 intrinsics
pub fn parse(s: &str) -> Result<u64, AtoiSimdError> {
    unsafe {
        match s.len() {
            1 => {
                let val = *s.as_bytes().first().unwrap();
                if val > 0x39 || val < 0x30 {
                    return Err(AtoiSimdError::Invalid(s));
                }
                Ok((val & 0xF) as u64)
            }
            2 => {
                let mut chunk = read(s);

                let cmp = _mm_set_epi8(
                    HIGH, HIGH, HIGH, HIGH, HIGH, HIGH, HIGH, HIGH, HIGH, HIGH, HIGH, HIGH, HIGH,
                    HIGH, CHAR_MAX, CHAR_MAX,
                );
                let check_high = process_gt(chunk, cmp);
                let cmp = _mm_set_epi8(
                    LOW, LOW, LOW, LOW, LOW, LOW, LOW, LOW, LOW, LOW, LOW, LOW, LOW, LOW, CHAR_MIN,
                    CHAR_MIN,
                );
                let check_low = process_gt(cmp, chunk);

                chunk = process_and(chunk, 0xF0F);
                chunk = mult_10(chunk);

                checker(check_high, check_low, s)?;

                Ok(to_u64(chunk))
                // std::mem::transmute::<__m128i, [u16; 8]>(chunk)[7] as u64 // same performance
            }
            3 => {
                let mut chunk = read(s);

                let cmp = _mm_set_epi8(
                    HIGH, HIGH, HIGH, HIGH, HIGH, HIGH, HIGH, HIGH, HIGH, HIGH, HIGH, HIGH, HIGH,
                    CHAR_MAX, CHAR_MAX, CHAR_MAX,
                );
                let check_high = process_gt(chunk, cmp);
                let cmp = _mm_set_epi8(
                    LOW, LOW, LOW, LOW, LOW, LOW, LOW, LOW, LOW, LOW, LOW, LOW, LOW, CHAR_MIN,
                    CHAR_MIN, CHAR_MIN,
                );
                let check_low = process_gt(cmp, chunk);

                chunk = _mm_bslli_si128(chunk, 1);
                process_small(chunk, check_high, check_low, s)
            }
            4 => {
                let chunk = read(s);

                let cmp = _mm_set_epi8(
                    HIGH, HIGH, HIGH, HIGH, HIGH, HIGH, HIGH, HIGH, HIGH, HIGH, HIGH, HIGH,
                    CHAR_MAX, CHAR_MAX, CHAR_MAX, CHAR_MAX,
                );
                let check_high = process_gt(chunk, cmp);
                let cmp = _mm_set_epi8(
                    LOW, LOW, LOW, LOW, LOW, LOW, LOW, LOW, LOW, LOW, LOW, LOW, CHAR_MIN, CHAR_MIN,
                    CHAR_MIN, CHAR_MIN,
                );
                let check_low = process_gt(cmp, chunk);

                process_small(chunk, check_high, check_low, s)
            }
            5 => {
                let mut chunk = read(s);

                let cmp = _mm_set_epi8(
                    HIGH, HIGH, HIGH, HIGH, HIGH, HIGH, HIGH, HIGH, HIGH, HIGH, HIGH, CHAR_MAX,
                    CHAR_MAX, CHAR_MAX, CHAR_MAX, CHAR_MAX,
                );
                let check_high = process_gt(chunk, cmp);
                let cmp = _mm_set_epi8(
                    LOW, LOW, LOW, LOW, LOW, LOW, LOW, LOW, LOW, LOW, LOW, CHAR_MIN, CHAR_MIN,
                    CHAR_MIN, CHAR_MIN, CHAR_MIN,
                );
                let check_low = process_gt(cmp, chunk);

                chunk = _mm_bslli_si128(chunk, 3);
                process_medium(chunk, check_high, check_low, s)
            }
            6 => {
                let mut chunk = read(s);

                let cmp = _mm_set_epi8(
                    HIGH, HIGH, HIGH, HIGH, HIGH, HIGH, HIGH, HIGH, HIGH, HIGH, CHAR_MAX, CHAR_MAX,
                    CHAR_MAX, CHAR_MAX, CHAR_MAX, CHAR_MAX,
                );
                let check_high = process_gt(chunk, cmp);
                let cmp = _mm_set_epi8(
                    LOW, LOW, LOW, LOW, LOW, LOW, LOW, LOW, LOW, LOW, CHAR_MIN, CHAR_MIN, CHAR_MIN,
                    CHAR_MIN, CHAR_MIN, CHAR_MIN,
                );
                let check_low = process_gt(cmp, chunk);

                chunk = _mm_bslli_si128(chunk, 2);
                process_medium(chunk, check_high, check_low, s)
            }
            7 => {
                let mut chunk = read(s);

                let cmp = _mm_set_epi8(
                    HIGH, HIGH, HIGH, HIGH, HIGH, HIGH, HIGH, HIGH, HIGH, CHAR_MAX, CHAR_MAX,
                    CHAR_MAX, CHAR_MAX, CHAR_MAX, CHAR_MAX, CHAR_MAX,
                );
                let check_high = process_gt(chunk, cmp);
                let cmp = _mm_set_epi8(
                    LOW, LOW, LOW, LOW, LOW, LOW, LOW, LOW, LOW, CHAR_MIN, CHAR_MIN, CHAR_MIN,
                    CHAR_MIN, CHAR_MIN, CHAR_MIN, CHAR_MIN,
                );
                let check_low = process_gt(cmp, chunk);

                chunk = _mm_bslli_si128(chunk, 1);
                process_medium(chunk, check_high, check_low, s)
            }
            8 => {
                let chunk = read(s);

                let cmp = _mm_set_epi8(
                    HIGH, HIGH, HIGH, HIGH, HIGH, HIGH, HIGH, HIGH, CHAR_MAX, CHAR_MAX, CHAR_MAX,
                    CHAR_MAX, CHAR_MAX, CHAR_MAX, CHAR_MAX, CHAR_MAX,
                );
                let check_high = process_gt(chunk, cmp);
                let cmp = _mm_set_epi8(
                    LOW, LOW, LOW, LOW, LOW, LOW, LOW, LOW, CHAR_MIN, CHAR_MIN, CHAR_MIN, CHAR_MIN,
                    CHAR_MIN, CHAR_MIN, CHAR_MIN, CHAR_MIN,
                );
                let check_low = process_gt(cmp, chunk);

                process_medium(chunk, check_high, check_low, s)
            }
            9 => {
                let mut chunk = read(s);

                let cmp = _mm_set_epi8(
                    HIGH, HIGH, HIGH, HIGH, HIGH, HIGH, HIGH, CHAR_MAX, CHAR_MAX, CHAR_MAX,
                    CHAR_MAX, CHAR_MAX, CHAR_MAX, CHAR_MAX, CHAR_MAX, CHAR_MAX,
                );
                let check_high = process_gt(chunk, cmp);
                let cmp = _mm_set_epi8(
                    LOW, LOW, LOW, LOW, LOW, LOW, LOW, CHAR_MIN, CHAR_MIN, CHAR_MIN, CHAR_MIN,
                    CHAR_MIN, CHAR_MIN, CHAR_MIN, CHAR_MIN, CHAR_MIN,
                );
                let check_low = process_gt(cmp, chunk);

                chunk = _mm_bslli_si128(chunk, 7);
                process_big(chunk, check_high, check_low, s)
            }
            10 => {
                let mut chunk = read(s);

                let cmp = _mm_set_epi8(
                    HIGH, HIGH, HIGH, HIGH, HIGH, HIGH, CHAR_MAX, CHAR_MAX, CHAR_MAX, CHAR_MAX,
                    CHAR_MAX, CHAR_MAX, CHAR_MAX, CHAR_MAX, CHAR_MAX, CHAR_MAX,
                );
                let check_high = process_gt(chunk, cmp);
                let cmp = _mm_set_epi8(
                    LOW, LOW, LOW, LOW, LOW, LOW, CHAR_MIN, CHAR_MIN, CHAR_MIN, CHAR_MIN, CHAR_MIN,
                    CHAR_MIN, CHAR_MIN, CHAR_MIN, CHAR_MIN, CHAR_MIN,
                );
                let check_low = process_gt(cmp, chunk);

                chunk = _mm_bslli_si128(chunk, 6);
                process_big(chunk, check_high, check_low, s)
            }
            11 => {
                let mut chunk = read(s);

                let cmp = _mm_set_epi8(
                    HIGH, HIGH, HIGH, HIGH, HIGH, CHAR_MAX, CHAR_MAX, CHAR_MAX, CHAR_MAX, CHAR_MAX,
                    CHAR_MAX, CHAR_MAX, CHAR_MAX, CHAR_MAX, CHAR_MAX, CHAR_MAX,
                );
                let check_high = process_gt(chunk, cmp);
                let cmp = _mm_set_epi8(
                    LOW, LOW, LOW, LOW, LOW, CHAR_MIN, CHAR_MIN, CHAR_MIN, CHAR_MIN, CHAR_MIN,
                    CHAR_MIN, CHAR_MIN, CHAR_MIN, CHAR_MIN, CHAR_MIN, CHAR_MIN,
                );
                let check_low = process_gt(cmp, chunk);

                chunk = _mm_bslli_si128(chunk, 5);
                process_big(chunk, check_high, check_low, s)
            }
            12 => {
                let mut chunk = read(s);

                let cmp = _mm_set_epi8(
                    HIGH, HIGH, HIGH, HIGH, CHAR_MAX, CHAR_MAX, CHAR_MAX, CHAR_MAX, CHAR_MAX,
                    CHAR_MAX, CHAR_MAX, CHAR_MAX, CHAR_MAX, CHAR_MAX, CHAR_MAX, CHAR_MAX,
                );
                let check_high = process_gt(chunk, cmp);
                let cmp = _mm_set_epi8(
                    LOW, LOW, LOW, LOW, CHAR_MIN, CHAR_MIN, CHAR_MIN, CHAR_MIN, CHAR_MIN, CHAR_MIN,
                    CHAR_MIN, CHAR_MIN, CHAR_MIN, CHAR_MIN, CHAR_MIN, CHAR_MIN,
                );
                let check_low = process_gt(cmp, chunk);

                chunk = _mm_bslli_si128(chunk, 4);
                process_big(chunk, check_high, check_low, s)
            }
            13 => {
                let mut chunk = read(s);

                let cmp = _mm_set_epi8(
                    HIGH, HIGH, HIGH, CHAR_MAX, CHAR_MAX, CHAR_MAX, CHAR_MAX, CHAR_MAX, CHAR_MAX,
                    CHAR_MAX, CHAR_MAX, CHAR_MAX, CHAR_MAX, CHAR_MAX, CHAR_MAX, CHAR_MAX,
                );
                let check_high = process_gt(chunk, cmp);
                let cmp = _mm_set_epi8(
                    LOW, LOW, LOW, CHAR_MIN, CHAR_MIN, CHAR_MIN, CHAR_MIN, CHAR_MIN, CHAR_MIN,
                    CHAR_MIN, CHAR_MIN, CHAR_MIN, CHAR_MIN, CHAR_MIN, CHAR_MIN, CHAR_MIN,
                );
                let check_low = process_gt(cmp, chunk);

                chunk = _mm_bslli_si128(chunk, 3);
                process_big(chunk, check_high, check_low, s)
            }
            14 => {
                let mut chunk = read(s);

                let cmp = _mm_set_epi8(
                    HIGH, HIGH, CHAR_MAX, CHAR_MAX, CHAR_MAX, CHAR_MAX, CHAR_MAX, CHAR_MAX,
                    CHAR_MAX, CHAR_MAX, CHAR_MAX, CHAR_MAX, CHAR_MAX, CHAR_MAX, CHAR_MAX, CHAR_MAX,
                );
                let check_high = process_gt(chunk, cmp);
                let cmp = _mm_set_epi8(
                    LOW, LOW, CHAR_MIN, CHAR_MIN, CHAR_MIN, CHAR_MIN, CHAR_MIN, CHAR_MIN, CHAR_MIN,
                    CHAR_MIN, CHAR_MIN, CHAR_MIN, CHAR_MIN, CHAR_MIN, CHAR_MIN, CHAR_MIN,
                );
                let check_low = process_gt(cmp, chunk);

                chunk = _mm_bslli_si128(chunk, 2);
                process_big(chunk, check_high, check_low, s)
            }
            15 => {
                let mut chunk = read(s);

                let cmp = _mm_set_epi8(
                    HIGH, CHAR_MAX, CHAR_MAX, CHAR_MAX, CHAR_MAX, CHAR_MAX, CHAR_MAX, CHAR_MAX,
                    CHAR_MAX, CHAR_MAX, CHAR_MAX, CHAR_MAX, CHAR_MAX, CHAR_MAX, CHAR_MAX, CHAR_MAX,
                );
                let check_high = process_gt(chunk, cmp);
                let cmp = _mm_set_epi8(
                    LOW, CHAR_MIN, CHAR_MIN, CHAR_MIN, CHAR_MIN, CHAR_MIN, CHAR_MIN, CHAR_MIN,
                    CHAR_MIN, CHAR_MIN, CHAR_MIN, CHAR_MIN, CHAR_MIN, CHAR_MIN, CHAR_MIN, CHAR_MIN,
                );
                let check_low = process_gt(cmp, chunk);

                chunk = _mm_bslli_si128(chunk, 1);
                process_big(chunk, check_high, check_low, s)
            }
            16 => {
                let chunk = read(s);

                let cmp = _mm_set_epi8(
                    CHAR_MAX, CHAR_MAX, CHAR_MAX, CHAR_MAX, CHAR_MAX, CHAR_MAX, CHAR_MAX, CHAR_MAX,
                    CHAR_MAX, CHAR_MAX, CHAR_MAX, CHAR_MAX, CHAR_MAX, CHAR_MAX, CHAR_MAX, CHAR_MAX,
                );
                let check_high = process_gt(chunk, cmp);
                let cmp = _mm_set_epi8(
                    CHAR_MIN, CHAR_MIN, CHAR_MIN, CHAR_MIN, CHAR_MIN, CHAR_MIN, CHAR_MIN, CHAR_MIN,
                    CHAR_MIN, CHAR_MIN, CHAR_MIN, CHAR_MIN, CHAR_MIN, CHAR_MIN, CHAR_MIN, CHAR_MIN,
                );
                let check_low = process_gt(cmp, chunk);

                process_big(chunk, check_high, check_low, s)
            }
            s_len => return Err(AtoiSimdError::Size(s_len)),
        }
    }
}

/// Uses AVX/AVX2 intrinsics
unsafe fn process_avx(
    mut chunk: __m256i,
    check: __m256i,
    check2: __m256i,
    s: &str,
) -> Result<u128, AtoiSimdError> {
    // to numbers
    chunk = _mm256_and_si256(chunk, _mm256_set1_epi8(0xF));

    let mut mult = _mm256_set_epi8(
        1, 10, 1, 10, 1, 10, 1, 10, 1, 10, 1, 10, 1, 10, 1, 10, 1, 10, 1, 10, 1, 10, 1, 10, 1, 10,
        1, 10, 1, 10, 1, 10,
    );
    // mult 10
    chunk = _mm256_maddubs_epi16(chunk, mult);

    checker_avx(check, check2, s)?;

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

    Ok(arr[0] * 10_000_000_000_000_000 + arr[1])
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
        let check_high: __m256i;
        let check_low: __m256i;

        match s.len() {
            17 => {
                let cmp = _mm256_set_epi8(
                    HIGH, HIGH, HIGH, HIGH, HIGH, HIGH, HIGH, HIGH, HIGH, HIGH, HIGH, HIGH, HIGH,
                    HIGH, HIGH, CHAR_MAX, CHAR_MAX, CHAR_MAX, CHAR_MAX, CHAR_MAX, CHAR_MAX,
                    CHAR_MAX, CHAR_MAX, CHAR_MAX, CHAR_MAX, CHAR_MAX, CHAR_MAX, CHAR_MAX, CHAR_MAX,
                    CHAR_MAX, CHAR_MAX, CHAR_MAX,
                );
                check_high = process_avx_gt(chunk, cmp);
                let cmp = _mm256_set_epi8(
                    LOW, LOW, LOW, LOW, LOW, LOW, LOW, LOW, LOW, LOW, LOW, LOW, LOW, LOW, LOW,
                    CHAR_MIN, CHAR_MIN, CHAR_MIN, CHAR_MIN, CHAR_MIN, CHAR_MIN, CHAR_MIN, CHAR_MIN,
                    CHAR_MIN, CHAR_MIN, CHAR_MIN, CHAR_MIN, CHAR_MIN, CHAR_MIN, CHAR_MIN, CHAR_MIN,
                    CHAR_MIN,
                );
                check_low = process_avx_gt(cmp, chunk);

                let mut mult = process_avx_permute2x128(chunk);

                mult = _mm256_bsrli_epi128(mult, 1);
                chunk = _mm256_bslli_epi128(chunk, 15);
                chunk = process_avx_or(chunk, mult);
            }
            18 => {
                let cmp = _mm256_set_epi8(
                    HIGH, HIGH, HIGH, HIGH, HIGH, HIGH, HIGH, HIGH, HIGH, HIGH, HIGH, HIGH, HIGH,
                    HIGH, CHAR_MAX, CHAR_MAX, CHAR_MAX, CHAR_MAX, CHAR_MAX, CHAR_MAX, CHAR_MAX,
                    CHAR_MAX, CHAR_MAX, CHAR_MAX, CHAR_MAX, CHAR_MAX, CHAR_MAX, CHAR_MAX, CHAR_MAX,
                    CHAR_MAX, CHAR_MAX, CHAR_MAX,
                );
                check_high = process_avx_gt(chunk, cmp);
                let cmp = _mm256_set_epi8(
                    LOW, LOW, LOW, LOW, LOW, LOW, LOW, LOW, LOW, LOW, LOW, LOW, LOW, LOW, CHAR_MIN,
                    CHAR_MIN, CHAR_MIN, CHAR_MIN, CHAR_MIN, CHAR_MIN, CHAR_MIN, CHAR_MIN, CHAR_MIN,
                    CHAR_MIN, CHAR_MIN, CHAR_MIN, CHAR_MIN, CHAR_MIN, CHAR_MIN, CHAR_MIN, CHAR_MIN,
                    CHAR_MIN,
                );
                check_low = process_avx_gt(cmp, chunk);

                let mut mult = process_avx_permute2x128(chunk);
                mult = _mm256_bsrli_epi128(mult, 2);
                chunk = _mm256_bslli_epi128(chunk, 14);
                chunk = process_avx_or(chunk, mult);
            }
            19 => {
                let cmp = _mm256_set_epi8(
                    HIGH, HIGH, HIGH, HIGH, HIGH, HIGH, HIGH, HIGH, HIGH, HIGH, HIGH, HIGH, HIGH,
                    CHAR_MAX, CHAR_MAX, CHAR_MAX, CHAR_MAX, CHAR_MAX, CHAR_MAX, CHAR_MAX, CHAR_MAX,
                    CHAR_MAX, CHAR_MAX, CHAR_MAX, CHAR_MAX, CHAR_MAX, CHAR_MAX, CHAR_MAX, CHAR_MAX,
                    CHAR_MAX, CHAR_MAX, CHAR_MAX,
                );
                check_high = process_avx_gt(chunk, cmp);
                let cmp = _mm256_set_epi8(
                    LOW, LOW, LOW, LOW, LOW, LOW, LOW, LOW, LOW, LOW, LOW, LOW, LOW, CHAR_MIN,
                    CHAR_MIN, CHAR_MIN, CHAR_MIN, CHAR_MIN, CHAR_MIN, CHAR_MIN, CHAR_MIN, CHAR_MIN,
                    CHAR_MIN, CHAR_MIN, CHAR_MIN, CHAR_MIN, CHAR_MIN, CHAR_MIN, CHAR_MIN, CHAR_MIN,
                    CHAR_MIN, CHAR_MIN,
                );
                check_low = process_avx_gt(cmp, chunk);

                let mut mult = process_avx_permute2x128(chunk);
                mult = _mm256_bsrli_epi128(mult, 3);
                chunk = _mm256_bslli_epi128(chunk, 13);
                chunk = process_avx_or(chunk, mult);
            }
            20 => {
                let cmp = _mm256_set_epi8(
                    HIGH, HIGH, HIGH, HIGH, HIGH, HIGH, HIGH, HIGH, HIGH, HIGH, HIGH, HIGH,
                    CHAR_MAX, CHAR_MAX, CHAR_MAX, CHAR_MAX, CHAR_MAX, CHAR_MAX, CHAR_MAX, CHAR_MAX,
                    CHAR_MAX, CHAR_MAX, CHAR_MAX, CHAR_MAX, CHAR_MAX, CHAR_MAX, CHAR_MAX, CHAR_MAX,
                    CHAR_MAX, CHAR_MAX, CHAR_MAX, CHAR_MAX,
                );
                check_high = process_avx_gt(chunk, cmp);
                let cmp = _mm256_set_epi8(
                    LOW, LOW, LOW, LOW, LOW, LOW, LOW, LOW, LOW, LOW, LOW, LOW, CHAR_MIN, CHAR_MIN,
                    CHAR_MIN, CHAR_MIN, CHAR_MIN, CHAR_MIN, CHAR_MIN, CHAR_MIN, CHAR_MIN, CHAR_MIN,
                    CHAR_MIN, CHAR_MIN, CHAR_MIN, CHAR_MIN, CHAR_MIN, CHAR_MIN, CHAR_MIN, CHAR_MIN,
                    CHAR_MIN, CHAR_MIN,
                );
                check_low = process_avx_gt(cmp, chunk);

                // maybe can be optimized even further with _mm256_permutevar8x32_epi32
                let mut mult = process_avx_permute2x128(chunk);
                mult = _mm256_bsrli_epi128(mult, 4);
                chunk = _mm256_bslli_epi128(chunk, 12);
                chunk = process_avx_or(chunk, mult);
            }
            21 => {
                let cmp = _mm256_set_epi8(
                    HIGH, HIGH, HIGH, HIGH, HIGH, HIGH, HIGH, HIGH, HIGH, HIGH, HIGH, CHAR_MAX,
                    CHAR_MAX, CHAR_MAX, CHAR_MAX, CHAR_MAX, CHAR_MAX, CHAR_MAX, CHAR_MAX, CHAR_MAX,
                    CHAR_MAX, CHAR_MAX, CHAR_MAX, CHAR_MAX, CHAR_MAX, CHAR_MAX, CHAR_MAX, CHAR_MAX,
                    CHAR_MAX, CHAR_MAX, CHAR_MAX, CHAR_MAX,
                );
                check_high = process_avx_gt(chunk, cmp);
                let cmp = _mm256_set_epi8(
                    LOW, LOW, LOW, LOW, LOW, LOW, LOW, LOW, LOW, LOW, LOW, CHAR_MIN, CHAR_MIN,
                    CHAR_MIN, CHAR_MIN, CHAR_MIN, CHAR_MIN, CHAR_MIN, CHAR_MIN, CHAR_MIN, CHAR_MIN,
                    CHAR_MIN, CHAR_MIN, CHAR_MIN, CHAR_MIN, CHAR_MIN, CHAR_MIN, CHAR_MIN, CHAR_MIN,
                    CHAR_MIN, CHAR_MIN, CHAR_MIN,
                );
                check_low = process_avx_gt(cmp, chunk);

                let mut mult = process_avx_permute2x128(chunk);
                mult = _mm256_bsrli_epi128(mult, 5);
                chunk = _mm256_bslli_epi128(chunk, 11);
                chunk = process_avx_or(chunk, mult);
            }
            22 => {
                let cmp = _mm256_set_epi8(
                    HIGH, HIGH, HIGH, HIGH, HIGH, HIGH, HIGH, HIGH, HIGH, HIGH, CHAR_MAX, CHAR_MAX,
                    CHAR_MAX, CHAR_MAX, CHAR_MAX, CHAR_MAX, CHAR_MAX, CHAR_MAX, CHAR_MAX, CHAR_MAX,
                    CHAR_MAX, CHAR_MAX, CHAR_MAX, CHAR_MAX, CHAR_MAX, CHAR_MAX, CHAR_MAX, CHAR_MAX,
                    CHAR_MAX, CHAR_MAX, CHAR_MAX, CHAR_MAX,
                );
                check_high = process_avx_gt(chunk, cmp);
                let cmp = _mm256_set_epi8(
                    LOW, LOW, LOW, LOW, LOW, LOW, LOW, LOW, LOW, LOW, CHAR_MIN, CHAR_MIN, CHAR_MIN,
                    CHAR_MIN, CHAR_MIN, CHAR_MIN, CHAR_MIN, CHAR_MIN, CHAR_MIN, CHAR_MIN, CHAR_MIN,
                    CHAR_MIN, CHAR_MIN, CHAR_MIN, CHAR_MIN, CHAR_MIN, CHAR_MIN, CHAR_MIN, CHAR_MIN,
                    CHAR_MIN, CHAR_MIN, CHAR_MIN,
                );
                check_low = process_avx_gt(cmp, chunk);

                let mut mult = process_avx_permute2x128(chunk);
                mult = _mm256_bsrli_epi128(mult, 6);
                chunk = _mm256_bslli_epi128(chunk, 10);
                chunk = process_avx_or(chunk, mult);
            }
            23 => {
                let cmp = _mm256_set_epi8(
                    HIGH, HIGH, HIGH, HIGH, HIGH, HIGH, HIGH, HIGH, HIGH, CHAR_MAX, CHAR_MAX,
                    CHAR_MAX, CHAR_MAX, CHAR_MAX, CHAR_MAX, CHAR_MAX, CHAR_MAX, CHAR_MAX, CHAR_MAX,
                    CHAR_MAX, CHAR_MAX, CHAR_MAX, CHAR_MAX, CHAR_MAX, CHAR_MAX, CHAR_MAX, CHAR_MAX,
                    CHAR_MAX, CHAR_MAX, CHAR_MAX, CHAR_MAX, CHAR_MAX,
                );
                check_high = process_avx_gt(chunk, cmp);
                let cmp = _mm256_set_epi8(
                    LOW, LOW, LOW, LOW, LOW, LOW, LOW, LOW, LOW, CHAR_MIN, CHAR_MIN, CHAR_MIN,
                    CHAR_MIN, CHAR_MIN, CHAR_MIN, CHAR_MIN, CHAR_MIN, CHAR_MIN, CHAR_MIN, CHAR_MIN,
                    CHAR_MIN, CHAR_MIN, CHAR_MIN, CHAR_MIN, CHAR_MIN, CHAR_MIN, CHAR_MIN, CHAR_MIN,
                    CHAR_MIN, CHAR_MIN, CHAR_MIN, CHAR_MIN,
                );
                check_low = process_avx_gt(cmp, chunk);

                let mut mult = process_avx_permute2x128(chunk);
                mult = _mm256_bsrli_epi128(mult, 7);
                chunk = _mm256_bslli_epi128(chunk, 9);
                chunk = process_avx_or(chunk, mult);
            }
            24 => {
                let cmp = _mm256_set_epi8(
                    HIGH, HIGH, HIGH, HIGH, HIGH, HIGH, HIGH, HIGH, CHAR_MAX, CHAR_MAX, CHAR_MAX,
                    CHAR_MAX, CHAR_MAX, CHAR_MAX, CHAR_MAX, CHAR_MAX, CHAR_MAX, CHAR_MAX, CHAR_MAX,
                    CHAR_MAX, CHAR_MAX, CHAR_MAX, CHAR_MAX, CHAR_MAX, CHAR_MAX, CHAR_MAX, CHAR_MAX,
                    CHAR_MAX, CHAR_MAX, CHAR_MAX, CHAR_MAX, CHAR_MAX,
                );
                check_high = process_avx_gt(chunk, cmp);
                let cmp = _mm256_set_epi8(
                    LOW, LOW, LOW, LOW, LOW, LOW, LOW, LOW, CHAR_MIN, CHAR_MIN, CHAR_MIN, CHAR_MIN,
                    CHAR_MIN, CHAR_MIN, CHAR_MIN, CHAR_MIN, CHAR_MIN, CHAR_MIN, CHAR_MIN, CHAR_MIN,
                    CHAR_MIN, CHAR_MIN, CHAR_MIN, CHAR_MIN, CHAR_MIN, CHAR_MIN, CHAR_MIN, CHAR_MIN,
                    CHAR_MIN, CHAR_MIN, CHAR_MIN, CHAR_MIN,
                );
                check_low = process_avx_gt(cmp, chunk);

                // maybe can be optimized even further with _mm256_permute4x64_epi64
                let mut mult = process_avx_permute2x128(chunk);
                mult = _mm256_bsrli_epi128(mult, 8);
                chunk = _mm256_bslli_epi128(chunk, 8);
                chunk = process_avx_or(chunk, mult);
            }
            25 => {
                let cmp = _mm256_set_epi8(
                    HIGH, HIGH, HIGH, HIGH, HIGH, HIGH, HIGH, CHAR_MAX, CHAR_MAX, CHAR_MAX,
                    CHAR_MAX, CHAR_MAX, CHAR_MAX, CHAR_MAX, CHAR_MAX, CHAR_MAX, CHAR_MAX, CHAR_MAX,
                    CHAR_MAX, CHAR_MAX, CHAR_MAX, CHAR_MAX, CHAR_MAX, CHAR_MAX, CHAR_MAX, CHAR_MAX,
                    CHAR_MAX, CHAR_MAX, CHAR_MAX, CHAR_MAX, CHAR_MAX, CHAR_MAX,
                );
                check_high = process_avx_gt(chunk, cmp);
                let cmp = _mm256_set_epi8(
                    LOW, LOW, LOW, LOW, LOW, LOW, LOW, CHAR_MIN, CHAR_MIN, CHAR_MIN, CHAR_MIN,
                    CHAR_MIN, CHAR_MIN, CHAR_MIN, CHAR_MIN, CHAR_MIN, CHAR_MIN, CHAR_MIN, CHAR_MIN,
                    CHAR_MIN, CHAR_MIN, CHAR_MIN, CHAR_MIN, CHAR_MIN, CHAR_MIN, CHAR_MIN, CHAR_MIN,
                    CHAR_MIN, CHAR_MIN, CHAR_MIN, CHAR_MIN, CHAR_MIN,
                );
                check_low = process_avx_gt(cmp, chunk);

                let mut mult = process_avx_permute2x128(chunk);
                mult = _mm256_bsrli_epi128(mult, 9);
                chunk = _mm256_bslli_epi128(chunk, 7);
                chunk = process_avx_or(chunk, mult);
            }
            26 => {
                let cmp = _mm256_set_epi8(
                    HIGH, HIGH, HIGH, HIGH, HIGH, HIGH, CHAR_MAX, CHAR_MAX, CHAR_MAX, CHAR_MAX,
                    CHAR_MAX, CHAR_MAX, CHAR_MAX, CHAR_MAX, CHAR_MAX, CHAR_MAX, CHAR_MAX, CHAR_MAX,
                    CHAR_MAX, CHAR_MAX, CHAR_MAX, CHAR_MAX, CHAR_MAX, CHAR_MAX, CHAR_MAX, CHAR_MAX,
                    CHAR_MAX, CHAR_MAX, CHAR_MAX, CHAR_MAX, CHAR_MAX, CHAR_MAX,
                );
                check_high = process_avx_gt(chunk, cmp);
                let cmp = _mm256_set_epi8(
                    LOW, LOW, LOW, LOW, LOW, LOW, CHAR_MIN, CHAR_MIN, CHAR_MIN, CHAR_MIN, CHAR_MIN,
                    CHAR_MIN, CHAR_MIN, CHAR_MIN, CHAR_MIN, CHAR_MIN, CHAR_MIN, CHAR_MIN, CHAR_MIN,
                    CHAR_MIN, CHAR_MIN, CHAR_MIN, CHAR_MIN, CHAR_MIN, CHAR_MIN, CHAR_MIN, CHAR_MIN,
                    CHAR_MIN, CHAR_MIN, CHAR_MIN, CHAR_MIN, CHAR_MIN,
                );
                check_low = process_avx_gt(cmp, chunk);

                let mut mult = process_avx_permute2x128(chunk);
                mult = _mm256_bsrli_epi128(mult, 10);
                chunk = _mm256_bslli_epi128(chunk, 6);
                chunk = process_avx_or(chunk, mult);
            }
            27 => {
                let cmp = _mm256_set_epi8(
                    HIGH, HIGH, HIGH, HIGH, HIGH, CHAR_MAX, CHAR_MAX, CHAR_MAX, CHAR_MAX, CHAR_MAX,
                    CHAR_MAX, CHAR_MAX, CHAR_MAX, CHAR_MAX, CHAR_MAX, CHAR_MAX, CHAR_MAX, CHAR_MAX,
                    CHAR_MAX, CHAR_MAX, CHAR_MAX, CHAR_MAX, CHAR_MAX, CHAR_MAX, CHAR_MAX, CHAR_MAX,
                    CHAR_MAX, CHAR_MAX, CHAR_MAX, CHAR_MAX, CHAR_MAX, CHAR_MAX,
                );
                check_high = process_avx_gt(chunk, cmp);
                let cmp = _mm256_set_epi8(
                    LOW, LOW, LOW, LOW, LOW, CHAR_MIN, CHAR_MIN, CHAR_MIN, CHAR_MIN, CHAR_MIN,
                    CHAR_MIN, CHAR_MIN, CHAR_MIN, CHAR_MIN, CHAR_MIN, CHAR_MIN, CHAR_MIN, CHAR_MIN,
                    CHAR_MIN, CHAR_MIN, CHAR_MIN, CHAR_MIN, CHAR_MIN, CHAR_MIN, CHAR_MIN, CHAR_MIN,
                    CHAR_MIN, CHAR_MIN, CHAR_MIN, CHAR_MIN, CHAR_MIN, CHAR_MIN,
                );
                check_low = process_avx_gt(cmp, chunk);

                let mut mult = process_avx_permute2x128(chunk);
                mult = _mm256_bsrli_epi128(mult, 11);
                chunk = _mm256_bslli_epi128(chunk, 5);
                chunk = process_avx_or(chunk, mult);
            }
            28 => {
                let cmp = _mm256_set_epi8(
                    HIGH, HIGH, HIGH, HIGH, CHAR_MAX, CHAR_MAX, CHAR_MAX, CHAR_MAX, CHAR_MAX,
                    CHAR_MAX, CHAR_MAX, CHAR_MAX, CHAR_MAX, CHAR_MAX, CHAR_MAX, CHAR_MAX, CHAR_MAX,
                    CHAR_MAX, CHAR_MAX, CHAR_MAX, CHAR_MAX, CHAR_MAX, CHAR_MAX, CHAR_MAX, CHAR_MAX,
                    CHAR_MAX, CHAR_MAX, CHAR_MAX, CHAR_MAX, CHAR_MAX, CHAR_MAX, CHAR_MAX,
                );
                check_high = process_avx_gt(chunk, cmp);
                let cmp = _mm256_set_epi8(
                    LOW, LOW, LOW, LOW, CHAR_MIN, CHAR_MIN, CHAR_MIN, CHAR_MIN, CHAR_MIN, CHAR_MIN,
                    CHAR_MIN, CHAR_MIN, CHAR_MIN, CHAR_MIN, CHAR_MIN, CHAR_MIN, CHAR_MIN, CHAR_MIN,
                    CHAR_MIN, CHAR_MIN, CHAR_MIN, CHAR_MIN, CHAR_MIN, CHAR_MIN, CHAR_MIN, CHAR_MIN,
                    CHAR_MIN, CHAR_MIN, CHAR_MIN, CHAR_MIN, CHAR_MIN, CHAR_MIN,
                );
                check_low = process_avx_gt(cmp, chunk);

                // maybe can be optimized even further with _mm256_permutevar8x32_epi32
                // let mult = _mm256_set_epi32(6, 5, 4, 3, 2, 1, 0, 0);
                // chunk = _mm256_permutevar8x32_epi32(chunk, mult);
                let mut mult = process_avx_permute2x128(chunk);
                mult = _mm256_bsrli_epi128(mult, 12);
                chunk = _mm256_bslli_epi128(chunk, 4);
                chunk = process_avx_or(chunk, mult);
            }
            29 => {
                let cmp = _mm256_set_epi8(
                    HIGH, HIGH, HIGH, CHAR_MAX, CHAR_MAX, CHAR_MAX, CHAR_MAX, CHAR_MAX, CHAR_MAX,
                    CHAR_MAX, CHAR_MAX, CHAR_MAX, CHAR_MAX, CHAR_MAX, CHAR_MAX, CHAR_MAX, CHAR_MAX,
                    CHAR_MAX, CHAR_MAX, CHAR_MAX, CHAR_MAX, CHAR_MAX, CHAR_MAX, CHAR_MAX, CHAR_MAX,
                    CHAR_MAX, CHAR_MAX, CHAR_MAX, CHAR_MAX, CHAR_MAX, CHAR_MAX, CHAR_MAX,
                );
                check_high = process_avx_gt(chunk, cmp);
                let cmp = _mm256_set_epi8(
                    LOW, LOW, LOW, CHAR_MIN, CHAR_MIN, CHAR_MIN, CHAR_MIN, CHAR_MIN, CHAR_MIN,
                    CHAR_MIN, CHAR_MIN, CHAR_MIN, CHAR_MIN, CHAR_MIN, CHAR_MIN, CHAR_MIN, CHAR_MIN,
                    CHAR_MIN, CHAR_MIN, CHAR_MIN, CHAR_MIN, CHAR_MIN, CHAR_MIN, CHAR_MIN, CHAR_MIN,
                    CHAR_MIN, CHAR_MIN, CHAR_MIN, CHAR_MIN, CHAR_MIN, CHAR_MIN, CHAR_MIN,
                );
                check_low = process_avx_gt(cmp, chunk);

                let mut mult = process_avx_permute2x128(chunk);
                mult = _mm256_bsrli_epi128(mult, 13);
                chunk = _mm256_bslli_epi128(chunk, 3);
                chunk = process_avx_or(chunk, mult);
            }
            30 => {
                let cmp = _mm256_set_epi8(
                    HIGH, HIGH, CHAR_MAX, CHAR_MAX, CHAR_MAX, CHAR_MAX, CHAR_MAX, CHAR_MAX,
                    CHAR_MAX, CHAR_MAX, CHAR_MAX, CHAR_MAX, CHAR_MAX, CHAR_MAX, CHAR_MAX, CHAR_MAX,
                    CHAR_MAX, CHAR_MAX, CHAR_MAX, CHAR_MAX, CHAR_MAX, CHAR_MAX, CHAR_MAX, CHAR_MAX,
                    CHAR_MAX, CHAR_MAX, CHAR_MAX, CHAR_MAX, CHAR_MAX, CHAR_MAX, CHAR_MAX, CHAR_MAX,
                );
                check_high = process_avx_gt(chunk, cmp);
                let cmp = _mm256_set_epi8(
                    LOW, LOW, CHAR_MIN, CHAR_MIN, CHAR_MIN, CHAR_MIN, CHAR_MIN, CHAR_MIN, CHAR_MIN,
                    CHAR_MIN, CHAR_MIN, CHAR_MIN, CHAR_MIN, CHAR_MIN, CHAR_MIN, CHAR_MIN, CHAR_MIN,
                    CHAR_MIN, CHAR_MIN, CHAR_MIN, CHAR_MIN, CHAR_MIN, CHAR_MIN, CHAR_MIN, CHAR_MIN,
                    CHAR_MIN, CHAR_MIN, CHAR_MIN, CHAR_MIN, CHAR_MIN, CHAR_MIN, CHAR_MIN,
                );
                check_low = process_avx_gt(cmp, chunk);

                let mut mult = process_avx_permute2x128(chunk);
                mult = _mm256_bsrli_epi128(mult, 14);
                chunk = _mm256_bslli_epi128(chunk, 2);
                chunk = process_avx_or(chunk, mult);
            }
            31 => {
                let cmp = _mm256_set_epi8(
                    HIGH, CHAR_MAX, CHAR_MAX, CHAR_MAX, CHAR_MAX, CHAR_MAX, CHAR_MAX, CHAR_MAX,
                    CHAR_MAX, CHAR_MAX, CHAR_MAX, CHAR_MAX, CHAR_MAX, CHAR_MAX, CHAR_MAX, CHAR_MAX,
                    CHAR_MAX, CHAR_MAX, CHAR_MAX, CHAR_MAX, CHAR_MAX, CHAR_MAX, CHAR_MAX, CHAR_MAX,
                    CHAR_MAX, CHAR_MAX, CHAR_MAX, CHAR_MAX, CHAR_MAX, CHAR_MAX, CHAR_MAX, CHAR_MAX,
                );
                check_high = process_avx_gt(chunk, cmp);
                let cmp = _mm256_set_epi8(
                    LOW, CHAR_MIN, CHAR_MIN, CHAR_MIN, CHAR_MIN, CHAR_MIN, CHAR_MIN, CHAR_MIN,
                    CHAR_MIN, CHAR_MIN, CHAR_MIN, CHAR_MIN, CHAR_MIN, CHAR_MIN, CHAR_MIN, CHAR_MIN,
                    CHAR_MIN, CHAR_MIN, CHAR_MIN, CHAR_MIN, CHAR_MIN, CHAR_MIN, CHAR_MIN, CHAR_MIN,
                    CHAR_MIN, CHAR_MIN, CHAR_MIN, CHAR_MIN, CHAR_MIN, CHAR_MIN, CHAR_MIN, CHAR_MIN,
                );
                check_low = process_avx_gt(cmp, chunk);

                let mut mult = process_avx_permute2x128(chunk);
                // somehow it'll be bugged, if you move permute after it
                mult = _mm256_bsrli_epi128(mult, 15);
                chunk = _mm256_bslli_epi128(chunk, 1);
                chunk = process_avx_or(chunk, mult);
            }
            32 => {
                let cmp = _mm256_set_epi8(
                    CHAR_MAX, CHAR_MAX, CHAR_MAX, CHAR_MAX, CHAR_MAX, CHAR_MAX, CHAR_MAX, CHAR_MAX,
                    CHAR_MAX, CHAR_MAX, CHAR_MAX, CHAR_MAX, CHAR_MAX, CHAR_MAX, CHAR_MAX, CHAR_MAX,
                    CHAR_MAX, CHAR_MAX, CHAR_MAX, CHAR_MAX, CHAR_MAX, CHAR_MAX, CHAR_MAX, CHAR_MAX,
                    CHAR_MAX, CHAR_MAX, CHAR_MAX, CHAR_MAX, CHAR_MAX, CHAR_MAX, CHAR_MAX, CHAR_MAX,
                );
                check_high = process_avx_gt(chunk, cmp);
                let cmp = _mm256_set_epi8(
                    CHAR_MIN, CHAR_MIN, CHAR_MIN, CHAR_MIN, CHAR_MIN, CHAR_MIN, CHAR_MIN, CHAR_MIN,
                    CHAR_MIN, CHAR_MIN, CHAR_MIN, CHAR_MIN, CHAR_MIN, CHAR_MIN, CHAR_MIN, CHAR_MIN,
                    CHAR_MIN, CHAR_MIN, CHAR_MIN, CHAR_MIN, CHAR_MIN, CHAR_MIN, CHAR_MIN, CHAR_MIN,
                    CHAR_MIN, CHAR_MIN, CHAR_MIN, CHAR_MIN, CHAR_MIN, CHAR_MIN, CHAR_MIN, CHAR_MIN,
                );
                check_low = process_avx_gt(cmp, chunk);
            }
            _ => return parse(s).map(|v| v as u128),
        }

        process_avx(chunk, check_high, check_low, s)
    }
}

/// Parses string of digits and first '-' char.
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

/// Parses string of digits and first '-' char.
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
            if parse(&(str.clone() + "!")).is_ok() {
                panic!("error {}", str);
            }
            if parse(&(str.clone() + "a")).is_ok() {
                panic!("error {}", str);
            }
            str.push(i);
            assert_eq!(parse(&str).unwrap(), str.parse::<u64>().unwrap());
        }
        for i in '0'..='6' {
            if parse(&(str.clone() + "!")).is_ok() {
                panic!("error {}", str);
            }
            if parse(&(str.clone() + "a")).is_ok() {
                panic!("error {}", str);
            }
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
            if parse_i64(&(str.clone() + "!")).is_ok() {
                panic!("error {}", str);
            }
            if parse_i64(&(str.clone() + "a")).is_ok() {
                panic!("error {}", str);
            }
            str.push(i);
            str_neg.push(i);
            assert_eq!(parse_i64(&str).unwrap(), str.parse::<i64>().unwrap());
            assert_eq!(
                parse_i64(&str_neg).unwrap(),
                str_neg.parse::<i64>().unwrap()
            );
        }
        for i in '0'..='6' {
            if parse_i64(&(str.clone() + "!")).is_ok() {
                panic!("error {}", str);
            }
            if parse_i64(&(str.clone() + "a")).is_ok() {
                panic!("error {}", str);
            }
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
            if parse_u128(&(str.clone() + "!")).is_ok() {
                panic!("error {}", str);
            }
            if parse_u128(&(str.clone() + "a")).is_ok() {
                panic!("error {}", str);
            }
            str.push(i);
            assert_eq!(parse_u128(&str).unwrap(), str.parse::<u128>().unwrap());
        }
        for _ in 0..2 {
            for i in '0'..='9' {
                if parse_u128(&(str.clone() + "!")).is_ok() {
                    panic!("error {}", str);
                }
                if parse_u128(&(str.clone() + "a")).is_ok() {
                    panic!("error {}", str);
                }
                str.push(i);
                assert_eq!(parse_u128(&str).unwrap(), str.parse::<u128>().unwrap());
            }
        }
        for i in '0'..='2' {
            if parse_u128(&(str.clone() + "!")).is_ok() {
                panic!("error {}", str);
            }
            if parse_u128(&(str.clone() + "a")).is_ok() {
                panic!("error {}", str);
            }
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
            if parse_i128(&(str.clone() + "!")).is_ok() {
                panic!("error {}", str);
            }
            if parse_i128(&(str.clone() + "a")).is_ok() {
                panic!("error {}", str);
            }
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
                if parse_i128(&(str.clone() + "!")).is_ok() {
                    panic!("error {}", str);
                }
                if parse_i128(&(str.clone() + "a")).is_ok() {
                    panic!("error {}", str);
                }
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
            if parse_i128(&(str.clone() + "!")).is_ok() {
                panic!("error {}", str);
            }
            if parse_i128(&(str.clone() + "a")).is_ok() {
                panic!("error {}", str);
            }
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
