//! # Rust fast `&[u8]` to integer parser (x86_64 SIMD, SSE4.1, AVX2)
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
#![allow(clippy::manual_range_contains)]

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
use std::{fmt, str::from_utf8};

#[derive(Debug, Clone, Copy)]
pub enum AtoiSimdError<'a> {
    Empty,
    Size(usize, &'a [u8]),
    Overflow(ParseType, &'a [u8]),
    Invalid(&'a [u8]),
    I64Min,
}

impl fmt::Display for AtoiSimdError<'_> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::Empty => write!(f, "atoi_simd string is empty"),
            Self::Size(len, val) => write!(
                f,
                "atoi_simd wrong size: {} input: {}",
                len,
                from_utf8(val).unwrap_or("not string")
            ),
            Self::Overflow(t, val) => {
                write!(
                    f,
                    "atoi_simd {:?} overflow: {}",
                    t,
                    from_utf8(val).unwrap_or("not string")
                )
            }
            Self::Invalid(val) => {
                write!(
                    f,
                    "atoi_simd invalid, it must contain only digits: {}",
                    from_utf8(val).unwrap_or("not string")
                )
            }
            Self::I64Min => write!(f, "atoi_simd i64::min"), // internal error
        }
    }
}

impl std::error::Error for AtoiSimdError<'_> {}

const HIGH: i8 = 0x7F;
const LOW: i8 = -0x80;
const CHAR_MAX: i8 = 0x39;
const CHAR_MIN: i8 = 0x30;

/// s = "1234567890123456"
unsafe fn read(s: &[u8]) -> __m128i {
    _mm_lddqu_si128(std::mem::transmute_copy(&s))
}

