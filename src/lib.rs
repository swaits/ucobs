//! COBS (Consistent Overhead Byte Stuffing) encoder and decoder.
//!
//! Implements the algorithm from Cheshire & Baker, "Consistent Overhead
//! Byte Stuffing," IEEE/ACM Transactions on Networking, Vol. 7, No. 2,
//! April 1999.
//!
//! COBS transforms a byte sequence so that `0x00` never appears in the
//! output, allowing `0x00` to be used as an unambiguous frame delimiter.
//! Overhead is at most 1 byte per 254 input bytes plus 1; see
//! [`max_encoded_len`] for the exact formula.
//!
//! This crate is `no_std`, zero-alloc, and `unsafe`-free — it runs
//! everywhere from 8-bit MCUs to servers.
//!
//! # Quick start
//!
//! ```
//! // Encode
//! let mut buf = [0u8; 16];
//! let n = ucobs::encode(&[0x11, 0x00, 0x33], &mut buf).unwrap();
//! assert_eq!(&buf[..n], &[0x02, 0x11, 0x02, 0x33]);
//!
//! // Decode
//! let mut out = [0u8; 16];
//! let m = ucobs::decode(&buf[..n], &mut out).unwrap();
//! assert_eq!(&out[..m], &[0x11, 0x00, 0x33]);
//! ```
//!
//! # Buffer sizing
//!
//! Use [`max_encoded_len`] to determine the required destination buffer
//! size before encoding:
//!
//! ```
//! let data = [0x01, 0x02, 0x03];
//! let max = ucobs::max_encoded_len(data.len()); // 4
//!
//! let mut buf = [0u8; 4];
//! let n = ucobs::encode(&data, &mut buf).unwrap();
//! assert_eq!(n, 4); // fits exactly
//! ```
//!
//! If the destination is too small, `encode` returns `None` rather than
//! panicking:
//!
//! ```
//! let mut tiny = [0u8; 1];
//! assert_eq!(ucobs::encode(&[0x01, 0x02], &mut tiny), None);
//! ```
//!
//! # Framing for transport
//!
//! The encoder does **not** append a trailing `0x00` sentinel. The decoder
//! expects input **without** a trailing sentinel. Append and strip the
//! sentinel yourself when framing for transport:
//!
//! ```
//! let data = [0x11, 0x00, 0x33];
//!
//! // Encode into a wire frame: [COBS bytes...] [0x00 sentinel]
//! let mut frame = [0u8; 16];
//! let n = ucobs::encode(&data, &mut frame).unwrap();
//! frame[n] = 0x00; // append sentinel
//! let wire = &frame[..n + 1];
//!
//! // On the receiving end, strip the sentinel before decoding
//! let cobs_data = &wire[..wire.len() - 1];
//! let mut out = [0u8; 16];
//! let m = ucobs::decode(cobs_data, &mut out).unwrap();
//! assert_eq!(&out[..m], &data);
//! ```
//!
//! # Parsing a stream of frames
//!
//! In a byte stream, split on `0x00` to extract individual COBS frames,
//! then decode each one:
//!
//! ```
//! // Simulate a stream containing two frames separated by 0x00 sentinels
//! let stream = [
//!     0x02, 0x11, 0x02, 0x33, 0x00,  // frame 1: encodes [0x11, 0x00, 0x33]
//!     0x03, 0xAA, 0xBB, 0x00,        // frame 2: encodes [0xAA, 0xBB]
//! ];
//!
//! let mut out = [0u8; 16];
//! let frames: Vec<&[u8]> = stream.split(|&b| b == 0x00)
//!     .filter(|f| !f.is_empty())
//!     .collect();
//!
//! let n = ucobs::decode(frames[0], &mut out).unwrap();
//! assert_eq!(&out[..n], &[0x11, 0x00, 0x33]);
//!
//! let n = ucobs::decode(frames[1], &mut out).unwrap();
//! assert_eq!(&out[..n], &[0xAA, 0xBB]);
//! ```
//!
//! # Compile-time encoding
//!
//! [`encode`] is `const fn`, so you can build COBS-encoded lookup tables
//! or protocol headers at compile time with zero runtime cost:
//!
//! ```
//! // Pre-encode a command table at compile time
//! const PING: [u8; 2] = {
//!     let mut buf = [0u8; 2];
//!     match ucobs::encode(&[0x01], &mut buf) {
//!         Some(_) => buf,
//!         None => panic!("buffer too small"),
//!     }
//! };
//!
//! const ACK: [u8; 3] = {
//!     let mut buf = [0u8; 3];
//!     match ucobs::encode(&[0x06, 0x00], &mut buf) {
//!         Some(_) => buf,
//!         None => panic!("buffer too small"),
//!     }
//! };
//!
//! // No runtime encoding needed — these are baked into the binary
//! assert_eq!(PING, [0x02, 0x01]);
//! assert_eq!(ACK, [0x02, 0x06, 0x01]);
//! ```
//!
//! # Error handling
//!
//! Both [`encode`] and [`decode`] return `Option<usize>` — `None` on
//! failure, never panic.
//!
//! ```
//! let mut buf = [0u8; 8];
//!
//! // Destination too small for encode
//! assert_eq!(ucobs::encode(&[1, 2, 3], &mut buf[..1]), None);
//!
//! // Malformed COBS data (zero byte in encoded stream)
//! assert_eq!(ucobs::decode(&[0x00], &mut buf), None);
//!
//! // Truncated frame (code byte promises more data than exists)
//! assert_eq!(ucobs::decode(&[0x05, 0x11], &mut buf), None);
//!
//! // Empty input is valid for both
//! assert_eq!(ucobs::encode(&[], &mut buf), Some(1)); // encodes to [0x01]
//! assert_eq!(ucobs::decode(&[], &mut buf), Some(0)); // decodes to []
//! ```

