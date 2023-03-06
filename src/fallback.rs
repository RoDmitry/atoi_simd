#![allow(dead_code)]

use crate::safe_unchecked::SafeUnchecked;
use crate::AtoiSimdError;

macro_rules! overflow {
    ($curr:ident, $shift:expr, $more:ident, $max:expr) => {
        $curr >= $max / $shift && ($curr > $max / $shift || $more > $max % $shift)
    };
}

macro_rules! overflow_neg {
    ($curr:ident, $shift:expr, $more:ident, $max:expr) => {
        $curr <= $max / $shift && ($curr < $max / $shift || $more > -($max % $shift))
    };
}

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

#[inline(always)]
fn parse_8(s: &[u8]) -> Result<(u64, usize), AtoiSimdError> {
    let val: u64 = unsafe { *(s.as_ptr().cast()) };
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
    let val: u128 = unsafe { *(s.as_ptr().cast()) };
    let len = check_16(val).min(s.len());
    if len == 0 {
        return Err(AtoiSimdError::Empty);
    }
    let val = process_16(val, len);
    Ok((val, len))
}

#[inline(always)]
pub(crate) fn parse_fb_pos<const MAX: u64>(s: &[u8]) -> Result<(u64, usize), AtoiSimdError> {
    let (val, len) = parse_16(s)?;
    let val = val as u64;
    if val > MAX {
        return Err(AtoiSimdError::Overflow64(MAX, s));
    }

    Ok((val, len))
}

#[inline(always)]
pub(crate) fn parse_fb_neg<const MIN: i64>(s: &[u8]) -> Result<(i64, usize), AtoiSimdError> {
    let (val, len) = parse_16(s)?;
    let val = -(val as i64);
    if val < MIN {
        return Err(AtoiSimdError::Overflow64Neg(MIN, s));
    }

    Ok((val, len))
}

#[inline(always)]
pub(crate) fn parse_fb_64_pos<const MAX: u64, const LEN_MORE: usize>(
    s: &[u8],
) -> Result<(u64, usize), AtoiSimdError> {
    let (val, len) = parse_16(s)?;
    let val = val as u64;
    if len < 16 {
        return Ok((val, len));
    }
    let (more, len) = match parse_8(s.safe_unchecked(16..)) {
        Ok((v, l)) => (v, l),
        Err(AtoiSimdError::Empty) => return Ok((val, len)),
        Err(e) => return Err(e),
    };
    let ten_to_power_len_more = 10_u64.pow(LEN_MORE as u32);
    if len > LEN_MORE {
        return Err(AtoiSimdError::Size(len + 16, s));
    } else if len == LEN_MORE && overflow!(val, ten_to_power_len_more, more, MAX) {
        return Err(AtoiSimdError::Overflow64(MAX, s));
    }
    let res = val * 10_u64.pow(len as u32) + more;

    Ok((res, len + 16))
}

#[inline(always)]
pub(crate) fn parse_fb_64_neg(s: &[u8]) -> Result<(i64, usize), AtoiSimdError> {
    let (val, len) = parse_16(s)?;
    let val = -(val as i64);
    if len < 16 {
        return Ok((val, len));
    }
    let (more, len) = match parse_8(s.safe_unchecked(16..)) {
        Ok((v, l)) => (v as i64, l),
        Err(AtoiSimdError::Empty) => return Ok((val, len)),
        Err(e) => return Err(e),
    };
    if len > 3 {
        return Err(AtoiSimdError::Size(len + 16, s));
    } else if len == 3 && overflow_neg!(val, 1000, more, i64::MIN) {
        return Err(AtoiSimdError::Overflow64Neg(i64::MIN, s));
    }
    let res = val * 10_i64.pow(len as u32) - more;

    Ok((res, len + 16))
}

