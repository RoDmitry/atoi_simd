use crate::safe_unchecked::SliceGetter;
use crate::AtoiSimdError;
use core::arch::aarch64::*;
use core::convert::TryInto;
use core::mem::transmute;

pub(crate) const SHORT: usize = 4;

const CHAR_MAX: u8 = b'9';
const CHAR_MIN: u8 = b'0';

/* macro_rules! load_lanes {
    ($s:expr, $($i:expr),*) => {
        {
            let mut res = vdupq_n_u8(0);
            // let mut res = vcombine_u8(vcreate_u8(0),vcreate_u8(0));
            $(
                // res = vld1q_lane_u8($s.get_unchecked($i) as *const u8, res, $i);
                res = vsetq_lane_u8(*$s.get_unchecked($i), res, 15 - $i);
            )*
            res
        }
    };
} */

/* #[inline(always)]
unsafe fn load(s: &[u8]) -> uint8x16_t {
    vld1q_u8(s.as_ptr())
} */

#[inline(always)]
unsafe fn load(s: &[u8]) -> uint8x16_t {
    let mut data = vdupq_n_u64(0);

    match s.len() {
        16.. => vld1q_u8(s.as_ptr()),
        15 => {
            data = vsetq_lane_u64(u64::from_le_bytes(s[0..8].try_into().unwrap()), data, 0);
            let mut data = vreinterpretq_u32_u64(data);
            data = vsetq_lane_u32(u32::from_le_bytes(s[8..12].try_into().unwrap()), data, 2);
            let mut data = vreinterpretq_u16_u32(data);
            data = vsetq_lane_u16(u16::from_le_bytes(s[12..14].try_into().unwrap()), data, 6);
            let data = vreinterpretq_u8_u16(data);
            vsetq_lane_u8(s[14], data, 14)
        }
        14 => {
            data = vsetq_lane_u64(u64::from_le_bytes(s[0..8].try_into().unwrap()), data, 0);
            let mut data = vreinterpretq_u32_u64(data);
            data = vsetq_lane_u32(u32::from_le_bytes(s[8..12].try_into().unwrap()), data, 2);
            let mut data = vreinterpretq_u16_u32(data);
            data = vsetq_lane_u16(u16::from_le_bytes(s[12..14].try_into().unwrap()), data, 6);
            vreinterpretq_u8_u16(data)
        }
        13 => {
            data = vsetq_lane_u64(u64::from_le_bytes(s[0..8].try_into().unwrap()), data, 0);
            let mut data = vreinterpretq_u32_u64(data);
            data = vsetq_lane_u32(u32::from_le_bytes(s[8..12].try_into().unwrap()), data, 2);
            let data = vreinterpretq_u8_u32(data);
            vsetq_lane_u8(s[12], data, 12)
        }
        12 => {
            data = vsetq_lane_u64(u64::from_le_bytes(s[0..8].try_into().unwrap()), data, 0);
            let mut data = vreinterpretq_u32_u64(data);
            data = vsetq_lane_u32(u32::from_le_bytes(s[8..12].try_into().unwrap()), data, 2);
            vreinterpretq_u8_u32(data)
        }
        11 => {
            data = vsetq_lane_u64(u64::from_le_bytes(s[0..8].try_into().unwrap()), data, 0);
            let mut data = vreinterpretq_u16_u64(data);
            data = vsetq_lane_u16(u16::from_le_bytes(s[8..10].try_into().unwrap()), data, 4);
            let data = vreinterpretq_u8_u16(data);
            vsetq_lane_u8(s[10], data, 10)
        }
        10 => {
            data = vsetq_lane_u64(u64::from_le_bytes(s[0..8].try_into().unwrap()), data, 0);
            let mut data = vreinterpretq_u16_u64(data);
            data = vsetq_lane_u16(u16::from_le_bytes(s[8..10].try_into().unwrap()), data, 4);
            vreinterpretq_u8_u16(data)
        }
        9 => {
            data = vsetq_lane_u64(u64::from_le_bytes(s[0..8].try_into().unwrap()), data, 0);
            let data = vreinterpretq_u8_u64(data);
            vsetq_lane_u8(s[8], data, 8)
        }
        8 => {
            data = vsetq_lane_u64(u64::from_le_bytes(s[0..8].try_into().unwrap()), data, 0);
            vreinterpretq_u8_u64(data)
        }
        7 => {
            let mut data = vreinterpretq_u32_u64(data);
            data = vsetq_lane_u32(u32::from_le_bytes(s[0..4].try_into().unwrap()), data, 0);
            let mut data = vreinterpretq_u16_u32(data);
            data = vsetq_lane_u16(u16::from_le_bytes(s[4..6].try_into().unwrap()), data, 2);
            let data = vreinterpretq_u8_u16(data);
            vsetq_lane_u8(s[6], data, 6)
        }
        6 => {
            let mut data = vreinterpretq_u32_u64(data);
            data = vsetq_lane_u32(u32::from_le_bytes(s[0..4].try_into().unwrap()), data, 0);
            let mut data = vreinterpretq_u16_u32(data);
            data = vsetq_lane_u16(u16::from_le_bytes(s[4..6].try_into().unwrap()), data, 2);
            vreinterpretq_u8_u16(data)
        }
        5 => {
            let mut data = vreinterpretq_u32_u64(data);
            data = vsetq_lane_u32(u32::from_le_bytes(s[0..4].try_into().unwrap()), data, 0);
            let data = vreinterpretq_u8_u32(data);
            vsetq_lane_u8(s[4], data, 4)
        }
        4 => {
            let mut data = vreinterpretq_u32_u64(data);
            data = vsetq_lane_u32(u32::from_le_bytes(s[0..4].try_into().unwrap()), data, 0);
            vreinterpretq_u8_u32(data)
        }
        3 => {
            let mut data = vreinterpretq_u16_u64(data);
            data = vsetq_lane_u16(u16::from_le_bytes(s[0..2].try_into().unwrap()), data, 0);
            let data = vreinterpretq_u8_u16(data);
            vsetq_lane_u8(s[2], data, 2)
        }
        2 => {
            let mut data = vreinterpretq_u16_u64(data);
            data = vsetq_lane_u16(u16::from_le_bytes(s[0..2].try_into().unwrap()), data, 0);
            vreinterpretq_u8_u16(data)
        }
        1 => {
            let data = vreinterpretq_u8_u64(data);
            vsetq_lane_u8(s[0], data, 0)
        }
        0 => vreinterpretq_u8_u64(data),
        _ => core::hint::unreachable_unchecked(),
    }
}

