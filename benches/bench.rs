use iai_callgrind::{library_benchmark, library_benchmark_group, main};
use std::hint::black_box;

// ── Test payloads ───────────────────────────────────────────────────

fn zeros(n: usize) -> Vec<u8> {
    vec![0u8; n]
}

fn nonzero(n: usize) -> Vec<u8> {
    (0..n).map(|i| (i % 255 + 1) as u8).collect()
}

fn mixed(n: usize) -> Vec<u8> {
    (0..n).map(|i| (i % 256) as u8).collect()
}

// ── Setup helpers ───────────────────────────────────────────────────

fn setup_encode(data: Vec<u8>) -> (Vec<u8>, Vec<u8>) {
    let buf = vec![0u8; ucobs::max_encoded_len(data.len()) + 1];
    (data, buf)
}

fn setup_decode_ucobs(data: Vec<u8>) -> (Vec<u8>, Vec<u8>) {
    let mut enc = vec![0u8; ucobs::max_encoded_len(data.len()) + 1];
    let n = ucobs::encode(&data, &mut enc).unwrap();
    enc.truncate(n);
    let dec = vec![0u8; data.len() + 1];
    (enc, dec)
}

fn setup_encode_corncobs(data: Vec<u8>) -> (Vec<u8>, Vec<u8>) {
    let buf = vec![0u8; corncobs::max_encoded_len(data.len())];
    (data, buf)
}

fn setup_decode_corncobs(data: Vec<u8>) -> (Vec<u8>, Vec<u8>) {
    let mut enc = vec![0u8; corncobs::max_encoded_len(data.len())];
    let n = corncobs::encode_buf(&data, &mut enc);
    enc.truncate(n);
    let dec = vec![0u8; data.len() + 1];
    (enc, dec)
}

fn setup_encode_cobs(data: Vec<u8>) -> (Vec<u8>, Vec<u8>) {
    let buf = vec![0u8; cobs::max_encoding_length(data.len())];
    (data, buf)
}

fn setup_decode_cobs(data: Vec<u8>) -> (Vec<u8>, Vec<u8>) {
    let mut enc = vec![0u8; cobs::max_encoding_length(data.len())];
    let n = cobs::encode(&data, &mut enc);
    enc.truncate(n);
    let dec = vec![0u8; data.len() + 1];
    (enc, dec)
}

// ── Encode benchmarks ──────────────────────────────────────────────

#[library_benchmark]
#[bench::zeros_0(setup_encode(zeros(0)))]
#[bench::zeros_1(setup_encode(zeros(1)))]
#[bench::zeros_10(setup_encode(zeros(10)))]
#[bench::zeros_64(setup_encode(zeros(64)))]
#[bench::zeros_254(setup_encode(zeros(254)))]
#[bench::zeros_255(setup_encode(zeros(255)))]
#[bench::zeros_256(setup_encode(zeros(256)))]
#[bench::zeros_1024(setup_encode(zeros(1024)))]
#[bench::zeros_4096(setup_encode(zeros(4096)))]
#[bench::nonzero_0(setup_encode(nonzero(0)))]
#[bench::nonzero_1(setup_encode(nonzero(1)))]
#[bench::nonzero_10(setup_encode(nonzero(10)))]
#[bench::nonzero_64(setup_encode(nonzero(64)))]
#[bench::nonzero_254(setup_encode(nonzero(254)))]
#[bench::nonzero_255(setup_encode(nonzero(255)))]
#[bench::nonzero_256(setup_encode(nonzero(256)))]
#[bench::nonzero_1024(setup_encode(nonzero(1024)))]
#[bench::nonzero_4096(setup_encode(nonzero(4096)))]
#[bench::mixed_0(setup_encode(mixed(0)))]
#[bench::mixed_1(setup_encode(mixed(1)))]
#[bench::mixed_10(setup_encode(mixed(10)))]
#[bench::mixed_64(setup_encode(mixed(64)))]
#[bench::mixed_254(setup_encode(mixed(254)))]
#[bench::mixed_255(setup_encode(mixed(255)))]
#[bench::mixed_256(setup_encode(mixed(256)))]
#[bench::mixed_1024(setup_encode(mixed(1024)))]
#[bench::mixed_4096(setup_encode(mixed(4096)))]
fn encode_ucobs((data, mut buf): (Vec<u8>, Vec<u8>)) {
    black_box(ucobs::encode(black_box(&data), &mut buf));
}

