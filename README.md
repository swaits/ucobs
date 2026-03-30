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

## Performance

Benchmarked against the two other `no_std` COBS crates using
[Criterion](https://github.com/bheisler/criterion.rs) across three payload
patterns: **all zeros** (worst case — every byte triggers a code emit),
**no zeros**, and **mixed** (0x00–0xFF cycling). Throughput in MB/s, higher
is better.

### Encode

| Payload | Pattern | μCOBS | `cobs` 0.3 | `corncobs` 0.1 |
|---------|---------|------:|-----------:|---------------:|
| 64 B    | zeros   |   595 |        748 |            180 |
| 64 B    | mixed   | 1,853 |      1,011 |          1,734 |
| 64 B    | nonzero | 1,884 |      1,012 |          2,110 |
| 256 B   | zeros   |   654 |        817 |            184 |
| 256 B   | mixed   | 1,888 |      1,002 |          1,887 |
| 256 B   | nonzero | 1,964 |      1,004 |          1,968 |
| 4096 B  | zeros   |   675 |        845 |            188 |
| 4096 B  | mixed   | 1,734 |      1,016 |          1,746 |
| 4096 B  | nonzero | 1,775 |      1,020 |          2,245 |

### Decode

| Payload | Pattern | μCOBS | `cobs` 0.3 | `corncobs` 0.1 |
|---------|---------|------:|-----------:|---------------:|
| 64 B    | zeros   |   595 |        440 |            732 |
| 64 B    | mixed   | 7,041 |        504 |          7,991 |
| 64 B    | nonzero | 9,900 |        502 |          9,807 |
| 256 B   | zeros   |   656 |        472 |            742 |
| 256 B   | mixed   |17,024 |        645 |         16,207 |
| 256 B   | nonzero |18,456 |        647 |         19,549 |
| 4096 B  | zeros   |   675 |        483 |            768 |
| 4096 B  | mixed   |17,522 |        656 |         22,959 |
| 4096 B  | nonzero |37,987 |        623 |         40,005 |

### Code size

Measured from `.text` section of release-optimized symbols (`encode` + `decode`
combined):

| Crate          | Code size |
|----------------|----------:|
| μCOBS          |   541 B   |
| `cobs` 0.3     |   537 B   |
| `corncobs` 0.1 |   637 B   |

All three crates are tiny. The difference is negligible for any target.

### Crate properties

| Property       | μCOBS  | `cobs` 0.3       | `corncobs` 0.1 |
|----------------|:------:|:-----------------:|:--------------:|
| `no_std`       | always | opt-in[^1]        | always         |
| Zero-alloc     | yes    | opt-in[^2]        | yes            |
| `unsafe`-free  | yes    | yes               | yes            |
| Implementation | ~120 LOC | ~790 LOC        | ~645 LOC       |

[^1]: Requires disabling the default `std` feature.
[^2]: `alloc` is enabled by default via the `std` feature.

> Measured on Intel N100, Rust 1.93.1, Linux 6.12. Run `cargo bench` to
> reproduce. Full Criterion reports are generated in `target/criterion/report/`.

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
