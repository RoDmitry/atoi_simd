use super::*;
// use crate::fallback::*;
use arrayvec::{ArrayString, ArrayVec};
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
        let parsed = crate::parse::<u8>(&input).expect("Failed to parse valid input!");
        assert_eq!(i, parsed);
    }
}

#[test]
fn roundtrip_all_i8() {
    let mut buf = [0; 64];
    for i in i8::MIN..=i8::MAX {
        let input = i.numtoa(10, &mut buf);
        let parsed = crate::parse::<i8>(&input).expect("Failed to parse valid input!");
        assert_eq!(i, parsed);
    }
}

#[test]
#[cfg_attr(miri, ignore)] // too slow in miri
fn roundtrip_all_u16() {
    let mut buf = [0; 64];
    for i in u16::MIN..=u16::MAX {
        let input = i.numtoa(10, &mut buf);
        let parsed = crate::parse::<u16>(&input).expect("Failed to parse valid input!");
        assert_eq!(i, parsed);
    }
}

#[test]
#[cfg_attr(miri, ignore)] // too slow in miri
fn roundtrip_all_i16() {
    let mut buf = [0; 64];
    for i in i16::MIN..=i16::MAX {
        let input = i.numtoa(10, &mut buf);
        let parsed = crate::parse::<i16>(&input).expect("Failed to parse valid input!");
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
    test_each_position(s, |s_new| parse::<u8>(s_new))
}

fn test_each_position_u16(s: &str) {
    test_each_position(s, |s_new| parse::<u16>(s_new))
}

fn test_each_position_u32(s: &str) {
    test_each_position(s, |s_new| parse::<u32>(s_new))
}

fn test_each_position_u64(s: &str) {
    test_each_position(s, |s_new| parse::<u64>(s_new))
}

/* fn test_each_position_fb_64_pos<const MAX: u64, const LEN_MORE: usize>(s: &str) {
    test_each_position(s, |s_new| parse_fb_checked_64_pos::<MAX, LEN_MORE>(s_new))
}

fn test_each_position_fb_64_neg<const MIN: i64>(s: &str) {
    test_each_position(s, |s_new| parse_fb_checked_64_neg(s_new))
} */

fn test_each_position_until_invalid_u64(s: &str) {
    test_each_position_until_invalid(s, |s_new| parse_until_invalid::<u64>(s_new))
}

#[test]
fn test_parse_u8() {
    if parse::<u8>("".as_bytes()).is_ok() {
        panic!("error");
    }

    assert_eq!(parse::<u8>("0".as_bytes()).unwrap(), 0_u8);

    let mut s = ArrayString::<10>::new();
    for i in '1'..='3' {
        test_each_position_u8(&s);
        s.push(i);
        assert_eq!(parse::<u8>(s.as_bytes()).unwrap(), s.parse::<u8>().unwrap());
    }

    assert_eq!(parse::<u8>("255".as_bytes()).unwrap(), u8::MAX);

    if parse::<u8>("256".as_bytes()).is_ok() {
        panic!("error");
    }

    if parse::<u8>("12345678".as_bytes()).is_ok() {
        panic!("error");
    }
}

#[test]
fn test_parse_i8() {
    if parse::<i8>("".as_bytes()).is_ok() {
        panic!("error");
    }

    assert_eq!(parse::<i8>("0".as_bytes()).unwrap(), 0_i8);
    assert_eq!(parse::<i8>("-0".as_bytes()).unwrap(), 0_i8);

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

    assert_eq!(parse::<i8>("127".as_bytes()).unwrap(), i8::MAX);

    if parse::<i8>("128".as_bytes()).is_ok() {
        panic!("error");
    }

    assert_eq!(parse::<i8>("-128".as_bytes()).unwrap(), i8::MIN);

    if parse::<i8>("-129".as_bytes()).is_ok() {
        panic!("error");
    }

    if parse::<i8>("255".as_bytes()).is_ok() {
        panic!("error");
    }

    if parse::<i8>("12345678".as_bytes()).is_ok() {
        panic!("error");
    }

    if parse::<i8>("-12345678".as_bytes()).is_ok() {
        panic!("error");
    }
}

#[test]
fn test_parse_u16() {
    if parse::<u16>("".as_bytes()).is_ok() {
        panic!("error");
    }

    assert_eq!(parse::<u16>("0".as_bytes()).unwrap(), 0_u16);

    let mut s = ArrayString::<10>::new();
    for i in '1'..='5' {
        test_each_position_u16(&s);
        s.push(i);
        assert_eq!(
            parse::<u16>(s.as_bytes()).unwrap(),
            s.parse::<u16>().unwrap()
        );
    }

    assert_eq!(parse::<u16>("65535".as_bytes()).unwrap(), u16::MAX);

    if parse::<u16>("65536".as_bytes()).is_ok() {
        panic!("error");
    }

    if parse::<u16>("12345678".as_bytes()).is_ok() {
        panic!("error");
    }
}

#[test]
fn test_parse_i16() {
    if parse::<i16>("".as_bytes()).is_ok() {
        panic!("error");
    }

    assert_eq!(parse::<i16>("0".as_bytes()).unwrap(), 0_i16);
    assert_eq!(parse::<i16>("-0".as_bytes()).unwrap(), 0_i16);

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

    assert_eq!(parse::<i16>("32767".as_bytes()).unwrap(), i16::MAX);

    if parse::<i16>("32768".as_bytes()).is_ok() {
        panic!("error");
    }

    assert_eq!(parse::<i16>("-32768".as_bytes()).unwrap(), i16::MIN);

    if parse::<i16>("-32769".as_bytes()).is_ok() {
        panic!("error");
    }

    if parse::<i16>("65535".as_bytes()).is_ok() {
        panic!("error");
    }

    if parse::<i16>("12345678".as_bytes()).is_ok() {
        panic!("error");
    }

    if parse::<i16>("-12345678".as_bytes()).is_ok() {
        panic!("error");
    }
}

#[test]
fn test_parse_u32() {
    if parse::<u32>("".as_bytes()).is_ok() {
        panic!("error");
    }

    assert_eq!(parse::<u32>("0".as_bytes()).unwrap(), 0_u32);

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

    assert_eq!(parse::<u32>("4294967295".as_bytes()).unwrap(), u32::MAX);

    if parse::<u32>("4294967296".as_bytes()).is_ok() {
        panic!("error");
    }

    if parse::<u32>("123456789012345".as_bytes()).is_ok() {
        panic!("error");
    }
}

#[test]
fn test_parse_i32() {
    if parse::<i32>("".as_bytes()).is_ok() {
        panic!("error");
    }

    assert_eq!(parse::<i32>("0".as_bytes()).unwrap(), 0_i32);
    assert_eq!(parse::<i32>("-0".as_bytes()).unwrap(), 0_i32);

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

    assert_eq!(parse::<i32>("2147483647".as_bytes()).unwrap(), i32::MAX);

    if parse::<i32>("2147483648".as_bytes()).is_ok() {
        panic!("error");
    }

    assert_eq!(parse::<i32>("-2147483648".as_bytes()).unwrap(), i32::MIN);

    if parse::<i32>("-2147483649".as_bytes()).is_ok() {
        panic!("error");
    }

    if parse::<i32>("4294967295".as_bytes()).is_ok() {
        panic!("error");
    }

    if parse::<i32>("123456789012345".as_bytes()).is_ok() {
        panic!("error");
    }

    if parse::<i32>("-123456789012345".as_bytes()).is_ok() {
        panic!("error");
    }
}

#[test]
fn test_parse_u64() {
    if parse::<u64>("".as_bytes()).is_ok() {
        panic!("error");
    }

    assert_eq!(parse::<u64>("0".as_bytes()).unwrap(), 0_u64);

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

    assert_eq!(
        parse::<u64>("18446744073709551615".as_bytes()).unwrap(),
        u64::MAX
    );

    if parse::<u64>("18446744073709551616".as_bytes()).is_ok() {
        panic!("error");
    }

    if parse::<u64>("99999999999999999999".as_bytes()).is_ok() {
        panic!("error");
    }
}

#[test]
fn test_parse_until_invalid_u64() {
    if parse_until_invalid::<u64>("".as_bytes()).is_ok() {
        panic!("error");
    }

    assert_eq!(
        parse_until_invalid::<u64>("0".as_bytes()).unwrap(),
        (0_u64, 1_usize)
    );

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
        parse_until_invalid::<u64>("18446744073709551615".as_bytes()).unwrap(),
        (u64::MAX, 20)
    );

    if parse_until_invalid::<u64>("18446744073709551616".as_bytes()).is_ok() {
        panic!("error");
    }

    if parse_until_invalid::<u64>("99999999999999999999".as_bytes()).is_ok() {
        panic!("error");
    }
}

