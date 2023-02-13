#![allow(dead_code)]

use crate::fallback::{parse_fb_128_pos, parse_fb_checked_neg, parse_fb_neg, parse_fb_pos};
use crate::AtoiSimdError;
#[cfg(target_arch = "x86")]
use core::arch::x86::{
    __m128i, __m256i, _mm256_and_si256, _mm256_bslli_epi128, _mm256_bsrli_epi128,
    _mm256_cmpgt_epi8, _mm256_extracti128_si256, _mm256_lddqu_si256, _mm256_madd_epi16,
    _mm256_maddubs_epi16, _mm256_movemask_epi8, _mm256_or_si256, _mm256_packus_epi32,
    _mm256_permute2x128_si256, _mm256_permute4x64_epi64, _mm256_set1_epi8, _mm256_set_epi16,
    _mm256_set_epi8, _mm_add_epi64, _mm_and_si128, _mm_cmpgt_epi8, _mm_lddqu_si128, _mm_madd_epi16,
    _mm_maddubs_epi16, _mm_movemask_epi8, _mm_mul_epu32, _mm_or_si128, _mm_packus_epi32,
    _mm_set1_epi8, _mm_set_epi16, _mm_set_epi32, _mm_set_epi8, _mm_srli_epi64,
};
#[cfg(target_arch = "x86_64")]
use core::arch::x86_64::{
    __m128i, __m256i, _mm256_and_si256, _mm256_bslli_epi128, _mm256_bsrli_epi128,
    _mm256_cmpgt_epi8, _mm256_extracti128_si256, _mm256_lddqu_si256, _mm256_madd_epi16,
    _mm256_maddubs_epi16, _mm256_movemask_epi8, _mm256_or_si256, _mm256_packus_epi32,
    _mm256_permute2x128_si256, _mm256_permute4x64_epi64, _mm256_set1_epi8, _mm256_set_epi16,
    _mm256_set_epi8, _mm_add_epi64, _mm_and_si128, _mm_cmpgt_epi8, _mm_cvtsi128_si64,
    _mm_lddqu_si128, _mm_madd_epi16, _mm_maddubs_epi16, _mm_movemask_epi8, _mm_mul_epu32,
    _mm_or_si128, _mm_packus_epi32, _mm_set1_epi8, _mm_set_epi16, _mm_set_epi32, _mm_set_epi8,
    _mm_srli_epi64,
};

const CHAR_MAX: i8 = b'9' as i8;
const CHAR_MIN: i8 = b'0' as i8;

/// s = "1234567890123456"
#[inline(always)]
unsafe fn read(s: &[u8]) -> __m128i {
    _mm_lddqu_si128(std::mem::transmute_copy(&s))
}

#[inline(always)]
unsafe fn read_avx(s: &[u8]) -> __m256i {
    _mm256_lddqu_si256(std::mem::transmute_copy(&s))
}

/// converts chars  [ 0x36353433323130393837363534333231 ]
/// to numbers      [ 0x06050403020100090807060504030201 ]
#[inline(always)]
unsafe fn to_numbers(chunk: __m128i) -> __m128i {
    let mult = _mm_set1_epi8(0xF);

    _mm_and_si128(chunk, mult)
}

#[inline(always)]
unsafe fn process_gt(cmp_left: __m128i, cmp_right: __m128i) -> __m128i {
    _mm_cmpgt_epi8(cmp_left, cmp_right)
}

#[inline(always)]
unsafe fn process_avx_gt(cmp_left: __m256i, cmp_right: __m256i) -> __m256i {
    _mm256_cmpgt_epi8(cmp_left, cmp_right)
}

#[inline(always)]
unsafe fn checker(check: __m128i, check2: __m128i) -> u32 {
    let chunk = _mm_or_si128(check, check2);
    let res = _mm_movemask_epi8(chunk);
    res.trailing_zeros()
}

#[inline(always)]
unsafe fn checker_avx(check: __m256i, check2: __m256i) -> u32 {
    let chunk = _mm256_or_si256(check, check2);
    let res = _mm256_movemask_epi8(chunk);
    res.trailing_zeros()
}

#[cfg(target_arch = "x86")]
#[inline(always)]
unsafe fn to_u64(chunk: __m128i) -> u64 {
    std::mem::transmute::<__m128i, [u64; 2]>(chunk)[0]
}

