# μCOBS development tasks

# default: list all recipes
default:
    @just --list

# run ALL quality gates (tests, clippy, fmt check, doc, fuzz, miri)
test: test-unit test-clippy test-fmt test-doc test-fuzz test-miri

# quick compile check (no tests)
check:
    cargo check

# unit + integration + proptest tests (via nextest for parallelism)
test-unit:
    cargo nextest run
    cargo test --doc

# clippy with all warnings as errors
test-clippy:
    cargo clippy -- -D warnings

# check formatting
test-fmt:
    cargo fmt -- --check

# doc build (catches broken doc links)
test-doc:
    cargo doc --no-deps

# fuzz all targets (60s each, requires nightly + cargo-fuzz)
test-fuzz:
    cd fuzz && cargo +nightly fuzz run fuzz_decode -- -max_total_time=60
    cd fuzz && cargo +nightly fuzz run fuzz_roundtrip -- -max_total_time=60
    cd fuzz && cargo +nightly fuzz run fuzz_small_dest -- -max_total_time=60

# run tests under miri (catches UB, requires nightly + miri component)
# proptest is excluded — its file-persistence layer calls getcwd which miri does not support
test-miri:
    cargo +nightly miri test -- --skip proptests

# run criterion benchmarks (ucobs vs cobs vs corncobs)
bench:
    cargo bench

# run python cobs benchmarks (requires: pip install cobs)
bench-python:
    python3 bench_python.py

# run all benchmarks (rust + python)
bench-all: bench bench-python

# format code
fmt:
    cargo fmt

# build in release mode
build:
    cargo build --release

# clean all build artifacts
clean:
    cargo clean
    rm -rf fuzz/target fuzz/corpus fuzz/artifacts
