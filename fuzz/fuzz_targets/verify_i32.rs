#![no_main]

use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    let result = atoi_simd::parse::<i32, false, false>(data);
    if let Some([first_digit, second_digit]) = data.get(0..=1) {
        // avoid leading zeroes and + sign
        if *first_digit != b'0' && *first_digit != b'+' && *second_digit != b'0' {
            // std only supports converting from &str, not &[u8]
            if let Ok(string) = ::core::str::from_utf8(data) {
                let std_result = string.parse::<i32>();
                match (result, std_result) {
                    (Ok(ours), Ok(std)) => assert_eq!(ours, std),
                    (Err(_), Err(_)) => (), // both failed, nothing to do
                    (ours, std) => panic!("Parsing discrepancy! Ours: {:?}, std: {:?}", ours, std),
                }
            }
        }
    }
});