#[cfg(target_arch = "x86_64")]
#[inline(always)]
unsafe fn to_u64(chunk: __m128i) -> u64 {
    _mm_cvtsi128_si64(chunk) as u64
}

#[inline(always)]
unsafe fn to_u32x4(chunk: __m128i) -> [u32; 4] {
    std::mem::transmute(chunk)
}

#[inline(always)]
unsafe fn process_mult1(chunk: __m128i, mult1: __m128i) -> __m128i {
    _mm_maddubs_epi16(chunk, mult1)
}

#[inline(always)]
unsafe fn process_small(mut chunk: __m128i, mult1: __m128i, mult2: __m128i) -> __m128i {
    // combine numbers [ 0x0038 | 0x0022 | 0x000c | 0x005a | 0x004e | 0x0038 | 0x0022 | 0x000c ( 56 | 34 | 12 | 90 | 78 | 56 | 34 | 12 ) ]
    chunk = process_mult1(chunk, mult1);

    // combine again   [ 0x0000 | 0x0d80 | 0x0000 | 0x2334 | 0x0000 | 0x162e | 0x0000 | 0x04d2 ( 0 | 3456 | 0 | 9012 | 0 | 5678 | 0 | 1234) ]
    _mm_madd_epi16(chunk, mult2)
}

#[inline(always)]
unsafe fn process_medium(
    mut chunk: __m128i,
    mult1: __m128i,
    mult2: __m128i,
    mult4: __m128i,
) -> __m128i {
    chunk = process_small(chunk, mult1, mult2);

    // remove extra bytes [ (64 bits, same as the right ) | 0x0d80 | 0x2334 | 0x162e | 0x04d2 ( 3456 | 9012 | 5678 | 1234) ]
    chunk = _mm_packus_epi32(chunk, chunk);

    // _mm_set_epi16(1, 10000, 0, 0, 0, 0, 1, 10000);

    // combine again [ (64 bits, zeroes) | 0x055f2cc0 | 0x00bc614e ( 90123456 | 12345678 ) ]
    _mm_madd_epi16(chunk, mult4)
}

#[inline(always)]
unsafe fn process_big(
    mut chunk: __m128i,
    mult1: __m128i,
    mult2: __m128i,
    mult4: __m128i,
    mult8: u64,
) -> u64 {
    chunk = process_medium(chunk, mult1, mult2, mult4);

    // let chunk = to_u64(chunk);
    // ((chunk & 0xFFFF_FFFF) * 100_000_000) + (chunk >> 32)

    let arr = to_u32x4(chunk);
    (arr[0] as u64 * mult8) + (arr[1] as u64)
}

#[inline(always)]
fn parse_unchecked_64(s: &[u8], len: usize) -> Result<(u64, usize), AtoiSimdError> {
    if len == 0 {
        return Err(AtoiSimdError::Empty);
    }
    Ok(((s[0] & 0xF) as u64, len))
}

