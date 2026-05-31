//! Performance benchmarks for FastLink

use criterion::{criterion_group, criterion_main, Criterion};

fn crypto_benchmark(c: &mut Criterion) {
    // TODO: Add crypto benchmarks
    c.bench_function("noop", |b| b.iter(|| {}));
}

criterion_group!(benches, crypto_benchmark);
criterion_main!(benches);