/* #[test]
fn test_parse_fb_64_pos() {
    if parse_fb_checked_64_pos::<{ u64::MAX }, 4>("".as_bytes()).is_ok() {
        panic!("error");
    }

    assert_eq!(
        parse_fb_checked_64_pos::<{ u64::MAX }, 4>("0".as_bytes()).unwrap(),
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
        parse_fb_checked_64_pos::<{ u64::MAX }, 4>("18446744073709551615".as_bytes()).unwrap(),
        u64::MAX
    );

    if parse_fb_checked_64_pos::<{ u64::MAX }, 4>("18446744073709551616".as_bytes()).is_ok() {
        panic!("error");
    }

    if parse_fb_checked_64_pos::<{ u64::MAX }, 4>("99999999999999999999".as_bytes()).is_ok() {
        panic!("error");
    }
}

#[test]
fn test_parse_fb_64_neg() {
    if parse_fb_checked_64_neg("".as_bytes()).is_ok() {
        panic!("error");
    }

    assert_eq!(parse_fb_checked_64_neg("0".as_bytes()).unwrap(), 0_i64);

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
        parse_fb_checked_64_neg("9223372036854775808".as_bytes()).unwrap(),
        i64::MIN
    );

    if parse_fb_checked_64_neg("9223372036854775809".as_bytes()).is_ok() {
        panic!("error");
    }

    if parse_fb_checked_64_neg("99999999999999999999".as_bytes()).is_ok() {
        panic!("error");
    }
} */