#[library_benchmark]
#[bench::zeros_0(setup_encode_corncobs(zeros(0)))]
#[bench::zeros_1(setup_encode_corncobs(zeros(1)))]
#[bench::zeros_10(setup_encode_corncobs(zeros(10)))]
#[bench::zeros_64(setup_encode_corncobs(zeros(64)))]
#[bench::zeros_254(setup_encode_corncobs(zeros(254)))]
#[bench::zeros_255(setup_encode_corncobs(zeros(255)))]
#[bench::zeros_256(setup_encode_corncobs(zeros(256)))]
#[bench::zeros_1024(setup_encode_corncobs(zeros(1024)))]
#[bench::zeros_4096(setup_encode_corncobs(zeros(4096)))]
#[bench::nonzero_0(setup_encode_corncobs(nonzero(0)))]
#[bench::nonzero_1(setup_encode_corncobs(nonzero(1)))]
#[bench::nonzero_10(setup_encode_corncobs(nonzero(10)))]
#[bench::nonzero_64(setup_encode_corncobs(nonzero(64)))]
#[bench::nonzero_254(setup_encode_corncobs(nonzero(254)))]
#[bench::nonzero_255(setup_encode_corncobs(nonzero(255)))]
#[bench::nonzero_256(setup_encode_corncobs(nonzero(256)))]
#[bench::nonzero_1024(setup_encode_corncobs(nonzero(1024)))]
#[bench::nonzero_4096(setup_encode_corncobs(nonzero(4096)))]
#[bench::mixed_0(setup_encode_corncobs(mixed(0)))]
#[bench::mixed_1(setup_encode_corncobs(mixed(1)))]
#[bench::mixed_10(setup_encode_corncobs(mixed(10)))]
#[bench::mixed_64(setup_encode_corncobs(mixed(64)))]
#[bench::mixed_254(setup_encode_corncobs(mixed(254)))]
#[bench::mixed_255(setup_encode_corncobs(mixed(255)))]
#[bench::mixed_256(setup_encode_corncobs(mixed(256)))]
#[bench::mixed_1024(setup_encode_corncobs(mixed(1024)))]
#[bench::mixed_4096(setup_encode_corncobs(mixed(4096)))]
fn encode_corncobs((data, mut buf): (Vec<u8>, Vec<u8>)) {
    black_box(corncobs::encode_buf(black_box(&data), &mut buf));
}

#[library_benchmark]
#[bench::zeros_0(setup_encode_cobs(zeros(0)))]
#[bench::zeros_1(setup_encode_cobs(zeros(1)))]
#[bench::zeros_10(setup_encode_cobs(zeros(10)))]
#[bench::zeros_64(setup_encode_cobs(zeros(64)))]
#[bench::zeros_254(setup_encode_cobs(zeros(254)))]
#[bench::zeros_255(setup_encode_cobs(zeros(255)))]
#[bench::zeros_256(setup_encode_cobs(zeros(256)))]
#[bench::zeros_1024(setup_encode_cobs(zeros(1024)))]
#[bench::zeros_4096(setup_encode_cobs(zeros(4096)))]
#[bench::nonzero_0(setup_encode_cobs(nonzero(0)))]
#[bench::nonzero_1(setup_encode_cobs(nonzero(1)))]
#[bench::nonzero_10(setup_encode_cobs(nonzero(10)))]
#[bench::nonzero_64(setup_encode_cobs(nonzero(64)))]
#[bench::nonzero_254(setup_encode_cobs(nonzero(254)))]
#[bench::nonzero_255(setup_encode_cobs(nonzero(255)))]
#[bench::nonzero_256(setup_encode_cobs(nonzero(256)))]
#[bench::nonzero_1024(setup_encode_cobs(nonzero(1024)))]
#[bench::nonzero_4096(setup_encode_cobs(nonzero(4096)))]
#[bench::mixed_0(setup_encode_cobs(mixed(0)))]
#[bench::mixed_1(setup_encode_cobs(mixed(1)))]
#[bench::mixed_10(setup_encode_cobs(mixed(10)))]
#[bench::mixed_64(setup_encode_cobs(mixed(64)))]
#[bench::mixed_254(setup_encode_cobs(mixed(254)))]
#[bench::mixed_255(setup_encode_cobs(mixed(255)))]
#[bench::mixed_256(setup_encode_cobs(mixed(256)))]
#[bench::mixed_1024(setup_encode_cobs(mixed(1024)))]
#[bench::mixed_4096(setup_encode_cobs(mixed(4096)))]
fn encode_cobs((data, mut buf): (Vec<u8>, Vec<u8>)) {
    black_box(cobs::encode(black_box(&data), &mut buf));
}

