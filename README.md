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
[iai-callgrind](https://github.com/iai-callgrind/iai-callgrind) (instruction
counts via Valgrind — deterministic, zero run-to-run variance). Three payload
patterns: **all zeros** (worst case), **no zeros**, and **mixed** (0x00–0xFF
cycling). Instruction count per call, lower is better.
Legend: 🥇 winner, 🥉 last, 🤝 tie (<3%).

### Encode

| Payload | Pattern | μCOBS | `cobs` 0.5 | `corncobs` 0.1 |
|---------|---------|------:|-----------:|---------------:|
| 64 B    | zeros   | 🤝 1,171 | 🤝 1,164 |    🥉 4,098   |
| 64 B    | nonzero | 🤝 512 | 🥉 1,100  |       🤝 514   |
| 64 B    | mixed   | 🥇 523 |  🥉 1,101 |          570   |
| 256 B   | zeros   | 🤝 4,243 | 🤝 4,236 |   🥉 15,810   |
| 256 B   | nonzero | 🤝 1,552 | 🥉 3,992 |   🤝 1,531    |
| 256 B   | mixed   | 🤝 1,561 | 🥉 3,993 |   🤝 1,585    |
| 4096 B  | zeros   | 🤝 65,904 | 🤝 65,897 | 🥉 250,271  |
| 4096 B  | nonzero | 🤝 22,077 | 🥉 61,993 | 🤝 21,711   |
| 4096 B  | mixed   | 🤝 22,932 | 🥉 62,009 | 🤝 22,656   |

### Decode

| Payload | Pattern | μCOBS | `cobs` 0.5 | `corncobs` 0.1 |
|---------|---------|------:|-----------:|---------------:|
| 64 B    | zeros   | 🥇 499 | 🥉 2,354  |        1,507   |
| 64 B    | nonzero | 🥇 179 | 🥉 1,778  |          185   |
| 64 B    | mixed   |    226 | 🥉 1,787  |       🥇 206   |
| 256 B   | zeros   | 🥇 1,473 | 🥉 8,882 |       5,539   |
| 256 B   | nonzero | 🤝 240 | 🥉 6,607  |       🤝 247   |
| 256 B   | mixed   |    285 | 🥉 6,613  |       🥇 266   |
| 4096 B  | zeros   | 🥇 25,018 | 🥉 139,698 |   86,435   |
| 4096 B  | nonzero | 🤝 1,315 | 🥉 103,253 |  🤝 1,337   |
| 4096 B  | mixed   | 🤝 2,086 | 🥉 103,394 |  🤝 2,067   |

### Code size

Measured from `.text` section of release-optimized symbols (`encode` + `decode`
combined). Run `just bench-size` to reproduce.

| Crate          | `encode` | `decode` | Total |
|----------------|--------:|---------:|------:|
| μCOBS          |   429 B |    392 B | 821 B |
| `cobs` 0.5     |   282 B |    494 B | 776 B |
| `corncobs` 0.1 |   375 B |    262 B | 637 B |

All three crates are under 1 KB. μCOBS is the largest because safe
`const fn` encoding with `copy_from_slice`/memcpy requires `split_at`
bounds checks that generate panic paths — the cost of combining
compile-time safety with runtime speed. Embedded users targeting size
over speed can build with `opt-level = "s"` which reduces μCOBS to
~738 B.

### Crate properties

| Property       | μCOBS  | `cobs` 0.5       | `corncobs` 0.1 |
|----------------|:------:|:-----------------:|:--------------:|
| `no_std`       | always | opt-in[^1]        | always         |
| Zero-alloc     | yes    | opt-in[^2]        | yes            |
| `unsafe`-free  | yes    | yes               | yes            |
| `const fn` encode | yes | no                | no             |
| Implementation | ~140 LOC | ~790 LOC        | ~645 LOC       |

[^1]: Requires disabling the default `std` feature.
[^2]: `alloc` is enabled by default via the `std` feature.

> Benchmarked with iai-callgrind (Valgrind instruction counting) for
> deterministic, reproducible results. Run `just bench` to reproduce.
> Requires `valgrind` installed.

## When to use μCOBS

| You need… | μCOBS | `cobs` 0.5 | `corncobs` 0.1 |
|---|:---:|:---:|:---:|
| Minimal code to audit | **~140 LOC** | ~790 LOC | ~645 LOC |
| Compile-time (`const fn`) encode | **yes** | no | no |
| Fewest encode instructions | **ties cobs/corncobs** | zeros only | nonzero only |
| Fewest decode instructions (zero-heavy) | **yes (3–5×)** | no | 2nd |
| Fewest decode instructions (nonzero) | **yes/tied** | no | tied |
| Fewest decode instructions (mixed) | tied at 4 KB | no | **slight edge at small sizes** |
| Dead-simple API (3 functions) | **yes** | yes | more surface area |
| In-place / streaming encode | no | no | **yes** |
| `no_std` + zero-alloc by default | **yes** | opt-in | **yes** |
| Thorough test suite | **111 tests, fuzz, proptest, mutants** | basic | basic |

**Choose μCOBS** if you want the smallest, most auditable COBS implementation
with a `const fn` encoder, a dead-simple 3-function API, and leading or tied
instruction efficiency across most workloads — plus dominant decode performance
on zero-heavy data (3–5× fewer instructions than alternatives). Wins or ties
16 of 18 benchmarks.

**Choose `corncobs`** if you need in-place encoding, an iterator-based API,
or your workload is dominated by small mixed-pattern decoding where it uses
~8% fewer instructions at 64–256 B.

**Choose `cobs`** if you need `std` convenience features or a streaming state
machine. Note: `cobs` uses 50–75× more instructions than μCOBS/corncobs for
decoding non-zero data.

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

## Minimum Supported Rust Version

The default build requires **Rust 1.93+** (for const `copy_from_slice`).

### `legacy-msrv` feature

If you're targeting a toolchain older than 1.93 (e.g. Xtensa), enable the
`legacy-msrv` feature to fall back to a manual byte copy in the encoder:

```toml
[dependencies]
ucobs = { version = "0.3", features = ["legacy-msrv"] }
```

All other optimizations (sub-slice scan, zero fast path, decode batch fill)
are available regardless of this feature. Only the encode copy phase is
affected — it uses a byte loop instead of `copy_from_slice`/memcpy.

## Testing

μCOBS has one of the most thorough test suites of any COBS implementation —
111 tests across 8 categories:

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
- **Mutation testing** — `cargo-mutants` verifies that the test suite catches
  injected bugs in every reachable code path
- **Boundary tests** — exhaustive coverage of 254/255-byte block boundaries,
  multi-block splits, buffer-exact fits, and off-by-one dest sizes
- **Compile-time verification** — `const` assertions that validate encode
  correctness at compile time

```sh
# Unit + property + interop tests
just test-unit

# All quality gates (tests, clippy, fmt, doc, fuzz, miri, mutants)
just test

# Mutation testing only
just test-mutants
```

## License

[MIT](LICENSE) — Stephen Waits