#[test]
fn test_parse_i64() {
    if parse::<i64>("".as_bytes()).is_ok() {
        panic!("error");
    }

    assert_eq!(parse::<i64>("0".as_bytes()).unwrap(), 0_i64);
    assert_eq!(parse::<i64>("-0".as_bytes()).unwrap(), 0_i64);

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

    assert_eq!(
        parse::<i64>("9223372036854775807".as_bytes()).unwrap(),
        i64::MAX
    );

    if parse::<i64>("9223372036854775808".as_bytes()).is_ok() {
        panic!("error");
    }

    assert_eq!(
        parse::<i64>("-9223372036854775808".as_bytes()).unwrap(),
        i64::MIN
    );

    if parse::<i64>("-9223372036854775809".as_bytes()).is_ok() {
        panic!("error");
    }

    if parse::<i64>("18446744073709551615".as_bytes()).is_ok() {
        panic!("error");
    }

    if parse::<i64>("99999999999999999999".as_bytes()).is_ok() {
        panic!("error");
    }

    if parse::<i64>("-99999999999999999999".as_bytes()).is_ok() {
        panic!("error");
    }
}

#[test]
fn test_parse_u128() {
    if parse::<u128>("".as_bytes()).is_ok() {
        panic!("error");
    }

    assert_eq!(parse::<u128>("0".as_bytes()).unwrap(), 0_u128);

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
        parse::<u128>("9999999999999999".as_bytes()).unwrap(),
        9_999_999_999_999_999_u128
    );

    assert_eq!(
        parse::<u128>("18446744073709551615".as_bytes()).unwrap(),
        u64::MAX as u128
    );

    assert_eq!(
        parse::<u128>("18446744073709551616".as_bytes()).unwrap(),
        18446744073709551616
    );

    assert_eq!(
        parse::<u128>("99999999999999999999".as_bytes()).unwrap(),
        99999999999999999999
    );

    assert_eq!(
        parse::<u128>("12345678901234567890123456789012".as_bytes()).unwrap(),
        1234567890_1234567890_1234567890_12_u128
    );

    assert_eq!(
        parse::<u128>("340282366920938463463374607431768211455".as_bytes()).unwrap(),
        u128::MAX
    );

    if parse::<u128>("340282366920938463463374607431768211456".as_bytes()).is_ok() {
        panic!("error");
    }
}