#[inline(always)]
unsafe fn check_len(mut chunk: uint8x16_t) -> u32 {
    let cmp_high = vld1q_dup_u8(&CHAR_MAX);
    let cmp_low = vld1q_dup_u8(&CHAR_MIN);
    let check_high = vcgtq_u8(chunk, cmp_high);
    let check_low = vcgtq_u8(cmp_low, chunk);

    chunk = vorrq_u8(check_high, check_low);

    let chunk = vreinterpretq_u16_u8(chunk);
    let chunk = vshrn_n_u16(chunk, 4);

    let chunk = vreinterpret_u64_u8(chunk);
    let res = vget_lane_u64(chunk, 0);
    res.trailing_zeros() >> 2
}

/* #[inline(always)]
unsafe fn process_mult1(mut chunk: uint8x16_t, mult1: uint8x16_t) -> uint8x8_t {
    chunk = vmulq_u8(chunk, mult1);
    let (chunk1, chunk2): (uint8x8_t, uint8x8_t) = transmute(chunk);
    vpadd_u8(chunk1, chunk2)
}

#[inline(always)]
unsafe fn process_small(chunk: uint8x16_t, mult1: uint8x16_t, mult2: uint8x8_t) -> uint16x4_t {
    let chunk = process_mult1(chunk, mult1);

    let chunk = vmull_u8(chunk, mult2);
    let (chunk1, chunk2): (uint16x4_t, uint16x4_t) = transmute(chunk);
    vpadd_u16(chunk1, chunk2)
}

#[inline(always)]
unsafe fn process_medium(
    chunk: uint8x16_t,
    mult1: uint8x16_t,
    mult2: uint8x8_t,
    mult4: uint16x4_t,
) -> uint32x2_t {
    let chunk = process_small(chunk, mult1, mult2);

    let chunk = vmull_u16(chunk, mult4);
    let (chunk1, chunk2): (uint32x2_t, uint32x2_t) = transmute(chunk);
    vpadd_u32(chunk1, chunk2)
}

#[inline(always)]
unsafe fn process_big(
    chunk: uint8x16_t,
    mult1: uint8x16_t,
    mult2: uint8x8_t,
    mult4: uint16x4_t,
    mult8: u64,
) -> u64 {
    let chunk = process_medium(chunk, mult1, mult2, mult4);

    vget_lane_u32(chunk, 0) as u64 * mult8 + vget_lane_u32(chunk, 1) as u64
} */

