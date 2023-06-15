#![allow(dead_code)]

use crate::safe_unchecked::SafeUnchecked;
use crate::short::{parse_short_neg, parse_short_pos};
use crate::AtoiSimdError;
use core::ptr::read_unaligned;

macro_rules! overflow {
    ($curr:ident, $shift:expr, $more:ident, $max:expr) => {
        $curr >= $max / $shift && ($curr > $max / $shift || $more > $max % $shift)
    };
}

/* #[inline(always)]
fn check_4(val: u32) -> usize {
    ((((val & 0xF0F0_F0F0) | (((val.wrapping_add(0x0606_0606)) & 0xF0F0_F0F0) >> 4)) ^ 0x3333_3333)
        .trailing_zeros()
        >> 3) as usize // same as divide by 8 (drops extra bits from right)
} */

#[inline(always)]
fn check_8(val: u64) -> usize {
    ((((val & 0xF0F0_F0F0_F0F0_F0F0)
        | (((val.wrapping_add(0x0606_0606_0606_0606)) & 0xF0F0_F0F0_F0F0_F0F0) >> 4))
        ^ 0x3333_3333_3333_3333)
        .trailing_zeros()
        >> 3) as usize // same as divide by 8 (drops extra bits from right)
}

#[inline(always)]
fn check_16(val: u128) -> usize {
    ((((val & 0xF0F0_F0F0_F0F0_F0F0_F0F0_F0F0_F0F0_F0F0)
        | (((val.wrapping_add(0x0606_0606_0606_0606_0606_0606_0606_0606))
            & 0xF0F0_F0F0_F0F0_F0F0_F0F0_F0F0_F0F0_F0F0)
            >> 4))
        ^ 0x3333_3333_3333_3333_3333_3333_3333_3333)
        .trailing_zeros()
        >> 3) as usize // same as divide by 8 (drops extra bits from right)
}

#[inline(always)]
fn process_4(mut val: u32, len: usize) -> u32 {
    val <<= 32_usize.saturating_sub(len << 3); // << 3 - same as mult by 8
    val = (val & 0x0F0F_0F0F).wrapping_mul(0xA01) >> 8;
    (val & 0x00FF_00FF).wrapping_mul(0x64_0001) >> 16
}

#[inline(always)]
fn process_8(mut val: u64, len: usize) -> u64 {
    val <<= 64_usize.saturating_sub(len << 3); // << 3 - same as mult by 8
    val = (val & 0x0F0F_0F0F_0F0F_0F0F).wrapping_mul(0xA01) >> 8;
    val = (val & 0x00FF_00FF_00FF_00FF).wrapping_mul(0x64_0001) >> 16;
    (val & 0x0000_FFFF_0000_FFFF).wrapping_mul(0x2710_0000_0001) >> 32
}

#[inline(always)]
fn process_16(mut val: u128, len: usize) -> u128 {
    val <<= 128_usize.saturating_sub(len << 3); // << 3 - same as mult by 8
    val = (val & 0x0F0F_0F0F_0F0F_0F0F_0F0F_0F0F_0F0F_0F0F).wrapping_mul(0xA01) >> 8;
    val = (val & 0x00FF_00FF_00FF_00FF_00FF_00FF_00FF_00FF).wrapping_mul(0x64_0001) >> 16;
    val = (val & 0x0000_FFFF_0000_FFFF_0000_FFFF_0000_FFFF).wrapping_mul(0x2710_0000_0001) >> 32;
    (val & 0x0000_0000_FFFF_FFFF_0000_0000_FFFF_FFFF).wrapping_mul(0x5F5_E100_0000_0000_0000_0001)
        >> 64
}

/* #[inline(always)]
fn parse_4(s: &[u8]) -> Result<(u32, usize), AtoiSimdError> {
    let val: u32 = unsafe { read_unaligned(s.as_ptr().cast()) };
    // let val: u64 = unsafe { *core::mem::transmute_copy::<&[u8], *const u64>(&s) };
    let len = check_4(val).min(s.len());
    if len == 0 {
        return Err(AtoiSimdError::Empty);
    }
    let val = process_4(val, len);
    Ok((val, len))
} */

#[inline(always)]
fn parse_8(s: &[u8]) -> Result<(u64, usize), AtoiSimdError> {
    let val: u64 = unsafe { read_unaligned(s.as_ptr().cast()) };
    // let val: u64 = unsafe { *core::mem::transmute_copy::<&[u8], *const u64>(&s) };
    let len = check_8(val).min(s.len());
    if len == 0 {
        return Err(AtoiSimdError::Empty);
    }
    let val = process_8(val, len);
    Ok((val, len))
}