#[test]
fn test_parse_i128() {
    if parse::<i128>("".as_bytes()).is_ok() {
        panic!("error");
    }

    assert_eq!(parse::<i128>("0".as_bytes()).unwrap(), 0_i128);
    assert_eq!(parse::<i128>("-0".as_bytes()).unwrap(), 0_i128);

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
        parse::<i128>("-9999999999999999".as_bytes()).unwrap(),
        -9_999_999_999_999_999_i128
    );

    assert_eq!(
        parse::<i128>("9999999999999999".as_bytes()).unwrap(),
        9_999_999_999_999_999_i128
    );

    assert_eq!(
        parse::<i128>("-99999999999999999999999999999999".as_bytes()).unwrap(),
        -99_999_999_999_999_999_999_999_999_999_999_i128
    );

    assert_eq!(
        parse::<i128>("99999999999999999999999999999999".as_bytes()).unwrap(),
        99_999_999_999_999_999_999_999_999_999_999_i128
    );

    assert_eq!(
        parse::<i128>("12345678901234567890123456789012".as_bytes()).unwrap(),
        1234567890_1234567890_1234567890_12_i128
    );

    assert_eq!(
        parse::<i128>("-12345678901234567890123456789012".as_bytes()).unwrap(),
        -1234567890_1234567890_1234567890_12_i128
    );

    assert_eq!(
        parse::<i128>("170141183460469231731687303715884105727".as_bytes()).unwrap(),
        i128::MAX
    );

    assert_eq!(
        parse::<i128>("-170141183460469231731687303715884105728".as_bytes()).unwrap(),
        i128::MIN
    );

    if parse::<i128>("170141183460469231731687303715884105728".as_bytes()).is_ok() {
        panic!("error");
    }

    if parse::<i128>("-170141183460469231731687303715884105729".as_bytes()).is_ok() {
        panic!("error");
    }
}

#[test]
fn test_parse_types() {
    let tmp: u8 = parse("123".as_bytes()).unwrap();
    assert_eq!(tmp, 123_u8);

    let tmp: i8 = parse("-123".as_bytes()).unwrap();
    assert_eq!(tmp, -123_i8);

    let tmp: u16 = parse("1234".as_bytes()).unwrap();
    assert_eq!(tmp, 1234_u16);

    let tmp: i16 = parse("-1234".as_bytes()).unwrap();
    assert_eq!(tmp, -1234_i16);

    let tmp: u32 = parse("1234".as_bytes()).unwrap();
    assert_eq!(tmp, 1234_u32);

    let tmp: i32 = parse("-1234".as_bytes()).unwrap();
    assert_eq!(tmp, -1234_i32);

    let tmp: usize = parse("1234".as_bytes()).unwrap();
    assert_eq!(tmp, 1234_usize);

    let tmp: isize = parse("-1234".as_bytes()).unwrap();
    assert_eq!(tmp, -1234_isize);

    let tmp: u64 = parse("1234".as_bytes()).unwrap();
    assert_eq!(tmp, 1234_u64);

    let tmp: i64 = parse("-1234".as_bytes()).unwrap();
    assert_eq!(tmp, -1234_i64);

    let tmp: u128 = parse("999999".as_bytes()).unwrap();
    assert_eq!(tmp, 999999_u128);

    let tmp: i128 = parse("-999999".as_bytes()).unwrap();
    assert_eq!(tmp, -999999_i128);
}