/*#[inline(always)]
unsafe fn parse_simd_neon(
    len: usize,
    chunk: uint8x16_t,
) -> Result<(u64, usize), AtoiSimdError<'static>> {
    let res = match len {
        0 => return Err(AtoiSimdError::Empty),
        1 => vgetq_lane_u8(chunk, 0) as u64,
        2 => {
            let mult = vld1q_u8([10, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0].as_ptr());
            let chunk = process_mult1(chunk, mult);
            transmute(chunk)
        }
        3 => {
            let chunk = process_small(
                chunk,
                vld1q_u8([10, 1, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0].as_ptr()),
                vld1_u8([10, 1, 0, 0, 0, 0, 0, 0].as_ptr()),
            );
            transmute(chunk)
        }
        4 => {
            let chunk = process_small(
                chunk,
                vld1q_u8([10, 1, 10, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0].as_ptr()),
                vld1_u8([100, 1, 0, 0, 0, 0, 0, 0].as_ptr()),
            );
            transmute(chunk)
        }
        5 => {
            let chunk = process_medium(
                chunk,
                vld1q_u8([10, 1, 10, 1, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0].as_ptr()),
                vld1_u8([100, 1, 1, 0, 0, 0, 0, 0].as_ptr()),
                vld1_u16([10, 1, 0, 0].as_ptr()),
            );
            transmute(chunk)
        }
        6 => {
            let chunk = process_medium(
                chunk,
                vld1q_u8([10, 1, 10, 1, 10, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0].as_ptr()),
                vld1_u8([100, 1, 1, 0, 0, 0, 0, 0].as_ptr()),
                vld1_u16([100, 1, 0, 0].as_ptr()),
            );
            transmute(chunk)
        }
        7 => {
            let chunk = process_medium(
                chunk,
                vld1q_u8([10, 1, 10, 1, 10, 1, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0].as_ptr()),
                vld1_u8([100, 1, 10, 1, 0, 0, 0, 0].as_ptr()),
                vld1_u16([1000, 1, 0, 0].as_ptr()),
            );
            transmute(chunk)
        }
        8 => {
            let chunk = process_medium(
                chunk,
                vld1q_u8([10, 1, 10, 1, 10, 1, 10, 1, 0, 0, 0, 0, 0, 0, 0, 0].as_ptr()),
                vld1_u8([100, 1, 100, 1, 0, 0, 0, 0].as_ptr()),
                vld1_u16([10000, 1, 0, 0].as_ptr()),
            );
            transmute(chunk)
        }
        9 => process_big(
            chunk,
            vld1q_u8([10, 1, 10, 1, 10, 1, 10, 1, 1, 0, 0, 0, 0, 0, 0, 0].as_ptr()),
            vld1_u8([100, 1, 100, 1, 1, 0, 0, 0].as_ptr()),
            vld1_u16([10000, 1, 1, 0].as_ptr()),
            10,
        ),
        10 => process_big(
            chunk,
            vld1q_u8([10, 1, 10, 1, 10, 1, 10, 1, 10, 1, 0, 0, 0, 0, 0, 0].as_ptr()),
            vld1_u8([100, 1, 100, 1, 1, 0, 0, 0].as_ptr()),
            vld1_u16([10000, 1, 1, 0].as_ptr()),
            100,
        ),
        11 => process_big(
            chunk,
            vld1q_u8([10, 1, 10, 1, 10, 1, 10, 1, 10, 1, 1, 0, 0, 0, 0, 0].as_ptr()),
            vld1_u8([100, 1, 100, 1, 10, 1, 0, 0].as_ptr()),
            vld1_u16([10000, 1, 1, 0].as_ptr()),
            1000,
        ),
        12 => process_big(
            chunk,
            vld1q_u8([10, 1, 10, 1, 10, 1, 10, 1, 10, 1, 10, 1, 0, 0, 0, 0].as_ptr()),
            vld1_u8([100, 1, 100, 1, 100, 1, 0, 0].as_ptr()),
            vld1_u16([10000, 1, 1, 0].as_ptr()),
            10_000,
        ),
        13 => process_big(
            chunk,
            vld1q_u8([10, 1, 10, 1, 10, 1, 10, 1, 10, 1, 10, 1, 1, 0, 0, 0].as_ptr()),
            vld1_u8([100, 1, 100, 1, 100, 1, 1, 0].as_ptr()),
            vld1_u16([10000, 1, 10, 1].as_ptr()),
            100_000,
        ),
        14 => process_big(
            chunk,
            vld1q_u8([10, 1, 10, 1, 10, 1, 10, 1, 10, 1, 10, 1, 10, 1, 0, 0].as_ptr()),
            vld1_u8([100, 1, 100, 1, 100, 1, 1, 0].as_ptr()),
            vld1_u16([10000, 1, 100, 1].as_ptr()),
            1_000_000,
        ),
        15 => process_big(
            chunk,
            vld1q_u8([10, 1, 10, 1, 10, 1, 10, 1, 10, 1, 10, 1, 10, 1, 1, 0].as_ptr()),
            vld1_u8([100, 1, 100, 1, 100, 1, 10, 1].as_ptr()),
            vld1_u16([10000, 1, 1000, 1].as_ptr()),
            10_000_000,
        ),
        16 => process_big(
            chunk,
            vld1q_u8([10, 1, 10, 1, 10, 1, 10, 1, 10, 1, 10, 1, 10, 1, 10, 1].as_ptr()),
            vld1_u8([100, 1, 100, 1, 100, 1, 100, 1].as_ptr()),
            vld1_u16([10000, 1, 10000, 1].as_ptr()),
            100_000_000,
        ),
        _ => {
            if cfg!(debug_assertions) {
                panic!("parse_simd_neon: wrong size {}", len);
            } else {
                core::hint::unreachable_unchecked()
            }
        }
    };
    Ok((res, len))
}*/

