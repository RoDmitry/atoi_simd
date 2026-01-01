#![allow(unused)]
pub use atoi_simd::{AtoiSimdError, Parse, ParseNeg};

pub fn parse_skipped<T: Parse>(s: &[u8]) -> Result<T, AtoiSimdError<'_>> {
    atoi_simd::parse::<_, true, true>(s)
}

pub fn parse<T: Parse + PartialEq>(s: &[u8]) -> Result<T, AtoiSimdError<'_>> {
    let res = atoi_simd::parse::<_, false, false>(s);
    let skipped = atoi_simd::parse::<_, true, true>(s);
    if res == skipped {
        res
    } else {
        panic!("mismatch");
    }
}

pub fn parse_pos<T: Parse + PartialEq>(s: &[u8]) -> Result<T, AtoiSimdError<'_>> {
    let res = atoi_simd::parse_pos::<_, false>(s);
    let skipped = atoi_simd::parse_pos::<_, true>(s);
    if res == skipped {
        res
    } else {
        panic!("mismatch");
    }
}

pub fn parse_neg<T: Parse + ParseNeg + PartialEq>(s: &[u8]) -> Result<T, AtoiSimdError<'_>> {
    let res = atoi_simd::parse_neg::<_, false>(s);
    let skipped = atoi_simd::parse_neg::<_, true>(s);
    if res == skipped {
        res
    } else {
        panic!("mismatch");
    }
}

pub fn parse_prefix<T: Parse + PartialEq>(s: &[u8]) -> Result<(T, usize), AtoiSimdError<'_>> {
    let res = atoi_simd::parse_prefix::<_, false, false>(s);
    let skipped = atoi_simd::parse_prefix::<_, true, true>(s);
    if res == skipped {
        res
    } else {
        panic!("mismatch");
    }
}

pub fn parse_prefix_pos<T: Parse + PartialEq>(s: &[u8]) -> Result<(T, usize), AtoiSimdError<'_>> {
    let res = atoi_simd::parse_prefix_pos::<_, false>(s);
    let skipped = atoi_simd::parse_prefix_pos::<_, true>(s);
    if res == skipped {
        res
    } else {
        panic!("mismatch");
    }
}

pub fn parse_prefix_neg<T: Parse + ParseNeg + PartialEq>(
    s: &[u8],
) -> Result<(T, usize), AtoiSimdError<'_>> {
    let res = atoi_simd::parse_prefix_neg::<_, false>(s);
    let skipped = atoi_simd::parse_prefix_neg::<_, true>(s);
    if res == skipped {
        res
    } else {
        panic!("mismatch");
    }
}
