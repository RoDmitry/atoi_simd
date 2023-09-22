use super::*;
// use crate::fallback::*;
use arrayvec::ArrayString;
use core::cmp::PartialEq;
use core::fmt::Debug;
use core::str::FromStr;
use numtoa::NumToA;

const INVALID_CHARS: [char; 6] = ['/', ':', '\0', '\x7f', '!', 'a'];

#[test]
fn roundtrip_all_u8() {
    let mut buf = [0; 64];
    for i in u8::MIN..=u8::MAX {
        let input = i.numtoa(10, &mut buf);
        let parsed = crate::parse::<u8>(input).expect("Failed to parse valid input!");
        assert_eq!(i, parsed);
    }
}

#[test]
fn roundtrip_all_i8() {
    let mut buf = [0; 64];
    for i in i8::MIN..=i8::MAX {
        let input = i.numtoa(10, &mut buf);
        let parsed = crate::parse::<i8>(input).expect("Failed to parse valid input!");
        assert_eq!(i, parsed);
    }
}

#[test]
#[cfg_attr(miri, ignore)] // too slow in miri
fn roundtrip_all_u16() {
    let mut buf = [0; 64];
    for i in u16::MIN..=u16::MAX {
        let input = i.numtoa(10, &mut buf);
        let parsed = crate::parse::<u16>(input).expect("Failed to parse valid input!");
        assert_eq!(i, parsed);
    }
}

#[test]
#[cfg_attr(miri, ignore)] // too slow in miri
fn roundtrip_all_i16() {
    let mut buf = [0; 64];
    for i in i16::MIN..=i16::MAX {
        let input = i.numtoa(10, &mut buf);
        let parsed = crate::parse::<i16>(input).expect("Failed to parse valid input!");
        assert_eq!(i, parsed);
    }
}

fn test_each_position<T: Copy>(s: &str, func: fn(&[u8]) -> Result<T, AtoiSimdError>) {
    let mut s_new = ArrayString::<40>::new();
    for j in 0..=s.len() {
        for &ch_str in INVALID_CHARS.iter() {
            s_new.push_str(&s[0..j]);
            s_new.push(ch_str);
            s_new.push_str(&s[j..s.len()]);
            if func(s_new.as_bytes()).is_ok() {
                panic!("error {}", s_new);
            }
            s_new.clear();
        }
    }
}

fn test_each_position_until_invalid<T: Copy + Debug + PartialEq + FromStr>(
    s: &str,
    func: fn(&[u8]) -> Result<(T, usize), AtoiSimdError>,
) where
    <T as FromStr>::Err: Debug,
{
    let mut s_new = ArrayString::<40>::new();
    for j in 1..=s.len() {
        for &ch_str in INVALID_CHARS.iter() {
            let ts = &s[0..j];
            s_new.push_str(ts);
            s_new.push(ch_str);
            s_new.push_str(&s[j..s.len()]);
            assert_eq!(
                func(s_new.as_bytes()).unwrap(),
                (ts.parse::<T>().unwrap(), j)
            );
            s_new.clear();
        }
    }
}

fn test_each_position_u8(s: &str) {
    test_each_position(s, parse::<u8>)
}

fn test_each_position_u16(s: &str) {
    test_each_position(s, parse::<u16>)
}

fn test_each_position_u32(s: &str) {
    test_each_position(s, parse::<u32>)
}

fn test_each_position_u64(s: &str) {
    test_each_position(s, parse::<u64>)
}

/* fn test_each_position_fb_64_pos<const MAX: u64, const LEN_MORE: usize>(s: &str) {
    test_each_position(s, |s_new| parse_fb_checked_64_pos::<MAX, LEN_MORE>(s_new))
}

fn test_each_position_fb_64_neg<const MIN: i64>(s: &str) {
    test_each_position(s, |s_new| parse_fb_checked_64_neg(s_new))
} */

fn test_each_position_until_invalid_u64(s: &str) {
    test_each_position_until_invalid(s, parse_until_invalid::<u64>)
}

#[test]
fn test_parse_u8() {
    if parse::<u8>(b"").is_ok() {
        panic!("error");
    }

    assert_eq!(parse::<u8>(b"0").unwrap(), 0_u8);

    let mut s = ArrayString::<10>::new();
    for i in '1'..='3' {
        test_each_position_u8(&s);
        s.push(i);
        assert_eq!(parse::<u8>(s.as_bytes()).unwrap(), s.parse::<u8>().unwrap());
    }

    assert_eq!(parse::<u8>(b"255").unwrap(), u8::MAX);

    if parse::<u8>(b"256").is_ok() {
        panic!("error");
    }

    if parse::<u8>(b"12345678").is_ok() {
        panic!("error");
    }

    if parse::<u8>(b"1234567890123456789012345").is_ok() {
        panic!("error");
    }
}