#[inline(always)]
unsafe fn parse_simd_neon(
    len: usize,
    mut chunk: uint8x16_t,
) -> Result<(u64, usize), AtoiSimdError<'static>> {
    chunk = match len {
        0 => return Err(AtoiSimdError::Empty),
        1 => return Ok((vgetq_lane_u8(chunk, 0) as u64, 1)),
        2 => vextq_u8(vdupq_n_u8(0), chunk, 2),
        3 => vextq_u8(vdupq_n_u8(0), chunk, 3),
        4 => vextq_u8(vdupq_n_u8(0), chunk, 4),
        5 => vextq_u8(vdupq_n_u8(0), chunk, 5),
        6 => vextq_u8(vdupq_n_u8(0), chunk, 6),
        7 => vextq_u8(vdupq_n_u8(0), chunk, 7),
        8 => vextq_u8(vdupq_n_u8(0), chunk, 8),
        9 => vextq_u8(vdupq_n_u8(0), chunk, 9),
        10 => vextq_u8(vdupq_n_u8(0), chunk, 10),
        11 => vextq_u8(vdupq_n_u8(0), chunk, 11),
        12 => vextq_u8(vdupq_n_u8(0), chunk, 12),
        13 => vextq_u8(vdupq_n_u8(0), chunk, 13),
        14 => vextq_u8(vdupq_n_u8(0), chunk, 14),
        15 => vextq_u8(vdupq_n_u8(0), chunk, 15),
        16 => chunk,
        _ => {
            if cfg!(debug_assertions) {
                panic!("parse_simd_neon: wrong size {}", len);
            } else {
                core::hint::unreachable_unchecked()
            }
        }
    };

    /* slower
    let (sum, chunk) = odd_even_8(chunk);
    let mult = vdup_n_u8(10);
    let chunk = vmla_u8(sum, chunk, mult);

    let (sum, chunk) = odd_even_small_16(chunk);
    let mult = vdup_n_u16(100);
    let chunk = vmla_u16(sum, chunk, mult);

    let (sum, chunk) = odd_even_small_32(chunk);
    let mult = vdup_n_u32(10000);
    let chunk = vmla_u32(sum, chunk, mult);*/

    chunk = vmulq_u8(
        chunk,
        vld1q_u8([10, 1, 10, 1, 10, 1, 10, 1, 10, 1, 10, 1, 10, 1, 10, 1].as_ptr()),
    );
    let (chunk1, chunk2): (uint8x8_t, uint8x8_t) = transmute(chunk);
    let chunk = vpadd_u8(chunk1, chunk2);

    let chunk = vmull_u8(chunk, vld1_u8([100, 1, 100, 1, 100, 1, 100, 1].as_ptr()));
    let (chunk1, chunk2): (uint16x4_t, uint16x4_t) = transmute(chunk);
    let chunk = vpadd_u16(chunk1, chunk2);

    let chunk = vmull_u16(chunk, vld1_u16([10000, 1, 10000, 1].as_ptr()));
    let (chunk1, chunk2): (uint32x2_t, uint32x2_t) = transmute(chunk);
    let chunk = vpadd_u32(chunk1, chunk2);

    let res = (vget_lane_u32(chunk, 0) as u64) * 100_000_000 + (vget_lane_u32(chunk, 1) as u64);

    Ok((res, len))
}

