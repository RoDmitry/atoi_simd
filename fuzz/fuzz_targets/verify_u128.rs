#![no_main]

use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    let result = atoi_simd::parse::<u128>(data);
    if let Some(first_digit) = data.get(0) {
        // atoi_simd doesn't support leading zeroes or + sign
        if *first_digit != b'0' && *first_digit != b'+' {
            // std only supports converting from &str, not &[u8]
            if let Ok(string) = ::core::str::from_utf8(data) {
                let std_result = string.parse::<u128>();
                match (result, std_result) {
                    (Ok(ours), Ok(std)) => assert_eq!(ours, std),
                    (Err(_), Err(_)) => (), // both failed, nothing to do
                    (ours, std) => panic!("Parsing discrepancy! Ours: {:?}, std: {:?}", ours, std),
                }
            }
        }
    }
});
