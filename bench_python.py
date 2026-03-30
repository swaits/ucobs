#!/usr/bin/env python3
"""Benchmark Python cobs encode/decode vs ucobs (reference timing only).

Requires: pip install cobs
"""

import time


def payload_zeros(n: int) -> bytes:
    return b"\x00" * n


def payload_nonzero(n: int) -> bytes:
    return bytes((i % 255 + 1) for i in range(n))


def payload_mixed(n: int) -> bytes:
    return bytes(i % 256 for i in range(n))


def bench(func, data, iterations=10_000):
    start = time.perf_counter()
    for _ in range(iterations):
        func(data)
    elapsed = time.perf_counter() - start
    return elapsed


def main():
    try:
        from cobs import cobs as cobs_mod
    except ImportError:
        print("ERROR: python cobs package not installed. Run: pip install cobs")
        return

    sizes = [0, 1, 10, 64, 254, 255, 256, 1024, 4096]
    payloads = [
        ("zeros", payload_zeros),
        ("nonzero", payload_nonzero),
        ("mixed", payload_mixed),
    ]
    iterations = 10_000

    print(f"{'operation':<30} {'size':>6} {'iters':>7} {'total_ms':>10} {'per_op_us':>10} {'MB/s':>10}")
    print("-" * 80)

    for name, make in payloads:
        for size in sizes:
            data = make(size)

            # Encode
            elapsed = bench(cobs_mod.encode, data, iterations)
            per_op_us = (elapsed / iterations) * 1_000_000
            mbps = (size * iterations / elapsed / 1_000_000) if elapsed > 0 and size > 0 else 0
            print(f"{'python-cobs encode/' + name:<30} {size:>6} {iterations:>7} {elapsed*1000:>10.2f} {per_op_us:>10.3f} {mbps:>10.1f}")

            # Decode
            encoded = cobs_mod.encode(data)
            elapsed = bench(cobs_mod.decode, encoded, iterations)
            per_op_us = (elapsed / iterations) * 1_000_000
            mbps = (size * iterations / elapsed / 1_000_000) if elapsed > 0 and size > 0 else 0
            print(f"{'python-cobs decode/' + name:<30} {size:>6} {iterations:>7} {elapsed*1000:>10.2f} {per_op_us:>10.3f} {mbps:>10.1f}")


if __name__ == "__main__":
    main()