#[inline(always)]
unsafe fn simd_neon_len(s: &[u8]) -> Result<(usize, uint8x16_t), AtoiSimdError> {
    let mut chunk = load(s);
    let len = check_len(chunk) as usize;

    chunk = vandq_u8(chunk, vdupq_n_u8(0xF));

    Ok((len, chunk))
}

#[inline(always)]
pub(crate) fn parse_simd_16(s: &[u8]) -> Result<(u64, usize), AtoiSimdError> {
    unsafe {
        let (len, chunk) = simd_neon_len(s)?;
        parse_simd_neon(len, chunk)
    }
}

#[inline(always)]
unsafe fn odd_even_8(chunk: uint8x16_t) -> (uint8x8_t, uint8x8_t) {
    let chunk = vreinterpretq_u16_u8(chunk);
    let sum = vshrn_n_u16::<8>(chunk);
    let chunk = vmovn_u16(chunk);

    (sum, chunk)
}

#[inline(always)]
unsafe fn odd_even_16(chunk: uint8x16_t) -> (uint16x8_t, uint8x8_t) {
    let chunk = vreinterpretq_u16_u8(chunk);
    let sum = vshrq_n_u16::<8>(chunk);
    let chunk = vmovn_u16(chunk);

    (sum, chunk)
}

#[inline(always)]
unsafe fn odd_even_32(chunk: uint16x8_t) -> (uint32x4_t, uint16x4_t) {
    let chunk = vreinterpretq_u32_u16(chunk);
    let sum = vshrq_n_u32::<16>(chunk);
    let chunk = vmovn_u32(chunk);

    (sum, chunk)
}

#[inline(always)]
unsafe fn odd_even_64(chunk: int32x4_t) -> (uint64x2_t, uint32x2_t) {
    let chunk = vreinterpretq_u64_s32(chunk);
    let sum = vshrq_n_u64::<32>(chunk);
    let chunk = vmovn_u64(chunk);

    (sum, chunk)
}

/* #[inline(always)]
unsafe fn odd_even_small_16(chunk: uint8x8_t) -> (uint16x4_t, uint16x4_t) {
    let chunk = vreinterpret_u16_u8(chunk);
    let sum = vshr_n_u16::<8>(chunk);
    let chunk = vand_u16(chunk, vdup_n_u16(0xFF));

    (sum, chunk)
}

#[inline(always)]
unsafe fn odd_even_small_32(chunk: uint16x4_t) -> (uint32x2_t, uint32x2_t) {
    let chunk = vreinterpret_u32_u16(chunk);
    let sum = vshr_n_u32::<16>(chunk);
    let chunk = vand_u32(chunk, vdup_n_u32(0xFFFF));

    (sum, chunk)
} */