#[inline(always)]
pub(crate) fn parse_16(s: &[u8]) -> Result<(u128, usize), AtoiSimdError> {
    let val: u128 = unsafe { read_unaligned(s.as_ptr().cast()) };
    let len = check_16(val).min(s.len());
    if len == 0 {
        return Err(AtoiSimdError::Empty);
    }
    let val = process_16(val, len);
    Ok((val, len))
}

#[inline(always)]
pub(crate) fn parse_fb_pos<const MAX: u64>(s: &[u8]) -> Result<(u64, usize), AtoiSimdError> {
    let (val, len) = if MAX < 1_000_000_000 {
        parse_short_pos::<MAX>(s)
    } else {
        parse_16(s).map(|(v, l)| (v as u64, l))
    }?;
    if val > MAX {
        return Err(AtoiSimdError::Overflow(MAX as u128, s));
    }

    Ok((val, len))
}

#[inline(always)]
pub(crate) fn parse_fb_neg<const MIN: i64>(s: &[u8]) -> Result<(i64, usize), AtoiSimdError> {
    debug_assert!(MIN < 0);
    let (val, len) = if MIN > -1_000_000_000 {
        parse_short_neg::<MIN>(s)
    } else {
        parse_16(s).map(|(v, l)| (-(v as i64), l))
    }?;
    if val < MIN {
        return Err(AtoiSimdError::Overflow(-MIN as u128, s));
    }

    Ok((val, len))
}

#[inline(always)]
pub(crate) fn parse_fb_64_pos<const MAX: u64, const LEN_MORE: usize>(
    s: &[u8],
) -> Result<(u64, usize), AtoiSimdError> {
    if s.len() < 9 {
        return parse_short_pos::<MAX>(s);
    }
    let val: u64 = unsafe { read_unaligned(s.as_ptr().cast()) };
    let mut len = check_8(val).min(s.len());
    let val = match len {
        0 => return Err(AtoiSimdError::Empty),
        1 => return Ok((val & 0xF, len)),
        2..=4 => return Ok((process_4(val as u32, len) as u64, len)),
        5..=7 => return Ok((process_8(val, len), len)),
        8 => {
            let mut val: u64 = unsafe { read_unaligned(s.get_safe_unchecked(8..).as_ptr().cast()) };
            len = (check_8(val) + 8).min(s.len());
            val = process_16(unsafe { read_unaligned(s.as_ptr().cast()) }, len) as u64;
            if len < 16 {
                return Ok((val, len));
            }
            val
        }
        _ => unsafe { core::hint::unreachable_unchecked() },
    };

    let (more, len) = match parse_8(s.get_safe_unchecked(16..)) {
        Ok((v, l)) => (v, l),
        Err(AtoiSimdError::Empty) => return Ok((val, len)),
        Err(e) => return Err(e),
    };
    if len > LEN_MORE {
        return Err(AtoiSimdError::Size(len + 16, s));
    }
    let shift = 10_u64.pow(len as u32);
    if len == LEN_MORE && overflow!(val, shift, more, MAX) {
        return Err(AtoiSimdError::Overflow(MAX as u128, s));
    }
    let res = val * shift + more;

    Ok((res, len + 16))
}

#[inline(always)]
pub(crate) fn parse_fb_64_neg(s: &[u8]) -> Result<(i64, usize), AtoiSimdError> {
    let (val, len) = parse_fb_64_pos::<{ i64::MAX as u64 + 1 }, 3>(s)?;
    if val > i64::MAX as u64 {
        return Ok((i64::MIN, len));
    }

    Ok((-(val as i64), len))
}

#[inline(always)]
pub(crate) fn parse_fb_128_pos<const MAX: u128>(s: &[u8]) -> Result<(u128, usize), AtoiSimdError> {
    if s.len() < 9 {
        return parse_short_pos::<{ u64::MAX }>(s).map(|(v, l)| (v as u128, l));
    }
    let val: u64 = unsafe { read_unaligned(s.as_ptr().cast()) };
    let mut len = check_8(val).min(s.len());
    let mut val = match len {
        0 => return Err(AtoiSimdError::Empty),
        1 => return Ok(((val & 0xF) as u128, len)),
        2..=4 => return Ok((process_4(val as u32, len) as u128, len)),
        5..=7 => return Ok((process_8(val, len) as u128, len)),
        8 => {
            let val: u64 = unsafe { read_unaligned(s.get_safe_unchecked(8..).as_ptr().cast()) };
            len = (check_8(val) + 8).min(s.len());
            let val = process_16(unsafe { read_unaligned(s.as_ptr().cast()) }, len);
            if len < 16 {
                return Ok((val, len));
            }
            val
        }
        _ => unsafe { core::hint::unreachable_unchecked() },
    };

    // can be optimized parsing by 64 bits as above
    let (more, len) = match parse_16(s.get_safe_unchecked(16..)) {
        Ok((v, l)) => (v, l),
        Err(AtoiSimdError::Empty) => return Ok((val, len)),
        Err(e) => return Err(e),
    };
    val = val * 10_u128.pow(len as u32) + more;
    if len < 16 {
        return Ok((val, len + 16));
    }

    let (more, len) = match parse_8(s.get_safe_unchecked(32..)) {
        Ok((v, l)) => (v as u128, l),
        Err(AtoiSimdError::Empty) => return Ok((val, 32)),
        Err(e) => return Err(e),
    };
    if len > 7 {
        return Err(AtoiSimdError::Size(len + 32, s));
    } else if len == 7 && overflow!(val, 10_000_000, more, MAX) {
        return Err(AtoiSimdError::Overflow(MAX, s));
    }
    let res = val * 10_u128.pow(len as u32) + more;

    Ok((res, len + 32))
}