#[test]
fn test_parse_i8() {
    if parse::<i8>(b"").is_ok() {
        panic!("error");
    }

    assert_eq!(parse::<i8>(b"0").unwrap(), 0_i8);
    assert_eq!(parse::<i8>(b"-0").unwrap(), 0_i8);

    let mut s = ArrayString::<19>::new();
    let mut s_neg = ArrayString::<20>::new();
    s_neg.push('-');
    for i in '1'..='3' {
        test_each_position(&s, parse::<i8>);
        s.push(i);
        s_neg.push(i);
        assert_eq!(parse::<i8>(s.as_bytes()).unwrap(), s.parse::<i8>().unwrap());
        assert_eq!(
            parse::<i8>(s_neg.as_bytes()).unwrap(),
            s_neg.parse::<i8>().unwrap()
        );
    }

    assert_eq!(parse::<i8>(b"127").unwrap(), i8::MAX);

    if parse::<i8>(b"128").is_ok() {
        panic!("error");
    }

    assert_eq!(parse::<i8>(b"-128").unwrap(), i8::MIN);

    if parse::<i8>(b"-129").is_ok() {
        panic!("error");
    }

    if parse::<i8>(b"255").is_ok() {
        panic!("error");
    }

    if parse::<i8>(b"12345678").is_ok() {
        panic!("error");
    }

    if parse::<i8>(b"-12345678").is_ok() {
        panic!("error");
    }

    if parse::<i8>(b"-1234567890123456789012345").is_ok() {
        panic!("error");
    }
}

#[test]
fn test_parse_u16() {
    if parse::<u16>(b"").is_ok() {
        panic!("error");
    }

    assert_eq!(parse::<u16>(b"0").unwrap(), 0_u16);

    let mut s = ArrayString::<10>::new();
    for i in '1'..='5' {
        test_each_position_u16(&s);
        s.push(i);
        assert_eq!(
            parse::<u16>(s.as_bytes()).unwrap(),
            s.parse::<u16>().unwrap()
        );
    }

    assert_eq!(parse::<u16>(b"65535").unwrap(), u16::MAX);

    if parse::<u16>(b"65536").is_ok() {
        panic!("error");
    }

    if parse::<u16>(b"12345678").is_ok() {
        panic!("error");
    }

    if parse::<u16>(b"1234567890123456789012345").is_ok() {
        panic!("error");
    }
}

#[test]
fn test_parse_i16() {
    if parse::<i16>(b"").is_ok() {
        panic!("error");
    }

    assert_eq!(parse::<i16>(b"0").unwrap(), 0_i16);
    assert_eq!(parse::<i16>(b"-0").unwrap(), 0_i16);

    let mut s = ArrayString::<19>::new();
    let mut s_neg = ArrayString::<20>::new();
    s_neg.push('-');
    for i in '1'..='5' {
        test_each_position(&s, parse::<i16>);
        s.push(i);
        s_neg.push(i);
        assert_eq!(
            parse::<i16>(s.as_bytes()).unwrap(),
            s.parse::<i16>().unwrap()
        );
        assert_eq!(
            parse::<i16>(s_neg.as_bytes()).unwrap(),
            s_neg.parse::<i16>().unwrap()
        );
    }

    assert_eq!(parse::<i16>(b"32767").unwrap(), i16::MAX);

    if parse::<i16>(b"32768").is_ok() {
        panic!("error");
    }

    assert_eq!(parse::<i16>(b"-32768").unwrap(), i16::MIN);

    if parse::<i16>(b"-32769").is_ok() {
        panic!("error");
    }

    if parse::<i16>(b"65535").is_ok() {
        panic!("error");
    }

    if parse::<i16>(b"12345678").is_ok() {
        panic!("error");
    }

    if parse::<i16>(b"-12345678").is_ok() {
        panic!("error");
    }

    if parse::<i16>(b"-1234567890123456789012345").is_ok() {
        panic!("error");
    }
}

#[test]
fn test_parse_u32() {
    if parse::<u32>(b"").is_ok() {
        panic!("error");
    }

    assert_eq!(parse::<u32>(b"0").unwrap(), 0_u32);

    let mut s = ArrayString::<10>::new();
    for i in '1'..='9' {
        test_each_position_u32(&s);
        s.push(i);
        assert_eq!(
            parse::<u32>(s.as_bytes()).unwrap(),
            s.parse::<u32>().unwrap()
        );
    }
    test_each_position_u32(&s);
    s.push('0');
    assert_eq!(
        parse::<u32>(s.as_bytes()).unwrap(),
        s.parse::<u32>().unwrap()
    );

    assert_eq!(parse::<u32>(b"4294967295").unwrap(), u32::MAX);

    if parse::<u32>(b"4294967296").is_ok() {
        panic!("error");
    }

    if parse::<u32>(b"123456789012345").is_ok() {
        panic!("error");
    }

    if parse::<u32>(b"1234567890123456789012345").is_ok() {
        panic!("error");
    }
}

