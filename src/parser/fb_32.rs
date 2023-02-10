use super::*;
use crate::fallback::*;

impl ParserPos<u8> for u8 {
    #[inline]
    fn atoi_simd_parse_pos(s: &[u8]) -> Result<u8, AtoiSimdError> {
        parse_fb_checked_pos::<{ u8::MAX as u64 }>(s).map(|v| v as u8)
    }

    #[inline]
    fn atoi_simd_parse_until_invalid_pos(s: &[u8]) -> Result<(u8, usize), AtoiSimdError> {
        parse_fb_pos::<{ u8::MAX as u64 }>(s).map(|(v, i)| (v as u8, i))
    }
}
impl Parser<u8> for u8 {}

impl ParserPos<i8> for i8 {
    #[inline]
    fn atoi_simd_parse_pos(s: &[u8]) -> Result<i8, AtoiSimdError> {
        parse_fb_checked_pos::<{ i8::MAX as u64 }>(s).map(|v| v as i8)
    }

    #[inline]
    fn atoi_simd_parse_until_invalid_pos(s: &[u8]) -> Result<(i8, usize), AtoiSimdError> {
        parse_fb_pos::<{ i8::MAX as u64 }>(s).map(|(v, i)| (v as i8, i))
    }
}

impl ParserNeg<i8> for i8 {
    #[inline]
    fn atoi_simd_parse_neg(s: &[u8]) -> Result<i8, AtoiSimdError> {
        parse_fb_checked_neg::<{ i8::MIN as i64 }>(s).map(|v| v as i8)
    }

    #[inline]
    fn atoi_simd_parse_until_invalid_neg(s: &[u8]) -> Result<(i8, usize), AtoiSimdError> {
        parse_fb_neg::<{ i8::MIN as i64 }>(s).map(|(v, i)| (v as i8, i))
    }
}

impl Parser<i8> for i8 {
    #[inline]
    fn atoi_simd_parse(s: &[u8]) -> Result<i8, AtoiSimdError> {
        atoi_simd_parse_signed(s)
    }

    #[inline]
    fn atoi_simd_parse_until_invalid(s: &[u8]) -> Result<(i8, usize), AtoiSimdError> {
        atoi_simd_parse_until_invalid_signed(s)
    }
}

impl ParserPos<u16> for u16 {
    #[inline]
    fn atoi_simd_parse_pos(s: &[u8]) -> Result<u16, AtoiSimdError> {
        parse_fb_checked_pos::<{ u16::MAX as u64 }>(s).map(|v| v as u16)
    }

    #[inline]
    fn atoi_simd_parse_until_invalid_pos(s: &[u8]) -> Result<(u16, usize), AtoiSimdError> {
        parse_fb_pos::<{ u16::MAX as u64 }>(s).map(|(v, i)| (v as u16, i))
    }
}
impl Parser<u16> for u16 {}

impl ParserPos<i16> for i16 {
    #[inline]
    fn atoi_simd_parse_pos(s: &[u8]) -> Result<i16, AtoiSimdError> {
        parse_fb_checked_pos::<{ i16::MAX as u64 }>(s).map(|v| v as i16)
    }

    #[inline]
    fn atoi_simd_parse_until_invalid_pos(s: &[u8]) -> Result<(i16, usize), AtoiSimdError> {
        parse_fb_pos::<{ i16::MAX as u64 }>(s).map(|(v, i)| (v as i16, i))
    }
}

impl ParserNeg<i16> for i16 {
    #[inline]
    fn atoi_simd_parse_neg(s: &[u8]) -> Result<i16, AtoiSimdError> {
        parse_fb_checked_neg::<{ i16::MIN as i64 }>(s).map(|v| v as i16)
    }

    #[inline]
    fn atoi_simd_parse_until_invalid_neg(s: &[u8]) -> Result<(i16, usize), AtoiSimdError> {
        parse_fb_neg::<{ i16::MIN as i64 }>(s).map(|(v, i)| (v as i16, i))
    }
}

impl Parser<i16> for i16 {
    #[inline]
    fn atoi_simd_parse(s: &[u8]) -> Result<i16, AtoiSimdError> {
        atoi_simd_parse_signed(s)
    }

    #[inline]
    fn atoi_simd_parse_until_invalid(s: &[u8]) -> Result<(i16, usize), AtoiSimdError> {
        atoi_simd_parse_until_invalid_signed(s)
    }
}

