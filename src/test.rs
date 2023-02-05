use super::*;

const INVALID_CHARS: [&str; 6] = ["/", ":", "\0", "\x7f", "!", "a"];

fn test_each_position<T: Copy>(s: &str, func: fn(&[u8]) -> Result<T, AtoiSimdError>) {
    for j in 0..=s.len() {
        for &ch_str in INVALID_CHARS.iter() {
            let s_new = (&s[0..j]).to_owned() + ch_str + &s[j..s.len()];
            if func(s_new.as_bytes()).is_ok() {
                panic!("error {}", s_new);
            }
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

#[test]
fn test_parse_u8() {
    if parse::<u8>("".as_bytes()).is_ok() {
        panic!("error");
    }

    assert_eq!(parse::<u8>("0".as_bytes()).unwrap(), 0_u8);

    let mut s = String::with_capacity(10);
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

    let mut s = String::with_capacity(19);
    let mut s_neg = String::with_capacity(20);
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

    let mut s = String::with_capacity(10);
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

    let mut s = String::with_capacity(19);
    let mut s_neg = String::with_capacity(20);
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

    let mut s = String::with_capacity(10);
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

    let mut s = String::with_capacity(19);
    let mut s_neg = String::with_capacity(20);
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

    let mut s = String::with_capacity(20);
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
fn test_parse_i64() {
    if parse::<i64>("".as_bytes()).is_ok() {
        panic!("error");
    }

    assert_eq!(parse::<i64>("0".as_bytes()).unwrap(), 0_i64);

    assert_eq!(parse::<i64>("-0".as_bytes()).unwrap(), 0_i64);

    let mut s = String::with_capacity(19);
    let mut s_neg = String::with_capacity(20);
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

    let mut s = String::with_capacity(32);
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
    for i in '0'..='2' {
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
        parse::<u128>("12345678901234567890123456789012".as_bytes()).unwrap(),
        1234567890_1234567890_1234567890_12_u128
    );

    if parse::<u128>("123456789012345678901234567890123".as_bytes()).is_ok() {
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

    let mut s = String::with_capacity(32);
    let mut s_neg = String::with_capacity(33);
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
    for i in '0'..='2' {
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

    if parse::<i128>("123456789012345678901234567890123".as_bytes()).is_ok() {
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