/// Parses string of *only* digits
/// Uses SSE intrinsics
#[inline(always)]
unsafe fn parse_simd_sse(
    s: &[u8],
    len: usize,
    mut chunk: __m128i,
) -> Result<(u64, usize), AtoiSimdError> {
    let res = match len {
        2 => {
            let mult = _mm_set_epi8(0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 10);
            chunk = process_mult1(chunk, mult);
            to_u64(chunk)
        }
        3 => {
            chunk = process_small(
                chunk,
                _mm_set_epi8(0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 1, 10),
                _mm_set_epi16(0, 0, 0, 0, 0, 0, 1, 10),
            );
            to_u64(chunk)
        }
        4 => {
            chunk = process_small(
                chunk,
                _mm_set_epi8(0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 10, 1, 10),
                _mm_set_epi16(0, 0, 0, 0, 0, 0, 1, 100),
            );
            to_u64(chunk)
        }
        5 => {
            chunk = process_medium(
                chunk,
                _mm_set_epi8(0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 1, 10, 1, 10),
                _mm_set_epi16(0, 0, 0, 0, 0, 1, 1, 100),
                _mm_set_epi16(0, 0, 0, 0, 0, 0, 1, 10),
            );
            to_u64(chunk)
        }
        6 => {
            chunk = process_medium(
                chunk,
                _mm_set_epi8(0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 10, 1, 10, 1, 10),
                _mm_set_epi16(0, 0, 0, 0, 0, 1, 1, 100),
                _mm_set_epi16(0, 0, 0, 0, 0, 0, 1, 100),
            );
            to_u64(chunk)
        }
        7 => {
            chunk = process_medium(
                chunk,
                _mm_set_epi8(0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 1, 10, 1, 10, 1, 10),
                _mm_set_epi16(0, 0, 0, 0, 1, 10, 1, 100),
                _mm_set_epi16(0, 0, 0, 0, 0, 0, 1, 1000),
            );
            to_u64(chunk)
        }
        8 => {
            chunk = process_medium(
                chunk,
                _mm_set_epi8(0, 0, 0, 0, 0, 0, 0, 0, 1, 10, 1, 10, 1, 10, 1, 10),
                _mm_set_epi16(0, 0, 0, 0, 1, 100, 1, 100),
                _mm_set_epi16(0, 0, 0, 0, 0, 0, 1, 10000),
            );
            to_u64(chunk)
        }
        9 => process_big(
            chunk,
            _mm_set_epi8(0, 0, 0, 0, 0, 0, 0, 1, 1, 10, 1, 10, 1, 10, 1, 10),
            _mm_set_epi16(0, 0, 0, 1, 1, 100, 1, 100),
            _mm_set_epi16(0, 0, 0, 0, 0, 1, 1, 10000),
            10,
        ),
        10 => process_big(
            chunk,
            _mm_set_epi8(0, 0, 0, 0, 0, 0, 1, 10, 1, 10, 1, 10, 1, 10, 1, 10),
            _mm_set_epi16(0, 0, 0, 1, 1, 100, 1, 100),
            _mm_set_epi16(0, 0, 0, 0, 0, 1, 1, 10000),
            100,
        ),
        11 => process_big(
            chunk,
            _mm_set_epi8(0, 0, 0, 0, 0, 1, 1, 10, 1, 10, 1, 10, 1, 10, 1, 10),
            _mm_set_epi16(0, 0, 1, 10, 1, 100, 1, 100),
            _mm_set_epi16(0, 0, 0, 0, 0, 1, 1, 10000),
            1000,
        ),
        12 => process_big(
            chunk,
            _mm_set_epi8(0, 0, 0, 0, 1, 10, 1, 10, 1, 10, 1, 10, 1, 10, 1, 10),
            _mm_set_epi16(0, 0, 1, 100, 1, 100, 1, 100),
            _mm_set_epi16(0, 0, 0, 0, 0, 1, 1, 10000),
            10_000,
        ),
        13 => process_big(
            chunk,
            _mm_set_epi8(0, 0, 0, 1, 1, 10, 1, 10, 1, 10, 1, 10, 1, 10, 1, 10),
            _mm_set_epi16(0, 1, 1, 100, 1, 100, 1, 100),
            _mm_set_epi16(0, 0, 0, 0, 1, 10, 1, 10000),
            100_000,
        ),
        14 => process_big(
            chunk,
            _mm_set_epi8(0, 0, 1, 10, 1, 10, 1, 10, 1, 10, 1, 10, 1, 10, 1, 10),
            _mm_set_epi16(0, 1, 1, 100, 1, 100, 1, 100),
            _mm_set_epi16(0, 0, 0, 0, 1, 100, 1, 10000),
            1_000_000,
        ),
        15 => process_big(
            chunk,
            _mm_set_epi8(0, 1, 1, 10, 1, 10, 1, 10, 1, 10, 1, 10, 1, 10, 1, 10),
            _mm_set_epi16(1, 10, 1, 100, 1, 100, 1, 100),
            _mm_set_epi16(0, 0, 0, 0, 1, 1000, 1, 10000),
            10_000_000,
        ),
        16 => process_big(
            chunk,
            _mm_set_epi8(1, 10, 1, 10, 1, 10, 1, 10, 1, 10, 1, 10, 1, 10, 1, 10),
            _mm_set_epi16(1, 100, 1, 100, 1, 100, 1, 100),
            _mm_set_epi16(0, 0, 0, 0, 1, 10000, 1, 10000),
            100_000_000,
        ),
        // somehow it's faster that way
        0..=1 => return parse_unchecked_64(s, len),
        s_len => return Err(AtoiSimdError::Size(s_len, s)),
        // Do not try to separate this function to three,
        // and chain them with `_ => parse_u32(s).map(|v| v as u64)`,
        // I've tried it, and the performance is not good (even with #[inline]).
    };
    Ok((res, len))
}

