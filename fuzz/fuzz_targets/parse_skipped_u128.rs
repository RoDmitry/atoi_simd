#![no_main]

use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    atoi_simd::parse_skipped::<u128>(data);
});