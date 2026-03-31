# Changelog

## 0.3.1 — 2026-03-31

### Performance

- **Decode: rewritten with shrinking-slice pattern** — decode zeros is 14%
  faster (25,018 vs 29,114 instructions at 4 KB), decode nonzero flipped from
  5% behind corncobs to 2% ahead, decode mixed closed from 8% behind to tied.
  ucobs now wins or ties 16 of 18 benchmarks.
- **Encode zeros: 6% faster** — zero fast path moved before `split_at` calls,
  eliminating per-byte overhead. Now tied with cobs (previously 6% behind).
- **Code size reduced** — 821 B total (was 869 B). Encode 429 B, decode 392 B.

### Changed

- **Replaced criterion with iai-callgrind** — benchmarks now measure
  deterministic instruction counts via Valgrind instead of wall-clock time.
  Zero run-to-run variance. Requires `valgrind` installed.
- **Expanded benchmark coverage** — 9 payload sizes (0–4096 B) × 3 patterns
  × 3 crates × encode/decode = 162 benchmarks, all running in ~2 minutes.
- **Code size measurement** added to `just bench` via `just bench-size`.
- **Mutation testing** added via `cargo-mutants` (`just test-mutants`).
- Added `mise.toml` for dev tool management.

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
