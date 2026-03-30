//! Fuzz target: feed arbitrary bytes to decode(). Must never panic.
#![no_main]
use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    let mut buf = [0u8; 2048];
    // Must not panic — may return None on invalid input.
    let _ = ucobs::decode(data, &mut buf);
});