#[test]
fn test_parse_i32() {
    if parse::<i32>(b"").is_ok() {
        panic!("error");
    }

    assert_eq!(parse::<i32>(b"0").unwrap(), 0_i32);
    assert_eq!(parse::<i32>(b"-0").unwrap(), 0_i32);

    let mut s = ArrayString::<19>::new();
    let mut s_neg = ArrayString::<20>::new();
    s_neg.push('-');
    for i in '1'..='9' {
        test_each_position(&s, parse::<i32>);
        s.push(i);
        s_neg.push(i);
        assert_eq!(
            parse::<i32>(s.as_bytes()).unwrap(),
            s.parse::<i32>().unwrap()
        );
        assert_eq!(
            parse::<i32>(s_neg.as_bytes()).unwrap(),
            s_neg.parse::<i32>().unwrap()
        );
    }
    test_each_position(&s, parse::<i32>);
    s.push('0');
    s_neg.push('0');
    assert_eq!(
        parse::<i32>(s.as_bytes()).unwrap(),
        s.parse::<i32>().unwrap()
    );
    assert_eq!(
        parse::<i32>(s_neg.as_bytes()).unwrap(),
        s_neg.parse::<i32>().unwrap()
    );

    assert_eq!(parse::<i32>(b"2147483647").unwrap(), i32::MAX);

    if parse::<i32>(b"2147483648").is_ok() {
        panic!("error");
    }

    assert_eq!(parse::<i32>(b"-2147483648").unwrap(), i32::MIN);

    if parse::<i32>(b"-2147483649").is_ok() {
        panic!("error");
    }

    if parse::<i32>(b"4294967295").is_ok() {
        panic!("error");
    }

    if parse::<i32>(b"123456789012345").is_ok() {
        panic!("error");
    }

    if parse::<i32>(b"-123456789012345").is_ok() {
        panic!("error");
    }

    if parse::<i32>(b"-1234567890123456789012345").is_ok() {
        panic!("error");
    }
}

#[test]
fn test_parse_u64() {
    if parse::<u64>(b"").is_ok() {
        panic!("error");
    }

    assert_eq!(parse::<u64>(b"0").unwrap(), 0_u64);

    let mut s = ArrayString::<20>::new();
    for i in '1'..='9' {
        test_each_position_u64(&s);
        s.push(i);
        assert_eq!(
            parse::<u64>(s.as_bytes()).unwrap(),
            s.parse::<u64>().unwrap()
        );
    }
    for i in '0'..='9' {
        test_each_position_u64(&s);
        s.push(i);
        assert_eq!(
            parse::<u64>(s.as_bytes()).unwrap(),
            s.parse::<u64>().unwrap()
        );
    }
    test_each_position_u64(&s);
    s.push('0');
    assert_eq!(
        parse::<u64>(s.as_bytes()).unwrap(),
        s.parse::<u64>().unwrap()
    );

    assert_eq!(parse::<u64>(b"18446744073709551615").unwrap(), u64::MAX);

    if parse::<u64>(b"18446744073709551616").is_ok() {
        panic!("error");
    }

    if parse::<u64>(b"99999999999999999999").is_ok() {
        panic!("error");
    }

    if parse::<u64>(b"1234567890123456789012345").is_ok() {
        panic!("error");
    }
}

#[test]
fn test_parse_until_invalid_u64() {
    if parse_until_invalid::<u64>(b"").is_ok() {
        panic!("error");
    }

    assert_eq!(parse_until_invalid::<u64>(b"0").unwrap(), (0_u64, 1_usize));

    let mut s = ArrayString::<20>::new();
    for i in '1'..='9' {
        test_each_position_until_invalid_u64(&s);
        s.push(i);
        assert_eq!(
            parse_until_invalid::<u64>(s.as_bytes()).unwrap(),
            (s.parse::<u64>().unwrap(), s.len())
        );
    }
    for i in '0'..='9' {
        test_each_position_until_invalid_u64(&s);
        s.push(i);
        assert_eq!(
            parse_until_invalid::<u64>(s.as_bytes()).unwrap(),
            (s.parse::<u64>().unwrap(), s.len())
        );
    }
    test_each_position_until_invalid_u64(&s);
    s.push('0');
    assert_eq!(
        parse_until_invalid::<u64>(s.as_bytes()).unwrap(),
        (s.parse::<u64>().unwrap(), s.len())
    );

    assert_eq!(
        parse_until_invalid::<u64>(b"18446744073709551615").unwrap(),
        (u64::MAX, 20)
    );

    if parse_until_invalid::<u64>(b"18446744073709551616").is_ok() {
        panic!("error");
    }

    if parse_until_invalid::<u64>(b"99999999999999999999").is_ok() {
        panic!("error");
    }

    if parse_until_invalid::<u64>(b"1234567890123456789012345").is_ok() {
        panic!("error");
    }
}

