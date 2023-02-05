use crate::{AtoiSimdError, ParseType};
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

const HIGH: i8 = i8::MAX;
const LOW: i8 = i8::MIN;
const CHAR_MAX: i8 = b'9' as i8;
const CHAR_MIN: i8 = b'0' as i8;

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

/// Parses string of *only* digits
#[target_feature(enable = "sse2,sse3,sse4.1,ssse3,avx,avx2")]
pub(crate) unsafe fn parse_u64(s: &[u8], parse_type: ParseType) -> Result<u64, AtoiSimdError> {
    match s.len() {
        0 => Err(AtoiSimdError::Empty),
        1 => match s[0] {
            c @ b'0'..=b'9' => Ok((c & 0xF) as u64),
            _ => Err(AtoiSimdError::Invalid(s)),
        },
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
pub(crate) unsafe fn parse_u128(s: &[u8]) -> Result<u128, AtoiSimdError> {
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