// ── Decode benchmarks ──────────────────────────────────────────────

#[library_benchmark]
#[bench::zeros_0(setup_decode_ucobs(zeros(0)))]
#[bench::zeros_1(setup_decode_ucobs(zeros(1)))]
#[bench::zeros_10(setup_decode_ucobs(zeros(10)))]
#[bench::zeros_64(setup_decode_ucobs(zeros(64)))]
#[bench::zeros_254(setup_decode_ucobs(zeros(254)))]
#[bench::zeros_255(setup_decode_ucobs(zeros(255)))]
#[bench::zeros_256(setup_decode_ucobs(zeros(256)))]
#[bench::zeros_1024(setup_decode_ucobs(zeros(1024)))]
#[bench::zeros_4096(setup_decode_ucobs(zeros(4096)))]
#[bench::nonzero_0(setup_decode_ucobs(nonzero(0)))]
#[bench::nonzero_1(setup_decode_ucobs(nonzero(1)))]
#[bench::nonzero_10(setup_decode_ucobs(nonzero(10)))]
#[bench::nonzero_64(setup_decode_ucobs(nonzero(64)))]
#[bench::nonzero_254(setup_decode_ucobs(nonzero(254)))]
#[bench::nonzero_255(setup_decode_ucobs(nonzero(255)))]
#[bench::nonzero_256(setup_decode_ucobs(nonzero(256)))]
#[bench::nonzero_1024(setup_decode_ucobs(nonzero(1024)))]
#[bench::nonzero_4096(setup_decode_ucobs(nonzero(4096)))]
#[bench::mixed_0(setup_decode_ucobs(mixed(0)))]
#[bench::mixed_1(setup_decode_ucobs(mixed(1)))]
#[bench::mixed_10(setup_decode_ucobs(mixed(10)))]
#[bench::mixed_64(setup_decode_ucobs(mixed(64)))]
#[bench::mixed_254(setup_decode_ucobs(mixed(254)))]
#[bench::mixed_255(setup_decode_ucobs(mixed(255)))]
#[bench::mixed_256(setup_decode_ucobs(mixed(256)))]
#[bench::mixed_1024(setup_decode_ucobs(mixed(1024)))]
#[bench::mixed_4096(setup_decode_ucobs(mixed(4096)))]
fn decode_ucobs((enc, mut buf): (Vec<u8>, Vec<u8>)) {
    black_box(ucobs::decode(black_box(&enc), &mut buf));
}