#![no_std]
#![forbid(unsafe_code)]
#![warn(missing_docs)]

/// COBS-encode `src` into `dest`.
///
/// Returns the number of bytes written, or `None` if `dest` is too small.
/// Use [`max_encoded_len`] to size the destination buffer.
///
/// The output does **not** include a trailing `0x00` sentinel — append it
/// yourself when framing for transport.
///
/// This function is `const fn`, so it can encode data at compile time.
///
/// # Examples
///
/// ```
/// let mut buf = [0u8; 8];
/// let n = ucobs::encode(&[0x00], &mut buf).unwrap();
/// assert_eq!(&buf[..n], &[0x01, 0x01]);
/// ```
///
/// Compile-time encoding:
///
/// ```
/// const ENCODED: [u8; 4] = {
///     let mut buf = [0u8; 4];
///     match ucobs::encode(&[0x11, 0x00, 0x33], &mut buf) {
///         Some(_) => buf,
///         None => panic!("buffer too small"),
///     }
/// };
/// assert_eq!(ENCODED, [0x02, 0x11, 0x02, 0x33]);
/// ```
#[must_use]
pub const fn encode(src: &[u8], dest: &mut [u8]) -> Option<usize> {
    let mut si = 0;
    let mut di = 0;

    loop {
        let remaining = src.len() - si;
        let max_run = if remaining < 254 { remaining } else { 254 };

        // Create bounded source window (split_at is const since Rust 1.71).
        let (_, src_tail) = src.split_at(si);
        let (src_window, _) = src_tail.split_at(max_run);

        // Fast path: leading zero byte avoids scan loop overhead.
        // Always continue — the next iteration emits the trailing group code byte.
        if max_run > 0 && src_window[0] == 0x00 {
            if di >= dest.len() {
                return None;
            }
            dest[di] = 0x01;
            di += 1;
            si += 1;
            continue;
        }

        // Phase 1: Scan for next zero byte.
        // LLVM sees `run < src_window.len()` → eliminates bounds check.
        let mut run = 0;
        while run < src_window.len() && src_window[run] != 0x00 {
            run += 1;
        }
        let full_block = run == 254;

        // Write code byte.
        if di >= dest.len() {
            return None;
        }
        dest[di] = if full_block { 0xFF } else { (run + 1) as u8 };
        di += 1;

        // Phase 2: Copy via const copy_from_slice (Rust 1.93+) → memcpy.
        if run > 0 {
            if di + run > dest.len() {
                return None;
            }
            let (_, dest_tail) = dest.split_at_mut(di);
            let (dest_chunk, _) = dest_tail.split_at_mut(run);
            let (src_chunk, _) = src_window.split_at(run);
            dest_chunk.copy_from_slice(src_chunk);
            di += run;
        }
        si += run;

        if full_block {
            if si >= src.len() {
                break;
            }
        } else {
            if si >= src.len() {
                break;
            }
            si += 1; // skip the zero byte
        }
    }

    Some(di)
}