/* slower
#[inline(always)]
unsafe fn parse_simd_extra(s: &[u8], mult: uint8x16_t) -> Result<(u64, usize, u32), AtoiSimdError> {
    let mut chunk = load(s);
    let len = check_len(chunk) as usize;
    chunk = vandq_u8(chunk, mult);
    let (res, mult) = match len {
        0 => (0, 0),
        1 => (vgetq_lane_u8(chunk, 0) as u64, 10),
        2 => {
            let mult10 = vld1q_u8([10, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0].as_ptr());
            let chunk = process_mult1(chunk, mult10);
            (transmute(chunk), 100)
        }
        3 => {
            let chunk = process_small(
                chunk,
                vld1q_u8([10, 1, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0].as_ptr()),
                vld1_u8([10, 1, 0, 0, 0, 0, 0, 0].as_ptr()),
            );
            (transmute(chunk), 1000)
        }
        4 => {
            let chunk = process_small(
                chunk,
                vld1q_u8([10, 1, 10, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0].as_ptr()),
                vld1_u8([100, 1, 0, 0, 0, 0, 0, 0].as_ptr()),
            );
            (transmute(chunk), 10_000)
        }
        5 => {
            let chunk = process_medium(
                chunk,
                vld1q_u8([10, 1, 10, 1, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0].as_ptr()),
                vld1_u8([100, 1, 1, 0, 0, 0, 0, 0].as_ptr()),
                vld1_u16([10, 1, 0, 0].as_ptr()),
            );
            (transmute(chunk), 100_000)
        }
        6 => {
            let chunk = process_medium(
                chunk,
                vld1q_u8([10, 1, 10, 1, 10, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0].as_ptr()),
                vld1_u8([100, 1, 1, 0, 0, 0, 0, 0].as_ptr()),
                vld1_u16([100, 1, 0, 0].as_ptr()),
            );
            (transmute(chunk), 1_000_000)
        }
        7 => {
            let chunk = process_medium(
                chunk,
                vld1q_u8([10, 1, 10, 1, 10, 1, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0].as_ptr()),
                vld1_u8([100, 1, 10, 1, 0, 0, 0, 0].as_ptr()),
                vld1_u16([1000, 1, 0, 0].as_ptr()),
            );
            (transmute(chunk), 10_000_000)
        }
        s_len => return Err(AtoiSimdError::Size(s_len, s)),
    };

    Ok((res, len, mult))
}*/

/* slower
#[inline(always)]
unsafe fn parse_simd_extra(s: &[u8], mult: uint8x16_t) -> Result<(u64, usize, u32), AtoiSimdError> {
    let mut chunk = load(s);
    let len = check_len(chunk) as usize;
    chunk = vandq_u8(chunk, mult);
    let (mult1, mult2, mult4, extra_mult) = match len {
        0 => return Ok((0, 0, 0)),
        1 => return Ok((vgetq_lane_u8(chunk, 0) as u64, 1, 10)),
        2 => (
            vld1q_u8([10, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0].as_ptr()),
            vld1_u8([1, 0, 0, 0, 0, 0, 0, 0].as_ptr()),
            vld1_u16([1, 0, 0, 0].as_ptr()),
            100,
        ),
        3 => (
            vld1q_u8([10, 1, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0].as_ptr()),
            vld1_u8([10, 1, 0, 0, 0, 0, 0, 0].as_ptr()),
            vld1_u16([1, 0, 0, 0].as_ptr()),
            1000,
        ),
        4 => (
            vld1q_u8([10, 1, 10, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0].as_ptr()),
            vld1_u8([100, 1, 0, 0, 0, 0, 0, 0].as_ptr()),
            vld1_u16([1, 1, 0, 0].as_ptr()),
            10_000,
        ),
        5 => (
            vld1q_u8([10, 1, 10, 1, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0].as_ptr()),
            vld1_u8([100, 1, 1, 0, 0, 0, 0, 0].as_ptr()),
            vld1_u16([10, 1, 0, 0].as_ptr()),
            100_000,
        ),
        6 => (
            vld1q_u8([10, 1, 10, 1, 10, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0].as_ptr()),
            vld1_u8([100, 1, 1, 0, 0, 0, 0, 0].as_ptr()),
            vld1_u16([100, 1, 0, 0].as_ptr()),
            1_000_000,
        ),
        7 => (
            vld1q_u8([10, 1, 10, 1, 10, 1, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0].as_ptr()),
            vld1_u8([100, 1, 10, 1, 0, 0, 0, 0].as_ptr()),
            vld1_u16([1000, 1, 0, 0].as_ptr()),
            10_000_000,
        ),
        s_len => return Err(AtoiSimdError::Size(s_len, s)),
    };

    let chunk = process_medium(chunk, mult1, mult2, mult4);
    let res = transmute(chunk);
    // println!("{} {}", res, extra_mult);

    Ok((res, len, extra_mult))
} */

