mod reimpl;
#[allow(unused_imports)]
use reimpl::*;

use ::core::{fmt::Debug, str::FromStr};
use arrayvec::ArrayString;

const INVALID_CHARS: [char; 6] = ['/', ':', '\0', '\x7f', '!', 'a'];

fn test_each_position<T: Copy>(s: &str, func: fn(&[u8]) -> Result<T, AtoiSimdError<'_>>) {
    let mut s_new = ArrayString::<101>::new();
    for j in 0..=s.len() {
        for ch_invalid in INVALID_CHARS {
            s_new.push_str(&s[0..j]);
            s_new.push(ch_invalid);
            s_new.push_str(&s[j..s.len()]);
            assert!(func(s_new.as_bytes()).is_err(), "input: {}", s_new);
            s_new.clear();
        }
    }
}

fn test_each_position_prefix<T: Copy + Debug + PartialEq + FromStr>(
    s: &str,
    func: fn(&[u8]) -> Result<(T, usize), AtoiSimdError<'_>>,
) where
    <T as FromStr>::Err: Debug,
{
    let mut s_new = ArrayString::<101>::new();
    for j in 1..=s.len() {
        for ch_invalid in INVALID_CHARS {
            let ts = &s[0..j];
            s_new.push_str(ts);
            s_new.push(ch_invalid);
            s_new.push_str(&s[j..s.len()]);
            assert_eq!(
                func(s_new.as_bytes()).unwrap(),
                (ts.parse::<T>().unwrap(), j),
                "input: {}",
                s_new
            );
            s_new.clear();
        }
    }
}

fn test_each_zeroes<T: Copy + Debug + PartialEq + FromStr>(
    s: &str,
    func: fn(&[u8]) -> Result<T, AtoiSimdError<'_>>,
) where
    <T as FromStr>::Err: Debug,
{
    let mut s_new = ArrayString::<100>::new();
    for i in 0..=(99 - s.len()) {
        s_new.truncate(i);
        s_new.push('0');
        s_new.push_str(s);
        assert_eq!(
            func(s_new.as_bytes()).unwrap(),
            s_new.parse::<T>().unwrap(),
            "input: {}",
            s_new
        );
        test_each_position(&s_new, func);
    }
}

fn test_each_zeroes_prefix<T: Copy + Debug + PartialEq + FromStr>(
    s: &str,
    func: fn(&[u8]) -> Result<(T, usize), AtoiSimdError<'_>>,
) where
    <T as FromStr>::Err: Debug,
{
    let mut s_new = ArrayString::<100>::new();
    for i in 0..=(99 - s.len()) {
        s_new.truncate(i);
        s_new.push('0');
        s_new.push_str(s);
        assert_eq!(
            func(s_new.as_bytes()).unwrap(),
            (s_new.parse::<T>().unwrap(), s_new.len()),
            "input: {}",
            s_new
        );
        test_each_position_prefix(&s_new, func);
    }
}

fn parse_tester<
    T: Copy + Debug + PartialEq + FromStr + Parse,
    const LEN: usize,
    const LEN_NEG: usize,
    const SKIP_ZEROES: bool,
    I,
>(
    chars: I,
) where
    <T as FromStr>::Err: Debug,
    I: Iterator<Item = char>,
{
    let mut s = ArrayString::<LEN>::new();
    let mut s_neg = ArrayString::<LEN_NEG>::new();
    if LEN_NEG > 0 {
        s_neg.push('-');
    }
    test_each_position(&s, atoi_simd::parse::<T, SKIP_ZEROES, true>);
    if SKIP_ZEROES {
        test_each_zeroes(&s, atoi_simd::parse::<T, true, true>);
    }
    for ch in chars {
        s.push(ch);
        assert_eq!(
            atoi_simd::parse::<T, SKIP_ZEROES, true>(s.as_bytes()).unwrap(),
            s.parse::<T>().unwrap()
        );

        if LEN_NEG > 0 {
            s_neg.push(ch);
            assert_eq!(
                atoi_simd::parse::<T, SKIP_ZEROES, true>(s_neg.as_bytes()).unwrap(),
                s_neg.parse::<T>().unwrap()
            );
        }
        test_each_position(&s, atoi_simd::parse::<T, SKIP_ZEROES, true>);
        if SKIP_ZEROES {
            test_each_zeroes(&s, atoi_simd::parse::<T, true, true>);
        }
    }
    assert_eq!(s.len(), LEN);
    assert_eq!(s_neg.len(), LEN_NEG);
}

fn parse_prefix_tester<
    T: Copy + Debug + PartialEq + FromStr + Parse,
    const LEN: usize,
    const LEN_NEG: usize,
    const SKIP_ZEROES: bool,
    I,