/// Decode a COBS-encoded buffer.
///
/// Returns the number of decoded bytes written to `dest`, or `None` if the
/// input is malformed or `dest` is too small.
///
/// `src` should **not** include the trailing `0x00` frame sentinel — strip
/// it before calling this function.
///
/// # Examples
///
/// ```
/// let mut out = [0u8; 8];
/// let n = ucobs::decode(&[0x02, 0x11, 0x02, 0x33], &mut out).unwrap();
/// assert_eq!(&out[..n], &[0x11, 0x00, 0x33]);
/// ```
///
/// Malformed input returns `None`:
///
/// ```
/// let mut out = [0u8; 8];
/// assert_eq!(ucobs::decode(&[0x00], &mut out), None); // unexpected zero
/// assert_eq!(ucobs::decode(&[0x05, 0x11], &mut out), None); // truncated
/// ```
#[must_use]
pub fn decode(src: &[u8], dest: &mut [u8]) -> Option<usize> {
    let mut si = 0;
    let mut di = 0;
    while si < src.len() {
        let code = src[si] as usize;
        si += 1;
        if code == 0 {
            return None; // unexpected zero in COBS data
        }

        if code == 1 {
            // Batch path: count consecutive 0x01 codes, fill zeros in bulk.
            let mut count = 1usize;
            while si < src.len() && src[si] == 0x01 {
                si += 1;
                count += 1;
            }
            // Each 0x01 inserts an implicit zero, except the last at end-of-input.
            let zeros = if si < src.len() { count } else { count - 1 };
            if zeros > 0 {
                if di + zeros > dest.len() {
                    return None;
                }
                dest[di..di + zeros].fill(0);
                di += zeros;
            }
            continue;
        }

        // Bulk-copy `code - 1` data bytes.
        let n = code - 1;
        if si + n > src.len() || di + n > dest.len() {
            return None;
        }
        dest[di..di + n].copy_from_slice(&src[si..si + n]);
        di += n;
        si += n;
        // If code < 0xFF, an implicit zero follows (unless we're at the end).
        if code != 0xFF && si < src.len() {
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
///
/// Use this to allocate a destination buffer for [`encode`]. The formula
/// is `src_len + (src_len / 254) + 1`.
///
/// # Examples
///
/// ```
/// assert_eq!(ucobs::max_encoded_len(0), 1);
/// assert_eq!(ucobs::max_encoded_len(1), 2);
/// assert_eq!(ucobs::max_encoded_len(254), 256);
/// ```
#[must_use]
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

    // ── Compile-time (const fn) encode ────────────────────────────

    const _: () = {
        let input = [0x11, 0x00, 0x33];
        let mut buf = [0u8; 8];
        let Some(n) = encode(&input, &mut buf) else {
            panic!("const encode failed");
        };
        assert!(n == 4);
        assert!(buf[0] == 0x02);
        assert!(buf[1] == 0x11);
        assert!(buf[2] == 0x02);
        assert!(buf[3] == 0x33);
    };

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
        assert_eq!(dec(&[0x01, 0x02, 0x11, 0x01]), Some(vec![0x00, 0x11, 0x00]));
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

    // ── External corpus: decode error cases (cobs-c, nanocobs) ──

    #[test]
    fn decode_err_truncated_code_block() {
        // Code says 5 data bytes, only 3 follow.
        assert_eq!(dec(&[0x05, 0x31, 0x32, 0x33]), None);
    }

    #[test]
    fn decode_err_zero_after_code_block() {
        // Zero appears after a valid code block's data.
        assert_eq!(dec(&[0x05, 0x31, 0x32, 0x33, 0x34, 0x00]), None);
    }

    #[test]
    fn decode_zero_in_data_position_is_passthrough() {
        // A 0x00 in a data position is not checked by the decoder — it is
        // copied through as-is. This is correct COBS behavior: the decoder
        // only validates code-byte positions, not data bytes.
        assert_eq!(
            dec(&[0x05, 0x31, 0x32, 0x00, 0x34]),
            Some(vec![0x31, 0x32, 0x00, 0x34])
        );
    }

    // ── External corpus: Wikipedia vectors 9-11 ─────────────────

    #[test]
    fn corpus_wiki_vec9() {
        // 01..FF (255 bytes) → [FF 01..FE 02 FF]
        let input: Vec<u8> = (1..=255).map(|i| i as u8).collect();
        let mut expected = vec![0xFF];
        expected.extend(1u8..=254);
        expected.extend([0x02, 0xFF]);
        assert_eq!(enc(&input), Some(expected.clone()));
        assert_eq!(dec(&expected), Some(input));
    }

    #[test]
    fn corpus_wiki_vec10() {
        // 02..FF 00 (255 bytes) → [FF 02..FF 01 01]
        let mut input: Vec<u8> = (2..=255).map(|i| i as u8).collect();
        input.push(0x00);
        let mut expected = vec![0xFF];
        expected.extend(2u8..=255);
        expected.extend([0x01, 0x01]);
        assert_eq!(enc(&input), Some(expected.clone()));
        assert_eq!(dec(&expected), Some(input));
    }

    #[test]
    fn corpus_wiki_vec11() {
        // 03..FF 00 01 (255 bytes) → [FE 03..FF 02 01]
        let mut input: Vec<u8> = (3..=255).map(|i| i as u8).collect();
        input.push(0x00);
        input.push(0x01);
        let mut expected = vec![0xFE];
        expected.extend(3u8..=255);
        expected.extend([0x02, 0x01]);
        assert_eq!(enc(&input), Some(expected.clone()));
        assert_eq!(dec(&expected), Some(input));
    }

    // ── External corpus: Cheshire & Baker 1997 paper (Figure 2) ─

    #[test]
    fn corpus_paper_ipv4_header() {
        // IPv4 header fragment from the original SIGCOMM paper.
        let input = [
            0x45, 0x00, 0x00, 0x2C, 0x4C, 0x79, 0x00, 0x00, 0x40, 0x06, 0x4F, 0x37,
        ];
        let expected = [
            0x02, 0x45, 0x01, 0x04, 0x2C, 0x4C, 0x79, 0x01, 0x05, 0x40, 0x06, 0x4F, 0x37,
        ];
        assert_eq!(enc(&input), Some(expected.to_vec()));
        assert_eq!(dec(&expected), Some(input.to_vec()));
    }

    // ── External corpus: nanocobs vectors ───────────────────────

    #[test]
    fn corpus_nanocobs_single_nonzero() {
        assert_eq!(enc(&[0x34]), Some(vec![0x02, 0x34]));
    }

    #[test]
    fn corpus_nanocobs_two_nonzero() {
        assert_eq!(enc(&[0x34, 0x56]), Some(vec![0x03, 0x34, 0x56]));
    }

    #[test]
    fn corpus_nanocobs_eight_nonzero() {
        let input = [0x12, 0x34, 0x56, 0x78, 0x9A, 0xBC, 0xDE, 0xFF];
        let expected = [0x09, 0x12, 0x34, 0x56, 0x78, 0x9A, 0xBC, 0xDE, 0xFF];
        assert_eq!(enc(&input), Some(expected.to_vec()));
        assert_eq!(dec(&expected), Some(input.to_vec()));
    }

    #[test]
    fn corpus_nanocobs_eight_zeros() {
        let input = [0x00; 8];
        let expected = [0x01, 0x01, 0x01, 0x01, 0x01, 0x01, 0x01, 0x01, 0x01];
        assert_eq!(enc(&input), Some(expected.to_vec()));
        assert_eq!(dec(&expected), Some(input.to_vec()));
    }

    #[test]
    fn corpus_nanocobs_interleaved_00_11() {
        // 00 11 00 22
        let input = [0x00, 0x11, 0x00, 0x22];
        let expected = [0x01, 0x02, 0x11, 0x02, 0x22];
        assert_eq!(enc(&input), Some(expected.to_vec()));
        assert_eq!(dec(&expected), Some(input.to_vec()));
    }

    #[test]
    fn corpus_nanocobs_trailing_zero() {
        // 11 00 22 00
        let input = [0x11, 0x00, 0x22, 0x00];
        let expected = [0x02, 0x11, 0x02, 0x22, 0x01];
        assert_eq!(enc(&input), Some(expected.to_vec()));
        assert_eq!(dec(&expected), Some(input.to_vec()));
    }

    #[test]
    fn corpus_nanocobs_253_fill() {
        // 253 bytes of 0x42 → [FE 42 42 ... 42]
        let input = vec![0x42u8; 253];
        let mut expected = vec![0xFE];
        expected.extend(vec![0x42u8; 253]);
        assert_eq!(enc(&input), Some(expected.clone()));
        assert_eq!(dec(&expected), Some(input));
    }

    #[test]
    fn corpus_nanocobs_255_ones() {
        // 255 bytes of 0x01 → [FF 01x254 02 01]
        let input = vec![0x01u8; 255];
        let mut expected = vec![0xFF];
        expected.extend(vec![0x01u8; 254]);
        expected.extend([0x02, 0x01]);
        assert_eq!(enc(&input), Some(expected.clone()));
        assert_eq!(dec(&expected), Some(input));
    }

    #[test]
    fn corpus_nanocobs_508_fill() {
        // 508 bytes of 0xAA → two full blocks [FF AAx254 FF AAx254]
        let input = vec![0xAAu8; 508];
        let mut expected = vec![0xFF];
        expected.extend(vec![0xAAu8; 254]);
        expected.push(0xFF);
        expected.extend(vec![0xAAu8; 254]);
        assert_eq!(enc(&input), Some(expected.clone()));
        assert_eq!(dec(&expected), Some(input));
    }

    // ── External corpus: cobs-c vectors ─────────────────────────

    #[test]
    fn corpus_cobsc_five_nonzero() {
        let input = [0x31, 0x32, 0x33, 0x34, 0x35];
        let expected = [0x06, 0x31, 0x32, 0x33, 0x34, 0x35];
        assert_eq!(enc(&input), Some(expected.to_vec()));
        assert_eq!(dec(&expected), Some(input.to_vec()));
    }

    #[test]
    fn corpus_cobsc_two_segments() {
        // "12345\06789"
        let input = [0x31, 0x32, 0x33, 0x34, 0x35, 0x00, 0x36, 0x37, 0x38, 0x39];
        let expected = [
            0x06, 0x31, 0x32, 0x33, 0x34, 0x35, 0x05, 0x36, 0x37, 0x38, 0x39,
        ];
        assert_eq!(enc(&input), Some(expected.to_vec()));
        assert_eq!(dec(&expected), Some(input.to_vec()));
    }

    #[test]
    fn corpus_cobsc_leading_zero_two_segments() {
        // "\012345\06789"
        let input = [
            0x00, 0x31, 0x32, 0x33, 0x34, 0x35, 0x00, 0x36, 0x37, 0x38, 0x39,
        ];
        let expected = [
            0x01, 0x06, 0x31, 0x32, 0x33, 0x34, 0x35, 0x05, 0x36, 0x37, 0x38, 0x39,
        ];
        assert_eq!(enc(&input), Some(expected.to_vec()));
        assert_eq!(dec(&expected), Some(input.to_vec()));
    }

    #[test]
    fn corpus_cobsc_trailing_zero_two_segments() {
        // "12345\06789\0"
        let input = [
            0x31, 0x32, 0x33, 0x34, 0x35, 0x00, 0x36, 0x37, 0x38, 0x39, 0x00,
        ];
        let expected = [
            0x06, 0x31, 0x32, 0x33, 0x34, 0x35, 0x05, 0x36, 0x37, 0x38, 0x39, 0x01,
        ];
        assert_eq!(enc(&input), Some(expected.to_vec()));
        assert_eq!(dec(&expected), Some(input.to_vec()));
    }

    #[test]
    fn corpus_cobsc_three_zeros() {
        assert_eq!(enc(&[0x00, 0x00, 0x00]), Some(vec![0x01, 0x01, 0x01, 0x01]));
    }

    #[test]
    fn corpus_cobsc_253_ascending() {
        // 01..FD (253 bytes) → [FE 01..FD]
        let input: Vec<u8> = (1..=253).map(|i| i as u8).collect();
        let mut expected = vec![0xFE];
        expected.extend(1u8..=253);
        assert_eq!(enc(&input), Some(expected.clone()));
        assert_eq!(dec(&expected), Some(input));
    }

    #[test]
    fn corpus_cobsc_zero_then_256_bytes() {
        // 00 01..FF (256 bytes) → [01 FF 01..FE 02 FF]
        let mut input = vec![0x00u8];
        input.extend(1u8..=255);
        let mut expected = vec![0x01, 0xFF];
        expected.extend(1u8..=254);
        expected.extend([0x02, 0xFF]);
        assert_eq!(enc(&input), Some(expected.clone()));
        assert_eq!(dec(&expected), Some(input));
    }

    // ── External corpus: cobs2-rs vector ────────────────────────

    #[test]
    fn corpus_cobs2rs_abc_ghij_xyz() {
        // "ABC\0ghij\0xyz"
        let input = [
            0x41, 0x42, 0x43, 0x00, 0x67, 0x68, 0x69, 0x6A, 0x00, 0x78, 0x79, 0x7A,
        ];
        let expected = [
            0x04, 0x41, 0x42, 0x43, 0x05, 0x67, 0x68, 0x69, 0x6A, 0x04, 0x78, 0x79, 0x7A,
        ];
        assert_eq!(enc(&input), Some(expected.to_vec()));
        assert_eq!(dec(&expected), Some(input.to_vec()));
    }

    // ── External corpus: Jacques Fortier vector ─────────────────

    #[test]
    fn corpus_fortier_254_nonzero_then_zero() {
        // 01..FE 00 (255 bytes) → [FF 01..FE 01 01]
        let mut input: Vec<u8> = (1..=254).collect();
        input.push(0x00);
        let mut expected = vec![0xFF];
        expected.extend(1u8..=254);
        expected.extend([0x01, 0x01]);
        assert_eq!(enc(&input), Some(expected.clone()));
        assert_eq!(dec(&expected), Some(input));
    }

    // ── External corpus: Python cobs package ────────────────────

    #[test]
    fn corpus_python_hello_world() {
        // "Hello world\0This is a test"
        let input = b"Hello world\x00This is a test";
        let expected = [
            0x0C, 0x48, 0x65, 0x6C, 0x6C, 0x6F, 0x20, 0x77, 0x6F, 0x72, 0x6C, 0x64, 0x0F, 0x54,
            0x68, 0x69, 0x73, 0x20, 0x69, 0x73, 0x20, 0x61, 0x20, 0x74, 0x65, 0x73, 0x74,
        ];
        assert_eq!(enc(input), Some(expected.to_vec()));
        assert_eq!(dec(&expected), Some(input.to_vec()));
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
        let data_end = their_encoded
            .iter()
            .rposition(|&b| b != 0x00)
            .map_or(0, |i| i + 1);
        let their_no_sentinel = &their_encoded[..data_end];

        let mut our_buf = vec![0u8; input.len() + 1];
        let our_len =
            decode(their_no_sentinel, &mut our_buf).expect("our decode failed on corncobs output");
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
        use rand::RngExt;
        let mut rng = rand::rng();
        for _ in 0..500 {
            let len = rng.random_range(0..=512);
            let data: Vec<u8> = (0..len).map(|_| rng.random()).collect();
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

    // ── Dest-size edge cases ────────────────────────────────────

    #[test]
    fn encode_dest_exact_size() {
        let input = [0x11, 0x00, 0x33];
        let needed = max_encoded_len(input.len());
        let mut buf = vec![0u8; needed];
        assert_eq!(encode(&input, &mut buf), Some(4));
    }

    #[test]
    fn encode_dest_one_too_small() {
        let input = [0x11, 0x00, 0x33];
        let needed = max_encoded_len(input.len());
        let mut buf = vec![0u8; needed - 1];
        assert_eq!(encode(&input, &mut buf), None);
    }

    #[test]
    fn encode_dest_empty() {
        assert_eq!(encode(&[0x11], &mut []), None);
    }

    #[test]
    fn encode_empty_into_single_byte() {
        let mut buf = [0u8; 1];
        assert_eq!(encode(&[], &mut buf), Some(1));
        assert_eq!(buf[0], 0x01);
    }

    #[test]
    fn decode_dest_too_small() {
        // Encoded [0x11, 0x22] = [0x03, 0x11, 0x22], needs 2 bytes output
        assert_eq!(decode(&[0x03, 0x11, 0x22], &mut [0u8; 1]), None);
    }

    #[test]
    fn decode_dest_exact_size() {
        let mut out = [0u8; 2];
        assert_eq!(decode(&[0x03, 0x11, 0x22], &mut out), Some(2));
        assert_eq!(out, [0x11, 0x22]);
    }

    // ── Multi-block boundary tests ──────────────────────────────

    #[test]
    fn roundtrip_508_nonzero() {
        // Two full 254-byte blocks back-to-back
        let data: Vec<u8> = (0..508).map(|i| (i % 254 + 1) as u8).collect();
        roundtrip(&data);
    }

    #[test]
    fn roundtrip_762_nonzero() {
        // Three full 254-byte blocks
        let data: Vec<u8> = (0..762).map(|i| (i % 254 + 1) as u8).collect();
        roundtrip(&data);
    }

    #[test]
    fn roundtrip_254_nonzero_then_zero() {
        // Full block immediately followed by a zero
        let mut data: Vec<u8> = (1..=254).collect();
        data.push(0x00);
        roundtrip(&data);
    }

    #[test]
    fn roundtrip_254_nonzero_then_zeros() {
        // Full block followed by multiple zeros
        let mut data: Vec<u8> = (1..=254).collect();
        data.extend([0x00, 0x00, 0x00]);
        roundtrip(&data);
    }

    // ── max_encoded_len edge cases ──────────────────────────────

    #[test]
    fn max_encoded_len_zero() {
        assert_eq!(max_encoded_len(0), 1);
    }

    #[test]
    fn max_encoded_len_one() {
        assert_eq!(max_encoded_len(1), 2);
    }

    #[test]
    fn max_encoded_len_254() {
        assert_eq!(max_encoded_len(254), 256);
    }

    #[test]
    fn max_encoded_len_255() {
        assert_eq!(max_encoded_len(255), 257);
    }

    // ── Encode always fits in max_encoded_len ───────────────────

    #[test]
    fn encode_fits_max_encoded_len_all_patterns() {
        for len in [0, 1, 2, 10, 100, 253, 254, 255, 256, 508, 512, 1024] {
            for fill in [0x00u8, 0x01, 0x7F, 0xFF] {
                let data = vec![fill; len];
                let max = max_encoded_len(len);
                let mut buf = vec![0u8; max];
                let n = encode(&data, &mut buf)
                    .unwrap_or_else(|| panic!("encode failed for len={len}, fill=0x{fill:02X}"));
                assert!(
                    n <= max,
                    "encoded len {n} exceeds max {max} for len={len}, fill=0x{fill:02X}"
                );
            }
        }
    }
}

// ── Property-based tests (proptest) ─────────────────────────────

#[cfg(test)]
mod proptests {
    extern crate alloc;
    use super::*;
    use alloc::vec;
    use proptest::prelude::*;

    // Bump case count: default 256 is too low for a codec this critical.
    const CASES: u32 = 25_000;

    fn config() -> ProptestConfig {
        ProptestConfig {
            cases: CASES,
            ..ProptestConfig::default()
        }
    }

    // ── Round-trip invariants ───────────────────────────────────

    proptest! {
        #![proptest_config(config())]

        // Core invariant: encode then decode recovers the original.
        #[test]
        fn roundtrip_any(data in proptest::collection::vec(any::<u8>(), 0..4096)) {
            let mut enc_buf = vec![0u8; max_encoded_len(data.len()) + 1];
            let enc_len = encode(&data, &mut enc_buf).unwrap();
            let encoded = &enc_buf[..enc_len];

            // Encoded output must never contain 0x00.
            prop_assert!(!encoded.contains(&0x00));

            let mut dec_buf = vec![0u8; data.len() + 1];
            let dec_len = decode(encoded, &mut dec_buf).unwrap();
            prop_assert_eq!(&dec_buf[..dec_len], &data[..]);
        }

        // Round-trip with exact-size dest (no extra room).
        #[test]
        fn roundtrip_exact_dest(data in proptest::collection::vec(any::<u8>(), 0..4096)) {
            let max = max_encoded_len(data.len());
            let mut enc_buf = vec![0u8; max];
            let enc_len = encode(&data, &mut enc_buf).unwrap();
            prop_assert!(enc_len <= max);

            let mut dec_buf = vec![0u8; data.len()];
            let dec_len = decode(&enc_buf[..enc_len], &mut dec_buf).unwrap();
            prop_assert_eq!(&dec_buf[..dec_len], &data[..]);
        }
    }

    // ── Encoding properties ─────────────────────────────────────

    proptest! {
        #![proptest_config(config())]

        // Encoded length never exceeds the theoretical maximum.
        #[test]
        fn encoded_length_bounded(data in proptest::collection::vec(any::<u8>(), 0..4096)) {
            let mut enc_buf = vec![0u8; max_encoded_len(data.len()) + 1];
            let enc_len = encode(&data, &mut enc_buf).unwrap();
            prop_assert!(enc_len <= max_encoded_len(data.len()));
        }

        // Encoded output never contains 0x00 (the sentinel).
        #[test]
        fn encoded_no_zeros(data in proptest::collection::vec(any::<u8>(), 0..4096)) {
            let mut enc_buf = vec![0u8; max_encoded_len(data.len()) + 1];
            let enc_len = encode(&data, &mut enc_buf).unwrap();
            prop_assert!(!enc_buf[..enc_len].contains(&0x00));
        }

        // Encoding into a too-small dest returns None, never panics.
        #[test]
        fn encode_small_dest_never_panics(
            data in proptest::collection::vec(any::<u8>(), 1..512),
            shrink in 1usize..256,
        ) {
            let max = max_encoded_len(data.len());
            let dest_size = if shrink >= max { 0 } else { max - shrink };
            let mut buf = vec![0u8; dest_size];
            let _ = encode(&data, &mut buf); // may return None, must not panic
        }

        // Empty input always encodes to [0x01].
        #[test]
        fn encode_empty_is_0x01(_dummy in 0u8..1) {
            let mut buf = [0u8; 1];
            let n = encode(&[], &mut buf).unwrap();
            prop_assert_eq!(n, 1);
            prop_assert_eq!(buf[0], 0x01);
        }
    }

    // ── Decoding properties ─────────────────────────────────────

    proptest! {
        #![proptest_config(config())]

        // Decoding random garbage must never panic.
        #[test]
        fn decode_garbage_never_panics(data in proptest::collection::vec(any::<u8>(), 0..4096)) {
            let mut dec_buf = vec![0u8; data.len() + 256];
            let _ = decode(&data, &mut dec_buf);
        }

        // Decoding into a too-small dest returns None, never panics.
        #[test]
        fn decode_small_dest_never_panics(
            data in proptest::collection::vec(any::<u8>(), 0..512),
            dest_size in 0usize..64,
        ) {
            let mut buf = vec![0u8; dest_size];
            let _ = decode(&data, &mut buf);
        }

        // A 0x00 at a code-byte position is always rejected.
        #[test]
        fn decode_rejects_zero_code_byte(
            suffix in proptest::collection::vec(any::<u8>(), 0..64),
        ) {
            // 0x00 as the very first byte (always a code-byte position).
            let mut data = vec![0x00];
            data.extend(&suffix);
            let mut buf = vec![0u8; data.len() + 256];
            prop_assert_eq!(decode(&data, &mut buf), None);
        }

        // Decoding the output of encode always succeeds.
        #[test]
        fn decode_of_encode_always_succeeds(data in proptest::collection::vec(any::<u8>(), 0..4096)) {
            let mut enc_buf = vec![0u8; max_encoded_len(data.len()) + 1];
            let enc_len = encode(&data, &mut enc_buf).unwrap();
            let mut dec_buf = vec![0u8; data.len() + 1];
            prop_assert!(decode(&enc_buf[..enc_len], &mut dec_buf).is_some());
        }
    }

    // ── Cross-validation against corncobs ───────────────────────

    proptest! {
        #![proptest_config(config())]

        // Our encode → corncobs decode must match.
        #[test]
        fn interop_our_encode_their_decode(data in proptest::collection::vec(any::<u8>(), 0..4096)) {
            let mut enc_buf = vec![0u8; max_encoded_len(data.len()) + 2];
            let enc_len = encode(&data, &mut enc_buf).unwrap();
            enc_buf[enc_len] = 0x00; // append sentinel for corncobs

            let mut dec_buf = vec![0u8; data.len() + 1];
            let dec_len = corncobs::decode_buf(&enc_buf[..enc_len + 1], &mut dec_buf).unwrap();
            prop_assert_eq!(&dec_buf[..dec_len], &data[..]);
        }

        // corncobs encode → our decode must match.
        #[test]
        fn interop_their_encode_our_decode(data in proptest::collection::vec(any::<u8>(), 0..4096)) {
            let mut enc_buf = vec![0u8; corncobs::max_encoded_len(data.len())];
            let enc_len = corncobs::encode_buf(&data, &mut enc_buf);
            let data_end = enc_buf[..enc_len].iter().rposition(|&b| b != 0x00).map_or(0, |i| i + 1);

            let mut dec_buf = vec![0u8; data.len() + 1];
            let dec_len = decode(&enc_buf[..data_end], &mut dec_buf).unwrap();
            prop_assert_eq!(&dec_buf[..dec_len], &data[..]);
        }

        // Both crates must produce byte-identical encoded output for the same input.
        #[test]
        fn interop_encode_byte_identical(data in proptest::collection::vec(any::<u8>(), 0..4096)) {
            // Encode with ucobs.
            let mut our_buf = vec![0u8; max_encoded_len(data.len()) + 2];
            let our_len = encode(&data, &mut our_buf).unwrap();

            // Encode with corncobs (includes trailing sentinel).
            let mut their_buf = vec![0u8; corncobs::max_encoded_len(data.len())];
            let their_len = corncobs::encode_buf(&data, &mut their_buf);
            // Strip corncobs sentinel for comparison.
            let their_end = their_buf[..their_len]
                .iter()
                .rposition(|&b| b != 0x00)
                .map_or(0, |i| i + 1);

            prop_assert_eq!(
                &our_buf[..our_len],
                &their_buf[..their_end],
                "encoded output differs for input len={}",
                data.len()
            );
        }
    }

    // ── Structural / algebraic properties ───────────────────────

    proptest! {
        #![proptest_config(config())]

        // Encoding is deterministic: same input always produces same output.
        #[test]
        fn encode_deterministic(data in proptest::collection::vec(any::<u8>(), 0..2048)) {
            let mut buf1 = vec![0u8; max_encoded_len(data.len()) + 1];
            let mut buf2 = vec![0u8; max_encoded_len(data.len()) + 1];
            let n1 = encode(&data, &mut buf1).unwrap();
            let n2 = encode(&data, &mut buf2).unwrap();
            prop_assert_eq!(n1, n2);
            prop_assert_eq!(&buf1[..n1], &buf2[..n2]);
        }

        // Decoding is deterministic.
        #[test]
        fn decode_deterministic(data in proptest::collection::vec(any::<u8>(), 0..2048)) {
            let mut enc_buf = vec![0u8; max_encoded_len(data.len()) + 1];
            let enc_len = encode(&data, &mut enc_buf).unwrap();
            let encoded = &enc_buf[..enc_len];

            let mut buf1 = vec![0u8; data.len() + 1];
            let mut buf2 = vec![0u8; data.len() + 1];
            let n1 = decode(encoded, &mut buf1).unwrap();
            let n2 = decode(encoded, &mut buf2).unwrap();
            prop_assert_eq!(n1, n2);
            prop_assert_eq!(&buf1[..n1], &buf2[..n2]);
        }

        // Encoded length is always >= 1 (even for empty input).
        #[test]
        fn encoded_length_at_least_one(data in proptest::collection::vec(any::<u8>(), 0..4096)) {
            let mut enc_buf = vec![0u8; max_encoded_len(data.len()) + 1];
            let enc_len = encode(&data, &mut enc_buf).unwrap();
            prop_assert!(enc_len >= 1);
        }

        // Encoded length is always > input length (COBS always adds overhead).
        #[test]
        fn encoded_length_strictly_greater(data in proptest::collection::vec(any::<u8>(), 0..4096)) {
            let mut enc_buf = vec![0u8; max_encoded_len(data.len()) + 1];
            let enc_len = encode(&data, &mut enc_buf).unwrap();
            prop_assert!(enc_len > data.len());
        }

        // Decoded length equals original input length (bijection).
        #[test]
        fn decoded_length_matches_original(data in proptest::collection::vec(any::<u8>(), 0..4096)) {
            let mut enc_buf = vec![0u8; max_encoded_len(data.len()) + 1];
            let enc_len = encode(&data, &mut enc_buf).unwrap();
            let mut dec_buf = vec![0u8; data.len() + 1];
            let dec_len = decode(&enc_buf[..enc_len], &mut dec_buf).unwrap();
            prop_assert_eq!(dec_len, data.len());
        }

        // First byte of encoded output is always a valid code byte (1..=255).
        #[test]
        fn first_encoded_byte_is_valid_code(data in proptest::collection::vec(any::<u8>(), 0..4096)) {
            let mut enc_buf = vec![0u8; max_encoded_len(data.len()) + 1];
            let enc_len = encode(&data, &mut enc_buf).unwrap();
            prop_assert!(enc_len >= 1);
            prop_assert!(enc_buf[0] >= 1); // code byte is never 0x00
        }
    }

    // ── Targeted boundary-region tests ──────────────────────────

    proptest! {
        #![proptest_config(config())]

        // Payloads near the 254-byte block boundary (the critical COBS edge).
        #[test]
        fn roundtrip_near_block_boundary(
            data in proptest::collection::vec(any::<u8>(), 250..260)
        ) {
            let mut enc_buf = vec![0u8; max_encoded_len(data.len()) + 1];
            let enc_len = encode(&data, &mut enc_buf).unwrap();
            let encoded = &enc_buf[..enc_len];
            prop_assert!(!encoded.contains(&0x00));

            let mut dec_buf = vec![0u8; data.len() + 1];
            let dec_len = decode(encoded, &mut dec_buf).unwrap();
            prop_assert_eq!(&dec_buf[..dec_len], &data[..]);
        }

        // Payloads spanning multiple block boundaries.
        #[test]
        fn roundtrip_multi_block(
            data in proptest::collection::vec(any::<u8>(), 500..520)
        ) {
            let mut enc_buf = vec![0u8; max_encoded_len(data.len()) + 1];
            let enc_len = encode(&data, &mut enc_buf).unwrap();
            let encoded = &enc_buf[..enc_len];
            prop_assert!(!encoded.contains(&0x00));

            let mut dec_buf = vec![0u8; data.len() + 1];
            let dec_len = decode(encoded, &mut dec_buf).unwrap();
            prop_assert_eq!(&dec_buf[..dec_len], &data[..]);
        }

        // Very small payloads (0–8 bytes) — high code-byte-to-data ratio.
        #[test]
        fn roundtrip_tiny(data in proptest::collection::vec(any::<u8>(), 0..8)) {
            let mut enc_buf = vec![0u8; max_encoded_len(data.len()) + 1];
            let enc_len = encode(&data, &mut enc_buf).unwrap();
            let encoded = &enc_buf[..enc_len];
            prop_assert!(!encoded.contains(&0x00));

            let mut dec_buf = vec![0u8; data.len() + 1];
            let dec_len = decode(encoded, &mut dec_buf).unwrap();
            prop_assert_eq!(&dec_buf[..dec_len], &data[..]);
        }

        // All-zeros payloads of varying length — worst-case overhead.
        #[test]
        fn roundtrip_all_zeros(len in 0usize..2048) {
            let data = vec![0u8; len];
            let mut enc_buf = vec![0u8; max_encoded_len(len) + 1];
            let enc_len = encode(&data, &mut enc_buf).unwrap();
            let encoded = &enc_buf[..enc_len];
            prop_assert!(!encoded.contains(&0x00));

            let mut dec_buf = vec![0u8; len + 1];
            let dec_len = decode(encoded, &mut dec_buf).unwrap();
            prop_assert_eq!(&dec_buf[..dec_len], &data[..]);
        }

        // All-0xFF payloads — no zeros at all, tests block-boundary logic.
        #[test]
        fn roundtrip_all_ff(len in 0usize..2048) {
            let data = vec![0xFFu8; len];
            let mut enc_buf = vec![0u8; max_encoded_len(len) + 1];
            let enc_len = encode(&data, &mut enc_buf).unwrap();
            let encoded = &enc_buf[..enc_len];
            prop_assert!(!encoded.contains(&0x00));

            let mut dec_buf = vec![0u8; len + 1];
            let dec_len = decode(encoded, &mut dec_buf).unwrap();
            prop_assert_eq!(&dec_buf[..dec_len], &data[..]);
        }

        // Single repeated byte — catches any byte-value-specific bugs.
        #[test]
        fn roundtrip_single_byte_repeated(byte in any::<u8>(), len in 0usize..1024) {
            let data = vec![byte; len];
            let mut enc_buf = vec![0u8; max_encoded_len(len) + 1];
            let enc_len = encode(&data, &mut enc_buf).unwrap();
            let encoded = &enc_buf[..enc_len];
            prop_assert!(!encoded.contains(&0x00));

            let mut dec_buf = vec![0u8; len + 1];
            let dec_len = decode(encoded, &mut dec_buf).unwrap();
            prop_assert_eq!(&dec_buf[..dec_len], &data[..]);
        }

        // Two-byte alphabet — high density of zeros mixed with non-zeros.
        #[test]
        fn roundtrip_binary_alphabet(
            data in proptest::collection::vec(prop_oneof![Just(0x00u8), Just(0xFFu8)], 0..2048)
        ) {
            let mut enc_buf = vec![0u8; max_encoded_len(data.len()) + 1];
            let enc_len = encode(&data, &mut enc_buf).unwrap();
            let encoded = &enc_buf[..enc_len];
            prop_assert!(!encoded.contains(&0x00));

            let mut dec_buf = vec![0u8; data.len() + 1];
            let dec_len = decode(encoded, &mut dec_buf).unwrap();
            prop_assert_eq!(&dec_buf[..dec_len], &data[..]);
        }
    }
}
