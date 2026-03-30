//! COBS (Consistent Overhead Byte Stuffing) encoder and decoder.
//!
//! Implements the algorithm from Cheshire & Baker, "Consistent Overhead
//! Byte Stuffing," IEEE/ACM Transactions on Networking, Vol. 7, No. 2,
//! April 1999.
//!
//! COBS transforms a byte sequence so that `0x00` never appears in the
//! output, allowing `0x00` to be used as an unambiguous frame delimiter.
//! Overhead is exactly 1 byte per 254 input bytes (worst case).
//!
//! This crate is `no_std` and zero-alloc — suitable for embedded firmware.
//!
//! # Examples
//!
//! ```
//! let mut buf = [0u8; 16];
//!
//! // Encode
//! let n = ucobs::encode(&[0x11, 0x00, 0x33], &mut buf).unwrap();
//! assert_eq!(&buf[..n], &[0x02, 0x11, 0x02, 0x33]);
//!
//! // Decode
//! let mut out = [0u8; 16];
//! let n = ucobs::decode(&[0x02, 0x11, 0x02, 0x33], &mut out).unwrap();
//! assert_eq!(&out[..n], &[0x11, 0x00, 0x33]);
//! ```

#![no_std]

/// Encode `src` into `dest` using COBS. Returns the number of bytes written,
/// or `None` if `dest` is too small.
///
/// The output does NOT include a trailing `0x00` sentinel — the caller
/// should append it as a frame delimiter.
pub fn encode(src: &[u8], dest: &mut [u8]) -> Option<usize> {
    let mut si = 0;
    let mut di = 0;
    let mut code_idx = di;
    let mut need_final_code = true;

    if di >= dest.len() && !src.is_empty() {
        return None;
    }
    // Reserve space for the first code byte.
    di += 1;
    let mut code: u8 = 1;

    while si < src.len() {
        if src[si] == 0x00 {
            if code_idx >= dest.len() {
                return None;
            }
            dest[code_idx] = code;
            code_idx = di;
            if di >= dest.len() {
                return None;
            }
            di += 1;
            code = 1;
            si += 1;
            need_final_code = true;
        } else {
            if di >= dest.len() {
                return None;
            }
            dest[di] = src[si];
            di += 1;
            si += 1;
            code += 1;
            if code == 0xFF {
                // Block full (254 data bytes) — emit code.
                if code_idx >= dest.len() {
                    return None;
                }
                dest[code_idx] = code;

                if si < src.len() {
                    // More input: start a new block.
                    code_idx = di;
                    if di >= dest.len() {
                        return None;
                    }
                    di += 1;
                    code = 1;
                    need_final_code = true;
                } else {
                    // Input exhausted right at the block boundary.
                    need_final_code = false;
                }
            }
        }
    }

    if need_final_code {
        if code_idx >= dest.len() {
            return None;
        }
        dest[code_idx] = code;
    }

    Some(di)
}

/// Decode a COBS-encoded buffer. Returns the number of decoded bytes
/// written to `dest`, or `None` if the input is malformed.
///
/// `src` should NOT include the trailing `0x00` frame sentinel — strip
/// it before calling this function.
pub fn decode(src: &[u8], dest: &mut [u8]) -> Option<usize> {
    let mut si = 0;
    let mut di = 0;
    while si < src.len() {
        let code = src[si] as usize;
        si += 1;
        if code == 0 {
            return None; // unexpected zero in COBS data
        }
        // Copy `code - 1` data bytes verbatim.
        for _ in 1..code {
            if si >= src.len() || di >= dest.len() {
                return None;
            }
            dest[di] = src[si];
            di += 1;
            si += 1;
        }
        // If code < 0xFF, an implicit zero follows (unless we're at the end).
        if code < 0xFF && si < src.len() {
            if di >= dest.len() {
                return None;
            }
            dest[di] = 0;
            di += 1;
        }
    }
    Some(di)
}

/// Maximum encoded size for a given source length (excluding sentinel).
pub const fn max_encoded_len(src_len: usize) -> usize {
    src_len + (src_len / 254) + 1
}

#[cfg(test)]
mod tests {
    use super::*;
    extern crate alloc;
    use alloc::vec;
    use alloc::vec::Vec;

    fn dec(encoded: &[u8]) -> Option<Vec<u8>> {
        let mut buf = vec![0u8; 512];
        decode(encoded, &mut buf).map(|n| buf[..n].to_vec())
    }

    fn enc(input: &[u8]) -> Option<Vec<u8>> {
        let mut buf = vec![0u8; max_encoded_len(input.len()) + 1];
        encode(input, &mut buf).map(|n| buf[..n].to_vec())
    }

