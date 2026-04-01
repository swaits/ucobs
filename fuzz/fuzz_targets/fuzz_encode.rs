//! Fuzz target: encode arbitrary input into buffers of varying sizes.
//! Exercises encode error paths (buffer too small) and validates output properties.
#![no_main]
use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    if data.len() > 1024 {
        return;
    }

    let max = ucobs::max_encoded_len(data.len());

    // Encode into a correctly-sized buffer — must always succeed.
    let mut buf = vec![0u8; max];
    let n = ucobs::encode(data, &mut buf)
        .expect("encode failed with max_encoded_len buffer");
    let encoded = &buf[..n];

    // Output must never contain 0x00.
    assert!(!encoded.contains(&0x00), "encoded output contains 0x00");

    // Output length must be bounded by max_encoded_len.
    assert!(n <= max, "encoded length exceeds max_encoded_len");

    // Output must be at least 1 byte (even empty input encodes to [0x01]).
    assert!(n >= 1, "encoded length is zero");

    // Encode into every possible undersized buffer — must return None, never panic.
    for size in 0..n {
        let mut small = vec![0u8; size];
        assert_eq!(ucobs::encode(data, &mut small), None);
    }
});
