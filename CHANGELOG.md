# Changelog

## 0.3.0 — 2026-03-31

### Performance

- **Encode nonzero/mixed: ~2x faster** — uses const `copy_from_slice` (memcpy at
  runtime) via Rust 1.93's stabilization of const slice methods. Sub-slice scan
  pattern helps LLVM eliminate bounds checks.
- **Encode zeros: ~50% faster** — fast path skips scan loop overhead for leading
  zero bytes.
- **Decode zeros: ~4x faster** — batch consecutive `0x01` codes and fill zeros
  via `fill(0)` (memset) instead of per-byte iteration.
- Encode now wins or ties corncobs on nonzero/mixed workloads.
- Decode zeros now leads all competitors by 3x.

### Added

- `legacy-msrv` feature flag — enables compilation on Rust 1.83+ (e.g. Xtensa
  toolchain) by replacing const `copy_from_slice` with a manual byte loop.
  All other optimizations (sub-slice scan, zero fast path, decode batch fill)
  work without this feature on any supported Rust version.

### Changed

- **MSRV bumped from 1.83 to 1.93** — required for const `copy_from_slice`.
  Use the `legacy-msrv` feature for older toolchains.
- Dev-dependencies updated: `cobs` 0.3 to 0.5, `criterion` 0.5 to 0.8,
  `rand` 0.8 to 0.10.

## 0.2.0 — 2026-03-30

### Added

- `const fn` encode — encode data at compile time with zero runtime cost.
- Comprehensive documentation with 7 usage examples (basic, buffer sizing,
  framing, stream parsing, compile-time encoding, error handling).
- Initial encode/decode throughput optimizations.

### Changed

- MSRV set to 1.83.

## 0.1.0 — 2026-03-29

- Initial release.
- `no_std`, zero-alloc, `unsafe`-free COBS encoder and decoder.
- 106 tests: canonical vectors, property-based (proptest), fuzz targets,
  cross-crate interop with `corncobs` and `cobs`.