    // ── Wikipedia canonical decode vectors (Cheshire & Baker 1999) ──

    #[test]
    fn decode_empty() {
        assert_eq!(dec(&[0x01]), Some(vec![]));
    }

    #[test]
    fn decode_single_zero() {
        assert_eq!(dec(&[0x01, 0x01]), Some(vec![0x00]));
    }

    #[test]
    fn decode_two_zeros() {
        assert_eq!(dec(&[0x01, 0x01, 0x01]), Some(vec![0x00, 0x00]));
    }

    #[test]
    fn decode_single_nonzero() {
        assert_eq!(dec(&[0x02, 0x11]), Some(vec![0x11]));
    }

    #[test]
    fn decode_zero_delimited() {
        assert_eq!(
            dec(&[0x01, 0x02, 0x11, 0x01]),
            Some(vec![0x00, 0x11, 0x00])
        );
    }

    #[test]
    fn decode_mixed() {
        assert_eq!(
            dec(&[0x03, 0x11, 0x22, 0x02, 0x33]),
            Some(vec![0x11, 0x22, 0x00, 0x33])
        );
    }

    #[test]
    fn decode_no_zeros() {
        assert_eq!(
            dec(&[0x05, 0x11, 0x22, 0x33, 0x44]),
            Some(vec![0x11, 0x22, 0x33, 0x44])
        );
    }

    #[test]
    fn decode_trailing_zeros() {
        assert_eq!(
            dec(&[0x02, 0x11, 0x01, 0x01, 0x01]),
            Some(vec![0x11, 0x00, 0x00, 0x00])
        );
    }

    #[test]
    fn decode_all_zeros_4() {
        assert_eq!(
            dec(&[0x01, 0x01, 0x01, 0x01, 0x01]),
            Some(vec![0x00, 0x00, 0x00, 0x00])
        );
    }

    #[test]
    fn decode_all_ff_4() {
        assert_eq!(
            dec(&[0x05, 0xFF, 0xFF, 0xFF, 0xFF]),
            Some(vec![0xFF, 0xFF, 0xFF, 0xFF])
        );
    }

    #[test]
    fn decode_alternating() {
        assert_eq!(
            dec(&[0x01, 0x02, 0x01, 0x02, 0x02, 0x02, 0x03]),
            Some(vec![0x00, 0x01, 0x00, 0x02, 0x00, 0x03])
        );
    }

    // ── Block boundary tests (254/255 bytes) ────────────────────

    #[test]
    fn decode_254_nonzero() {
        let input: Vec<u8> = (1..=254).map(|i| i as u8).collect();
        let mut encoded = vec![0xFF];
        encoded.extend(1u8..=254);
        assert_eq!(dec(&encoded), Some(input));
    }

    #[test]
    fn decode_255_nonzero_split() {
        let input: Vec<u8> = (1..=255).map(|i| i as u8).collect();
        let mut encoded = vec![0xFF];
        encoded.extend(1u8..=254);
        encoded.push(0x02);
        encoded.push(0xFF);
        assert_eq!(dec(&encoded), Some(input));
    }

    // ── Ping regression (tag byte 0x00 was stripped by old decoder) ──

    #[test]
    fn decode_ping_tag_zero() {
        assert_eq!(dec(&[0x01, 0x01]), Some(vec![0x00]));
    }

    // ── Error cases ─────────────────────────────────────────────

    #[test]
    fn decode_unexpected_zero() {
        assert_eq!(dec(&[0x00]), None);
    }

    #[test]
    fn decode_truncated() {
        assert_eq!(dec(&[0x04, 0x11]), None);
    }

    #[test]
    fn decode_empty_input() {
        assert_eq!(dec(&[]), Some(vec![]));
    }

    // ── Encode tests ────────────────────────────────────────────

    #[test]
    fn encode_empty() {
        assert_eq!(enc(&[]), Some(vec![0x01]));
    }

    #[test]
    fn encode_single_zero() {
        assert_eq!(enc(&[0x00]), Some(vec![0x01, 0x01]));
    }

    #[test]
    fn encode_single_nonzero() {
        assert_eq!(enc(&[0x11]), Some(vec![0x02, 0x11]));
    }

    #[test]
    fn encode_mixed() {
        assert_eq!(
            enc(&[0x11, 0x22, 0x00, 0x33]),
            Some(vec![0x03, 0x11, 0x22, 0x02, 0x33])
        );
    }