#[test]
fn test_parse_pos() {
    let tmp: i8 = parse_pos("123".as_bytes()).unwrap();
    assert_eq!(tmp, 123_i8);

    let tmp: i16 = parse_pos("1234".as_bytes()).unwrap();
    assert_eq!(tmp, 1234_i16);

    let tmp: i32 = parse_pos("1234".as_bytes()).unwrap();
    assert_eq!(tmp, 1234_i32);

    let tmp: isize = parse_pos("1234".as_bytes()).unwrap();
    assert_eq!(tmp, 1234_isize);

    let tmp: i64 = parse_pos("1234".as_bytes()).unwrap();
    assert_eq!(tmp, 1234_i64);

    let tmp: i128 = parse_pos("999999".as_bytes()).unwrap();
    assert_eq!(tmp, 999999_i128);
}

#[test]
fn test_parse_neg() {
    let tmp: i8 = parse_neg("123".as_bytes()).unwrap();
    assert_eq!(tmp, -123_i8);

    let tmp: i16 = parse_neg("1234".as_bytes()).unwrap();
    assert_eq!(tmp, -1234_i16);

    let tmp: i32 = parse_neg("1234".as_bytes()).unwrap();
    assert_eq!(tmp, -1234_i32);

    let tmp: isize = parse_neg("1234".as_bytes()).unwrap();
    assert_eq!(tmp, -1234_isize);

    let tmp: i64 = parse_neg("1234".as_bytes()).unwrap();
    assert_eq!(tmp, -1234_i64);

    let tmp: i128 = parse_neg("999999".as_bytes()).unwrap();
    assert_eq!(tmp, -999999_i128);
}