/* #[test]
fn test_parse_fb_64_pos() {
    if parse_fb_checked_64_pos::<{ u64::MAX }, 4>(b"").is_ok() {
        panic!("error");
    }

    assert_eq!(
        parse_fb_checked_64_pos::<{ u64::MAX }, 4>(b"0").unwrap(),
        0_u64
    );

    let mut s = ArrayString::<20>::new();
    for i in '1'..='9' {
        test_each_position_fb_64_pos::<{ u64::MAX }, 4>(&s);
        s.push(i);
        assert_eq!(
            parse_fb_checked_64_pos::<{ u64::MAX }, 4>(s.as_bytes()).unwrap(),
            s.parse::<u64>().unwrap()
        );
    }
    for i in '0'..='9' {
        test_each_position_fb_64_pos::<{ u64::MAX }, 4>(&s);
        s.push(i);
        assert_eq!(
            parse_fb_checked_64_pos::<{ u64::MAX }, 4>(s.as_bytes()).unwrap(),
            s.parse::<u64>().unwrap()
        );
    }
    test_each_position_fb_64_pos::<{ u64::MAX }, 4>(&s);
    s.push('0');
    assert_eq!(
        parse_fb_checked_64_pos::<{ u64::MAX }, 4>(s.as_bytes()).unwrap(),
        s.parse::<u64>().unwrap()
    );

    assert_eq!(
        parse_fb_checked_64_pos::<{ u64::MAX }, 4>(b"18446744073709551615").unwrap(),
        u64::MAX
    );

    if parse_fb_checked_64_pos::<{ u64::MAX }, 4>(b"18446744073709551616").is_ok() {
        panic!("error");
    }

    if parse_fb_checked_64_pos::<{ u64::MAX }, 4>(b"99999999999999999999").is_ok() {
        panic!("error");
    }
}

#[test]
fn test_parse_fb_64_neg() {
    if parse_fb_checked_64_neg(b"").is_ok() {
        panic!("error");
    }

    assert_eq!(parse_fb_checked_64_neg(b"0").unwrap(), 0_i64);

    let mut s = ArrayString::<20>::new();
    for i in '1'..='9' {
        test_each_position_fb_64_neg::<{ i64::MIN }>(&s);
        s.push(i);
        assert_eq!(
            parse_fb_checked_64_neg(s.as_bytes()).unwrap(),
            -s.parse::<i64>().unwrap()
        );
    }
    for i in '0'..='9' {
        test_each_position_fb_64_neg::<{ i64::MIN }>(&s);
        s.push(i);
        assert_eq!(
            parse_fb_checked_64_neg(s.as_bytes()).unwrap(),
            -s.parse::<i64>().unwrap()
        );
    }

    assert_eq!(
        parse_fb_checked_64_neg(b"9223372036854775808").unwrap(),
        i64::MIN
    );

    if parse_fb_checked_64_neg(b"9223372036854775809").is_ok() {
        panic!("error");
    }

    if parse_fb_checked_64_neg(b"99999999999999999999").is_ok() {
        panic!("error");
    }
} */

