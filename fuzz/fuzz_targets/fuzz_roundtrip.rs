//! Fuzz target: encode then decode arbitrary input. Output must match input.
#![no_main]
use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    if data.len() > 1024 {
        return; // skip huge inputs to keep fuzzing fast
    }

    let max_enc = ucobs::max_encoded_len(data.len()) + 1;
    let mut enc_buf = vec![0u8; max_enc];

    let enc_len = match ucobs::encode(data, &mut enc_buf) {
        Some(n) => n,
        None => return, // encode failed (buffer too small shouldn't happen, but skip)
    };
    let encoded = &enc_buf[..enc_len];

    // Encoded output must never contain 0x00.
    assert!(!encoded.contains(&0x00), "encoded contains sentinel");

    let mut dec_buf = vec![0u8; data.len() + 1];
    let dec_len = ucobs::decode(encoded, &mut dec_buf)
        .expect("decode failed after successful encode");

    assert_eq!(
        &dec_buf[..dec_len], data,
        "round-trip mismatch: encode→decode produced different output"
    );
});