#[test]
fn test_parse_until_invalid() {
    let tmp = parse_until_invalid::<u8>("123s".as_bytes()).unwrap();
    assert_eq!(tmp, (123_u8, 3));

    let tmp = parse_until_invalid::<i8>("-123s".as_bytes()).unwrap();
    assert_eq!(tmp, (-123_i8, 4));

    let tmp = parse_until_invalid::<u16>("1234s".as_bytes()).unwrap();
    assert_eq!(tmp, (1234_u16, 4));

    let tmp = parse_until_invalid::<i16>("-1234s".as_bytes()).unwrap();
    assert_eq!(tmp, (-1234_i16, 5));

    let tmp = parse_until_invalid::<u32>("1234s".as_bytes()).unwrap();
    assert_eq!(tmp, (1234_u32, 4));

    let tmp = parse_until_invalid::<i32>("-1234s".as_bytes()).unwrap();
    assert_eq!(tmp, (-1234_i32, 5));

    let tmp = parse_until_invalid::<u64>("1234s".as_bytes()).unwrap();
    assert_eq!(tmp, (1234_u64, 4));

    let tmp = parse_until_invalid::<i64>("-1234s".as_bytes()).unwrap();
    assert_eq!(tmp, (-1234_i64, 5));

    let tmp = parse_until_invalid::<u128>("1234s".as_bytes()).unwrap();
    assert_eq!(tmp, (1234_u128, 4));

    let tmp = parse_until_invalid::<i128>("-1234s".as_bytes()).unwrap();
    assert_eq!(tmp, (-1234_i128, 5));

    let tmp = parse_until_invalid::<u128>("12345678901234567890s".as_bytes()).unwrap();
    assert_eq!(tmp, (12345678901234567890_u128, 20));

    let tmp = parse_until_invalid::<i128>("-12345678901234567890s".as_bytes()).unwrap();
    assert_eq!(tmp, (-12345678901234567890_i128, 21));

    let tmp = parse_until_invalid::<u64>(
        "12345678901234567890s11111111111111111111111111111111111111111111111111111111111111"
            .as_bytes(),
    )
    .unwrap();
    assert_eq!(tmp, (12345678901234567890_u64, 20));

    let tmp = parse_until_invalid::<i64>(
        "-1234567890123456789s11111111111111111111111111111111111111111111111111111111111111"
            .as_bytes(),
    )
    .unwrap();
    assert_eq!(tmp, (-1234567890123456789_i64, 20));

    let tmp = parse_until_invalid::<u128>(
        "12345678901234567890s11111111111111111111111111111111111111111111111111111111111111"
            .as_bytes(),
    )
    .unwrap();
    assert_eq!(tmp, (12345678901234567890_u128, 20));

    let tmp = parse_until_invalid::<i128>(
        "-12345678901234567890s11111111111111111111111111111111111111111111111111111111111111"
            .as_bytes(),
    )
    .unwrap();
    assert_eq!(tmp, (-12345678901234567890_i128, 21));

    assert_eq!(
        parse_until_invalid::<u64>("0 asdf".as_bytes()).unwrap(),
        (0, 1)
    );

    assert_eq!(
        parse_until_invalid::<u64>("1:2 3:4 1:5s".as_bytes()).unwrap(),
        (1, 1)
    );

    assert_eq!(
        parse_until_invalid::<u64>("12:2 3:4 1:5s".as_bytes()).unwrap(),
        (12, 2)
    );

    assert_eq!(
        parse_until_invalid::<u64>("123:2 3:4 1:5s".as_bytes()).unwrap(),
        (123, 3)
    );

    assert_eq!(
        parse_until_invalid::<u64>("1234:2 3:4 1:5s".as_bytes()).unwrap(),
        (1234, 4)
    );

    assert_eq!(
        parse_until_invalid::<u64>("18446744073709551615".as_bytes()).unwrap(),
        (u64::MAX, 20)
    );

    if parse_until_invalid::<u64>("18446744073709551616".as_bytes()).is_ok() {
        panic!("error");
    }

    if parse_until_invalid::<u64>("99999999999999999999".as_bytes()).is_ok() {
        panic!("error");
    }

    assert_eq!(
        parse_until_invalid::<i64>("9223372036854775807".as_bytes()).unwrap(),
        (i64::MAX, 19)
    );

    if parse_until_invalid::<i64>("9223372036854775808".as_bytes()).is_ok() {
        panic!("error");
    }

    assert_eq!(
        parse_until_invalid::<i64>("-9223372036854775808".as_bytes()).unwrap(),
        (i64::MIN, 20)
    );

    if parse_until_invalid::<i64>("-9223372036854775809".as_bytes()).is_ok() {
        panic!("error");
    }

    if parse_until_invalid::<i64>("18446744073709551615".as_bytes()).is_ok() {
        panic!("error");
    }

    if parse_until_invalid::<i64>("99999999999999999999".as_bytes()).is_ok() {
        panic!("error");
    }

    if parse_until_invalid::<i64>("-99999999999999999999".as_bytes()).is_ok() {
        panic!("error");
    }

    assert_eq!(
        parse_until_invalid::<u128>("9999999999999999".as_bytes()).unwrap(),
        (9_999_999_999_999_999_u128, 16)
    );

    assert_eq!(
        parse_until_invalid::<u128>("12345678901234567890123456789012".as_bytes()).unwrap(),
        (1234567890_1234567890_1234567890_12_u128, 32)
    );

    assert_eq!(
        parse_until_invalid::<i128>("-9999999999999999".as_bytes()).unwrap(),
        (-9_999_999_999_999_999_i128, 17)
    );

    assert_eq!(
        parse_until_invalid::<i128>("9999999999999999".as_bytes()).unwrap(),
        (9_999_999_999_999_999_i128, 16)
    );

    assert_eq!(
        parse_until_invalid::<i128>("-99999999999999999999999999999999".as_bytes()).unwrap(),
        (-99_999_999_999_999_999_999_999_999_999_999_i128, 33)
    );

    assert_eq!(
        parse_until_invalid::<i128>("99999999999999999999999999999999".as_bytes()).unwrap(),
        (99_999_999_999_999_999_999_999_999_999_999_i128, 32)
    );

    assert_eq!(
        parse_until_invalid::<i128>("12345678901234567890123456789012".as_bytes()).unwrap(),
        (1234567890_1234567890_1234567890_12_i128, 32)
    );

    assert_eq!(
        parse_until_invalid::<i128>("-12345678901234567890123456789012".as_bytes()).unwrap(),
        (-1234567890_1234567890_1234567890_12_i128, 33)
    );
}