#[test]
fn test_parse_i64() {
    if parse::<i64>(b"").is_ok() {
        panic!("error");
    }

    assert_eq!(parse::<i64>(b"0").unwrap(), 0_i64);
    assert_eq!(parse::<i64>(b"-0").unwrap(), 0_i64);

    let mut s = ArrayString::<19>::new();
    let mut s_neg = ArrayString::<20>::new();
    s_neg.push('-');
    for i in '1'..='9' {
        test_each_position(&s, parse::<i64>);
        s.push(i);
        s_neg.push(i);
        assert_eq!(
            parse::<i64>(s.as_bytes()).unwrap(),
            s.parse::<i64>().unwrap()
        );
        assert_eq!(
            parse::<i64>(s_neg.as_bytes()).unwrap(),
            s_neg.parse::<i64>().unwrap()
        );
    }
    for i in '0'..='9' {
        test_each_position(&s, parse::<i64>);
        s.push(i);
        s_neg.push(i);
        assert_eq!(
            parse::<i64>(s.as_bytes()).unwrap(),
            s.parse::<i64>().unwrap()
        );
        assert_eq!(
            parse::<i64>(s_neg.as_bytes()).unwrap(),
            s_neg.parse::<i64>().unwrap()
        );
    }

    assert_eq!(parse::<i64>(b"9223372036854775807").unwrap(), i64::MAX);

    if parse::<i64>(b"9223372036854775808").is_ok() {
        panic!("error");
    }

    assert_eq!(parse::<i64>(b"-9223372036854775808").unwrap(), i64::MIN);

    if parse::<i64>(b"-9223372036854775809").is_ok() {
        panic!("error");
    }

    if parse::<i64>(b"18446744073709551615").is_ok() {
        panic!("error");
    }

    if parse::<i64>(b"99999999999999999999").is_ok() {
        panic!("error");
    }

    if parse::<i64>(b"-99999999999999999999").is_ok() {
        panic!("error");
    }

    if parse::<i64>(b"-1234567890123456789012345").is_ok() {
        panic!("error");
    }
}

#[test]
fn test_parse_u128() {
    if parse::<u128>(b"").is_ok() {
        panic!("error");
    }

    assert_eq!(parse::<u128>(b"0").unwrap(), 0_u128);

    let mut s = ArrayString::<39>::new();
    for i in '1'..='9' {
        test_each_position(&s, parse::<u128>);
        s.push(i);
        assert_eq!(
            parse::<u128>(s.as_bytes()).unwrap(),
            s.parse::<u128>().unwrap()
        );
    }
    for _ in 0..2 {
        for i in '0'..='9' {
            test_each_position(&s, parse::<u128>);
            s.push(i);
            assert_eq!(
                parse::<u128>(s.as_bytes()).unwrap(),
                s.parse::<u128>().unwrap()
            );
        }
    }
    for i in '0'..='9' {
        test_each_position(&s, parse::<u128>);
        s.push(i);
        assert_eq!(
            parse::<u128>(s.as_bytes()).unwrap(),
            s.parse::<u128>().unwrap()
        );
    }

    assert_eq!(
        parse::<u128>(b"9999999999999999").unwrap(),
        9_999_999_999_999_999_u128
    );

    assert_eq!(
        parse::<u128>(b"18446744073709551615").unwrap(),
        u64::MAX as u128
    );

    assert_eq!(
        parse::<u128>(b"18446744073709551616").unwrap(),
        18446744073709551616
    );

    assert_eq!(
        parse::<u128>(b"99999999999999999999").unwrap(),
        99999999999999999999
    );

    assert_eq!(
        parse::<u128>(b"12345678901234567890123456789012").unwrap(),
        1234567890_1234567890_1234567890_12_u128
    );

    assert_eq!(
        parse::<u128>(b"340282366920938463463374607431768211455").unwrap(),
        u128::MAX
    );

    if parse::<u128>(b"340282366920938463463374607431768211456").is_ok() {
        panic!("error");
    }

    if parse::<u128>(b"999999999999999999999999999999999999999").is_ok() {
        panic!("error");
    }

    if parse::<u128>(b"9999999999999999999999999999999999999999999").is_ok() {
        panic!("error");
    }
}

