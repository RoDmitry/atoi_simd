use atoi_simd::{parse, parse_i64};
use criterion::{black_box, criterion_group, criterion_main, Criterion};

fn benchmark_long_std(c: &mut Criterion) {
    c.bench_function(
        "long string std u64                  1234567890123456",
        |b| b.iter(|| black_box("1234567890123456").parse::<u64>().unwrap()),
    );
}

fn benchmark_long_neg_std(c: &mut Criterion) {
    c.bench_function(
        "long string negative std i64         -1234567890123456",
        |b| b.iter(|| black_box("-1234567890123456").parse::<i64>().unwrap()),
    );
}

fn benchmark_long_u64(c: &mut Criterion) {
    c.bench_function(
        "long string u64                      1234567890123456",
        |b| b.iter(|| parse(black_box("1234567890123456")).unwrap()),
    );
}

fn benchmark_long_i64(c: &mut Criterion) {
    c.bench_function(
        "long string i64                      1234567890123456",
        |b| b.iter(|| parse_i64(black_box("1234567890123456")).unwrap()),
    );
}

fn benchmark_long_neg_i64(c: &mut Criterion) {
    c.bench_function(
        "long string negative i64             -1234567890123456",
        |b| b.iter(|| parse_i64(black_box("-123456789012345")).unwrap()),
    );
}

fn benchmark_short_std(c: &mut Criterion) {
    c.bench_function("short string std u64                       1", |b| {
        b.iter(|| black_box("1").parse::<u64>().unwrap())
    });
}

fn benchmark_short_neg_std(c: &mut Criterion) {
    c.bench_function("short string negative std i64              -1", |b| {
        b.iter(|| black_box("-1").parse::<i64>().unwrap())
    });
}

fn benchmark_short_u64(c: &mut Criterion) {
    c.bench_function("short string u64                           1", |b| {
        b.iter(|| parse(black_box("1")).unwrap())
    });
}

fn benchmark_short_i64(c: &mut Criterion) {
    c.bench_function("short string i64                           1", |b| {
        b.iter(|| parse_i64(black_box("1")).unwrap())
    });
}

fn benchmark_short_neg_i64(c: &mut Criterion) {
    c.bench_function("short string negative i64                  -1", |b| {
        b.iter(|| parse_i64(black_box("-1")).unwrap())
    });
}

fn benchmark_long15_std(c: &mut Criterion) {
    c.bench_function(
        "15 chars string std u64              123456789012345",
        |b| b.iter(|| black_box("123456789012345").parse::<u64>().unwrap()),
    );
}

fn benchmark_long15_neg_std(c: &mut Criterion) {
    c.bench_function(
        "15 chars string negative std i64     -123456789012345",
        |b| b.iter(|| black_box("-123456789012345").parse::<i64>().unwrap()),
    );
}

fn benchmark_long15_u64(c: &mut Criterion) {
    c.bench_function(
        "15 chars string u64                  123456789012345",
        |b| b.iter(|| parse(black_box("123456789012345")).unwrap()),
    );
}

fn benchmark_long15_i64(c: &mut Criterion) {
    c.bench_function(
        "15 chars string i64                  123456789012345",
        |b| b.iter(|| parse_i64(black_box("123456789012345")).unwrap()),
    );
}

fn benchmark_long15_neg_i64(c: &mut Criterion) {
    c.bench_function(
        "15 chars string negative i64         -123456789012345",
        |b| b.iter(|| parse_i64(black_box("-123456789012345")).unwrap()),
    );
}

criterion_group!(
    benches,
    benchmark_long_std,
    benchmark_long_neg_std,
    benchmark_long_u64,
    benchmark_long_i64,
    benchmark_long_neg_i64,
    benchmark_short_std,
    benchmark_short_neg_std,
    benchmark_short_u64,
    benchmark_short_i64,
    benchmark_short_neg_i64,
    benchmark_long15_std,
    benchmark_long15_neg_std,
    benchmark_long15_u64,
    benchmark_long15_i64,
    benchmark_long15_neg_i64
);
criterion_main!(benches);