>(
    chars: I,
) where
    <T as FromStr>::Err: Debug,
    I: Iterator<Item = char>,
{
    let mut s = ArrayString::<LEN>::new();
    let mut s_neg = ArrayString::<LEN_NEG>::new();
    if LEN_NEG > 0 {
        s_neg.push('-');
    }
    for ch in chars {
        s.push(ch);
        assert_eq!(
            atoi_simd::parse_prefix::<T, SKIP_ZEROES, true>(s.as_bytes()).unwrap(),
            (s.parse::<T>().unwrap(), s.len())
        );

        if LEN_NEG > 0 {
            s_neg.push(ch);
            assert_eq!(
                atoi_simd::parse_prefix::<T, SKIP_ZEROES, true>(s_neg.as_bytes()).unwrap(),
                (s_neg.parse::<T>().unwrap(), s_neg.len())
            );
        }
        test_each_position_prefix(&s, atoi_simd::parse_prefix::<T, SKIP_ZEROES, true>);
        if SKIP_ZEROES {
            test_each_zeroes_prefix(&s, atoi_simd::parse_prefix::<T, true, true>);
        }
    }
    assert_eq!(s.len(), LEN);
    assert_eq!(s_neg.len(), LEN_NEG);
}

#[test]
fn test_parse_u8() {
    assert!(parse::<u8>(b"").is_err());

    assert_eq!(parse::<u8>(b"0").unwrap(), 0_u8);

    parse_tester::<u8, 3, 0, false, _>('1'..='3');
    parse_tester::<u8, 4, 0, true, _>('0'..='3');

    assert_eq!(parse::<u8>(b"255").unwrap(), u8::MAX);

    assert!(parse::<u8>(b"256").is_err());
    assert!(parse::<u8>(b"12345678").is_err());
    assert!(parse::<u8>(b"1234567890123456789012345").is_err());
}

#[test]
fn test_parse_prefix_u8() {
    assert!(parse_prefix::<u8>(b"").is_err());

    assert_eq!(parse_prefix::<u8>(b"0").unwrap(), (0_u8, 1_usize));

    parse_prefix_tester::<u8, 3, 0, false, _>('1'..='3');
    parse_prefix_tester::<u8, 4, 0, true, _>('0'..='3');
}

#[test]
fn test_parse_i8() {
    assert!(parse::<i8>(b"").is_err());
    assert!(parse::<i8>(b"-").is_err());
    assert_eq!(parse::<i8>(b"0").unwrap(), 0_i8);
    assert_eq!(parse::<i8>(b"-0").unwrap(), 0_i8);

    parse_tester::<i8, 3, 4, false, _>('1'..='3');
    parse_tester::<i8, 4, 5, true, _>('0'..='3');

    assert_eq!(parse::<i8>(b"127").unwrap(), i8::MAX);
    assert_eq!(parse::<i8>(b"-128").unwrap(), i8::MIN);

    assert!(parse::<i8>(b"128").is_err());
    assert!(parse::<i8>(b"-129").is_err());
    assert!(parse::<i8>(b"255").is_err());
    assert!(parse::<i8>(b"12345678").is_err());
    assert!(parse::<i8>(b"-12345678").is_err());
    assert!(parse::<i8>(b"-1234567890123456789012345").is_err());
}

#[test]
fn test_parse_u16() {
    assert!(parse::<u16>(b"").is_err());

    assert_eq!(parse::<u16>(b"0").unwrap(), 0_u16);

    parse_tester::<u16, 5, 0, false, _>('1'..='5');
    parse_tester::<u16, 6, 0, true, _>('0'..='5');

    assert_eq!(parse::<u16>(b"65535").unwrap(), u16::MAX);

    assert!(parse::<u16>(b"65536").is_err());
    assert!(parse::<u16>(b"12345678").is_err());
    assert!(parse::<u16>(b"1234567890123456789012345").is_err());
}

#[test]
fn test_parse_prefix_u16() {
    assert!(parse_prefix::<u16>(b"").is_err());

    assert_eq!(parse_prefix::<u16>(b"0").unwrap(), (0_u16, 1_usize));

    parse_prefix_tester::<u16, 5, 0, false, _>('1'..='5');
    parse_prefix_tester::<u16, 6, 0, true, _>('0'..='5');
}

#[test]
fn test_parse_i16() {
    assert!(parse::<i16>(b"").is_err());
    assert!(parse::<i16>(b"-").is_err());

    assert_eq!(parse::<i16>(b"0").unwrap(), 0_i16);
    assert_eq!(parse::<i16>(b"-0").unwrap(), 0_i16);

    parse_tester::<i16, 5, 6, false, _>('1'..='5');
    parse_tester::<i16, 6, 7, true, _>('0'..='5');

    assert_eq!(parse::<i16>(b"32767").unwrap(), i16::MAX);
    assert_eq!(parse::<i16>(b"-32768").unwrap(), i16::MIN);

    assert!(parse::<i16>(b"32768").is_err());
    assert!(parse::<i16>(b"-32769").is_err());
    assert!(parse::<i16>(b"65535").is_err());
    assert!(parse::<i16>(b"12345678").is_err());
    assert!(parse::<i16>(b"-12345678").is_err());
    assert!(parse::<i16>(b"-1234567890123456789012345").is_err());
}

#[test]
fn test_parse_u32() {
    assert!(parse::<u32>(b"").is_err());

    assert_eq!(parse::<u32>(b"0").unwrap(), 0_u32);

    parse_tester::<u32, 10, 0, false, _>(('1'..='9').chain('0'..='0'));
    parse_tester::<u32, 11, 0, true, _>(('0'..='9').chain('0'..='0'));

    assert_eq!(parse::<u32>(b"4294967295").unwrap(), u32::MAX);

    assert!(parse::<u32>(b"4294967296").is_err());
    assert!(parse::<u32>(b"123456789012345").is_err());
    assert!(parse::<u32>(b"1234567890123456789012345").is_err());
}