#[test]
fn test_parse_i128() {
    if parse::<i128>(b"").is_ok() {
        panic!("error");
    }

    assert_eq!(parse::<i128>(b"0").unwrap(), 0_i128);
    assert_eq!(parse::<i128>(b"-0").unwrap(), 0_i128);

    let mut s = ArrayString::<39>::new();
    let mut s_neg = ArrayString::<40>::new();
    s_neg.push('-');
    for i in '1'..='9' {
        test_each_position(&s, parse::<i128>);
        s.push(i);
        s_neg.push(i);
        assert_eq!(
            parse::<i128>(s.as_bytes()).unwrap(),
            s.parse::<i128>().unwrap()
        );
        assert_eq!(
            parse::<i128>(s_neg.as_bytes()).unwrap(),
            s_neg.parse::<i128>().unwrap()
        );
    }
    for _ in 0..2 {
        for i in '0'..='9' {
            test_each_position(&s, parse::<i128>);
            s.push(i);
            s_neg.push(i);
            assert_eq!(
                parse::<i128>(s.as_bytes()).unwrap(),
                s.parse::<i128>().unwrap()
            );
            assert_eq!(
                parse::<i128>(s_neg.as_bytes()).unwrap(),
                s_neg.parse::<i128>().unwrap()
            );
        }
    }
    for i in '0'..='9' {
        test_each_position(&s, parse::<i128>);
        s.push(i);
        s_neg.push(i);
        assert_eq!(
            parse::<i128>(s.as_bytes()).unwrap(),
            s.parse::<i128>().unwrap()
        );
        assert_eq!(
            parse::<i128>(s_neg.as_bytes()).unwrap(),
            s_neg.parse::<i128>().unwrap()
        );
    }

    assert_eq!(
        parse::<i128>(b"-9999999999999999").unwrap(),
        -9_999_999_999_999_999_i128
    );

    assert_eq!(
        parse::<i128>(b"9999999999999999").unwrap(),
        9_999_999_999_999_999_i128
    );

    assert_eq!(
        parse::<i128>(b"-99999999999999999999999999999999").unwrap(),
        -99_999_999_999_999_999_999_999_999_999_999_i128
    );

    assert_eq!(
        parse::<i128>(b"99999999999999999999999999999999").unwrap(),
        99_999_999_999_999_999_999_999_999_999_999_i128
    );

    assert_eq!(
        parse::<i128>(b"12345678901234567890123456789012").unwrap(),
        1234567890_1234567890_1234567890_12_i128
    );

    assert_eq!(
        parse::<i128>(b"-12345678901234567890123456789012").unwrap(),
        -1234567890_1234567890_1234567890_12_i128
    );

    assert_eq!(
        parse::<i128>(b"170141183460469231731687303715884105727").unwrap(),
        i128::MAX
    );

    assert_eq!(
        parse::<i128>(b"-170141183460469231731687303715884105728").unwrap(),
        i128::MIN
    );

    if parse::<i128>(b"170141183460469231731687303715884105728").is_ok() {
        panic!("error");
    }

    if parse::<i128>(b"-170141183460469231731687303715884105729").is_ok() {
        panic!("error");
    }

    if parse::<u128>(b"-999999999999999999999999999999999999999").is_ok() {
        panic!("error");
    }

    if parse::<u128>(b"-9999999999999999999999999999999999999999999").is_ok() {
        panic!("error");
    }
}

#[test]
fn test_parse_types() {
    let tmp: u8 = parse(b"123").unwrap();
    assert_eq!(tmp, 123_u8);

    let tmp: i8 = parse(b"-123").unwrap();
    assert_eq!(tmp, -123_i8);

    let tmp: u16 = parse(b"1234").unwrap();
    assert_eq!(tmp, 1234_u16);

    let tmp: i16 = parse(b"-1234").unwrap();
    assert_eq!(tmp, -1234_i16);

    let tmp: u32 = parse(b"1234").unwrap();
    assert_eq!(tmp, 1234_u32);

    let tmp: i32 = parse(b"-1234").unwrap();
    assert_eq!(tmp, -1234_i32);

    let tmp: usize = parse(b"1234").unwrap();
    assert_eq!(tmp, 1234_usize);

    let tmp: isize = parse(b"-1234").unwrap();
    assert_eq!(tmp, -1234_isize);

    let tmp: u64 = parse(b"1234").unwrap();
    assert_eq!(tmp, 1234_u64);

    let tmp: i64 = parse(b"-1234").unwrap();
    assert_eq!(tmp, -1234_i64);

    let tmp: u128 = parse(b"999999").unwrap();
    assert_eq!(tmp, 999999_u128);

    let tmp: i128 = parse(b"-999999").unwrap();
    assert_eq!(tmp, -999999_i128);
}

#[test]
fn test_parse_pos() {
    let tmp: i8 = parse_pos(b"123").unwrap();
    assert_eq!(tmp, 123_i8);

    let tmp: i16 = parse_pos(b"1234").unwrap();
    assert_eq!(tmp, 1234_i16);

    let tmp: i32 = parse_pos(b"1234").unwrap();
    assert_eq!(tmp, 1234_i32);

    let tmp: isize = parse_pos(b"1234").unwrap();
    assert_eq!(tmp, 1234_isize);

    let tmp: i64 = parse_pos(b"1234").unwrap();
    assert_eq!(tmp, 1234_i64);

    let tmp: i128 = parse_pos(b"999999").unwrap();
    assert_eq!(tmp, 999999_i128);
}

#[test]
fn test_parse_neg() {
    let tmp: i8 = parse_neg(b"123").unwrap();
    assert_eq!(tmp, -123_i8);

    let tmp: i16 = parse_neg(b"1234").unwrap();
    assert_eq!(tmp, -1234_i16);

    let tmp: i32 = parse_neg(b"1234").unwrap();
    assert_eq!(tmp, -1234_i32);

    let tmp: isize = parse_neg(b"1234").unwrap();
    assert_eq!(tmp, -1234_isize);

    let tmp: i64 = parse_neg(b"1234").unwrap();
    assert_eq!(tmp, -1234_i64);

    let tmp: i128 = parse_neg(b"999999").unwrap();
    assert_eq!(tmp, -999999_i128);
}

