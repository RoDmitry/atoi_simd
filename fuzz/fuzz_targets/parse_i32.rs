#![no_main]

use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    let _ = atoi_simd::parse::<i32, false, false>(data);
});
