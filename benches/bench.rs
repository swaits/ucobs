use std::hint::black_box;

use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};

// ── Test payloads ───────────────────────────────────────────────────

fn payload_zeros(len: usize) -> Vec<u8> {
    vec![0u8; len]
}

fn payload_nonzero(len: usize) -> Vec<u8> {
    (0..len).map(|i| (i % 255 + 1) as u8).collect()
}

fn payload_mixed(len: usize) -> Vec<u8> {
    (0..len).map(|i| (i % 256) as u8).collect()
}

// ── ucobs ───────────────────────────────────────────────────────────

fn ucobs_encode(data: &[u8]) -> Vec<u8> {
    let mut buf = vec![0u8; ucobs::max_encoded_len(data.len()) + 1];
    let n = ucobs::encode(data, &mut buf).unwrap();
    buf.truncate(n);
    buf
}

fn ucobs_decode(encoded: &[u8], out: &mut [u8]) -> usize {
    ucobs::decode(encoded, out).unwrap()
}

// ── corncobs ────────────────────────────────────────────────────────

fn corncobs_encode(data: &[u8]) -> Vec<u8> {
    let mut buf = vec![0u8; corncobs::max_encoded_len(data.len())];
    let n = corncobs::encode_buf(data, &mut buf);
    buf.truncate(n);
    buf
}

fn corncobs_decode(encoded: &[u8], out: &mut [u8]) -> usize {
    // corncobs expects trailing sentinel
    corncobs::decode_buf(encoded, out).unwrap()
}

// ── cobs crate ──────────────────────────────────────────────────────

fn cobs_encode(data: &[u8]) -> Vec<u8> {
    let mut buf = vec![0u8; cobs::max_encoding_length(data.len())];
    let n = cobs::encode(data, &mut buf);
    buf.truncate(n);
    buf
}

fn cobs_decode(encoded: &[u8], out: &mut [u8]) -> usize {
    // The cobs crate returns DecodeReport on success or Err(EmptyFrame)
    // for empty input. We extract parsed_size, treating errors as 0.
    cobs::decode(encoded, out)
        .map(|report| report.parsed_size())
        .unwrap_or(0)
}

// ── Benchmarks ──────────────────────────────────────────────────────

const SIZES: &[usize] = &[0, 1, 10, 64, 254, 255, 256, 1024, 4096];

fn bench_encode(c: &mut Criterion) {
    let payloads: Vec<(&str, Box<dyn Fn(usize) -> Vec<u8>>)> = vec![
        ("zeros", Box::new(payload_zeros)),
        ("nonzero", Box::new(payload_nonzero)),
        ("mixed", Box::new(payload_mixed)),
    ];

    for (name, make) in &payloads {
        let mut group = c.benchmark_group(format!("encode/{name}"));
        for &size in SIZES {
            let data = make(size);
            group.throughput(Throughput::Bytes(size as u64));

            group.bench_with_input(BenchmarkId::new("ucobs", size), &data, |b, d| {
                let mut buf = vec![0u8; ucobs::max_encoded_len(d.len()) + 1];
                b.iter(|| {
                    black_box(ucobs::encode(black_box(d), &mut buf).unwrap());
                });
            });

            group.bench_with_input(BenchmarkId::new("corncobs", size), &data, |b, d| {
                let mut buf = vec![0u8; corncobs::max_encoded_len(d.len())];
                b.iter(|| {
                    black_box(corncobs::encode_buf(black_box(d), &mut buf));
                });
            });

            group.bench_with_input(BenchmarkId::new("cobs", size), &data, |b, d| {
                let mut buf = vec![0u8; cobs::max_encoding_length(d.len())];
                b.iter(|| {
                    black_box(cobs::encode(black_box(d), &mut buf));
                });
            });
        }
        group.finish();
    }
}

fn bench_decode(c: &mut Criterion) {
    let payloads: Vec<(&str, Box<dyn Fn(usize) -> Vec<u8>>)> = vec![
        ("zeros", Box::new(payload_zeros)),
        ("nonzero", Box::new(payload_nonzero)),
        ("mixed", Box::new(payload_mixed)),
    ];

    for (name, make) in &payloads {
        let mut group = c.benchmark_group(format!("decode/{name}"));
        for &size in SIZES {
            let data = make(size);
            let ucobs_enc = ucobs_encode(&data);
            // corncobs expects sentinel in encoded form
            let corncobs_enc = corncobs_encode(&data);
            let cobs_enc = cobs_encode(&data);

            group.throughput(Throughput::Bytes(size as u64));

            group.bench_with_input(BenchmarkId::new("ucobs", size), &ucobs_enc, |b, enc| {
                let mut buf = vec![0u8; size + 1];
                b.iter(|| {
                    black_box(ucobs_decode(black_box(enc), &mut buf));
                });
            });

            group.bench_with_input(
                BenchmarkId::new("corncobs", size),
                &corncobs_enc,
                |b, enc| {
                    let mut buf = vec![0u8; size + 1];
                    b.iter(|| {
                        black_box(corncobs_decode(black_box(enc), &mut buf));
                    });
                },
            );

            group.bench_with_input(BenchmarkId::new("cobs", size), &cobs_enc, |b, enc| {
                let mut buf = vec![0u8; size + 1];
                b.iter(|| {
                    black_box(cobs_decode(black_box(enc), &mut buf));
                });
            });
        }
        group.finish();
    }
}

criterion_group!(benches, bench_encode, bench_decode);
criterion_main!(benches);