#[test]
fn test_parse_until_invalid() {
    let tmp = parse_until_invalid::<u8>(b"123s").unwrap();
    assert_eq!(tmp, (123_u8, 3));

    let tmp = parse_until_invalid::<i8>(b"-123s").unwrap();
    assert_eq!(tmp, (-123_i8, 4));

    let tmp = parse_until_invalid::<u16>(b"1234s").unwrap();
    assert_eq!(tmp, (1234_u16, 4));

    let tmp = parse_until_invalid::<i16>(b"-1234s").unwrap();
    assert_eq!(tmp, (-1234_i16, 5));

    let tmp = parse_until_invalid::<u32>(b"1234s").unwrap();
    assert_eq!(tmp, (1234_u32, 4));

    let tmp = parse_until_invalid::<i32>(b"-1234s").unwrap();
    assert_eq!(tmp, (-1234_i32, 5));

    let tmp = parse_until_invalid::<u64>(b"1234s").unwrap();
    assert_eq!(tmp, (1234_u64, 4));

    let tmp = parse_until_invalid::<i64>(b"-1234s").unwrap();
    assert_eq!(tmp, (-1234_i64, 5));

    let tmp = parse_until_invalid::<u128>(b"1234s").unwrap();
    assert_eq!(tmp, (1234_u128, 4));

    let tmp = parse_until_invalid::<i128>(b"-1234s").unwrap();
    assert_eq!(tmp, (-1234_i128, 5));

    let tmp = parse_until_invalid::<u128>(b"12345678901234567890s").unwrap();
    assert_eq!(tmp, (12345678901234567890_u128, 20));

    let tmp = parse_until_invalid::<i128>(b"-12345678901234567890s").unwrap();
    assert_eq!(tmp, (-12345678901234567890_i128, 21));

    let tmp = parse_until_invalid::<u64>(
        b"12345678901234567890s11111111111111111111111111111111111111111111111111111111111111",
    )
    .unwrap();
    assert_eq!(tmp, (12345678901234567890_u64, 20));

    let tmp = parse_until_invalid::<i64>(
        b"-1234567890123456789s11111111111111111111111111111111111111111111111111111111111111",
    )
    .unwrap();
    assert_eq!(tmp, (-1234567890123456789_i64, 20));

    let tmp = parse_until_invalid::<u128>(
        b"12345678901234567890s11111111111111111111111111111111111111111111111111111111111111",
    )
    .unwrap();
    assert_eq!(tmp, (12345678901234567890_u128, 20));

    let tmp = parse_until_invalid::<i128>(
        b"-12345678901234567890s11111111111111111111111111111111111111111111111111111111111111",
    )
    .unwrap();
    assert_eq!(tmp, (-12345678901234567890_i128, 21));

    assert_eq!(parse_until_invalid::<u64>(b"0 asdf").unwrap(), (0, 1));

    assert_eq!(parse_until_invalid::<u64>(b"1:2 3:4 1:5s").unwrap(), (1, 1));

    assert_eq!(
        parse_until_invalid::<u64>(b"12:2 3:4 1:5s").unwrap(),
        (12, 2)
    );

    assert_eq!(
        parse_until_invalid::<u64>(b"123:2 3:4 1:5s").unwrap(),
        (123, 3)
    );

    assert_eq!(
        parse_until_invalid::<u64>(b"1234:2 3:4 1:5s").unwrap(),
        (1234, 4)
    );

    assert_eq!(
        parse_until_invalid::<u64>(b"18446744073709551615").unwrap(),
        (u64::MAX, 20)
    );

    if parse_until_invalid::<u64>(b"18446744073709551616").is_ok() {
        panic!("error");
    }

    if parse_until_invalid::<u64>(b"99999999999999999999").is_ok() {
        panic!("error");
    }

    assert_eq!(
        parse_until_invalid::<i64>(b"9223372036854775807").unwrap(),
        (i64::MAX, 19)
    );

    if parse_until_invalid::<i64>(b"9223372036854775808").is_ok() {
        panic!("error");
    }

    assert_eq!(
        parse_until_invalid::<i64>(b"-9223372036854775808").unwrap(),
        (i64::MIN, 20)
    );

    if parse_until_invalid::<i64>(b"-9223372036854775809").is_ok() {
        panic!("error");
    }

    if parse_until_invalid::<i64>(b"18446744073709551615").is_ok() {
        panic!("error");
    }

    if parse_until_invalid::<i64>(b"99999999999999999999").is_ok() {
        panic!("error");
    }

    if parse_until_invalid::<i64>(b"-99999999999999999999").is_ok() {
        panic!("error");
    }

    assert_eq!(
        parse_until_invalid::<u128>(b"9999999999999999").unwrap(),
        (9_999_999_999_999_999_u128, 16)
    );

    assert_eq!(
        parse_until_invalid::<u128>(b"12345678901234567890123456789012").unwrap(),
        (1234567890_1234567890_1234567890_12_u128, 32)
    );

    assert_eq!(
        parse_until_invalid::<i128>(b"-9999999999999999").unwrap(),
        (-9_999_999_999_999_999_i128, 17)
    );

    assert_eq!(
        parse_until_invalid::<i128>(b"9999999999999999").unwrap(),
        (9_999_999_999_999_999_i128, 16)
    );

    assert_eq!(
        parse_until_invalid::<i128>(b"-99999999999999999999999999999999").unwrap(),
        (-99_999_999_999_999_999_999_999_999_999_999_i128, 33)
    );

    assert_eq!(
        parse_until_invalid::<i128>(b"99999999999999999999999999999999").unwrap(),
        (99_999_999_999_999_999_999_999_999_999_999_i128, 32)
    );

    assert_eq!(
        parse_until_invalid::<i128>(b"12345678901234567890123456789012").unwrap(),
        (1234567890_1234567890_1234567890_12_i128, 32)
    );

    assert_eq!(
        parse_until_invalid::<i128>(b"-12345678901234567890123456789012").unwrap(),
        (-1234567890_1234567890_1234567890_12_i128, 33)
    );
}