#[test]
fn test_parse_prefix_u32() {
    assert!(parse_prefix::<u32>(b"").is_err());

    assert_eq!(parse_prefix::<u32>(b"0").unwrap(), (0_u32, 1_usize));

    parse_prefix_tester::<u32, 10, 0, false, _>(('1'..='9').chain('0'..='0'));
    parse_prefix_tester::<u32, 11, 0, true, _>(('0'..='9').chain('0'..='0'));
}

#[test]
fn test_parse_i32() {
    assert!(parse::<i32>(b"").is_err());
    assert!(parse::<i32>(b"-").is_err());

    assert_eq!(parse::<i32>(b"0").unwrap(), 0_i32);
    assert_eq!(parse::<i32>(b"-0").unwrap(), 0_i32);

    parse_tester::<i32, 10, 11, false, _>(('1'..='9').chain('0'..='0'));
    parse_tester::<i32, 11, 12, true, _>(('0'..='9').chain('0'..='0'));

    assert_eq!(parse::<i32>(b"2147483647").unwrap(), i32::MAX);
    assert_eq!(parse::<i32>(b"-2147483648").unwrap(), i32::MIN);

    assert!(parse::<i32>(b"2147483648").is_err());
    assert!(parse::<i32>(b"-2147483649").is_err());
    assert!(parse::<i32>(b"4294967295").is_err());
    assert!(parse::<i32>(b"123456789012345").is_err());
    assert!(parse::<i32>(b"-123456789012345").is_err());
    assert!(parse::<i32>(b"-1234567890123456789012345").is_err());
}

#[test]
fn test_parse_u64() {
    assert!(parse::<u64>(b"").is_err());

    assert_eq!(parse::<u64>(b"0").unwrap(), 0_u64);

    parse_tester::<u64, 20, 0, false, _>(('1'..='9').chain('0'..='9').chain('0'..='0'));
    parse_tester::<u64, 21, 0, true, _>(('0'..='9').chain('0'..='9').chain('0'..='0'));

    assert_eq!(parse::<u64>(b"18446744073709551615").unwrap(), u64::MAX);

    assert!(parse::<u64>(b"18446744073709551616").is_err());
    assert!(parse::<u64>(b"99999999999999999999").is_err());
    assert!(parse::<u64>(b"1234567890123456789012345").is_err());
}

#[test]
fn test_parse_prefix_u64() {
    assert!(parse_prefix::<u64>(b"").is_err());

    assert_eq!(parse_prefix::<u64>(b"0").unwrap(), (0_u64, 1_usize));

    parse_prefix_tester::<u64, 20, 0, false, _>(('1'..='9').chain('0'..='9').chain('0'..='0'));
    parse_prefix_tester::<u64, 21, 0, true, _>(('0'..='9').chain('0'..='9').chain('0'..='0'));

    assert_eq!(
        parse_prefix::<u64>(b"18446744073709551615").unwrap(),
        (u64::MAX, 20)
    );

    assert_eq!(
        parse_prefix::<u64>(b"18446744073709551615s").unwrap(),
        (u64::MAX, 20)
    );

    assert!(parse_prefix::<u64>(b"18446744073709551616").is_err());
    assert!(parse_prefix::<u64>(b"18446744073709551616s").is_err());
    assert!(parse_prefix::<u64>(b"99999999999999999999").is_err());
    assert!(parse_prefix::<u64>(b"99999999999999999999s").is_err());
    assert!(parse_prefix::<u64>(b"1234567890123456789012345").is_err());
}

#[test]
fn test_parse_i64() {
    assert!(parse::<i64>(b"").is_err());
    assert!(parse::<i64>(b"-").is_err());

    assert_eq!(parse::<i64>(b"0").unwrap(), 0_i64);
    assert_eq!(parse::<i64>(b"-0").unwrap(), 0_i64);

    parse_tester::<i64, 19, 20, false, _>(('1'..='9').chain('0'..='9'));
    parse_tester::<i64, 20, 21, true, _>(('0'..='9').chain('0'..='9'));

    assert_eq!(parse::<i64>(b"9223372036854775807").unwrap(), i64::MAX);
    assert_eq!(parse::<i64>(b"-9223372036854775808").unwrap(), i64::MIN);

    assert!(parse::<i64>(b"9223372036854775808").is_err());
    assert!(parse::<i64>(b"-9223372036854775809").is_err());
    assert!(parse::<i64>(b"18446744073709551615").is_err());
    assert!(parse::<i64>(b"99999999999999999999").is_err());
    assert!(parse::<i64>(b"-99999999999999999999").is_err());
    assert!(parse::<i64>(b"-1234567890123456789012345").is_err());
}