#[library_benchmark]
#[bench::zeros_0(setup_decode_corncobs(zeros(0)))]
#[bench::zeros_1(setup_decode_corncobs(zeros(1)))]
#[bench::zeros_10(setup_decode_corncobs(zeros(10)))]
#[bench::zeros_64(setup_decode_corncobs(zeros(64)))]
#[bench::zeros_254(setup_decode_corncobs(zeros(254)))]
#[bench::zeros_255(setup_decode_corncobs(zeros(255)))]
#[bench::zeros_256(setup_decode_corncobs(zeros(256)))]
#[bench::zeros_1024(setup_decode_corncobs(zeros(1024)))]
#[bench::zeros_4096(setup_decode_corncobs(zeros(4096)))]
#[bench::nonzero_0(setup_decode_corncobs(nonzero(0)))]
#[bench::nonzero_1(setup_decode_corncobs(nonzero(1)))]
#[bench::nonzero_10(setup_decode_corncobs(nonzero(10)))]
#[bench::nonzero_64(setup_decode_corncobs(nonzero(64)))]
#[bench::nonzero_254(setup_decode_corncobs(nonzero(254)))]
#[bench::nonzero_255(setup_decode_corncobs(nonzero(255)))]
#[bench::nonzero_256(setup_decode_corncobs(nonzero(256)))]
#[bench::nonzero_1024(setup_decode_corncobs(nonzero(1024)))]
#[bench::nonzero_4096(setup_decode_corncobs(nonzero(4096)))]
#[bench::mixed_0(setup_decode_corncobs(mixed(0)))]
#[bench::mixed_1(setup_decode_corncobs(mixed(1)))]
#[bench::mixed_10(setup_decode_corncobs(mixed(10)))]
#[bench::mixed_64(setup_decode_corncobs(mixed(64)))]
#[bench::mixed_254(setup_decode_corncobs(mixed(254)))]
#[bench::mixed_255(setup_decode_corncobs(mixed(255)))]
#[bench::mixed_256(setup_decode_corncobs(mixed(256)))]
#[bench::mixed_1024(setup_decode_corncobs(mixed(1024)))]
#[bench::mixed_4096(setup_decode_corncobs(mixed(4096)))]
fn decode_corncobs((enc, mut buf): (Vec<u8>, Vec<u8>)) {
    let _ = black_box(corncobs::decode_buf(black_box(&enc), &mut buf));
}

#[library_benchmark]
#[bench::zeros_0(setup_decode_cobs(zeros(0)))]
#[bench::zeros_1(setup_decode_cobs(zeros(1)))]
#[bench::zeros_10(setup_decode_cobs(zeros(10)))]
#[bench::zeros_64(setup_decode_cobs(zeros(64)))]
#[bench::zeros_254(setup_decode_cobs(zeros(254)))]
#[bench::zeros_255(setup_decode_cobs(zeros(255)))]
#[bench::zeros_256(setup_decode_cobs(zeros(256)))]
#[bench::zeros_1024(setup_decode_cobs(zeros(1024)))]
#[bench::zeros_4096(setup_decode_cobs(zeros(4096)))]
#[bench::nonzero_0(setup_decode_cobs(nonzero(0)))]
#[bench::nonzero_1(setup_decode_cobs(nonzero(1)))]
#[bench::nonzero_10(setup_decode_cobs(nonzero(10)))]
#[bench::nonzero_64(setup_decode_cobs(nonzero(64)))]
#[bench::nonzero_254(setup_decode_cobs(nonzero(254)))]
#[bench::nonzero_255(setup_decode_cobs(nonzero(255)))]
#[bench::nonzero_256(setup_decode_cobs(nonzero(256)))]
#[bench::nonzero_1024(setup_decode_cobs(nonzero(1024)))]
#[bench::nonzero_4096(setup_decode_cobs(nonzero(4096)))]
#[bench::mixed_0(setup_decode_cobs(mixed(0)))]
#[bench::mixed_1(setup_decode_cobs(mixed(1)))]
#[bench::mixed_10(setup_decode_cobs(mixed(10)))]
#[bench::mixed_64(setup_decode_cobs(mixed(64)))]
#[bench::mixed_254(setup_decode_cobs(mixed(254)))]
#[bench::mixed_255(setup_decode_cobs(mixed(255)))]
#[bench::mixed_256(setup_decode_cobs(mixed(256)))]
#[bench::mixed_1024(setup_decode_cobs(mixed(1024)))]
#[bench::mixed_4096(setup_decode_cobs(mixed(4096)))]
fn decode_cobs((enc, mut buf): (Vec<u8>, Vec<u8>)) {
    black_box(
        cobs::decode(black_box(&enc), &mut buf)
            .map(|r| r.parsed_size())
            .unwrap_or(0),
    );
}

// ── Groups ─────────────────────────────────────────────────────────

library_benchmark_group!(
    name = encode;
    benchmarks = encode_ucobs, encode_corncobs, encode_cobs
);

library_benchmark_group!(
    name = decode;
    benchmarks = decode_ucobs, decode_corncobs, decode_cobs
);

main!(library_benchmark_groups = encode, decode);