#[test]
fn test_parse_until_invalid_pos() {
    let tmp = parse_until_invalid_pos::<u8>(b"123s").unwrap();
    assert_eq!(tmp, (123_u8, 3));

    let tmp = parse_until_invalid_pos::<i8>(b"123s").unwrap();
    assert_eq!(tmp, (123_i8, 3));

    let tmp = parse_until_invalid_pos::<u16>(b"123s").unwrap();
    assert_eq!(tmp, (123_u16, 3));

    let tmp = parse_until_invalid_pos::<u16>(b"1234s").unwrap();
    assert_eq!(tmp, (1234_u16, 4));

    let tmp = parse_until_invalid_pos::<i16>(b"1234s").unwrap();
    assert_eq!(tmp, (1234_i16, 4));

    let tmp = parse_until_invalid_pos::<u32>(b"1234s").unwrap();
    assert_eq!(tmp, (1234_u32, 4));

    let tmp = parse_until_invalid_pos::<i32>(b"1234s").unwrap();
    assert_eq!(tmp, (1234_i32, 4));

    let tmp = parse_until_invalid_pos::<u64>(b"1234s").unwrap();
    assert_eq!(tmp, (1234_u64, 4));

    let tmp = parse_until_invalid_pos::<i64>(b"1234s").unwrap();
    assert_eq!(tmp, (1234_i64, 4));

    let tmp = parse_until_invalid_pos::<u128>(b"1234s").unwrap();
    assert_eq!(tmp, (1234_u128, 4));

    let tmp = parse_until_invalid_pos::<u128>(b"12345678901234567890s").unwrap();
    assert_eq!(tmp, (12345678901234567890_u128, 20));

    let tmp = parse_until_invalid_pos::<i128>(b"1234s").unwrap();
    assert_eq!(tmp, (1234_i128, 4));

    let tmp = parse_until_invalid_pos::<i128>(b"12345678901234567890s").unwrap();
    assert_eq!(tmp, (12345678901234567890_i128, 20));
}

#[test]
fn test_parse_until_invalid_neg() {
    let tmp = parse_until_invalid_neg::<i8>(b"123s").unwrap();
    assert_eq!(tmp, (-123_i8, 3));

    let tmp = parse_until_invalid_neg::<i16>(b"1234s").unwrap();
    assert_eq!(tmp, (-1234_i16, 4));

    let tmp = parse_until_invalid_neg::<i32>(b"1234s").unwrap();
    assert_eq!(tmp, (-1234_i32, 4));

    let tmp = parse_until_invalid_neg::<i64>(b"1234s").unwrap();
    assert_eq!(tmp, (-1234_i64, 4));

    let tmp = parse_until_invalid_neg::<i128>(b"1234s").unwrap();
    assert_eq!(tmp, (-1234_i128, 4));

    let tmp = parse_until_invalid_neg::<i128>(b"12345678901234567890s").unwrap();
    assert_eq!(tmp, (-12345678901234567890_i128, 20));
}

#[test]
fn overflow_u32() {}