#[test]
fn test_parse_u128() {
    assert!(parse::<u128>(b"").is_err());

    assert_eq!(parse::<u128>(b"0").unwrap(), 0_u128);

    parse_tester::<u128, 39, 0, false, _>(
        ('1'..='9')
            .chain('0'..='9')
            .chain('0'..='9')
            .chain('0'..='9'),
    );
    parse_tester::<u128, 40, 0, true, _>(
        ('0'..='9')
            .chain('0'..='9')
            .chain('0'..='9')
            .chain('0'..='9'),
    );

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

    assert!(parse::<u128>(b"340282366920938463463374607431768211456").is_err());
    assert!(parse::<u128>(b"999999999999999999999999999999999999999").is_err());
    assert!(parse::<u128>(b"9999999999999999999999999999999999999999999").is_err());
}

#[test]
fn test_parse_prefix_u128() {
    assert!(parse_prefix::<u128>(b"").is_err());

    assert_eq!(parse_prefix::<u128>(b"0").unwrap(), (0_u128, 1_usize));

    parse_prefix_tester::<u128, 39, 0, false, _>(
        ('1'..='9')
            .chain('0'..='9')
            .chain('0'..='9')
            .chain('0'..='9'),
    );
    parse_prefix_tester::<u128, 40, 0, true, _>(
        ('0'..='9')
            .chain('0'..='9')
            .chain('0'..='9')
            .chain('0'..='9'),
    );

    assert_eq!(
        parse_prefix::<u128>(b"340282366920938463463374607431768211455").unwrap(),
        (u128::MAX, 39)
    );

    assert_eq!(
        parse_prefix::<u128>(b"340282366920938463463374607431768211455s").unwrap(),
        (u128::MAX, 39)
    );

    assert!(parse_prefix::<u128>(b"340282366920938463463374607431768211456").is_err());
    assert!(parse_prefix::<u128>(b"340282366920938463463374607431768211456s").is_err());
    assert!(parse_prefix::<u128>(b"999999999999999999999999999999999999999").is_err());
    assert!(parse_prefix::<u128>(b"999999999999999999999999999999999999999s").is_err());
    assert!(parse_prefix::<u128>(b"9999999999999999999999999999999999999999999").is_err());
}