#[inline(always)]
fn parse_simd_sse_checked(s: &[u8]) -> Result<(u64, usize), AtoiSimdError> {
    unsafe {
        let mut chunk = read(s);
        let cmp_high = _mm_set_epi8(
            CHAR_MAX, CHAR_MAX, CHAR_MAX, CHAR_MAX, CHAR_MAX, CHAR_MAX, CHAR_MAX, CHAR_MAX,
            CHAR_MAX, CHAR_MAX, CHAR_MAX, CHAR_MAX, CHAR_MAX, CHAR_MAX, CHAR_MAX, CHAR_MAX,
        );
        let cmp_low = _mm_set_epi8(
            CHAR_MIN, CHAR_MIN, CHAR_MIN, CHAR_MIN, CHAR_MIN, CHAR_MIN, CHAR_MIN, CHAR_MIN,
            CHAR_MIN, CHAR_MIN, CHAR_MIN, CHAR_MIN, CHAR_MIN, CHAR_MIN, CHAR_MIN, CHAR_MIN,
        );
        let check_high = process_gt(chunk, cmp_high);
        let check_low = process_gt(cmp_low, chunk);

        chunk = to_numbers(chunk);

        let len = s.len().min(checker(check_high, check_low) as usize);

        parse_simd_sse(s, len, chunk)
    }
}

/// Parses string of *only* digits
/// Uses AVX/AVX2 intrinsics
#[inline(always)]
unsafe fn process_avx(mut chunk: __m256i) -> u128 {
    let mut mult = _mm256_set_epi8(
        1, 10, 1, 10, 1, 10, 1, 10, 1, 10, 1, 10, 1, 10, 1, 10, 1, 10, 1, 10, 1, 10, 1, 10, 1, 10,
        1, 10, 1, 10, 1, 10,
    );
    // mult 1 char
    chunk = _mm256_maddubs_epi16(chunk, mult);

    mult = _mm256_set_epi16(
        1, 100, 1, 100, 1, 100, 1, 100, 1, 100, 1, 100, 1, 100, 1, 100,
    );
    // mult 2
    chunk = _mm256_madd_epi16(chunk, mult);

    // remove extra bytes
    chunk = _mm256_packus_epi32(chunk, chunk);

    // used to move to the SSE intrinsics
    // move by 64 bits ( unused | unused | third [191:128] | first [63:0] )
    // but compiled assembly is different, and faster
    // vextracti128 xmm1, ymm0, 1
    // vpunpcklqdq xmm0, xmm0, xmm1
    chunk = _mm256_permute4x64_epi64(chunk, 8);
    let mut chunk = _mm256_extracti128_si256(chunk, 0);

    let mut mult = _mm_set_epi16(1, 10000, 1, 10000, 1, 10000, 1, 10000);
    // mult 4
    chunk = _mm_madd_epi16(chunk, mult);

    mult = _mm_set_epi32(0, 100_000_000, 0, 100_000_000);
    // mult 8
    mult = _mm_mul_epu32(chunk, mult);
    // add higher 32 bits of old 64 to mult
    chunk = _mm_srli_epi64(chunk, 32);
    chunk = _mm_add_epi64(chunk, mult);

    let arr = std::mem::transmute::<__m128i, [u64; 2]>(chunk);

    // mult 16
    arr[0] as u128 * 10_000_000_000_000_000 + arr[1] as u128

    // AVX intrinsics
    /* mult = _mm256_set_epi16(
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

    arr[0] * 10_000_000_000_000_000 + arr[1] */
}

#[inline(always)]
unsafe fn process_avx_permute2x128(chunk: __m256i) -> __m256i {
    _mm256_permute2x128_si256(chunk, chunk, 8)
}

#[inline(always)]
unsafe fn process_avx_or(chunk: __m256i, mult: __m256i) -> __m256i {
    _mm256_or_si256(chunk, mult)
}

#[inline(always)]
fn parse_unchecked_128(s: &[u8], len: usize) -> Result<(u128, usize), AtoiSimdError> {
    if len == 0 {
        return Err(AtoiSimdError::Empty);
    }
    Ok(((s[0] & 0xF) as u128, len))
}