#[inline(always)]
unsafe fn parse_simd_extra<'a>(
    s: &'a [u8],
    mult: uint8x16_t,
    chunk1: &mut uint8x16_t,
    chunk2: &mut uint8x16_t,
) -> Result<(u128, usize), AtoiSimdError<'a>> {
    let mut chunk3 = load(s);
    let mut len = check_len(chunk3) as usize;
    chunk3 = vandq_u8(chunk3, mult);
    match len {
        0 => {} //return Ok((0, 16)),
        1 => {
            let tmp = vextq_u8(*chunk1, *chunk2, 1);
            *chunk2 = vextq_u8(*chunk2, chunk3, 1);
            chunk3 = vextq_u8(vdupq_n_u8(0), *chunk1, 1);
            *chunk1 = tmp;
        }
        2 => {
            let tmp = vextq_u8(*chunk1, *chunk2, 2);
            *chunk2 = vextq_u8(*chunk2, chunk3, 2);
            chunk3 = vextq_u8(vdupq_n_u8(0), *chunk1, 2);
            *chunk1 = tmp;
        }
        3 => {
            let tmp = vextq_u8(*chunk1, *chunk2, 3);
            *chunk2 = vextq_u8(*chunk2, chunk3, 3);
            chunk3 = vextq_u8(vdupq_n_u8(0), *chunk1, 3);
            *chunk1 = tmp;
        }
        4 => {
            let tmp = vextq_u8(*chunk1, *chunk2, 4);
            *chunk2 = vextq_u8(*chunk2, chunk3, 4);
            chunk3 = vextq_u8(vdupq_n_u8(0), *chunk1, 4);
            *chunk1 = tmp;
        }
        5 => {
            let tmp = vextq_u8(*chunk1, *chunk2, 5);
            *chunk2 = vextq_u8(*chunk2, chunk3, 5);
            chunk3 = vextq_u8(vdupq_n_u8(0), *chunk1, 5);
            *chunk1 = tmp;
        }
        6 => {
            let tmp = vextq_u8(*chunk1, *chunk2, 6);
            *chunk2 = vextq_u8(*chunk2, chunk3, 6);
            chunk3 = vextq_u8(vdupq_n_u8(0), *chunk1, 6);
            *chunk1 = tmp;
        }
        7 => {
            let tmp = vextq_u8(*chunk1, *chunk2, 7);
            *chunk2 = vextq_u8(*chunk2, chunk3, 7);
            chunk3 = vextq_u8(vdupq_n_u8(0), *chunk1, 7);
            *chunk1 = tmp;
        }
        s_len => return Err(AtoiSimdError::Size(s_len, s)),
    }
    len += 16;

    chunk3 = vmulq_u8(
        chunk3,
        vld1q_u8([10, 1, 10, 1, 10, 1, 10, 1, 10, 1, 10, 1, 10, 1, 10, 1].as_ptr()),
    );
    let (chunk1, chunk2): (uint8x8_t, uint8x8_t) = transmute(chunk3);
    let chunk3 = vpadd_u8(chunk1, chunk2);

    let chunk3 = vmull_u8(chunk3, vld1_u8([100, 1, 100, 1, 100, 1, 100, 1].as_ptr()));
    let (chunk1, chunk2): (uint16x4_t, uint16x4_t) = transmute(chunk3);
    let chunk3 = vpadd_u16(chunk1, chunk2);

    let chunk3 = vmull_u16(chunk3, vld1_u16([10000, 1, 10000, 1].as_ptr()));
    let (chunk1, chunk2): (uint32x2_t, uint32x2_t) = transmute(chunk3);
    let chunk3 = vpadd_u32(chunk1, chunk2);

    let res = (vget_lane_u32(chunk3, 1) as u128)
        .checked_mul(100_000_000_000_000_000_000_000_000_000_000)
        .ok_or(AtoiSimdError::Overflow(u128::MAX, s))?;

    Ok((res, len))
}

