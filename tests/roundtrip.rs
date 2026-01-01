mod reimpl;
#[allow(unused_imports)]
use reimpl::*;

use numtoa::NumToA;

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