/// Parses string of *only* digits. String length must be 1..=32.
/// Uses AVX/AVX2 intrinsics
#[inline(always)]
pub(crate) unsafe fn parse_simd_u128(s: &[u8]) -> Result<(u128, usize), AtoiSimdError> {
    let mut len = s.len();
    if len < 4 {
        return parse_fb_128_pos(s);
    } else if len < 17 {
        return parse_simd_sse_checked(s).map(|(v, l)| (v as u128, l));
    }

    let mut chunk = read_avx(s);
    let cmp_max = _mm256_set_epi8(
        CHAR_MAX, CHAR_MAX, CHAR_MAX, CHAR_MAX, CHAR_MAX, CHAR_MAX, CHAR_MAX, CHAR_MAX, CHAR_MAX,
        CHAR_MAX, CHAR_MAX, CHAR_MAX, CHAR_MAX, CHAR_MAX, CHAR_MAX, CHAR_MAX, CHAR_MAX, CHAR_MAX,
        CHAR_MAX, CHAR_MAX, CHAR_MAX, CHAR_MAX, CHAR_MAX, CHAR_MAX, CHAR_MAX, CHAR_MAX, CHAR_MAX,
        CHAR_MAX, CHAR_MAX, CHAR_MAX, CHAR_MAX, CHAR_MAX,
    );
    let cmp_min = _mm256_set_epi8(
        CHAR_MIN, CHAR_MIN, CHAR_MIN, CHAR_MIN, CHAR_MIN, CHAR_MIN, CHAR_MIN, CHAR_MIN, CHAR_MIN,
        CHAR_MIN, CHAR_MIN, CHAR_MIN, CHAR_MIN, CHAR_MIN, CHAR_MIN, CHAR_MIN, CHAR_MIN, CHAR_MIN,
        CHAR_MIN, CHAR_MIN, CHAR_MIN, CHAR_MIN, CHAR_MIN, CHAR_MIN, CHAR_MIN, CHAR_MIN, CHAR_MIN,
        CHAR_MIN, CHAR_MIN, CHAR_MIN, CHAR_MIN, CHAR_MIN,
    );
    let check_high = process_avx_gt(chunk, cmp_max);
    let check_low = process_avx_gt(cmp_min, chunk);
    // to numbers
    chunk = _mm256_and_si256(chunk, _mm256_set1_epi8(0xF));

    len = len.min(checker_avx(check_high, check_low) as usize);

    let mut mult = process_avx_permute2x128(chunk);

    match len {
        17 => {
            mult = _mm256_bsrli_epi128(mult, 1);
            chunk = _mm256_bslli_epi128(chunk, 15);
            chunk = process_avx_or(chunk, mult);
        }
        18 => {
            mult = _mm256_bsrli_epi128(mult, 2);
            chunk = _mm256_bslli_epi128(chunk, 14);
            chunk = process_avx_or(chunk, mult);
        }
        19 => {
            mult = _mm256_bsrli_epi128(mult, 3);
            chunk = _mm256_bslli_epi128(chunk, 13);
            chunk = process_avx_or(chunk, mult);
        }
        20 => {
            // maybe can be optimized even further with _mm256_permutevar8x32_epi32
            mult = _mm256_bsrli_epi128(mult, 4);
            chunk = _mm256_bslli_epi128(chunk, 12);
            chunk = process_avx_or(chunk, mult);
        }
        21 => {
            mult = _mm256_bsrli_epi128(mult, 5);
            chunk = _mm256_bslli_epi128(chunk, 11);
            chunk = process_avx_or(chunk, mult);
        }
        22 => {
            mult = _mm256_bsrli_epi128(mult, 6);
            chunk = _mm256_bslli_epi128(chunk, 10);
            chunk = process_avx_or(chunk, mult);
        }
        23 => {
            mult = _mm256_bsrli_epi128(mult, 7);
            chunk = _mm256_bslli_epi128(chunk, 9);
            chunk = process_avx_or(chunk, mult);
        }
        24 => {
            // maybe can be optimized even further with _mm256_permute4x64_epi64
            mult = _mm256_bsrli_epi128(mult, 8);
            chunk = _mm256_bslli_epi128(chunk, 8);
            chunk = process_avx_or(chunk, mult);
        }
        25 => {
            mult = _mm256_bsrli_epi128(mult, 9);
            chunk = _mm256_bslli_epi128(chunk, 7);
            chunk = process_avx_or(chunk, mult);
        }
        26 => {
            mult = _mm256_bsrli_epi128(mult, 10);
            chunk = _mm256_bslli_epi128(chunk, 6);
            chunk = process_avx_or(chunk, mult);
        }
        27 => {
            mult = _mm256_bsrli_epi128(mult, 11);
            chunk = _mm256_bslli_epi128(chunk, 5);
            chunk = process_avx_or(chunk, mult);
        }
        28 => {
            // maybe can be optimized even further with _mm256_permutevar8x32_epi32
            // let mult = _mm256_set_epi32(6, 5, 4, 3, 2, 1, 0, 0);
            // chunk = _mm256_permutevar8x32_epi32(chunk, mult);
            mult = _mm256_bsrli_epi128(mult, 12);
            chunk = _mm256_bslli_epi128(chunk, 4);
            chunk = process_avx_or(chunk, mult);
        }
        29 => {
            mult = _mm256_bsrli_epi128(mult, 13);
            chunk = _mm256_bslli_epi128(chunk, 3);
            chunk = process_avx_or(chunk, mult);
        }
        30 => {
            mult = _mm256_bsrli_epi128(mult, 14);
            chunk = _mm256_bslli_epi128(chunk, 2);
            chunk = process_avx_or(chunk, mult);
        }
        31 => {
            mult = _mm256_bsrli_epi128(mult, 15);
            chunk = _mm256_bslli_epi128(chunk, 1);
            chunk = process_avx_or(chunk, mult);
        }
        32 => {}
        // somehow it's faster that way
        0..=1 => return parse_unchecked_128(s, len),
        s_len => {
            return parse_simd_sse(s, s_len, std::mem::transmute_copy(&chunk))
                .map(|(v, l)| (v as u128, l))
        }
    }

    Ok((process_avx(chunk), len))
}