impl ParserPos<u32> for u32 {
    #[inline]
    fn atoi_simd_parse_pos(s: &[u8]) -> Result<u32, AtoiSimdError> {
        parse_fb_checked_pos::<{ u32::MAX as u64 }>(s).map(|v| v as u32)
    }

    #[inline]
    fn atoi_simd_parse_until_invalid_pos(s: &[u8]) -> Result<(u32, usize), AtoiSimdError> {
        parse_fb_pos::<{ u32::MAX as u64 }>(s).map(|(v, i)| (v as u32, i))
    }
}
impl Parser<u32> for u32 {}

impl ParserPos<i32> for i32 {
    #[inline]
    fn atoi_simd_parse_pos(s: &[u8]) -> Result<i32, AtoiSimdError> {
        parse_fb_checked_pos::<{ i32::MAX as u64 }>(s).map(|v| v as i32)
    }

    #[inline]
    fn atoi_simd_parse_until_invalid_pos(s: &[u8]) -> Result<(i32, usize), AtoiSimdError> {
        parse_fb_pos::<{ i32::MAX as u64 }>(s).map(|(v, i)| (v as i32, i))
    }
}

impl ParserNeg<i32> for i32 {
    #[inline]
    fn atoi_simd_parse_neg(s: &[u8]) -> Result<i32, AtoiSimdError> {
        parse_fb_checked_neg::<{ i32::MIN as i64 }>(s).map(|v| v as i32)
    }

    #[inline]
    fn atoi_simd_parse_until_invalid_neg(s: &[u8]) -> Result<(i32, usize), AtoiSimdError> {
        parse_fb_neg::<{ i32::MIN as i64 }>(s).map(|(v, i)| (v as i32, i))
    }
}

impl Parser<i32> for i32 {
    #[inline]
    fn atoi_simd_parse(s: &[u8]) -> Result<i32, AtoiSimdError> {
        atoi_simd_parse_signed(s)
    }

    #[inline]
    fn atoi_simd_parse_until_invalid(s: &[u8]) -> Result<(i32, usize), AtoiSimdError> {
        atoi_simd_parse_until_invalid_signed(s)
    }
}

#[cfg(target_pointer_width = "32")]
impl ParserPos<usize> for usize {
    #[inline]
    fn atoi_simd_parse_pos(s: &[u8]) -> Result<usize, AtoiSimdError> {
        parse_fb_checked_pos::<{ u32::MAX as u64 }>(s).map(|v| v as usize)
    }

    fn atoi_simd_parse_until_invalid_pos(s: &[u8]) -> Result<(usize, usize), AtoiSimdError> {
        parse_fb_pos::<{ usize::MAX as u64 }>(s).map(|(v, i)| (v as usize, i))
    }
}
#[cfg(target_pointer_width = "32")]
impl Parser<usize> for usize {}

#[cfg(target_pointer_width = "32")]
impl ParserPos<isize> for isize {
    #[inline]
    fn atoi_simd_parse_pos(s: &[u8]) -> Result<isize, AtoiSimdError> {
        parse_fb_checked_pos::<{ isize::MAX as u64 }>(s).map(|v| v as isize)
    }

    fn atoi_simd_parse_until_invalid_pos(s: &[u8]) -> Result<(isize, usize), AtoiSimdError> {
        parse_fb_pos::<{ isize::MAX as u64 }>(s).map(|(v, i)| (v as isize, i))
    }
}

#[cfg(target_pointer_width = "32")]
impl ParserNeg<isize> for isize {
    #[inline]
    fn atoi_simd_parse_neg(s: &[u8]) -> Result<isize, AtoiSimdError> {
        parse_fb_checked_neg::<{ isize::MIN as i64 }>(s).map(|v| v as isize)
    }

    fn atoi_simd_parse_until_invalid_neg(s: &[u8]) -> Result<(isize, usize), AtoiSimdError> {
        parse_fb_neg::<{ isize::MIN as i64 }>(s).map(|(v, i)| (v as isize, i))
    }
}

#[cfg(target_pointer_width = "32")]
impl Parser<isize> for isize {
    #[inline]
    fn atoi_simd_parse(s: &[u8]) -> Result<isize, AtoiSimdError> {
        atoi_simd_parse_signed(s)
    }

    #[inline]
    fn atoi_simd_parse_until_invalid(s: &[u8]) -> Result<(isize, usize), AtoiSimdError> {
        atoi_simd_parse_until_invalid_signed(s)
    }
}