#[inline(always)]
pub(crate) fn parse_fb_128_pos<const MAX: u128>(s: &[u8]) -> Result<(u128, usize), AtoiSimdError> {
    let (mut val, len) = parse_16(s)?;
    if len < 16 {
        return Ok((val, len));
    }

    let (more, len) = match parse_16(s.safe_unchecked(16..)) {
        Ok((v, l)) => (v, l),
        Err(AtoiSimdError::Empty) => return Ok((val, len)),
        Err(e) => return Err(e),
    };
    val = val * 10_u128.pow(len as u32) + more;
    if len < 16 {
        return Ok((val, len + 16));
    }

    let (more, len) = match parse_8(s.safe_unchecked(32..)) {
        Ok((v, l)) => (v as u128, l),
        Err(AtoiSimdError::Empty) => return Ok((val, 32)),
        Err(e) => return Err(e),
    };
    if len > 7 {
        return Err(AtoiSimdError::Size(len + 32, s));
    } else if len == 7 && overflow!(val, 10_000_000, more, MAX) {
        return Err(AtoiSimdError::Overflow128(MAX, s));
    }
    let res = val * 10_u128.pow(len as u32) + more;

    Ok((res, len + 32))
}

#[inline(always)]
pub(crate) fn parse_fb_128_neg(s: &[u8]) -> Result<(i128, usize), AtoiSimdError> {
    let (val, len) = parse_16(s)?;
    let mut val = -(val as i128);
    if len < 16 {
        return Ok((val, len));
    }

    let (more, len) = match parse_16(s.safe_unchecked(16..)) {
        Ok((v, l)) => (v as i128, l),
        Err(AtoiSimdError::Empty) => return Ok((val, len)),
        Err(e) => return Err(e),
    };
    val = val * 10_i128.pow(len as u32) - more;
    if len < 16 {
        return Ok((val, len + 16));
    }

    let (more, len) = match parse_8(s.safe_unchecked(32..)) {
        Ok((v, l)) => (v as i128, l),
        Err(AtoiSimdError::Empty) => return Ok((val, 32)),
        Err(e) => return Err(e),
    };
    if len > 7 {
        return Err(AtoiSimdError::Size(len + 32, s));
    } else if len == 7 && overflow_neg!(val, 10_000_000, more, i128::MIN) {
        return Err(AtoiSimdError::Overflow128Neg(i128::MIN, s));
    }
    let res = val * 10_i128.pow(len as u32) - more;

    Ok((res, len + 32))
}

#[inline(always)]
pub(crate) fn parse_fb_checked_pos<const MAX: u64>(s: &[u8]) -> Result<u64, AtoiSimdError> {
    let (res, len) = parse_fb_pos::<MAX>(s)?;
    if len < s.len() {
        return Err(AtoiSimdError::Invalid64(res, len));
    }
    Ok(res)
}

#[inline(always)]
pub(crate) fn parse_fb_checked_neg<const MIN: i64>(s: &[u8]) -> Result<i64, AtoiSimdError> {
    let (res, len) = parse_fb_neg::<MIN>(s)?;
    if len < s.len() {
        return Err(AtoiSimdError::Invalid64(-res as u64, len));
    }
    Ok(res)
}

#[inline(always)]
pub(crate) fn parse_fb_checked_64_pos<const MAX: u64, const LEN_MORE: usize>(
    s: &[u8],
) -> Result<u64, AtoiSimdError> {
    let (res, len) = parse_fb_64_pos::<MAX, LEN_MORE>(s)?;
    if len < s.len() {
        return Err(AtoiSimdError::Invalid64(res, len));
    }
    Ok(res)
}

#[inline(always)]
pub(crate) fn parse_fb_checked_64_neg(s: &[u8]) -> Result<i64, AtoiSimdError> {
    let (res, len) = parse_fb_64_neg(s)?;
    if len < s.len() {
        return Err(AtoiSimdError::Invalid64(-res as u64, len));
    }
    Ok(res)
}

#[inline(always)]
pub(crate) fn parse_fb_checked_128_pos<const MAX: u128>(s: &[u8]) -> Result<u128, AtoiSimdError> {
    let (res, len) = parse_fb_128_pos::<MAX>(s)?;
    if len < s.len() {
        return Err(AtoiSimdError::Invalid128(res, len));
    }
    Ok(res)
}

#[inline(always)]
pub(crate) fn parse_fb_checked_128_neg(s: &[u8]) -> Result<i128, AtoiSimdError> {
    let (res, len) = parse_fb_128_neg(s)?;
    if len < s.len() {
        return Err(AtoiSimdError::Invalid128(-res as u128, len));
    }
    Ok(res)
}