#[inline(always)]
fn parse_simd_checked_pre_u64(s: &[u8]) -> Result<u64, AtoiSimdError> {
    let (res, len) = if s.len() < 4 {
        parse_fb_pos::<{ u64::MAX }>(s)
    } else {
        parse_simd_sse_checked(s)
    }?;
    if len < s.len() {
        return Err(AtoiSimdError::Invalid64(res, len));
    }
    Ok(res)
}

#[inline(always)]
fn parse_simd_checked_pre_i64_neg(s: &[u8]) -> Result<i64, AtoiSimdError> {
    let (res, len) = if s.len() < 4 {
        parse_fb_neg::<{ i64::MIN }>(s)
    } else {
        parse_simd_sse_checked(s).map(|(v, l)| (-(v as i64), l))
    }?;
    if len < s.len() {
        return Err(AtoiSimdError::Invalid64(res as u64, len));
    }
    Ok(res)
}

#[inline(always)]
pub(crate) fn parse_simd_checked_u128(s: &[u8]) -> Result<u128, AtoiSimdError> {
    let (res, len) = unsafe { parse_simd_u128(s)? };
    if len < s.len() {
        return Err(AtoiSimdError::Invalid128(res, len));
    }
    Ok(res)
}

#[inline(always)]
pub(crate) fn parse_simd<const MAX: u64>(s: &[u8]) -> Result<(u64, usize), AtoiSimdError> {
    if s.len() < 4 {
        return parse_fb_pos::<{ MAX }>(s);
    }
    let (res, len) = parse_simd_sse_checked(s)?;
    if res > MAX {
        Err(AtoiSimdError::Overflow64(MAX, s))
    } else {
        Ok((res, len))
    }
}

#[inline(always)]
pub(crate) fn parse_simd_checked<const MAX: u64>(s: &[u8]) -> Result<u64, AtoiSimdError> {
    let res = parse_simd_checked_pre_u64(s)?;
    if res > MAX {
        Err(AtoiSimdError::Overflow64(MAX, s))
    } else {
        Ok(res)
    }
}

#[inline(always)]
pub(crate) fn parse_simd_neg<const MIN: i64>(s: &[u8]) -> Result<(i64, usize), AtoiSimdError> {
    if s.len() < 4 {
        return parse_fb_neg::<{ MIN }>(s);
    }
    let (res, len) = parse_simd_sse_checked(s)?;
    let min = -MIN as u64;
    if res > min {
        Err(AtoiSimdError::Overflow64(MIN as u64, s))
    } else if res == min {
        Ok((MIN, len))
    } else {
        Ok((-(res as i64), len))
    }
}

