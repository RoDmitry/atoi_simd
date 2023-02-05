use crate::{AtoiSimdError, ParseType};

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

pub(crate) fn parse_fb_pos<const MAX: u64>(s: &[u8]) -> Result<u64, AtoiSimdError> {
    let mut i = 0;
    if s.len() == i {
        return Err(AtoiSimdError::Empty);
    }
    match s[i] {
        c @ b'0'..=b'9' => {
            let mut significand = (c - b'0') as u64;
            i += 1;
            while s.len() > i {
                match s[i] {
                    c @ b'0'..=b'9' => {
                        let digit = (c - b'0') as u64;

                        if overflow!(significand * 10 + digit, MAX) {
                            return Err(AtoiSimdError::Overflow(ParseType::None, s));
                        }

                        significand = significand * 10 + digit;
                        i += 1;
                    }
                    _ => {
                        return Err(AtoiSimdError::Invalid(s));
                    }
                }
            }
            Ok(significand)
        }
        _ => Err(AtoiSimdError::Invalid(s)),
    }
}

pub(crate) fn parse_fb_neg<const MIN: i64>(s: &[u8]) -> Result<i64, AtoiSimdError> {
    let mut i = 0;
    if s.len() == i {
        return Err(AtoiSimdError::Empty);
    }
    match s[i] {
        c @ b'0'..=b'9' => {
            let mut significand = -((c - b'0') as i64);
            i += 1;
            while s.len() > i {
                match s[i] {
                    c @ b'0'..=b'9' => {
                        let digit = (c - b'0') as i64;

                        if overflow_neg!(significand * 10 - digit, MIN) {
                            return Err(AtoiSimdError::Overflow(ParseType::I64Neg, s));
                        }

                        significand = significand * 10 - digit;
                        i += 1;
                    }
                    _ => {
                        return Err(AtoiSimdError::Invalid(s));
                    }
                }
            }
            Ok(significand)
        }
        _ => Err(AtoiSimdError::Invalid(s)),
    }
}

pub(crate) fn parse_fb_128_pos(s: &[u8]) -> Result<u128, AtoiSimdError> {
    let mut i = 0;
    if s.len() == i {
        return Err(AtoiSimdError::Empty);
    }
    match s[i] {
        c @ b'0'..=b'9' => {
            let mut significand = (c - b'0') as u128;
            i += 1;
            while s.len() > i {
                match s[i] {
                    c @ b'0'..=b'9' => {
                        let digit = (c - b'0') as u128;

                        if overflow!(significand * 10 + digit, u128::MAX) {
                            return Err(AtoiSimdError::Overflow(ParseType::None, s));
                        }

                        significand = significand * 10 + digit;
                        i += 1;
                    }
                    _ => {
                        return Err(AtoiSimdError::Invalid(s));
                    }
                }
            }
            Ok(significand)
        }
        _ => Err(AtoiSimdError::Invalid(s)),
    }
}

pub(crate) fn parse_fb_128_neg(s: &[u8]) -> Result<i128, AtoiSimdError> {
    let mut i = 0;
    if s.len() == i {
        return Err(AtoiSimdError::Empty);
    }
    match s[i] {
        c @ b'0'..=b'9' => {
            let mut significand = -((c - b'0') as i128);
            i += 1;
            while s.len() > i {
                match s[i] {
                    c @ b'0'..=b'9' => {
                        let digit = (c - b'0') as i128;

                        if overflow_neg!(significand * 10 - digit, i128::MIN) {
                            return Err(AtoiSimdError::Overflow(ParseType::None, s));
                        }

                        significand = significand * 10 - digit;
                        i += 1;
                    }
                    _ => {
                        return Err(AtoiSimdError::Invalid(s));
                    }
                }
            }
            Ok(significand)
        }
        _ => Err(AtoiSimdError::Invalid(s)),
    }
}
