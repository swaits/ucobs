# μCOBS

[![crates.io](https://img.shields.io/crates/v/ucobs.svg)](https://crates.io/crates/ucobs)
[![docs.rs](https://docs.rs/ucobs/badge.svg)](https://docs.rs/ucobs)
[![license](https://img.shields.io/crates/l/ucobs.svg)](LICENSE)

**The** COBS implementation for Rust.

[Consistent Overhead Byte Stuffing](https://en.wikipedia.org/wiki/Consistent_Overhead_Byte_Stuffing)
(COBS) encodes arbitrary bytes so that `0x00` never appears in the output,
allowing `0x00` to serve as an unambiguous frame delimiter. Overhead is exactly
1 byte per 254 input bytes (worst case).

## Features

- **`no_std`, zero-alloc** — runs anywhere, from 8-bit MCUs to servers
- **~140 lines of implementation** — small, auditable, easy to verify
- **100% spec-compliant** — Cheshire & Baker, IEEE/ACM Transactions on Networking, 1999
- **Proven interoperability** — cross-validated against `corncobs`, `cobs`, and Python `cobs`
- **Insanely tested** — canonical vectors, property-based tests, fuzz targets,
  cross-crate interop, randomized payloads, and more coming

## Usage

```rust
// Encode
let mut buf = [0u8; 16];
let n = ucobs::encode(&[0x11, 0x00, 0x33], &mut buf).unwrap();
assert_eq!(&buf[..n], &[0x02, 0x11, 0x02, 0x33]);

// Decode
let mut out = [0u8; 16];
let n = ucobs::decode(&[0x02, 0x11, 0x02, 0x33], &mut out).unwrap();
assert_eq!(&out[..n], &[0x11, 0x00, 0x33]);

// Buffer sizing
let max = ucobs::max_encoded_len(256); // 258
```

## API

| Function | Signature | Description |
|---|---|---|
| `encode` | `(src: &[u8], dest: &mut [u8]) -> Option<usize>` | COBS-encode `src` into `dest`. Returns byte count or `None` if buffer too small. |
| `decode` | `(src: &[u8], dest: &mut [u8]) -> Option<usize>` | Decode COBS data. Returns byte count or `None` if input is malformed. |
| `max_encoded_len` | `const fn(src_len: usize) -> usize` | Maximum encoded size for a given input length. |

**Note:** The encoder does *not* append a trailing `0x00` sentinel. Append it
yourself when framing for transport.

## Testing

```sh
# Unit + property + interop tests
cargo test

# Fuzz testing (requires nightly + cargo-fuzz)
cd fuzz
cargo +nightly fuzz run fuzz_decode -- -max_total_time=60
cargo +nightly fuzz run fuzz_roundtrip -- -max_total_time=60
```

## License

[MIT](LICENSE) — Stephen Waits
