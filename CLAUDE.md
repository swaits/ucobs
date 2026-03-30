# μCOBS (ucobs)

## Identity

- **Crate name**: `ucobs` (on crates.io)
- **Display name**: μCOBS
- **Mission**: Become THE COBS crate for Rust — embedded and non-embedded alike

## First Principles

1. **Correct** — 100% COBS spec-compliant (Cheshire & Baker 1999). No shortcuts, no deviations.
2. **Interoperable** — proven byte-for-byte compatibility with `corncobs`, `cobs` crates and Python `cobs` packages.
3. **Insanely tested** — canonical vectors, property-based tests (proptest), fuzz targets (cargo-fuzz), cross-crate interop, randomized payloads. If you think testing is done, you're 10% done.
4. **Fast** — benchmark against every competing implementation. Must be at least as fast, ideally fastest. Benchmarks are CI-enforced.
5. **Tiny** — `no_std`, zero-alloc, zero runtime dependencies. ~120 lines of implementation. Runs on 8-bit MCUs and servers alike.
6. **Trustworthy** — small enough to audit by hand. No unsafe. No panics in release. Fuzz-hardened.

## Architecture

Single-file library: `src/lib.rs`

### Public API (3 functions)

```rust
pub fn encode(src: &[u8], dest: &mut [u8]) -> Option<usize>
pub fn decode(src: &[u8], dest: &mut [u8]) -> Option<usize>
pub const fn max_encoded_len(src_len: usize) -> usize
```

**Convention**: Output does NOT include trailing `0x00` sentinel. Caller appends it for framing.

## Development

```sh
# Run all tests
cargo test

# Fuzz (requires nightly + cargo-fuzz)
cd fuzz
cargo +nightly fuzz run fuzz_decode -- -max_total_time=60
cargo +nightly fuzz run fuzz_roundtrip -- -max_total_time=60
```

## Dev Dependencies (test-only)

- `corncobs` — cross-validation against another COBS implementation
- `proptest` — property-based testing
- `rand` — randomized test payloads

## Roadmap

- More interop targets: `cobs` crate, Python `cobs` package
- Benchmarks (criterion) vs corncobs, cobs, and others
- quickcheck in addition to proptest
- Even more fuzz targets and edge-case generators
- MSRV policy and CI
- `#[deny(unsafe_code)]` badge
