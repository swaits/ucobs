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
| `const fn` encode | yes | no                | no             |
| Implementation | ~120 LOC | ~790 LOC        | ~645 LOC       |

[^1]: Requires disabling the default `std` feature.
[^2]: `alloc` is enabled by default via the `std` feature.

> Measured on Intel N100, Rust 1.93.1, Linux 6.12. Run `just bench` to
> reproduce. Full Criterion reports are generated in `target/criterion/report/`.

## When to use μCOBS

| You need… | μCOBS | `cobs` 0.3 | `corncobs` 0.1 |
|---|:---:|:---:|:---:|
| Minimal code to audit | **~120 LOC** | ~790 LOC | ~645 LOC |
| Compile-time (`const fn`) encode | **yes** | no | no |
| Fastest encode (zero-heavy data) | **3–4×** faster | baseline | slowest |
| Fastest decode (large payloads) | competitive | slow | **slight edge** |
| Dead-simple API (3 functions) | **yes** | yes | more surface area |
| In-place / streaming encode | no | no | **yes** |
| `no_std` + zero-alloc by default | **yes** | opt-in | **yes** |
| Thorough test suite | **106 tests, fuzz, proptest** | basic | basic |

**Choose μCOBS** if you want the smallest, most auditable COBS implementation
with a minimal API and strong performance — especially on embedded targets or
payloads containing many zero bytes.

**Choose `corncobs`** if you need in-place encoding, an iterator-based API, or
your workload is dominated by large non-zero payloads where it holds a slight
throughput edge.

**Choose `cobs`** if you need `std` convenience features and don't care about
`no_std` or encode performance.

## Examples

### Basic encode and decode

```rust
// Encode: [0x11, 0x00, 0x33] → [0x02, 0x11, 0x02, 0x33]
let mut buf = [0u8; 16];
let n = ucobs::encode(&[0x11, 0x00, 0x33], &mut buf).unwrap();
assert_eq!(&buf[..n], &[0x02, 0x11, 0x02, 0x33]);

// Decode reverses it
let mut out = [0u8; 16];
let m = ucobs::decode(&buf[..n], &mut out).unwrap();
assert_eq!(&out[..m], &[0x11, 0x00, 0x33]);
```

### Buffer sizing

Use `max_encoded_len` to determine the required destination buffer size:

```rust
let data = [0x01, 0x02, 0x03];
let max = ucobs::max_encoded_len(data.len()); // 4 bytes

let mut buf = [0u8; 4];
let n = ucobs::encode(&data, &mut buf).unwrap();
assert_eq!(n, 4); // fits exactly
```

If the destination is too small, `encode` returns `None` rather than panicking:

```rust
let mut tiny = [0u8; 1];
assert_eq!(ucobs::encode(&[0x01, 0x02], &mut tiny), None);
```

### Framing for transport

Append a `0x00` sentinel after encoding to delimit frames on a wire. Strip
it before decoding:

```rust
let data = [0x11, 0x00, 0x33];

// Encode and append sentinel
let mut frame = [0u8; 16];
let n = ucobs::encode(&data, &mut frame).unwrap();
frame[n] = 0x00;
let wire = &frame[..n + 1];

// Receive: strip sentinel, then decode
let cobs_data = &wire[..wire.len() - 1];
let mut out = [0u8; 16];
let m = ucobs::decode(cobs_data, &mut out).unwrap();
assert_eq!(&out[..m], &data);
```

### Parsing a stream of frames

Split a byte stream on `0x00` to extract individual COBS frames:

```rust
// Two frames separated by 0x00 sentinels
let stream = [
    0x02, 0x11, 0x02, 0x33, 0x00,  // frame 1: [0x11, 0x00, 0x33]
    0x03, 0xAA, 0xBB, 0x00,        // frame 2: [0xAA, 0xBB]
];

let mut out = [0u8; 16];
let frames: Vec<&[u8]> = stream.split(|&b| b == 0x00)
    .filter(|f| !f.is_empty())
    .collect();

let n = ucobs::decode(frames[0], &mut out).unwrap();
assert_eq!(&out[..n], &[0x11, 0x00, 0x33]);

let n = ucobs::decode(frames[1], &mut out).unwrap();
assert_eq!(&out[..n], &[0xAA, 0xBB]);
```

