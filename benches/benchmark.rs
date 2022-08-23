use atoi_simd::{parse, parse_i128, parse_i64, parse_u128};
use criterion::{
    criterion_group, criterion_main, measurement::WallTime, BenchmarkGroup, BenchmarkId, Criterion,
};

fn bench_64(bench_group: &mut BenchmarkGroup<WallTime>, str: &str) {
    let str_neg = "-".to_owned() + str;

    bench_group.bench_with_input(BenchmarkId::new("std u64", str.len()), str, |b, val| {
        b.iter(|| val.parse::<u64>().unwrap())
    });
    bench_group.bench_with_input(BenchmarkId::new("std i64", str.len()), str, |b, val| {
        b.iter(|| val.parse::<i64>().unwrap())
    });
    bench_group.bench_with_input(
        BenchmarkId::new("std neg i64", str.len()),
        &str_neg,
        |b, val| b.iter(|| val.parse::<i64>().unwrap()),
    );

    bench_group.bench_with_input(BenchmarkId::new("u64", str.len()), str, |b, val| {
        b.iter(|| parse(val).unwrap())
    });
    bench_group.bench_with_input(BenchmarkId::new("i64", str.len()), str, |b, val| {
        b.iter(|| parse_i64(val).unwrap())
    });
    bench_group.bench_with_input(
        BenchmarkId::new("neg i64", str.len()),
        &str_neg,
        |b, val| b.iter(|| parse_i64(val).unwrap()),
    );
}

fn bench_128(bench_group: &mut BenchmarkGroup<WallTime>, str: &str) {
    let str_neg = "-".to_owned() + str;

    bench_group.bench_with_input(BenchmarkId::new("std u128", str.len()), str, |b, val| {
        b.iter(|| val.parse::<u128>().unwrap())
    });
    bench_group.bench_with_input(BenchmarkId::new("std i128", str.len()), str, |b, val| {
        b.iter(|| val.parse::<i128>().unwrap())
    });
    bench_group.bench_with_input(
        BenchmarkId::new("std neg i128", str.len()),
        &str_neg,
        |b, val| b.iter(|| val.parse::<i128>().unwrap()),
    );

    bench_group.bench_with_input(BenchmarkId::new("u128", str.len()), str, |b, val| {
        b.iter(|| parse_u128(val).unwrap())
    });
    bench_group.bench_with_input(BenchmarkId::new("i128", str.len()), str, |b, val| {
        b.iter(|| parse_i128(val).unwrap())
    });
    bench_group.bench_with_input(
        BenchmarkId::new("neg i128", str.len()),
        &str_neg,
        |b, val| b.iter(|| parse_i128(val).unwrap()),
    );
}

fn benchmark(c: &mut Criterion) {
    let mut bench_group = c.benchmark_group("benchmark");
    let mut str = String::new();
    for i in '1'..='3' {
        str.push(i);
        bench_64(&mut bench_group, &str);
        bench_128(&mut bench_group, &str);
    }

    str = "12345".to_owned();
    bench_64(&mut bench_group, &str);
    bench_128(&mut bench_group, &str);

    str = "1234567890".to_owned();
    bench_64(&mut bench_group, &str);
    bench_128(&mut bench_group, &str);

    str = "12345678901234".to_owned();
    for i in '5'..='6' {
        str.push(i);
        bench_64(&mut bench_group, &str);
    }
    bench_128(&mut bench_group, &str);

    str = "12345678901234567".to_owned();
    bench_128(&mut bench_group, &str);

    str = "1234567890123456789012345".to_owned();
    bench_128(&mut bench_group, &str);

    str = "123456789012345678901234567890".to_owned();
    bench_128(&mut bench_group, &str);

    str = "12345678901234567890123456789012".to_owned();
    bench_128(&mut bench_group, &str);

    bench_group.finish();
}

criterion_group!(benches, benchmark);
criterion_main!(benches);