    #[test]
    fn encode_no_zeros() {
        assert_eq!(
            enc(&[0x11, 0x22, 0x33, 0x44]),
            Some(vec![0x05, 0x11, 0x22, 0x33, 0x44])
        );
    }

    #[test]
    fn encode_trailing_zeros() {
        assert_eq!(
            enc(&[0x11, 0x00, 0x00, 0x00]),
            Some(vec![0x02, 0x11, 0x01, 0x01, 0x01])
        );
    }

    #[test]
    fn encode_254_nonzero() {
        let input: Vec<u8> = (1..=254).map(|i| i as u8).collect();
        let mut expected = vec![0xFF];
        expected.extend(1u8..=254);
        assert_eq!(enc(&input), Some(expected));
    }

    // ── Round-trip tests ────────────────────────────────────────

    #[test]
    fn roundtrip_empty() {
        roundtrip(&[]);
    }

    #[test]
    fn roundtrip_single_zero() {
        roundtrip(&[0x00]);
    }

    #[test]
    fn roundtrip_ping() {
        roundtrip(&[0x00]); // Ping command = tag 0
    }

    #[test]
    fn roundtrip_mixed() {
        roundtrip(&[0x11, 0x22, 0x00, 0x33, 0x00, 0x00, 0x44]);
    }

    #[test]
    fn roundtrip_254_nonzero() {
        let data: Vec<u8> = (1..=254).map(|i| i as u8).collect();
        roundtrip(&data);
    }

    #[test]
    fn roundtrip_255_nonzero() {
        let data: Vec<u8> = (0..255).map(|i| (i + 1) as u8).collect();
        roundtrip(&data);
    }

    #[test]
    fn roundtrip_256_with_zeros() {
        let mut data = vec![0u8; 256];
        for (i, b) in data.iter_mut().enumerate() {
            *b = i as u8; // 0x00, 0x01, ..., 0xFF
        }
        roundtrip(&data);
    }

    fn roundtrip(input: &[u8]) {
        let mut enc_buf = vec![0u8; max_encoded_len(input.len()) + 1];
        let enc_len = encode(input, &mut enc_buf).expect("encode failed");
        let encoded = &enc_buf[..enc_len];

        // Verify no zero bytes in encoded output.
        assert!(
            !encoded.contains(&0x00),
            "encoded output contains 0x00: {:?}",
            encoded
        );

        let mut dec_buf = vec![0u8; input.len() + 1];
        let dec_len = decode(encoded, &mut dec_buf).expect("decode failed");
        assert_eq!(&dec_buf[..dec_len], input, "round-trip mismatch");
    }

    // ── Cross-validation against corncobs ────────────────────────

    /// Encode with ours, decode with corncobs — must match.
    /// Note: our encoder omits the trailing 0x00 sentinel; corncobs expects it.
    fn cross_encode(input: &[u8]) {
        let mut our_buf = vec![0u8; max_encoded_len(input.len()) + 2];
        let our_len = encode(input, &mut our_buf).expect("our encode failed");
        // Append sentinel for corncobs compatibility
        our_buf[our_len] = 0x00;
        let our_with_sentinel = &our_buf[..our_len + 1];

        let mut their_buf = vec![0u8; input.len() + 1];
        let their_len = corncobs::decode_buf(our_with_sentinel, &mut their_buf)
            .expect("corncobs failed to decode our output");
        assert_eq!(&their_buf[..their_len], input, "cross-encode mismatch");
    }

    /// Encode with corncobs, decode with ours — must match.
    /// Note: corncobs includes the trailing 0x00 sentinel; our decoder doesn't expect it.
    fn cross_decode(input: &[u8]) {
        let mut their_buf = vec![0u8; corncobs::max_encoded_len(input.len())];
        let their_len = corncobs::encode_buf(input, &mut their_buf);
        // corncobs includes the sentinel — strip it for our decoder
        let their_encoded = &their_buf[..their_len];
        // Find the sentinel and exclude it
        let data_end = their_encoded.iter().rposition(|&b| b != 0x00).map_or(0, |i| i + 1);
        let their_no_sentinel = &their_encoded[..data_end];

        let mut our_buf = vec![0u8; input.len() + 1];
        let our_len = decode(their_no_sentinel, &mut our_buf)
            .expect("our decode failed on corncobs output");
        assert_eq!(&our_buf[..our_len], input, "cross-decode mismatch");
    }

    #[test]
    fn interop_empty() {
        cross_encode(&[]);
        cross_decode(&[]);
    }

    #[test]
    fn interop_single_zero() {
        cross_encode(&[0x00]);
        cross_decode(&[0x00]);
    }