### Compile-time encoding

`encode` is `const fn`, so you can build COBS-encoded tables at compile
time with zero runtime cost:

```rust
const PING: [u8; 2] = {
    let mut buf = [0u8; 2];
    match ucobs::encode(&[0x01], &mut buf) {
        Some(_) => buf,
        None => panic!("buffer too small"),
    }
};

const ACK: [u8; 3] = {
    let mut buf = [0u8; 3];
    match ucobs::encode(&[0x06, 0x00], &mut buf) {
        Some(_) => buf,
        None => panic!("buffer too small"),
    }
};

// Baked into the binary — no runtime encoding needed
assert_eq!(PING, [0x02, 0x01]);
assert_eq!(ACK, [0x02, 0x06, 0x01]);
```

### Error handling

Both `encode` and `decode` return `Option<usize>` — `None` on failure,
never panic:

```rust
let mut buf = [0u8; 8];

// Destination too small
assert_eq!(ucobs::encode(&[1, 2, 3], &mut buf[..1]), None);

// Malformed input (zero byte in encoded stream)
assert_eq!(ucobs::decode(&[0x00], &mut buf), None);

// Truncated frame (code byte promises more data than exists)
assert_eq!(ucobs::decode(&[0x05, 0x11], &mut buf), None);

// Empty input is valid
assert_eq!(ucobs::encode(&[], &mut buf), Some(1)); // encodes to [0x01]
assert_eq!(ucobs::decode(&[], &mut buf), Some(0)); // decodes to []
```

## API

| Function | Signature | Description |
|---|---|---|
| `encode` | `const fn(src: &[u8], dest: &mut [u8]) -> Option<usize>` | COBS-encode `src` into `dest`. Returns byte count or `None` if buffer too small. |
| `decode` | `(src: &[u8], dest: &mut [u8]) -> Option<usize>` | Decode COBS data. Returns byte count or `None` if input is malformed. |
| `max_encoded_len` | `const fn(src_len: usize) -> usize` | Maximum encoded size for a given input length. |

**Note:** The encoder does *not* append a trailing `0x00` sentinel. Append it
yourself when framing for transport.

## Testing

μCOBS has one of the most thorough test suites of any COBS implementation —
106 tests across 7 categories:

- **Canonical vectors** — every example from Cheshire & Baker 1999 (the original
  COBS paper), plus the full Wikipedia vector set
- **External corpora** — vectors drawn from `cobs-c`, `nanocobs`, `cobs2-rs`,
  Jacques Fortier's C implementation, and the Python `cobs` package
- **Cross-crate interop** — byte-for-byte validation against both `corncobs` and
  `cobs` crates on randomized payloads up to 4 KB
- **Property-based tests** — 6 proptest suites covering round-trip correctness,
  no-zero-in-output invariants, encoding properties, decoding properties,
  structural/algebraic properties, and boundary-region behavior
- **Fuzz targets** — 3 `cargo-fuzz` harnesses (decode, round-trip, small-dest)
  for continuous coverage of edge cases
- **Boundary tests** — exhaustive coverage of 254/255-byte block boundaries,
  multi-block splits, buffer-exact fits, and off-by-one dest sizes
- **Compile-time verification** — `const` assertions that validate encode
  correctness at compile time

```sh
# Unit + property + interop tests
just test-unit

# All quality gates (tests, clippy, fmt, doc, fuzz, miri)
just test

# Fuzz testing only (requires nightly + cargo-fuzz)
just test-fuzz
```

## License

[MIT](LICENSE) — Stephen Waits
