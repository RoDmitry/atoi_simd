use crate::AtoiSimdError;

macro_rules! overflow {
    ($a:ident * 10 + $b:ident, $c:expr) => {
        match $c {
            c => $a >= c / 10 && ($a > c / 10 || $b > c % 10),
        }
    };
}

macro_rules! overflow_neg {
    ($a:ident * 10 - $b:ident, $c:expr) => {
        match $c {
            c => $a <= c / 10 && ($a < c / 10 || $b > -(c % 10)),
        }
    };
}

pub(crate) fn parse_fb_pos<const MAX: u64>(s: &[u8]) -> Result<(u64, usize), AtoiSimdError> {
    let mut i = 0;
    if s.len() == i {
        return Err(AtoiSimdError::Empty);
    }
    match s[i] {
        c @ b'0'..=b'9' => {
            let mut res = (c & 0xF) as u64;
            i += 1;
            while s.len() > i {
                match s[i] {
                    c @ b'0'..=b'9' => {
                        let digit = (c & 0xF) as u64;

                        if overflow!(res * 10 + digit, MAX) {
                            return Err(AtoiSimdError::Overflow64(MAX, s));
                        }

                        res = res * 10 + digit;
                        i += 1;
                    }
                    _ => return Err(AtoiSimdError::Invalid64(res, i)),
                }
            }
            Ok((res, i))
        }
        _ => Err(AtoiSimdError::Invalid64(0, i)),
    }
}

pub(crate) fn parse_fb_neg<const MIN: i64>(s: &[u8]) -> Result<(i64, usize), AtoiSimdError> {
    let mut i = 0;
    if s.len() == i {
        return Err(AtoiSimdError::Empty);
    }
    match s[i] {
        c @ b'0'..=b'9' => {
            let mut res = -((c & 0xF) as i64);
            i += 1;
            while s.len() > i {
                match s[i] {
                    c @ b'0'..=b'9' => {
                        let digit = (c & 0xF) as i64;

                        if overflow_neg!(res * 10 - digit, MIN) {
                            return Err(AtoiSimdError::Overflow64(MIN as u64, s));
                        }

                        res = res * 10 - digit;
                        i += 1;
                    }
                    _ => return Err(AtoiSimdError::Invalid64(res as u64, i)),
                }
            }
            Ok((res, i))
        }
        _ => Err(AtoiSimdError::Invalid64(0, i)),
    }
}

pub(crate) fn parse_fb_128_pos(s: &[u8]) -> Result<(u128, usize), AtoiSimdError> {
    let mut i = 0;
    if s.len() == i {
        return Err(AtoiSimdError::Empty);
    }
    match s[i] {
        c @ b'0'..=b'9' => {
            let mut res = (c & 0xF) as u128;
            i += 1;
            while s.len() > i {
                match s[i] {
                    c @ b'0'..=b'9' => {
                        let digit = (c & 0xF) as u128;

                        if overflow!(res * 10 + digit, u128::MAX) {
                            return Err(AtoiSimdError::Overflow128(u128::MAX, s));
                        }

                        res = res * 10 + digit;
                        i += 1;
                    }
                    _ => return Err(AtoiSimdError::Invalid128(res, i)),
                }
            }
            Ok((res, i))
        }
        _ => Err(AtoiSimdError::Invalid128(0, i)),
    }
}

pub(crate) fn parse_fb_128_neg(s: &[u8]) -> Result<(i128, usize), AtoiSimdError> {
    let mut i = 0;
    if s.len() == i {
        return Err(AtoiSimdError::Empty);
    }
    match s[i] {
        c @ b'0'..=b'9' => {
            let mut res = -((c & 0xF) as i128);
            i += 1;
            while s.len() > i {
                match s[i] {
                    c @ b'0'..=b'9' => {
                        let digit = (c & 0xF) as i128;

                        if overflow_neg!(res * 10 - digit, i128::MIN) {
                            return Err(AtoiSimdError::Overflow128(i128::MIN as u128, s));
                        }

                        res = res * 10 - digit;
                        i += 1;
                    }
                    _ => return Err(AtoiSimdError::Invalid128(res as u128, i)),
                }
            }
            Ok((res, i))
        }
        _ => Err(AtoiSimdError::Invalid128(0, i)),
    }
}

pub(crate) fn parse_fb_until_invalid_pos<const MAX: u64>(
    s: &[u8],
) -> Result<(u64, usize), AtoiSimdError> {
    match parse_fb_pos::<MAX>(s) {
        Err(AtoiSimdError::Invalid64(res, i)) => Ok((res, i)),
        res => res,
    }
}

pub(crate) fn parse_fb_until_invalid_neg<const MIN: i64>(
    s: &[u8],
) -> Result<(i64, usize), AtoiSimdError> {
    match parse_fb_neg::<MIN>(s) {
        Err(AtoiSimdError::Invalid64(res, i)) => Ok((res as i64, i)),
        res => res,
    }
}

pub(crate) fn parse_fb_until_invalid_128_pos(s: &[u8]) -> Result<(u128, usize), AtoiSimdError> {
    match parse_fb_128_pos(s) {
        Err(AtoiSimdError::Invalid128(res, i)) => Ok((res, i)),
        res => res,
    }
}

pub(crate) fn parse_fb_until_invalid_128_neg(s: &[u8]) -> Result<(i128, usize), AtoiSimdError> {
    match parse_fb_128_neg(s) {
        Err(AtoiSimdError::Invalid128(res, i)) => Ok((res as i128, i)),
        res => res,
    }
}