#[inline(always)]
pub(crate) fn parse_fb_128_neg(s: &[u8]) -> Result<(i128, usize), AtoiSimdError> {
    let (val, len) = parse_fb_128_pos::<{ i128::MAX as u128 + 1 }>(s)?;
    if val > i128::MAX as u128 {
        return Ok((i128::MIN, len));
    }

    Ok((-(val as i128), len))
}

#[inline(always)]
pub(crate) fn parse_fb_checked_pos<const MAX: u64>(s: &[u8]) -> Result<u64, AtoiSimdError> {
    let (res, len) = parse_fb_pos::<MAX>(s)?;
    if len < s.len() {
        return Err(AtoiSimdError::Invalid64(res, len, s));
    }
    Ok(res)
}

#[inline(always)]
pub(crate) fn parse_fb_checked_neg<const MIN: i64>(s: &[u8]) -> Result<i64, AtoiSimdError> {
    debug_assert!(MIN < 0);
    let (res, len) = parse_fb_neg::<MIN>(s)?;
    if len < s.len() {
        return Err(AtoiSimdError::Invalid64(-res as u64, len, s));
    }
    Ok(res)
}

#[inline(always)]
pub(crate) fn parse_fb_checked_64_pos<const MAX: u64, const LEN_MORE: usize>(
    s: &[u8],
) -> Result<u64, AtoiSimdError> {
    let (res, len) = parse_fb_64_pos::<MAX, LEN_MORE>(s)?;
    if len < s.len() {
        return Err(AtoiSimdError::Invalid64(res, len, s));
    }
    Ok(res)
}

#[inline(always)]
pub(crate) fn parse_fb_checked_64_neg(s: &[u8]) -> Result<i64, AtoiSimdError> {
    let (res, len) = parse_fb_64_neg(s)?;
    if len < s.len() {
        return Err(AtoiSimdError::Invalid64(-res as u64, len, s));
    }
    Ok(res)
}

#[inline(always)]
pub(crate) fn parse_fb_checked_128_pos<const MAX: u128>(s: &[u8]) -> Result<u128, AtoiSimdError> {
    let (res, len) = parse_fb_128_pos::<MAX>(s)?;
    if len < s.len() {
        return Err(AtoiSimdError::Invalid128(res, len, s));
    }
    Ok(res)
}

#[inline(always)]
pub(crate) fn parse_fb_checked_128_neg(s: &[u8]) -> Result<i128, AtoiSimdError> {
    let (res, len) = parse_fb_128_neg(s)?;
    if len < s.len() {
        return Err(AtoiSimdError::Invalid128(-res as u128, len, s));
    }
    Ok(res)
}

/* #[inline(always)]
pub(crate) fn parse_short_pos<const MAX: u64>(s: &[u8]) -> Result<(u64, usize), AtoiSimdError> {
    let (val, len) = parse_4(s)?;
    let val = val as u64;
    if val > MAX {
        return Err(AtoiSimdError::Overflow(MAX as u128, s));
    }

    Ok((val, len))
}

#[inline(always)]
pub(crate) fn parse_short_neg<const MIN: i64>(s: &[u8]) -> Result<(i64, usize), AtoiSimdError> {
    debug_assert!(MIN < 0);
    let (val, len) = parse_4(s)?;
    let val = -(val as i64);
    if val < MIN {
        return Err(AtoiSimdError::Overflow(-MIN as u128, s));
    }

    Ok((val, len))
}

#[inline(always)]
pub(crate) fn parse_short_checked_neg<const MIN: i64>(s: &[u8]) -> Result<i64, AtoiSimdError> {
    debug_assert!(MIN < 0);
    let (res, len) = parse_short_neg::<MIN>(s)?;
    if len < s.len() {
        return Err(AtoiSimdError::Invalid64(-res as u64, len));
    }

    Ok(res)
} */
