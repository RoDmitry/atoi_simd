use atoi_simd::{parse, parse_i64};
use criterion::{black_box, criterion_group, criterion_main, Criterion};

const LONG_STRING: &str = "1234567890123456";
const LONG_STRING_NEG: &str = "-1234567890123456";
const SHORT_STRING: &str = "1";
const SHORT_STRING_NEG: &str = "-1";
const FIFTEEN_STRING: &str = "123456789012345";
const FIFTEEN_STRING_NEG: &str = "-123456789012345";

fn benchmark_long_std(c: &mut Criterion) {
    c.bench_function(
        &("long string std u64                  ".to_owned() + LONG_STRING),
        |b| b.iter(|| black_box(LONG_STRING).parse::<u64>().unwrap()),
    );
}

fn benchmark_long_neg_std(c: &mut Criterion) {
    c.bench_function(
        &("long string negative std i64         ".to_owned() + LONG_STRING_NEG),
        |b| b.iter(|| black_box(LONG_STRING_NEG).parse::<i64>().unwrap()),
    );
}

fn benchmark_long_u64(c: &mut Criterion) {
    c.bench_function(
        &("long string u64                      ".to_owned() + LONG_STRING),
        |b| b.iter(|| parse(black_box(LONG_STRING)).unwrap()),
    );
}

fn benchmark_long_i64(c: &mut Criterion) {
    c.bench_function(
        &("long string i64                      ".to_owned() + LONG_STRING),
        |b| b.iter(|| parse_i64(black_box(LONG_STRING)).unwrap()),
    );
}

fn benchmark_long_neg_i64(c: &mut Criterion) {
    c.bench_function(
        &("long string negative i64             ".to_owned() + LONG_STRING_NEG),
        |b| b.iter(|| parse_i64(black_box(LONG_STRING_NEG)).unwrap()),
    );
}

fn benchmark_short_std(c: &mut Criterion) {
    c.bench_function(
        &("short string std u64                       ".to_owned() + SHORT_STRING),
        |b| b.iter(|| black_box(SHORT_STRING).parse::<u64>().unwrap()),
    );
}

fn benchmark_short_neg_std(c: &mut Criterion) {
    c.bench_function(
        &("short string negative std i64              ".to_owned() + SHORT_STRING_NEG),
        |b| b.iter(|| black_box(SHORT_STRING_NEG).parse::<i64>().unwrap()),
    );
}

fn benchmark_short_u64(c: &mut Criterion) {
    c.bench_function(
        &("short string u64                           ".to_owned() + SHORT_STRING),
        |b| b.iter(|| parse(black_box(SHORT_STRING)).unwrap()),
    );
}

fn benchmark_short_i64(c: &mut Criterion) {
    c.bench_function(
        &("short string i64                           ".to_owned() + SHORT_STRING),
        |b| b.iter(|| parse_i64(black_box(SHORT_STRING)).unwrap()),
    );
}

fn benchmark_short_neg_i64(c: &mut Criterion) {
    c.bench_function(
        &("short string negative i64                  ".to_owned() + SHORT_STRING_NEG),
        |b| b.iter(|| parse_i64(black_box(SHORT_STRING_NEG)).unwrap()),
    );
}

fn benchmark_long15_std(c: &mut Criterion) {
    c.bench_function(
        &("15 chars string std u64              ".to_owned() + FIFTEEN_STRING),
        |b| b.iter(|| black_box(FIFTEEN_STRING).parse::<u64>().unwrap()),
    );
}

fn benchmark_long15_neg_std(c: &mut Criterion) {
    c.bench_function(
        &("15 chars string negative std i64     ".to_owned() + FIFTEEN_STRING_NEG),
        |b| b.iter(|| black_box(FIFTEEN_STRING_NEG).parse::<i64>().unwrap()),
    );
}

fn benchmark_long15_u64(c: &mut Criterion) {
    c.bench_function(
        &("15 chars string u64                  ".to_owned() + FIFTEEN_STRING),
        |b| b.iter(|| parse(black_box(FIFTEEN_STRING)).unwrap()),
    );
}

fn benchmark_long15_i64(c: &mut Criterion) {
    c.bench_function(
        &("15 chars string i64                  ".to_owned() + FIFTEEN_STRING),
        |b| b.iter(|| parse_i64(black_box(FIFTEEN_STRING)).unwrap()),
    );
}

fn benchmark_long15_neg_i64(c: &mut Criterion) {
    c.bench_function(
        &("15 chars string negative i64         ".to_owned() + FIFTEEN_STRING_NEG),
        |b| b.iter(|| parse_i64(black_box(FIFTEEN_STRING_NEG)).unwrap()),
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
