use crate::short::{parse_short_neg, parse_short_pos};
use crate::AtoiSimdError;

#[inline(always)]
pub(crate) fn parse_simd_checked_pre_u64(s: &[u8]) -> Result<u64, AtoiSimdError> {
    let (res, len) = if s.len() < super::SHORT {
        parse_short_pos::<{ u64::MAX }>(s)
    } else {
        super::parse_simd_16(s)
    }?;
    if len < s.len() {
        return Err(AtoiSimdError::Invalid64(res, len, s));
    }
    Ok(res)
}

#[inline(always)]
pub(crate) fn parse_simd_checked_pre_i64_neg(s: &[u8]) -> Result<i64, AtoiSimdError> {
    let (res, len) = if s.len() < super::SHORT {
        parse_short_neg::<{ i64::MIN }>(s)
    } else {
        super::parse_simd_16(s).map(|(v, l)| (-(v as i64), l))
    }?;
    if len < s.len() {
        return Err(AtoiSimdError::Invalid64(-res as u64, len, s));
    }
    Ok(res)
}

#[inline(always)]
pub(crate) fn parse_simd<const MAX: u64>(s: &[u8]) -> Result<(u64, usize), AtoiSimdError> {
    let (res, len) = super::parse_simd_16(s)?;
    if res > MAX {
        Err(AtoiSimdError::Overflow(MAX as u128, s))
    } else {
        Ok((res, len))
    }
}

#[inline(always)]
pub(crate) fn parse_simd_checked<const MAX: u64>(s: &[u8]) -> Result<u64, AtoiSimdError> {
    let res = parse_simd_checked_pre_u64(s)?;
    if res > MAX {
        Err(AtoiSimdError::Overflow(MAX as u128, s))
    } else {
        Ok(res)
    }
}

#[inline(always)]
pub(crate) fn parse_simd_neg<const MIN: i64>(s: &[u8]) -> Result<(i64, usize), AtoiSimdError> {
    debug_assert!(MIN < 0);
    let (res, len) = super::parse_simd_16(s)?;
    let min = -MIN as u64;
    if res > min {
        Err(AtoiSimdError::Overflow(min as u128, s))
    } else if res == min {
        Ok((MIN, len))
    } else {
        Ok((-(res as i64), len))
    }
}

#[inline(always)]
pub(crate) fn parse_simd_checked_neg<const MIN: i64>(s: &[u8]) -> Result<i64, AtoiSimdError> {
    debug_assert!(MIN < 0);
    let res = parse_simd_checked_pre_i64_neg(s)?;
    if res < MIN {
        Err(AtoiSimdError::Overflow(-MIN as u128, s))
    } else {
        Ok(res)
    }
}