    #[test]
    fn interop_mixed() {
        let data = [0x11, 0x22, 0x00, 0x33, 0x00, 0x00, 0x44];
        cross_encode(&data);
        cross_decode(&data);
    }

    #[test]
    fn interop_254_block() {
        let data: Vec<u8> = (1..=254).map(|i| i as u8).collect();
        cross_encode(&data);
        cross_decode(&data);
    }

    #[test]
    fn interop_255_block() {
        let data: Vec<u8> = (0..255).map(|i| (i + 1) as u8).collect();
        cross_encode(&data);
        cross_decode(&data);
    }

    #[test]
    fn interop_random_payloads() {
        use rand::Rng;
        let mut rng = rand::thread_rng();
        for _ in 0..500 {
            let len = rng.gen_range(0..=512);
            let data: Vec<u8> = (0..len).map(|_| rng.gen()).collect();
            cross_encode(&data);
            cross_decode(&data);
        }
    }

    #[test]
    fn interop_all_zeros() {
        for len in [0, 1, 2, 10, 100, 254, 255, 256, 512] {
            let data = vec![0u8; len];
            cross_encode(&data);
            cross_decode(&data);
        }
    }

    #[test]
    fn interop_all_ff() {
        for len in [0, 1, 2, 10, 100, 254, 255, 256, 512] {
            let data = vec![0xFFu8; len];
            cross_encode(&data);
            cross_decode(&data);
        }
    }
}

// ── Property-based tests (proptest) ─────────────────────────────

#[cfg(test)]
mod proptests {
    extern crate alloc;
    use alloc::vec;
    use super::*;
    use proptest::prelude::*;

    // Round-trip: encode then decode must recover the original.
    proptest! {
        #[test]
        fn roundtrip_any(data in proptest::collection::vec(any::<u8>(), 0..1024)) {
            let mut enc_buf = vec![0u8; max_encoded_len(data.len()) + 1];
            let enc_len = encode(&data, &mut enc_buf).unwrap();
            let encoded = &enc_buf[..enc_len];

            // Encoded output must never contain 0x00.
            prop_assert!(!encoded.contains(&0x00));

            let mut dec_buf = vec![0u8; data.len() + 1];
            let dec_len = decode(encoded, &mut dec_buf).unwrap();
            prop_assert_eq!(&dec_buf[..dec_len], &data[..]);
        }

        // Our encode → corncobs decode must match.
        #[test]
        fn interop_our_encode_their_decode(data in proptest::collection::vec(any::<u8>(), 0..1024)) {
            let mut enc_buf = vec![0u8; max_encoded_len(data.len()) + 2];
            let enc_len = encode(&data, &mut enc_buf).unwrap();
            // Append sentinel for corncobs.
            enc_buf[enc_len] = 0x00;

            let mut dec_buf = vec![0u8; data.len() + 1];
            let dec_len = corncobs::decode_buf(&enc_buf[..enc_len + 1], &mut dec_buf).unwrap();
            prop_assert_eq!(&dec_buf[..dec_len], &data[..]);
        }

        // corncobs encode → our decode must match.
        #[test]
        fn interop_their_encode_our_decode(data in proptest::collection::vec(any::<u8>(), 0..1024)) {
            let mut enc_buf = vec![0u8; corncobs::max_encoded_len(data.len())];
            let enc_len = corncobs::encode_buf(&data, &mut enc_buf);
            // Strip trailing sentinel for our decoder.
            let data_end = enc_buf[..enc_len].iter().rposition(|&b| b != 0x00).map_or(0, |i| i + 1);

            let mut dec_buf = vec![0u8; data.len() + 1];
            let dec_len = decode(&enc_buf[..data_end], &mut dec_buf).unwrap();
            prop_assert_eq!(&dec_buf[..dec_len], &data[..]);
        }

        // Encoded length is always within the theoretical maximum.
        #[test]
        fn encoded_length_bounded(data in proptest::collection::vec(any::<u8>(), 0..1024)) {
            let mut enc_buf = vec![0u8; max_encoded_len(data.len()) + 1];
            let enc_len = encode(&data, &mut enc_buf).unwrap();
            prop_assert!(enc_len <= max_encoded_len(data.len()));
        }

        // Decoding random garbage must never panic (returns None on invalid input).
        #[test]
        fn decode_never_panics(data in proptest::collection::vec(any::<u8>(), 0..512)) {
            let mut dec_buf = vec![0u8; data.len() + 256];
            let _ = decode(&data, &mut dec_buf); // may return None, must not panic
        }
    }
}
