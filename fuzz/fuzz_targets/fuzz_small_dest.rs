//! Fuzz target: encode and decode with undersized destination buffers.
//! Must never panic — should return None when dest is too small.
#![no_main]
use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    if data.is_empty() {
        return;
    }

    // Split fuzz input: first byte selects dest size, rest is payload.
    let dest_size = data[0] as usize;
    let payload = &data[1..];

    // Encode with a potentially too-small buffer.
    let mut enc_buf = vec![0u8; dest_size];
    let enc_result = ucobs::encode(payload, &mut enc_buf);

    if let Some(enc_len) = enc_result {
        let encoded = &enc_buf[..enc_len];

        // Encoded output must never contain 0x00.
        assert!(!encoded.contains(&0x00), "encoded contains sentinel");

        // Decode with a potentially too-small buffer.
        let mut dec_buf = vec![0u8; dest_size];
        let dec_result = ucobs::decode(encoded, &mut dec_buf);

        if let Some(dec_len) = dec_result {
            assert_eq!(
                &dec_buf[..dec_len], payload,
                "round-trip mismatch with small buffers"
            );
        }
    }
});
