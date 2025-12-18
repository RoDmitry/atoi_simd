#![no_main]

use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    let _ = atoi_simd::parse::<u8, false, false>(data);
});