#[test]
fn test_parse_i128() {
    assert!(parse::<i128>(b"").is_err());
    assert!(parse::<i128>(b"-").is_err());

    assert_eq!(parse::<i128>(b"0").unwrap(), 0_i128);
    assert_eq!(parse::<i128>(b"-0").unwrap(), 0_i128);

    parse_tester::<i128, 39, 40, false, _>(
        ('1'..='9')
            .chain('0'..='9')
            .chain('0'..='9')
            .chain('0'..='9'),
    );
    parse_tester::<i128, 40, 41, true, _>(
        ('0'..='9')
            .chain('0'..='9')
            .chain('0'..='9')
            .chain('0'..='9'),
    );

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

    assert!(parse::<i128>(b"170141183460469231731687303715884105728").is_err());
    assert!(parse::<i128>(b"-170141183460469231731687303715884105729").is_err());
    assert!(parse::<i128>(b"-999999999999999999999999999999999999999").is_err());
    assert!(parse::<i128>(b"-9999999999999999999999999999999999999999999").is_err());
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
fn test_zeroes() {
    let tmp: u8 = parse(b"0000000000000000").unwrap();
    assert_eq!(tmp, 0_u8);

    let tmp: u8 = parse_skipped(b"000000000000000000000000000000000000000000000000").unwrap();
    assert_eq!(tmp, 0_u8);

    let tmp: u8 = parse(b"0000000000000001").unwrap();
    assert_eq!(tmp, 1_u8);

    let tmp: i8 = parse(b"-0000000000000000").unwrap();
    assert_eq!(tmp, 0_i8);

    let tmp: i8 = parse(b"-0000000000000001").unwrap();
    assert_eq!(tmp, -1_i8);

    let tmp: u16 = parse(b"0000000000000000").unwrap();
    assert_eq!(tmp, 0_u16);

    let tmp: u16 = parse_skipped(b"000000000000000000000000000000000000000000000000").unwrap();
    assert_eq!(tmp, 0_u16);

    let tmp: u16 = parse(b"0000000000000001").unwrap();
    assert_eq!(tmp, 1_u16);

    let tmp: i16 = parse(b"-0000000000000000").unwrap();
    assert_eq!(tmp, 0_i16);

    let tmp: i16 = parse(b"-0000000000000001").unwrap();
    assert_eq!(tmp, -1_i16);

    let tmp: u32 = parse(b"0000000000000000").unwrap();
    assert_eq!(tmp, 0_u32);

    let tmp: u32 = parse(b"0000000000000001").unwrap();
    assert_eq!(tmp, 1_u32);

    let tmp: i32 = parse(b"-0000000000000000").unwrap();
    assert_eq!(tmp, 0_i32);

    let tmp: i32 = parse(b"-0000000000000001").unwrap();
    assert_eq!(tmp, -1_i32);

    assert_eq!(
        parse_skipped::<i32>(b"0000000000000000000000000000000000000000000000002147483647")
            .unwrap(),
        i32::MAX
    );
    assert_eq!(
        parse_skipped::<i32>(b"-0000000000000000000000000000000000000000000000002147483648")
            .unwrap(),
        i32::MIN
    );

    let tmp: usize = parse(b"0000000000000000").unwrap();
    assert_eq!(tmp, 0_usize);

    let tmp: usize = parse(b"0000000000000001").unwrap();
    assert_eq!(tmp, 1_usize);

    let tmp: isize = parse(b"-0000000000000000").unwrap();
    assert_eq!(tmp, 0_isize);

    let tmp: isize = parse(b"-0000000000000001").unwrap();
    assert_eq!(tmp, -1_isize);

    let tmp: u64 = parse(b"00000000000000000000").unwrap();
    assert_eq!(tmp, 0_u64);

    let tmp: u64 = parse(b"0000000000000123").unwrap();
    assert_eq!(tmp, 123_u64);

    let tmp: u64 = parse(b"00000000000000000001").unwrap();
    assert_eq!(tmp, 1_u64);

    let tmp: i64 = parse(b"-0000000000000000000").unwrap();
    assert_eq!(tmp, 0_i64);

    let tmp: i64 = parse(b"-0000000000000000001").unwrap();
    assert_eq!(tmp, -1_i64);

    assert_eq!(
        atoi_simd::parse_pos::<i64, true>(b"0000000000000000009223372036854775807").unwrap(),
        i64::MAX
    );
    assert_eq!(
        parse_skipped::<i64>(b"0000000000000000009223372036854775807").unwrap(),
        i64::MAX
    );

    assert_eq!(
        atoi_simd::parse_pos::<i64, true>(
            b"000000000000000000000000000000000000009223372036854775807"
        )
        .unwrap(),
        i64::MAX
    );
    assert_eq!(
        parse_skipped::<i64>(b"000000000000000000000000000000000000009223372036854775807").unwrap(),
        i64::MAX
    );

    let tmp: i128 = parse(b"-000000000000000000000000000000000000000").unwrap();
    assert_eq!(tmp, 0_i128);

    let tmp: i128 = parse(b"-000000000000000000000000000000000000001").unwrap();
    assert_eq!(tmp, -1_i128);

    test_each_zeroes("4294967295", atoi_simd::parse::<u32, true, false>);
    test_each_zeroes("18446744073709551615", atoi_simd::parse::<u64, true, false>);
    test_each_zeroes(
        "340282366920938463463374607431768211455",
        atoi_simd::parse::<u128, true, false>,
    );

    test_each_zeroes_prefix("4294967295", atoi_simd::parse_prefix::<u32, true, false>);
    test_each_zeroes_prefix(
        "18446744073709551615",
        atoi_simd::parse_prefix::<u64, true, false>,
    );
    test_each_zeroes_prefix(
        "340282366920938463463374607431768211455",
        atoi_simd::parse_prefix::<u128, true, false>,
    );
}

#[test]
fn test_parse_pos() {
    let tmp: i8 = parse_pos(b"123").unwrap();
    assert_eq!(tmp, 123_i8);

    assert!(parse_pos::<i8>((i8::MAX as u32 + 1).to_string().as_bytes()).is_err());

    let tmp: i16 = parse_pos(b"1234").unwrap();
    assert_eq!(tmp, 1234_i16);

    assert!(parse_pos::<i16>((i16::MAX as u32 + 1).to_string().as_bytes()).is_err());

    let tmp: i32 = parse_pos(b"1234").unwrap();
    assert_eq!(tmp, 1234_i32);

    assert!(parse_pos::<i32>((i32::MAX as u32 + 1).to_string().as_bytes()).is_err());

    let tmp: isize = parse_pos(b"1234").unwrap();
    assert_eq!(tmp, 1234_isize);

    assert!(parse_pos::<isize>((isize::MAX as u64 + 1).to_string().as_bytes()).is_err());

    let tmp: i64 = parse_pos(b"1234").unwrap();
    assert_eq!(tmp, 1234_i64);

    assert!(parse_pos::<i64>((i64::MAX as u64 + 1).to_string().as_bytes()).is_err());

    let tmp: i128 = parse_pos(b"999999").unwrap();
    assert_eq!(tmp, 999999_i128);

    assert!(parse_pos::<i128>((i128::MAX as u128 + 1).to_string().as_bytes()).is_err());
}

#[test]
fn test_parse_neg() {
    let tmp: i8 = parse_neg(b"123").unwrap();
    assert_eq!(tmp, -123_i8);

    assert!(parse_neg::<i8>((i8::MAX as u32 + 2).to_string().as_bytes()).is_err());

    let tmp: i16 = parse_neg(b"1234").unwrap();
    assert_eq!(tmp, -1234_i16);

    assert!(parse_neg::<i16>((i16::MAX as u32 + 2).to_string().as_bytes()).is_err());

    let tmp: i32 = parse_neg(b"1234").unwrap();
    assert_eq!(tmp, -1234_i32);

    assert!(parse_neg::<i32>((i32::MAX as u32 + 2).to_string().as_bytes()).is_err());

    let tmp: isize = parse_neg(b"1234").unwrap();
    assert_eq!(tmp, -1234_isize);

    assert!(parse_neg::<isize>((isize::MAX as u64 + 2).to_string().as_bytes()).is_err());

    let tmp: i64 = parse_neg(b"1234").unwrap();
    assert_eq!(tmp, -1234_i64);

    assert!(parse_neg::<i64>((i64::MAX as u64 + 2).to_string().as_bytes()).is_err());

    let tmp: i128 = parse_neg(b"999999").unwrap();
    assert_eq!(tmp, -999999_i128);

    assert!(parse_neg::<i128>((i128::MAX as u128 + 2).to_string().as_bytes()).is_err());
}

#[test]
fn test_parse_prefix() {
    let tmp = parse_prefix::<u8>(b"123s").unwrap();
    assert_eq!(tmp, (123_u8, 3));

    let tmp = parse_prefix::<i8>(b"-123s").unwrap();
    assert_eq!(tmp, (-123_i8, 4));

    let tmp = parse_prefix::<u16>(b"1234s").unwrap();
    assert_eq!(tmp, (1234_u16, 4));

    let tmp = parse_prefix::<i16>(b"-1234s").unwrap();
    assert_eq!(tmp, (-1234_i16, 5));

    let tmp = parse_prefix::<u32>(b"1234s").unwrap();
    assert_eq!(tmp, (1234_u32, 4));

    let tmp = parse_prefix::<i32>(b"-1234s").unwrap();
    assert_eq!(tmp, (-1234_i32, 5));

    let tmp = parse_prefix::<u64>(b"1234s").unwrap();
    assert_eq!(tmp, (1234_u64, 4));

    let tmp = parse_prefix::<i64>(b"-1234s").unwrap();
    assert_eq!(tmp, (-1234_i64, 5));

    let tmp = parse_prefix::<u128>(b"1234s").unwrap();
    assert_eq!(tmp, (1234_u128, 4));

    let tmp = parse_prefix::<i128>(b"-1234s").unwrap();
    assert_eq!(tmp, (-1234_i128, 5));

    let tmp = parse_prefix::<u128>(b"12345678901234567890s").unwrap();
    assert_eq!(tmp, (12345678901234567890_u128, 20));

    let tmp = parse_prefix::<i128>(b"-12345678901234567890s").unwrap();
    assert_eq!(tmp, (-12345678901234567890_i128, 21));

    let tmp = parse_prefix::<u64>(
        b"12345678901234567890s11111111111111111111111111111111111111111111111111111111111111",
    )
    .unwrap();
    assert_eq!(tmp, (12345678901234567890_u64, 20));

    let tmp = parse_prefix::<i64>(
        b"-1234567890123456789s11111111111111111111111111111111111111111111111111111111111111",
    )
    .unwrap();
    assert_eq!(tmp, (-1234567890123456789_i64, 20));

    let tmp = parse_prefix::<u128>(
        b"12345678901234567890s11111111111111111111111111111111111111111111111111111111111111",
    )
    .unwrap();
    assert_eq!(tmp, (12345678901234567890_u128, 20));

    let tmp = parse_prefix::<i128>(
        b"-12345678901234567890s11111111111111111111111111111111111111111111111111111111111111",
    )
    .unwrap();
    assert_eq!(tmp, (-12345678901234567890_i128, 21));

    assert_eq!(parse_prefix::<u64>(b"0 asdf").unwrap(), (0, 1));

    assert_eq!(parse_prefix::<u64>(b"1:2 3:4 1:5s").unwrap(), (1, 1));

    assert_eq!(parse_prefix::<u64>(b"12:2 3:4 1:5s").unwrap(), (12, 2));

    assert_eq!(parse_prefix::<u64>(b"123:2 3:4 1:5s").unwrap(), (123, 3));

    assert_eq!(parse_prefix::<u64>(b"1234:2 3:4 1:5s").unwrap(), (1234, 4));

    assert_eq!(
        parse_prefix::<u64>(b"18446744073709551615").unwrap(),
        (u64::MAX, 20)
    );

    assert_eq!(
        parse_prefix::<u64>(b"18446744073709551615s11111111111").unwrap(),
        (u64::MAX, 20)
    );

    assert!(parse_prefix::<u64>(b"18446744073709551616").is_err());
    assert!(parse_prefix::<u64>(b"99999999999999999999").is_err());

    assert_eq!(
        parse_prefix::<i64>(b"9223372036854775807").unwrap(),
        (i64::MAX, 19)
    );

    assert_eq!(
        parse_prefix::<i64>(b"-9223372036854775808").unwrap(),
        (i64::MIN, 20)
    );

    assert!(parse_prefix::<i64>(b"-").is_err());
    assert!(parse_prefix::<i64>(b"9223372036854775808").is_err());
    assert!(parse_prefix::<i64>(b"-9223372036854775809").is_err());
    assert!(parse_prefix::<i64>(b"18446744073709551615").is_err());
    assert!(parse_prefix::<i64>(b"99999999999999999999").is_err());
    assert!(parse_prefix::<i64>(b"-99999999999999999999").is_err());

    assert_eq!(
        parse_prefix::<u128>(b"9999999999999999").unwrap(),
        (9_999_999_999_999_999_u128, 16)
    );

    assert_eq!(
        parse_prefix::<u128>(b"12345678901234567890123456789012").unwrap(),
        (1234567890_1234567890_1234567890_12_u128, 32)
    );

    assert_eq!(
        parse_prefix::<u128>(b"12345678901234567890123456789012s1111111111111").unwrap(),
        (1234567890_1234567890_1234567890_12_u128, 32)
    );

    assert_eq!(
        parse_prefix::<u128>(b"123456789012345678901234567890123456789s1111111111111").unwrap(),
        (1234567890_1234567890_1234567890_123456789_u128, 39)
    );

    assert_eq!(
        parse_prefix::<i128>(b"-9999999999999999").unwrap(),
        (-9_999_999_999_999_999_i128, 17)
    );

    assert_eq!(
        parse_prefix::<i128>(b"9999999999999999").unwrap(),
        (9_999_999_999_999_999_i128, 16)
    );

    assert_eq!(
        parse_prefix::<i128>(b"-99999999999999999999999999999999").unwrap(),
        (-99_999_999_999_999_999_999_999_999_999_999_i128, 33)
    );

    assert_eq!(
        parse_prefix::<i128>(b"99999999999999999999999999999999").unwrap(),
        (99_999_999_999_999_999_999_999_999_999_999_i128, 32)
    );

    assert_eq!(
        parse_prefix::<i128>(b"12345678901234567890123456789012").unwrap(),
        (1234567890_1234567890_1234567890_12_i128, 32)
    );

    assert_eq!(
        parse_prefix::<i128>(b"-12345678901234567890123456789012").unwrap(),
        (-1234567890_1234567890_1234567890_12_i128, 33)
    );
}

#[test]
fn test_parse_prefix_pos() {
    let tmp = parse_prefix_pos::<u8>(b"123s").unwrap();
    assert_eq!(tmp, (123_u8, 3));

    let tmp = parse_prefix_pos::<i8>(b"123s").unwrap();
    assert_eq!(tmp, (123_i8, 3));

    let tmp = parse_prefix_pos::<u16>(b"123s").unwrap();
    assert_eq!(tmp, (123_u16, 3));

    let tmp = parse_prefix_pos::<u16>(b"1234s").unwrap();
    assert_eq!(tmp, (1234_u16, 4));

    let tmp = parse_prefix_pos::<i16>(b"1234s").unwrap();
    assert_eq!(tmp, (1234_i16, 4));

    let tmp = parse_prefix_pos::<u32>(b"1234s").unwrap();
    assert_eq!(tmp, (1234_u32, 4));

    let tmp = parse_prefix_pos::<i32>(b"1234s").unwrap();
    assert_eq!(tmp, (1234_i32, 4));

    let tmp = parse_prefix_pos::<u64>(b"1234s").unwrap();
    assert_eq!(tmp, (1234_u64, 4));

    let tmp = parse_prefix_pos::<i64>(b"1234s").unwrap();
    assert_eq!(tmp, (1234_i64, 4));

    let tmp = parse_prefix_pos::<u128>(b"1234s").unwrap();
    assert_eq!(tmp, (1234_u128, 4));

    let tmp = parse_prefix_pos::<u128>(b"12345678901234567890s").unwrap();
    assert_eq!(tmp, (12345678901234567890_u128, 20));

    let tmp = parse_prefix_pos::<i128>(b"1234s").unwrap();
    assert_eq!(tmp, (1234_i128, 4));

    let tmp = parse_prefix_pos::<i128>(b"12345678901234567890s").unwrap();
    assert_eq!(tmp, (12345678901234567890_i128, 20));
}

#[test]
fn test_parse_prefix_neg() {
    let tmp = parse_prefix_neg::<i8>(b"123s").unwrap();
    assert_eq!(tmp, (-123_i8, 3));

    let tmp = parse_prefix_neg::<i16>(b"1234s").unwrap();
    assert_eq!(tmp, (-1234_i16, 4));

    let tmp = parse_prefix_neg::<i32>(b"1234s").unwrap();
    assert_eq!(tmp, (-1234_i32, 4));

    let tmp = parse_prefix_neg::<i64>(b"1234s").unwrap();
    assert_eq!(tmp, (-1234_i64, 4));

    let tmp = parse_prefix_neg::<i128>(b"1234s").unwrap();
    assert_eq!(tmp, (-1234_i128, 4));

    let tmp = parse_prefix_neg::<i128>(b"12345678901234567890s").unwrap();
    assert_eq!(tmp, (-12345678901234567890_i128, 20));
}

#[test]
fn test_parse_skipped() {
    let tmp = parse_skipped::<u8>(b"+0000000000000000000000000123").unwrap();
    assert_eq!(tmp, 123_u8);

    let tmp = parse_skipped::<u16>(b"+00000000000000000000000001234").unwrap();
    assert_eq!(tmp, 1234_u16);

    let tmp = parse_skipped::<u32>(b"+00000000000000000000000001234").unwrap();
    assert_eq!(tmp, 1234_u32);

    let tmp = parse_skipped::<u64>(b"+00000000000000000000000001234").unwrap();
    assert_eq!(tmp, 1234_u64);

    let tmp = parse_skipped::<u64>(b"000000000001234").unwrap();
    assert_eq!(tmp, 1234_u64);

    let tmp = parse_skipped::<u64>(b"0000000000001234").unwrap();
    assert_eq!(tmp, 1234_u64);

    let tmp = parse_skipped::<u64>(b"00000000000001234").unwrap();
    assert_eq!(tmp, 1234_u64);

    let tmp = parse_skipped::<u128>(b"+00000000000000000000000001234").unwrap();
    assert_eq!(tmp, 1234_u128);

    let tmp = parse_skipped::<u128>(b"+000000000000000000000000012345678901234567890").unwrap();
    assert_eq!(tmp, 12345678901234567890_u128);

    let tmp = parse_skipped::<i8>(b"-0000000000000000000000000123").unwrap();
    assert_eq!(tmp, -123_i8);

    let tmp = parse_skipped::<i16>(b"-00000000000000000000000001234").unwrap();
    assert_eq!(tmp, -1234_i16);

    let tmp = parse_skipped::<i32>(b"-00000000000000000000000001234").unwrap();
    assert_eq!(tmp, -1234_i32);

    let tmp = parse_skipped::<i64>(b"-00000000000000000000000001234").unwrap();
    assert_eq!(tmp, -1234_i64);

    let tmp = parse_skipped::<i64>(b"-0000000000001234").unwrap();
    assert_eq!(tmp, -1234_i64);

    let tmp = parse_skipped::<i64>(b"-00000000000001234").unwrap();
    assert_eq!(tmp, -1234_i64);

    let tmp = parse_skipped::<i128>(b"-00000000000000000000000001234").unwrap();
    assert_eq!(tmp, -1234_i128);

    let tmp = parse_skipped::<i128>(b"-000000000000000000000000012345678901234567890").unwrap();
    assert_eq!(tmp, -12345678901234567890_i128);

    assert!(parse_skipped::<u64>(b"").is_err());

    // Zeroes.
    assert_eq!(parse_skipped::<i8>(b"0"), Ok(0));
    assert_eq!(parse_skipped::<i8>(b"-0"), Ok(0));
    assert_eq!(parse_skipped::<i8>(b"-0000000000000000000000000000"), Ok(0));
    assert_eq!(parse_skipped::<i8>(b"+0"), Ok(0));
    assert_eq!(parse_skipped::<i8>(b"+0000000000000000000000000000"), Ok(0));
    assert_eq!(parse_skipped::<u8>(b"0"), Ok(0));
    assert_eq!(parse_skipped::<u8>(b"+0"), Ok(0));
    assert_eq!(parse_skipped::<u8>(b"+0000000000000000000000000000"), Ok(0));
    assert_eq!(parse_skipped::<i16>(b"0"), Ok(0));
    assert_eq!(parse_skipped::<i16>(b"-0"), Ok(0));
    assert_eq!(
        parse_skipped::<i16>(b"-0000000000000000000000000000"),
        Ok(0)
    );
    assert_eq!(parse_skipped::<i16>(b"+0"), Ok(0));
    assert_eq!(
        parse_skipped::<i16>(b"+0000000000000000000000000000"),
        Ok(0)
    );
    assert_eq!(parse_skipped::<u16>(b"0"), Ok(0));
    assert_eq!(parse_skipped::<u16>(b"+0"), Ok(0));
    assert_eq!(
        parse_skipped::<u16>(b"+0000000000000000000000000000"),
        Ok(0)
    );
    assert_eq!(parse_skipped::<i32>(b"0"), Ok(0));
    assert_eq!(parse_skipped::<i32>(b"-0"), Ok(0));
    assert_eq!(
        parse_skipped::<i32>(b"-0000000000000000000000000000"),
        Ok(0)
    );
    assert_eq!(parse_skipped::<i32>(b"+0"), Ok(0));
    assert_eq!(
        parse_skipped::<i32>(b"+0000000000000000000000000000"),
        Ok(0)
    );
    assert_eq!(parse_skipped::<u32>(b"0"), Ok(0));
    assert_eq!(parse_skipped::<u32>(b"+0"), Ok(0));
    assert_eq!(
        parse_skipped::<u32>(b"+0000000000000000000000000000"),
        Ok(0)
    );
    assert_eq!(parse_skipped::<i64>(b"0"), Ok(0));
    assert_eq!(parse_skipped::<i64>(b"-0"), Ok(0));
    assert_eq!(
        parse_skipped::<i64>(b"-0000000000000000000000000000"),
        Ok(0)
    );
    assert_eq!(parse_skipped::<i64>(b"+0"), Ok(0));
    assert_eq!(
        parse_skipped::<i64>(b"+0000000000000000000000000000"),
        Ok(0)
    );
    assert_eq!(parse_skipped::<u64>(b"0"), Ok(0));
    assert_eq!(parse_skipped::<u64>(b"+0"), Ok(0));
    assert_eq!(
        parse_skipped::<u64>(b"+0000000000000000000000000000"),
        Ok(0)
    );
    assert_eq!(parse_skipped::<i128>(b"0"), Ok(0));
    assert_eq!(parse_skipped::<i128>(b"-0"), Ok(0));
    assert_eq!(
        parse_skipped::<i128>(b"-0000000000000000000000000000"),
        Ok(0)
    );
    assert_eq!(parse_skipped::<i128>(b"+0"), Ok(0));
    assert_eq!(
        parse_skipped::<i128>(b"+0000000000000000000000000000"),
        Ok(0)
    );
    assert_eq!(parse_skipped::<u128>(b"0"), Ok(0));
    assert_eq!(parse_skipped::<u128>(b"+0"), Ok(0));
    assert_eq!(
        parse_skipped::<u128>(b"+0000000000000000000000000000"),
        Ok(0)
    );
}