#[inline(always)]
pub(crate) fn parse_simd_checked_neg<const MIN: i64>(s: &[u8]) -> Result<i64, AtoiSimdError> {
    if s.len() < 4 {
        return parse_fb_checked_neg::<{ MIN }>(s);
    }
    let res = parse_simd_checked_pre_u64(s)?;
    let min = -MIN as u64;
    if res > min {
        Err(AtoiSimdError::Overflow64(MIN as u64, s))
    } else if res == min {
        Ok(MIN)
    } else {
        Ok(-(res as i64))
    }
}

#[inline(always)]
pub(crate) fn parse_simd_u64(s: &[u8]) -> Result<(u64, usize), AtoiSimdError> {
    let len = s.len();
    if len < 4 {
        return parse_fb_pos::<{ u64::MAX }>(s);
    } else if len < 17 {
        return parse_simd_sse_checked(s);
    }
    let (res, len) = unsafe { parse_simd_u128(s)? };
    if len > 20 {
        return Err(AtoiSimdError::Size(len, s));
    } else if len == 20 && res > u64::MAX as u128 {
        return Err(AtoiSimdError::Overflow64(u64::MAX, s));
    }
    Ok((res as u64, len))
}

#[inline(always)]
pub(crate) fn parse_simd_checked_u64(s: &[u8]) -> Result<u64, AtoiSimdError> {
    let len = s.len();
    if len < 17 {
        return parse_simd_checked_pre_u64(s);
    } else if len > 20 {
        return Err(AtoiSimdError::Size(len, s));
    }
    let res = parse_simd_checked_u128(s)?;
    if len == 20 && res > u64::MAX as u128 {
        return Err(AtoiSimdError::Overflow64(u64::MAX, s));
    }
    Ok(res as u64)
}

#[inline(always)]
pub(crate) fn parse_simd_i64(s: &[u8]) -> Result<(i64, usize), AtoiSimdError> {
    let len = s.len();
    if len < 4 {
        return parse_fb_pos::<{ i64::MAX as u64 }>(s).map(|(v, i)| (v as i64, i));
    } else if len < 17 {
        return parse_simd_sse_checked(s).map(|(v, i)| (v as i64, i));
    }
    let (res, len) = unsafe { parse_simd_u128(s)? };
    if len > 19 {
        return Err(AtoiSimdError::Size(len, s));
    } else if len == 19 && res > i64::MAX as u128 {
        return Err(AtoiSimdError::Overflow64(i64::MAX as u64, s));
    }
    Ok((res as i64, len))
}

#[inline(always)]
pub(crate) fn parse_simd_checked_i64(s: &[u8]) -> Result<i64, AtoiSimdError> {
    let len = s.len();
    if len < 17 {
        return parse_simd_checked_pre_u64(s).map(|v| v as i64);
    } else if len > 19 {
        return Err(AtoiSimdError::Size(len, s));
    }
    let res = parse_simd_checked_u128(s)?;
    if len == 19 && res > i64::MAX as u128 {
        return Err(AtoiSimdError::Overflow64(i64::MAX as u64, s));
    }
    Ok(res as i64)
}

#[inline(always)]
pub(crate) fn parse_simd_i64_neg(s: &[u8]) -> Result<(i64, usize), AtoiSimdError> {
    let len = s.len();
    if len < 4 {
        return parse_fb_neg::<{ i64::MIN }>(s);
    } else if len < 17 {
        return parse_simd_sse_checked(s).map(|(v, i)| (-(v as i64), i));
    }
    let (res, len) = unsafe { parse_simd_u128(s)? };
    if len > 19 {
        return Err(AtoiSimdError::Size(len, s));
    } else if len == 19 {
        let min = -(i64::MIN as i128) as u128;
        if res > min {
            return Err(AtoiSimdError::Overflow64(i64::MIN as u64, s));
        } else if res == min {
            return Ok((i64::MIN, len));
        }
    }
    Ok((-(res as i64), len))
}

#[inline(always)]
pub(crate) fn parse_simd_checked_i64_neg(s: &[u8]) -> Result<i64, AtoiSimdError> {
    let len = s.len();
    if len < 17 {
        return parse_simd_checked_pre_i64_neg(s);
    } else if len > 19 {
        return Err(AtoiSimdError::Size(len, s));
    }
    let res = parse_simd_checked_u128(s)?;
    if len == 19 {
        let min = -(i64::MIN as i128) as u128;
        if res > min {
            return Err(AtoiSimdError::Overflow64(i64::MIN as u64, s));
        } else if res == min {
            return Ok(i64::MIN);
        }
    }
    Ok(-(res as i64))
}