unsafe fn read_avx(s: &[u8]) -> __m256i {
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

#[inline]
unsafe fn to_u64(chunk: __m128i) -> u64 {
    _mm_cvtsi128_si64(chunk) as u64
}

#[inline]
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

unsafe fn checker(check: __m128i, check2: __m128i, s: &[u8]) -> Result<(), AtoiSimdError> {
    let mut chunk = _mm_or_si128(check, check2);

    let mult = _mm_set_epi64x(u64::MAX as i64, u64::MAX as i64);
    // invert all bits
    chunk = _mm_andnot_si128(chunk, mult);

    let res = _mm_test_all_ones(chunk);
    if res == 0 {
        return Err(AtoiSimdError::Invalid(s));
    }

    Ok(())
}

unsafe fn checker_avx(check: __m256i, check2: __m256i, s: &[u8]) -> Result<(), AtoiSimdError> {
    let chunk = _mm256_or_si256(check, check2);

    let mult = _mm256_set_epi64x(
        u64::MAX as i64,
        u64::MAX as i64,
        u64::MAX as i64,
        u64::MAX as i64,
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
    s: &[u8],
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
    s: &[u8],
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
    s: &[u8],
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

/// Parses string of *only* digits
#[target_feature(enable = "sse2,sse3,sse4.1,ssse3,avx,avx2")]
unsafe fn parse_u64(s: &[u8], parse_type: ParseType) -> Result<u64, AtoiSimdError> {
    match s.len() {
        0 => Err(AtoiSimdError::Empty),
        1 => {
            let val = *s.first().unwrap() as u64;
            if val > 0x39 || val < 0x30 {
                return Err(AtoiSimdError::Invalid(s));
            }
            Ok(val & 0xF)
        }
        2 => {
            let mut chunk = read(s);

            let cmp = _mm_set_epi8(
                HIGH, HIGH, HIGH, HIGH, HIGH, HIGH, HIGH, HIGH, HIGH, HIGH, HIGH, HIGH, HIGH, HIGH,
                CHAR_MAX, CHAR_MAX,
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
                HIGH, HIGH, HIGH, HIGH, HIGH, HIGH, HIGH, HIGH, HIGH, HIGH, HIGH, HIGH, CHAR_MAX,
                CHAR_MAX, CHAR_MAX, CHAR_MAX,
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
                HIGH, HIGH, HIGH, HIGH, HIGH, HIGH, HIGH, HIGH, HIGH, CHAR_MAX, CHAR_MAX, CHAR_MAX,
                CHAR_MAX, CHAR_MAX, CHAR_MAX, CHAR_MAX,
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
                HIGH, HIGH, HIGH, HIGH, HIGH, HIGH, HIGH, CHAR_MAX, CHAR_MAX, CHAR_MAX, CHAR_MAX,
                CHAR_MAX, CHAR_MAX, CHAR_MAX, CHAR_MAX, CHAR_MAX,
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
                HIGH, HIGH, HIGH, HIGH, CHAR_MAX, CHAR_MAX, CHAR_MAX, CHAR_MAX, CHAR_MAX, CHAR_MAX,
                CHAR_MAX, CHAR_MAX, CHAR_MAX, CHAR_MAX, CHAR_MAX, CHAR_MAX,
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
                HIGH, HIGH, CHAR_MAX, CHAR_MAX, CHAR_MAX, CHAR_MAX, CHAR_MAX, CHAR_MAX, CHAR_MAX,
                CHAR_MAX, CHAR_MAX, CHAR_MAX, CHAR_MAX, CHAR_MAX, CHAR_MAX, CHAR_MAX,
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
        17 => parse_u128(s).map(|v| v as u64),
        18 => parse_u128(s).map(|v| v as u64),
        19 => {
            let val = parse_u128(s)? as u64;

            match parse_type {
                ParseType::I64Neg => {
                    if val > i64::MIN as u64 {
                        Err(AtoiSimdError::Overflow(parse_type, s))
                    } else if val == i64::MIN as u64 {
                        Err(AtoiSimdError::I64Min)
                    } else {
                        Ok(val)
                    }
                }
                ParseType::I64 => {
                    if val > i64::MAX as u64 {
                        Err(AtoiSimdError::Overflow(parse_type, s))
                    } else {
                        Ok(val)
                    }
                }
                _ => Ok(val),
            }
        }
        20 => {
            if parse_type != ParseType::None {
                return Err(AtoiSimdError::Overflow(parse_type, s));
            }

            let val = parse_u128(s)?;

            if val > u64::MAX as u128 {
                return Err(AtoiSimdError::Overflow(parse_type, s));
            }
            Ok(val as u64)
        }
        s_len => Err(AtoiSimdError::Size(s_len, s)),
        // Do not try to separate this function to three,
        // and chain them with `_ => parse_u32(s).map(|v| v as u64)`,
        // I've tried it, and the performance is not good (even with #[inline]).
    }
}

/// Parses string of *only* digits
/// Uses AVX/AVX2 intrinsics
unsafe fn process_avx(
    mut chunk: __m256i,
    check: __m256i,
    check2: __m256i,
    s: &[u8],
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

    chunk = _mm256_srli_epi64(chunk, 32);
    chunk = _mm256_add_epi64(chunk, mult);

    let arr = std::mem::transmute::<__m256i, [u128; 2]>(chunk);

    Ok(arr[0] * 10_000_000_000_000_000 + arr[1])
}

/*
/// SSE intrinsics for the previous function `process_avx()`
unsafe fn process_u128(s: &[u8]) -> u128 {
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
#[target_feature(enable = "sse2,sse3,sse4.1,ssse3,avx,avx2")]
unsafe fn parse_u128(s: &[u8]) -> Result<u128, AtoiSimdError> {
    let mut chunk: __m256i;
    let check_high: __m256i;
    let check_low: __m256i;

    match s.len() {
        17 => {
            chunk = read_avx(s);
            let cmp = _mm256_set_epi8(
                HIGH, HIGH, HIGH, HIGH, HIGH, HIGH, HIGH, HIGH, HIGH, HIGH, HIGH, HIGH, HIGH, HIGH,
                HIGH, CHAR_MAX, CHAR_MAX, CHAR_MAX, CHAR_MAX, CHAR_MAX, CHAR_MAX, CHAR_MAX,
                CHAR_MAX, CHAR_MAX, CHAR_MAX, CHAR_MAX, CHAR_MAX, CHAR_MAX, CHAR_MAX, CHAR_MAX,
                CHAR_MAX, CHAR_MAX,
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
            chunk = read_avx(s);
            let cmp = _mm256_set_epi8(
                HIGH, HIGH, HIGH, HIGH, HIGH, HIGH, HIGH, HIGH, HIGH, HIGH, HIGH, HIGH, HIGH, HIGH,
                CHAR_MAX, CHAR_MAX, CHAR_MAX, CHAR_MAX, CHAR_MAX, CHAR_MAX, CHAR_MAX, CHAR_MAX,
                CHAR_MAX, CHAR_MAX, CHAR_MAX, CHAR_MAX, CHAR_MAX, CHAR_MAX, CHAR_MAX, CHAR_MAX,
                CHAR_MAX, CHAR_MAX,
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
            chunk = read_avx(s);
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
            chunk = read_avx(s);
            let cmp = _mm256_set_epi8(
                HIGH, HIGH, HIGH, HIGH, HIGH, HIGH, HIGH, HIGH, HIGH, HIGH, HIGH, HIGH, CHAR_MAX,
                CHAR_MAX, CHAR_MAX, CHAR_MAX, CHAR_MAX, CHAR_MAX, CHAR_MAX, CHAR_MAX, CHAR_MAX,
                CHAR_MAX, CHAR_MAX, CHAR_MAX, CHAR_MAX, CHAR_MAX, CHAR_MAX, CHAR_MAX, CHAR_MAX,
                CHAR_MAX, CHAR_MAX, CHAR_MAX,
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
            chunk = read_avx(s);
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
            chunk = read_avx(s);
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
            chunk = read_avx(s);
            let cmp = _mm256_set_epi8(
                HIGH, HIGH, HIGH, HIGH, HIGH, HIGH, HIGH, HIGH, HIGH, CHAR_MAX, CHAR_MAX, CHAR_MAX,
                CHAR_MAX, CHAR_MAX, CHAR_MAX, CHAR_MAX, CHAR_MAX, CHAR_MAX, CHAR_MAX, CHAR_MAX,
                CHAR_MAX, CHAR_MAX, CHAR_MAX, CHAR_MAX, CHAR_MAX, CHAR_MAX, CHAR_MAX, CHAR_MAX,
                CHAR_MAX, CHAR_MAX, CHAR_MAX, CHAR_MAX,
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
            chunk = read_avx(s);
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
            chunk = read_avx(s);
            let cmp = _mm256_set_epi8(
                HIGH, HIGH, HIGH, HIGH, HIGH, HIGH, HIGH, CHAR_MAX, CHAR_MAX, CHAR_MAX, CHAR_MAX,
                CHAR_MAX, CHAR_MAX, CHAR_MAX, CHAR_MAX, CHAR_MAX, CHAR_MAX, CHAR_MAX, CHAR_MAX,
                CHAR_MAX, CHAR_MAX, CHAR_MAX, CHAR_MAX, CHAR_MAX, CHAR_MAX, CHAR_MAX, CHAR_MAX,
                CHAR_MAX, CHAR_MAX, CHAR_MAX, CHAR_MAX, CHAR_MAX,
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
            chunk = read_avx(s);
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
            chunk = read_avx(s);
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
            chunk = read_avx(s);
            let cmp = _mm256_set_epi8(
                HIGH, HIGH, HIGH, HIGH, CHAR_MAX, CHAR_MAX, CHAR_MAX, CHAR_MAX, CHAR_MAX, CHAR_MAX,
                CHAR_MAX, CHAR_MAX, CHAR_MAX, CHAR_MAX, CHAR_MAX, CHAR_MAX, CHAR_MAX, CHAR_MAX,
                CHAR_MAX, CHAR_MAX, CHAR_MAX, CHAR_MAX, CHAR_MAX, CHAR_MAX, CHAR_MAX, CHAR_MAX,
                CHAR_MAX, CHAR_MAX, CHAR_MAX, CHAR_MAX, CHAR_MAX, CHAR_MAX,
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
            chunk = read_avx(s);
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
            chunk = read_avx(s);
            let cmp = _mm256_set_epi8(
                HIGH, HIGH, CHAR_MAX, CHAR_MAX, CHAR_MAX, CHAR_MAX, CHAR_MAX, CHAR_MAX, CHAR_MAX,
                CHAR_MAX, CHAR_MAX, CHAR_MAX, CHAR_MAX, CHAR_MAX, CHAR_MAX, CHAR_MAX, CHAR_MAX,
                CHAR_MAX, CHAR_MAX, CHAR_MAX, CHAR_MAX, CHAR_MAX, CHAR_MAX, CHAR_MAX, CHAR_MAX,
                CHAR_MAX, CHAR_MAX, CHAR_MAX, CHAR_MAX, CHAR_MAX, CHAR_MAX, CHAR_MAX,
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
            chunk = read_avx(s);
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
            chunk = read_avx(s);
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
        _ => return parse_u64(s, ParseType::None).map(|v| v as u128),
    }

    process_avx(chunk, check_high, check_low, s)
}

#[inline]
fn parse_u8(s: &[u8], parse_type: ParseType) -> Result<u8, AtoiSimdError> {
    let val = unsafe { parse_u64(s, parse_type)? };
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
fn parse_i8(s: &[u8]) -> Result<i8, AtoiSimdError> {
    if *s.first().ok_or(AtoiSimdError::Empty)? == b'-' {
        let val = unsafe { parse_u64(&s[1..], ParseType::None)? };
        if val > i8::MAX as u64 + 1 {
            Err(AtoiSimdError::Overflow(ParseType::I8Neg, s))
        } else if val == i8::MAX as u64 + 1 {
            Ok(i8::MIN)
        } else {
            Ok(-(val as i8))
        }
    } else {
        parse_u8(s, ParseType::I8).map(|v| v as i8)
    }
}

#[inline]
fn parse_u16(s: &[u8], parse_type: ParseType) -> Result<u16, AtoiSimdError> {
    let val = unsafe { parse_u64(s, parse_type)? };
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
fn parse_i16(s: &[u8]) -> Result<i16, AtoiSimdError> {
    if *s.first().ok_or(AtoiSimdError::Empty)? == b'-' {
        let val = unsafe { parse_u64(&s[1..], ParseType::None)? };
        if val > i16::MAX as u64 + 1 {
            Err(AtoiSimdError::Overflow(ParseType::I16Neg, s))
        } else if val == i16::MAX as u64 + 1 {
            Ok(i16::MIN)
        } else {
            Ok(-(val as i16))
        }
    } else {
        parse_u16(s, ParseType::I16).map(|v| v as i16)
    }
}

#[inline]
fn parse_u32(s: &[u8], parse_type: ParseType) -> Result<u32, AtoiSimdError> {
    let val = unsafe { parse_u64(s, parse_type)? };
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
fn parse_i32(s: &[u8]) -> Result<i32, AtoiSimdError> {
    if *s.first().ok_or(AtoiSimdError::Empty)? == b'-' {
        let val = unsafe { parse_u64(&s[1..], ParseType::None)? };
        if val > i32::MAX as u64 + 1 {
            Err(AtoiSimdError::Overflow(ParseType::I32Neg, s))
        } else if val == i32::MAX as u64 + 1 {
            Ok(i32::MIN)
        } else {
            Ok(-(val as i32))
        }
    } else {
        parse_u32(s, ParseType::I32).map(|v| v as i32)
    }
}

/// Parses string of digits and first '-' char.
/// String length (except '-' char) must be 1..=20.
/// This function is slower than `parse_u64()`, because it checks for '-' sign.
/// Uses SSE4.1 intrinsics
#[inline]
fn parse_i64(s: &[u8]) -> Result<i64, AtoiSimdError> {
    if *s.first().ok_or(AtoiSimdError::Empty)? == b'-' {
        let res = unsafe { parse_u64(&s[1..], ParseType::I64Neg).map(|v| -(v as i64)) };

        if let Err(AtoiSimdError::I64Min) = res {
            return Ok(i64::MIN);
        }

        res
    } else {
        unsafe { parse_u64(s, ParseType::I64).map(|v| v as i64) }
    }
}

/// Parses string of digits and first '-' char.
/// String length (except '-' char) must be 1..=32.
/// This function is slower than `parse_u128()`, because it checks for '-' sign.
/// Uses AVX/AVX2 intrinsics
#[inline]
fn parse_i128(s: &[u8]) -> Result<i128, AtoiSimdError> {
    if *s.first().ok_or(AtoiSimdError::Empty)? == b'-' {
        unsafe { parse_u128(&s[1..]).map(|v| -(v as i128)) }
    } else {
        unsafe { parse_u128(s).map(|v| v as i128) }
    }
}

pub trait Parser<T> {
    fn atoi_simd_parse(s: &[u8]) -> Result<T, AtoiSimdError>;
}

impl Parser<u8> for u8 {
    #[inline]
    fn atoi_simd_parse(s: &[u8]) -> Result<u8, AtoiSimdError> {
        parse_u8(s, ParseType::None)
    }
}

impl Parser<i8> for i8 {
    #[inline]
    fn atoi_simd_parse(s: &[u8]) -> Result<i8, AtoiSimdError> {
        parse_i8(s)
    }
}

impl Parser<u16> for u16 {
    #[inline]
    fn atoi_simd_parse(s: &[u8]) -> Result<u16, AtoiSimdError> {
        parse_u16(s, ParseType::None)
    }
}

impl Parser<i16> for i16 {
    #[inline]
    fn atoi_simd_parse(s: &[u8]) -> Result<i16, AtoiSimdError> {
        parse_i16(s)
    }
}

impl Parser<u32> for u32 {
    #[inline]
    fn atoi_simd_parse(s: &[u8]) -> Result<u32, AtoiSimdError> {
        parse_u32(s, ParseType::None)
    }
}

impl Parser<i32> for i32 {
    #[inline]
    fn atoi_simd_parse(s: &[u8]) -> Result<i32, AtoiSimdError> {
        parse_i32(s)
    }
}

#[cfg(target_pointer_width = "32")]
impl Parser<usize> for usize {
    #[inline]
    fn atoi_simd_parse(s: &[u8]) -> Result<usize, AtoiSimdError> {
        parse_u32(s, ParseType::None).map(|v| v as usize)
    }
}

#[cfg(target_pointer_width = "32")]
impl Parser<isize> for isize {
    #[inline]
    fn atoi_simd_parse(s: &[u8]) -> Result<isize, AtoiSimdError> {
        parse_i32(s).map(|v| v as isize)
    }
}

#[cfg(target_pointer_width = "64")]
impl Parser<usize> for usize {
    #[inline]
    fn atoi_simd_parse(s: &[u8]) -> Result<usize, AtoiSimdError> {
        unsafe { parse_u64(s, ParseType::None).map(|v| v as usize) }
    }
}

#[cfg(target_pointer_width = "64")]
impl Parser<isize> for isize {
    #[inline]
    fn atoi_simd_parse(s: &[u8]) -> Result<isize, AtoiSimdError> {
        parse_i64(s).map(|v| v as isize)
    }
}

impl Parser<u64> for u64 {
    #[inline]
    fn atoi_simd_parse(s: &[u8]) -> Result<u64, AtoiSimdError> {
        unsafe { parse_u64(s, ParseType::None) }
    }
}

impl Parser<i64> for i64 {
    #[inline]
    fn atoi_simd_parse(s: &[u8]) -> Result<i64, AtoiSimdError> {
        parse_i64(s)
    }
}

impl Parser<u128> for u128 {
    #[inline]
    fn atoi_simd_parse(s: &[u8]) -> Result<u128, AtoiSimdError> {
        unsafe { parse_u128(s) }
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

#[cfg(test)]
mod tests {
    use super::*;

    const INVALID_CHARS: [&str; 6] = ["/", ":", "\0", "\x7f", "!", "a"];

    fn test_each_position<T: Copy>(s: &str, func: fn(&[u8]) -> Result<T, AtoiSimdError>) {
        for j in 0..=s.len() {
            for &ch_str in INVALID_CHARS.iter() {
                let s_new = (&s[0..j]).to_owned() + ch_str + &s[j..s.len()];
                if func(s_new.as_bytes()).is_ok() {
                    panic!("error {}", s_new);
                }
            }
        }
    }

    fn test_each_position_u8(s: &str) {
        test_each_position(s, |s_new| parse::<u8>(s_new))
    }

    fn test_each_position_u16(s: &str) {
        test_each_position(s, |s_new| parse::<u16>(s_new))
    }

    fn test_each_position_u32(s: &str) {
        test_each_position(s, |s_new| parse::<u32>(s_new))
    }

    fn test_each_position_u64(s: &str) {
        test_each_position(s, |s_new| parse::<u64>(s_new))
    }

    #[test]
    fn test_parse_u8() {
        if parse::<u8>("".as_bytes()).is_ok() {
            panic!("error");
        }

        assert_eq!(parse::<u8>("0".as_bytes()).unwrap(), 0_u8);

        let mut s = String::with_capacity(10);
        for i in '1'..='3' {
            test_each_position_u8(&s);
            s.push(i);
            assert_eq!(parse::<u8>(s.as_bytes()).unwrap(), s.parse::<u8>().unwrap());
        }

        assert_eq!(parse::<u8>("255".as_bytes()).unwrap(), u8::MAX);

        if parse::<u8>("256".as_bytes()).is_ok() {
            panic!("error");
        }

        if parse::<u8>("12345678".as_bytes()).is_ok() {
            panic!("error");
        }
    }

    #[test]
    fn test_parse_i8() {
        if parse::<i8>("".as_bytes()).is_ok() {
            panic!("error");
        }

        assert_eq!(parse::<i8>("0".as_bytes()).unwrap(), 0_i8);

        assert_eq!(parse::<i8>("-0".as_bytes()).unwrap(), 0_i8);

        let mut s = String::with_capacity(19);
        let mut s_neg = String::with_capacity(20);
        s_neg.push('-');
        for i in '1'..='3' {
            test_each_position(&s, parse::<i8>);
            s.push(i);
            s_neg.push(i);
            assert_eq!(parse::<i8>(s.as_bytes()).unwrap(), s.parse::<i8>().unwrap());
            assert_eq!(
                parse::<i8>(s_neg.as_bytes()).unwrap(),
                s_neg.parse::<i8>().unwrap()
            );
        }

        assert_eq!(parse::<i8>("127".as_bytes()).unwrap(), i8::MAX);

        if parse::<i8>("128".as_bytes()).is_ok() {
            panic!("error");
        }

        assert_eq!(parse::<i8>("-128".as_bytes()).unwrap(), i8::MIN);

        if parse::<i8>("-129".as_bytes()).is_ok() {
            panic!("error");
        }

        if parse::<i8>("255".as_bytes()).is_ok() {
            panic!("error");
        }

        if parse::<i8>("12345678".as_bytes()).is_ok() {
            panic!("error");
        }

        if parse::<i8>("-12345678".as_bytes()).is_ok() {
            panic!("error");
        }
    }

    #[test]
    fn test_parse_u16() {
        if parse::<u16>("".as_bytes()).is_ok() {
            panic!("error");
        }

        assert_eq!(parse::<u16>("0".as_bytes()).unwrap(), 0_u16);

        let mut s = String::with_capacity(10);
        for i in '1'..='5' {
            test_each_position_u16(&s);
            s.push(i);
            assert_eq!(
                parse::<u16>(s.as_bytes()).unwrap(),
                s.parse::<u16>().unwrap()
            );
        }

        assert_eq!(parse::<u16>("65535".as_bytes()).unwrap(), u16::MAX);

        if parse::<u16>("65536".as_bytes()).is_ok() {
            panic!("error");
        }

        if parse::<u16>("12345678".as_bytes()).is_ok() {
            panic!("error");
        }
    }

    #[test]
    fn test_parse_i16() {
        if parse::<i16>("".as_bytes()).is_ok() {
            panic!("error");
        }

        assert_eq!(parse::<i16>("0".as_bytes()).unwrap(), 0_i16);

        assert_eq!(parse::<i16>("-0".as_bytes()).unwrap(), 0_i16);

        let mut s = String::with_capacity(19);
        let mut s_neg = String::with_capacity(20);
        s_neg.push('-');
        for i in '1'..='5' {
            test_each_position(&s, parse::<i16>);
            s.push(i);
            s_neg.push(i);
            assert_eq!(
                parse::<i16>(s.as_bytes()).unwrap(),
                s.parse::<i16>().unwrap()
            );
            assert_eq!(
                parse::<i16>(s_neg.as_bytes()).unwrap(),
                s_neg.parse::<i16>().unwrap()
            );
        }

        assert_eq!(parse::<i16>("32767".as_bytes()).unwrap(), i16::MAX);

        if parse::<i16>("32768".as_bytes()).is_ok() {
            panic!("error");
        }

        assert_eq!(parse::<i16>("-32768".as_bytes()).unwrap(), i16::MIN);

        if parse::<i16>("-32769".as_bytes()).is_ok() {
            panic!("error");
        }

        if parse::<i16>("65535".as_bytes()).is_ok() {
            panic!("error");
        }

        if parse::<i16>("12345678".as_bytes()).is_ok() {
            panic!("error");
        }

        if parse::<i16>("-12345678".as_bytes()).is_ok() {
            panic!("error");
        }
    }

    #[test]
    fn test_parse_u32() {
        if parse::<u32>("".as_bytes()).is_ok() {
            panic!("error");
        }

        assert_eq!(parse::<u32>("0".as_bytes()).unwrap(), 0_u32);

        let mut s = String::with_capacity(10);
        for i in '1'..='9' {
            test_each_position_u32(&s);
            s.push(i);
            assert_eq!(
                parse::<u32>(s.as_bytes()).unwrap(),
                s.parse::<u32>().unwrap()
            );
        }
        test_each_position_u32(&s);
        s.push('0');
        assert_eq!(
            parse::<u32>(s.as_bytes()).unwrap(),
            s.parse::<u32>().unwrap()
        );

        assert_eq!(parse::<u32>("4294967295".as_bytes()).unwrap(), u32::MAX);

        if parse::<u32>("4294967296".as_bytes()).is_ok() {
            panic!("error");
        }

        if parse::<u32>("123456789012345".as_bytes()).is_ok() {
            panic!("error");
        }
    }

    #[test]
    fn test_parse_i32() {
        if parse::<i32>("".as_bytes()).is_ok() {
            panic!("error");
        }

        assert_eq!(parse::<i32>("0".as_bytes()).unwrap(), 0_i32);

        assert_eq!(parse::<i32>("-0".as_bytes()).unwrap(), 0_i32);

        let mut s = String::with_capacity(19);
        let mut s_neg = String::with_capacity(20);
        s_neg.push('-');
        for i in '1'..='9' {
            test_each_position(&s, parse::<i32>);
            s.push(i);
            s_neg.push(i);
            assert_eq!(
                parse::<i32>(s.as_bytes()).unwrap(),
                s.parse::<i32>().unwrap()
            );
            assert_eq!(
                parse::<i32>(s_neg.as_bytes()).unwrap(),
                s_neg.parse::<i32>().unwrap()
            );
        }
        test_each_position(&s, parse::<i32>);
        s.push('0');
        s_neg.push('0');
        assert_eq!(
            parse::<i32>(s.as_bytes()).unwrap(),
            s.parse::<i32>().unwrap()
        );
        assert_eq!(
            parse::<i32>(s_neg.as_bytes()).unwrap(),
            s_neg.parse::<i32>().unwrap()
        );

        assert_eq!(parse::<i32>("2147483647".as_bytes()).unwrap(), i32::MAX);

        if parse::<i32>("2147483648".as_bytes()).is_ok() {
            panic!("error");
        }

        assert_eq!(parse::<i32>("-2147483648".as_bytes()).unwrap(), i32::MIN);

        if parse::<i32>("-2147483649".as_bytes()).is_ok() {
            panic!("error");
        }

        if parse::<i32>("4294967295".as_bytes()).is_ok() {
            panic!("error");
        }

        if parse::<i32>("123456789012345".as_bytes()).is_ok() {
            panic!("error");
        }

        if parse::<i32>("-123456789012345".as_bytes()).is_ok() {
            panic!("error");
        }
    }

    #[test]
    fn test_parse_u64() {
        if parse::<u64>("".as_bytes()).is_ok() {
            panic!("error");
        }

        assert_eq!(parse::<u64>("0".as_bytes()).unwrap(), 0_u64);

        let mut s = String::with_capacity(20);
        for i in '1'..='9' {
            test_each_position_u64(&s);
            s.push(i);
            assert_eq!(
                parse::<u64>(s.as_bytes()).unwrap(),
                s.parse::<u64>().unwrap()
            );
        }
        for i in '0'..='9' {
            test_each_position_u64(&s);
            s.push(i);
            assert_eq!(
                parse::<u64>(s.as_bytes()).unwrap(),
                s.parse::<u64>().unwrap()
            );
        }
        test_each_position_u64(&s);
        s.push('0');
        assert_eq!(
            parse::<u64>(s.as_bytes()).unwrap(),
            s.parse::<u64>().unwrap()
        );

        assert_eq!(
            parse::<u64>("18446744073709551615".as_bytes()).unwrap(),
            u64::MAX
        );

        if parse::<u64>("18446744073709551616".as_bytes()).is_ok() {
            panic!("error");
        }

        if parse::<u64>("99999999999999999999".as_bytes()).is_ok() {
            panic!("error");
        }
    }

    #[test]
    fn test_parse_i64() {
        if parse::<i64>("".as_bytes()).is_ok() {
            panic!("error");
        }

        assert_eq!(parse::<i64>("0".as_bytes()).unwrap(), 0_i64);

        assert_eq!(parse::<i64>("-0".as_bytes()).unwrap(), 0_i64);

        let mut s = String::with_capacity(19);
        let mut s_neg = String::with_capacity(20);
        s_neg.push('-');
        for i in '1'..='9' {
            test_each_position(&s, parse::<i64>);
            s.push(i);
            s_neg.push(i);
            assert_eq!(
                parse::<i64>(s.as_bytes()).unwrap(),
                s.parse::<i64>().unwrap()
            );
            assert_eq!(
                parse::<i64>(s_neg.as_bytes()).unwrap(),
                s_neg.parse::<i64>().unwrap()
            );
        }
        for i in '0'..='9' {
            test_each_position(&s, parse::<i64>);
            s.push(i);
            s_neg.push(i);
            assert_eq!(
                parse::<i64>(s.as_bytes()).unwrap(),
                s.parse::<i64>().unwrap()
            );
            assert_eq!(
                parse::<i64>(s_neg.as_bytes()).unwrap(),
                s_neg.parse::<i64>().unwrap()
            );
        }

        assert_eq!(
            parse::<i64>("9223372036854775807".as_bytes()).unwrap(),
            i64::MAX
        );

        if parse::<i64>("9223372036854775808".as_bytes()).is_ok() {
            panic!("error");
        }

        assert_eq!(
            parse::<i64>("-9223372036854775808".as_bytes()).unwrap(),
            i64::MIN
        );

        if parse::<i64>("-9223372036854775809".as_bytes()).is_ok() {
            panic!("error");
        }

        if parse::<i64>("18446744073709551615".as_bytes()).is_ok() {
            panic!("error");
        }

        if parse::<i64>("99999999999999999999".as_bytes()).is_ok() {
            panic!("error");
        }

        if parse::<i64>("-99999999999999999999".as_bytes()).is_ok() {
            panic!("error");
        }
    }

    #[test]
    fn test_parse_u128() {
        if parse::<u128>("".as_bytes()).is_ok() {
            panic!("error");
        }

        assert_eq!(parse::<u128>("0".as_bytes()).unwrap(), 0_u128);

        let mut s = String::with_capacity(32);
        for i in '1'..='9' {
            test_each_position(&s, parse::<u128>);
            s.push(i);
            assert_eq!(
                parse::<u128>(s.as_bytes()).unwrap(),
                s.parse::<u128>().unwrap()
            );
        }
        for _ in 0..2 {
            for i in '0'..='9' {
                test_each_position(&s, parse::<u128>);
                s.push(i);
                assert_eq!(
                    parse::<u128>(s.as_bytes()).unwrap(),
                    s.parse::<u128>().unwrap()
                );
            }
        }
        for i in '0'..='2' {
            test_each_position(&s, parse::<u128>);
            s.push(i);
            assert_eq!(
                parse::<u128>(s.as_bytes()).unwrap(),
                s.parse::<u128>().unwrap()
            );
        }

        assert_eq!(
            parse::<u128>("9999999999999999".as_bytes()).unwrap(),
            9_999_999_999_999_999_u128
        );

        assert_eq!(
            parse::<u128>("12345678901234567890123456789012".as_bytes()).unwrap(),
            1234567890_1234567890_1234567890_12_u128
        );

        if parse::<u128>("123456789012345678901234567890123".as_bytes()).is_ok() {
            panic!("error");
        }
    }

    #[test]
    fn test_parse_i128() {
        if parse::<i128>("".as_bytes()).is_ok() {
            panic!("error");
        }

        assert_eq!(parse::<i128>("0".as_bytes()).unwrap(), 0_i128);

        assert_eq!(parse::<i128>("-0".as_bytes()).unwrap(), 0_i128);

        let mut s = String::with_capacity(32);
        let mut s_neg = String::with_capacity(33);
        s_neg.push('-');
        for i in '1'..='9' {
            test_each_position(&s, parse::<i128>);
            s.push(i);
            s_neg.push(i);
            assert_eq!(
                parse::<i128>(s.as_bytes()).unwrap(),
                s.parse::<i128>().unwrap()
            );
            assert_eq!(
                parse::<i128>(s_neg.as_bytes()).unwrap(),
                s_neg.parse::<i128>().unwrap()
            );
        }
        for _ in 0..2 {
            for i in '0'..='9' {
                test_each_position(&s, parse::<i128>);
                s.push(i);
                s_neg.push(i);
                assert_eq!(
                    parse::<i128>(s.as_bytes()).unwrap(),
                    s.parse::<i128>().unwrap()
                );
                assert_eq!(
                    parse::<i128>(s_neg.as_bytes()).unwrap(),
                    s_neg.parse::<i128>().unwrap()
                );
            }
        }
        for i in '0'..='2' {
            test_each_position(&s, parse::<i128>);
            s.push(i);
            s_neg.push(i);
            assert_eq!(
                parse::<i128>(s.as_bytes()).unwrap(),
                s.parse::<i128>().unwrap()
            );
            assert_eq!(
                parse::<i128>(s_neg.as_bytes()).unwrap(),
                s_neg.parse::<i128>().unwrap()
            );
        }

        assert_eq!(
            parse::<i128>("-9999999999999999".as_bytes()).unwrap(),
            -9_999_999_999_999_999_i128
        );

        assert_eq!(
            parse::<i128>("9999999999999999".as_bytes()).unwrap(),
            9_999_999_999_999_999_i128
        );

        assert_eq!(
            parse::<i128>("-99999999999999999999999999999999".as_bytes()).unwrap(),
            -99_999_999_999_999_999_999_999_999_999_999_i128
        );

        assert_eq!(
            parse::<i128>("99999999999999999999999999999999".as_bytes()).unwrap(),
            99_999_999_999_999_999_999_999_999_999_999_i128
        );

        assert_eq!(
            parse::<i128>("12345678901234567890123456789012".as_bytes()).unwrap(),
            1234567890_1234567890_1234567890_12_i128
        );

        assert_eq!(
            parse::<i128>("-12345678901234567890123456789012".as_bytes()).unwrap(),
            -1234567890_1234567890_1234567890_12_i128
        );

        if parse::<i128>("123456789012345678901234567890123".as_bytes()).is_ok() {
            panic!("error");
        }
    }

    #[test]
    fn test_parse_types() {
        let tmp: u8 = parse("123".as_bytes()).unwrap();
        assert_eq!(tmp, 123_u8);

        let tmp: i8 = parse("-123".as_bytes()).unwrap();
        assert_eq!(tmp, -123_i8);

        let tmp: u16 = parse("1234".as_bytes()).unwrap();
        assert_eq!(tmp, 1234_u16);

        let tmp: i16 = parse("-1234".as_bytes()).unwrap();
        assert_eq!(tmp, -1234_i16);

        let tmp: u32 = parse("1234".as_bytes()).unwrap();
        assert_eq!(tmp, 1234_u32);

        let tmp: i32 = parse("-1234".as_bytes()).unwrap();
        assert_eq!(tmp, -1234_i32);

        let tmp: usize = parse("1234".as_bytes()).unwrap();
        assert_eq!(tmp, 1234_usize);

        let tmp: isize = parse("-1234".as_bytes()).unwrap();
        assert_eq!(tmp, -1234_isize);

        let tmp: u64 = parse("1234".as_bytes()).unwrap();
        assert_eq!(tmp, 1234_u64);

        let tmp: i64 = parse("-1234".as_bytes()).unwrap();
        assert_eq!(tmp, -1234_i64);

        let tmp: u128 = parse("999999".as_bytes()).unwrap();
        assert_eq!(tmp, 999999_u128);

        let tmp: i128 = parse("-999999".as_bytes()).unwrap();
        assert_eq!(tmp, -999999_i128);
    }
}