#[test]
fn test_parse_until_invalid_pos() {
    let tmp = parse_until_invalid_pos::<u8>("123s".as_bytes()).unwrap();
    assert_eq!(tmp, (123_u8, 3));

    let tmp = parse_until_invalid_pos::<i8>("123s".as_bytes()).unwrap();
    assert_eq!(tmp, (123_i8, 3));

    let tmp = parse_until_invalid_pos::<u16>("123s".as_bytes()).unwrap();
    assert_eq!(tmp, (123_u16, 3));

    let tmp = parse_until_invalid_pos::<u16>("1234s".as_bytes()).unwrap();
    assert_eq!(tmp, (1234_u16, 4));

    let tmp = parse_until_invalid_pos::<i16>("1234s".as_bytes()).unwrap();
    assert_eq!(tmp, (1234_i16, 4));

    let tmp = parse_until_invalid_pos::<u32>("1234s".as_bytes()).unwrap();
    assert_eq!(tmp, (1234_u32, 4));

    let tmp = parse_until_invalid_pos::<i32>("1234s".as_bytes()).unwrap();
    assert_eq!(tmp, (1234_i32, 4));

    let tmp = parse_until_invalid_pos::<u64>("1234s".as_bytes()).unwrap();
    assert_eq!(tmp, (1234_u64, 4));

    let tmp = parse_until_invalid_pos::<i64>("1234s".as_bytes()).unwrap();
    assert_eq!(tmp, (1234_i64, 4));

    let tmp = parse_until_invalid_pos::<u128>("1234s".as_bytes()).unwrap();
    assert_eq!(tmp, (1234_u128, 4));

    let tmp = parse_until_invalid_pos::<u128>("12345678901234567890s".as_bytes()).unwrap();
    assert_eq!(tmp, (12345678901234567890_u128, 20));

    let tmp = parse_until_invalid_pos::<i128>("1234s".as_bytes()).unwrap();
    assert_eq!(tmp, (1234_i128, 4));

    let tmp = parse_until_invalid_pos::<i128>("12345678901234567890s".as_bytes()).unwrap();
    assert_eq!(tmp, (12345678901234567890_i128, 20));
}

#[test]
fn test_parse_until_invalid_neg() {
    let tmp = parse_until_invalid_neg::<i8>("123s".as_bytes()).unwrap();
    assert_eq!(tmp, (-123_i8, 3));

    let tmp = parse_until_invalid_neg::<i16>("1234s".as_bytes()).unwrap();
    assert_eq!(tmp, (-1234_i16, 4));

    let tmp = parse_until_invalid_neg::<i32>("1234s".as_bytes()).unwrap();
    assert_eq!(tmp, (-1234_i32, 4));

    let tmp = parse_until_invalid_neg::<i64>("1234s".as_bytes()).unwrap();
    assert_eq!(tmp, (-1234_i64, 4));

    let tmp = parse_until_invalid_neg::<i128>("1234s".as_bytes()).unwrap();
    assert_eq!(tmp, (-1234_i128, 4));

    let tmp = parse_until_invalid_neg::<i128>("12345678901234567890s".as_bytes()).unwrap();
    assert_eq!(tmp, (-12345678901234567890_i128, 20));
}