#[inline(always)]
pub(crate) fn parse_simd_u128(s: &[u8]) -> Result<(u128, usize), AtoiSimdError> {
    unsafe {
        let mut chunk1 = load(s);
        let mut len = check_len(chunk1) as usize;

        let mult = vdupq_n_u8(0xF);
        chunk1 = vandq_u8(chunk1, mult);

        if len < 16 {
            return parse_simd_neon(len, chunk1).map(|(v, l)| (v as u128, l));
        };

        let mut chunk2 = load(s.get_safe_unchecked(16..));

        len = check_len(chunk2) as usize;
        chunk2 = vandq_u8(chunk2, mult);
        let mut extra = 0;
        // let mut extra_mult = 0;
        match len {
            0 => return parse_simd_neon(16, chunk1).map(|(v, l)| (v as u128, l)),
            1 => {
                chunk2 = vextq_u8(chunk1, chunk2, 1);
                chunk1 = vextq_u8(vdupq_n_u8(0), chunk1, 1);
            }
            2 => {
                chunk2 = vextq_u8(chunk1, chunk2, 2);
                chunk1 = vextq_u8(vdupq_n_u8(0), chunk1, 2);
            }
            3 => {
                chunk2 = vextq_u8(chunk1, chunk2, 3);
                chunk1 = vextq_u8(vdupq_n_u8(0), chunk1, 3);
            }
            4 => {
                chunk2 = vextq_u8(chunk1, chunk2, 4);
                chunk1 = vextq_u8(vdupq_n_u8(0), chunk1, 4);
            }
            5 => {
                chunk2 = vextq_u8(chunk1, chunk2, 5);
                chunk1 = vextq_u8(vdupq_n_u8(0), chunk1, 5);
            }
            6 => {
                chunk2 = vextq_u8(chunk1, chunk2, 6);
                chunk1 = vextq_u8(vdupq_n_u8(0), chunk1, 6);
            }
            7 => {
                chunk2 = vextq_u8(chunk1, chunk2, 7);
                chunk1 = vextq_u8(vdupq_n_u8(0), chunk1, 7);
            }
            8 => {
                chunk2 = vextq_u8(chunk1, chunk2, 8);
                chunk1 = vextq_u8(vdupq_n_u8(0), chunk1, 8);
            }
            9 => {
                chunk2 = vextq_u8(chunk1, chunk2, 9);
                chunk1 = vextq_u8(vdupq_n_u8(0), chunk1, 9);
            }
            10 => {
                chunk2 = vextq_u8(chunk1, chunk2, 10);
                chunk1 = vextq_u8(vdupq_n_u8(0), chunk1, 10);
            }
            11 => {
                chunk2 = vextq_u8(chunk1, chunk2, 11);
                chunk1 = vextq_u8(vdupq_n_u8(0), chunk1, 11);
            }
            12 => {
                chunk2 = vextq_u8(chunk1, chunk2, 12);
                chunk1 = vextq_u8(vdupq_n_u8(0), chunk1, 12);
            }
            13 => {
                chunk2 = vextq_u8(chunk1, chunk2, 13);
                chunk1 = vextq_u8(vdupq_n_u8(0), chunk1, 13);
            }
            14 => {
                chunk2 = vextq_u8(chunk1, chunk2, 14);
                chunk1 = vextq_u8(vdupq_n_u8(0), chunk1, 14);
            }
            15 => {
                chunk2 = vextq_u8(chunk1, chunk2, 15);
                chunk1 = vextq_u8(vdupq_n_u8(0), chunk1, 15);
            }
            16 => {
                // (extra, len, extra_mult) = parse_simd_extra(s.get_safe_unchecked(32..), mult)?;
                // len += 16;
                (extra, len) =
                    parse_simd_extra(s.get_safe_unchecked(32..), mult, &mut chunk1, &mut chunk2)?;
            }
            _ => core::hint::unreachable_unchecked(),
        };

        let (sum1, chunk1) = odd_even_8(chunk1);
        let (sum2, chunk2) = odd_even_8(chunk2);
        let mult = vdupq_n_u8(10);
        let chunk = vmlaq_u8(vcombine_u8(sum1, sum2), vcombine_u8(chunk1, chunk2), mult);

        let (sum, chunk) = odd_even_16(chunk);
        let mult = vdup_n_u8(100);
        let chunk = vmlal_u8(sum, chunk, mult);

        let (sum, chunk) = odd_even_32(chunk);
        let chunk = vqdmlal_n_s16(
            vreinterpretq_s32_u32(sum),
            vreinterpret_s16_u16(chunk),
            5000, // because it's doubling
        );

        let (sum, chunk) = odd_even_64(chunk);
        let chunk = vqdmlal_n_s32(
            vreinterpretq_s64_u64(sum),
            vreinterpret_s32_u32(chunk),
            50_000_000, // because it's doubling
        );

        let mut res = vgetq_lane_s64(chunk, 0) as u128 * 10_000_000_000_000_000
            + vgetq_lane_s64(chunk, 1) as u128;
        if extra > 0 {
            res = res
                // .checked_mul(extra_mult as u128)
                // .ok_or(AtoiSimdError::Overflow(u128::MAX, s))?
                .checked_add(extra as u128)
                .ok_or(AtoiSimdError::Overflow(u128::MAX, s))?;
        }

        Ok((res, len + 16))
    }
}
