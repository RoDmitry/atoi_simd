use atoi_simd::{parse, parse_until_invalid};
use criterion::{
    criterion_group, criterion_main, measurement::WallTime, BenchmarkGroup, BenchmarkId, Criterion,
};

fn bench_64(bench_group: &mut BenchmarkGroup<WallTime>, str: &str) {
    let str_neg = "-".to_owned() + str;

    bench_group.bench_with_input(BenchmarkId::new("u64", str.len()), str, |b, val| {
        b.iter(|| parse::<u64>(val.as_bytes()).unwrap())
    });
    if str.len() < 20 {
        bench_group.bench_with_input(BenchmarkId::new("i64", str.len()), str, |b, val| {
            b.iter(|| parse::<i64>(val.as_bytes()).unwrap())
        });
        bench_group.bench_with_input(
            BenchmarkId::new("neg i64", str.len()),
            &str_neg,
            |b, val| b.iter(|| parse::<i64>(val.as_bytes()).unwrap()),
        );
    }

    bench_group.bench_with_input(BenchmarkId::new("std u64", str.len()), str, |b, val| {
        b.iter(|| val.parse::<u64>().unwrap())
    });
    if str.len() < 20 {
        bench_group.bench_with_input(BenchmarkId::new("std i64", str.len()), str, |b, val| {
            b.iter(|| val.parse::<i64>().unwrap())
        });
        bench_group.bench_with_input(
            BenchmarkId::new("std neg i64", str.len()),
            &str_neg,
            |b, val| b.iter(|| val.parse::<i64>().unwrap()),
        );
    }
}

fn bench_128(bench_group: &mut BenchmarkGroup<WallTime>, str: &str) {
    let str_neg = "-".to_owned() + str;

    bench_group.bench_with_input(BenchmarkId::new("u128", str.len()), str, |b, val| {
        b.iter(|| parse::<u128>(val.as_bytes()).unwrap())
    });
    bench_group.bench_with_input(BenchmarkId::new("i128", str.len()), str, |b, val| {
        b.iter(|| parse::<i128>(val.as_bytes()).unwrap())
    });
    bench_group.bench_with_input(
        BenchmarkId::new("neg i128", str.len()),
        &str_neg,
        |b, val| b.iter(|| parse::<i128>(val.as_bytes()).unwrap()),
    );

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
}

fn bench_until_invalid_128(bench_group: &mut BenchmarkGroup<WallTime>, str: &str) {
    let len = str.len();
    let str = str.to_owned() + "s1111111111111111111111111111111111111111111111111111111111111";
    let str_neg = "-".to_owned() + &str;

    bench_group.bench_with_input(
        BenchmarkId::new("u128 until_invalid", len),
        &str,
        |b, val| b.iter(|| parse_until_invalid::<u128>(val.as_bytes()).unwrap()),
    );
    bench_group.bench_with_input(BenchmarkId::new("i128", len), &str, |b, val| {
        b.iter(|| parse_until_invalid::<i128>(val.as_bytes()).unwrap())
    });
    bench_group.bench_with_input(BenchmarkId::new("neg i128", len), &str_neg, |b, val| {
        b.iter(|| parse_until_invalid::<i128>(val.as_bytes()).unwrap())
    });
}

fn benchmark_group_max_20(
    bench_group: &mut BenchmarkGroup<WallTime>,
    func: fn(bench_group: &mut BenchmarkGroup<WallTime>, str: &str),
) {
    let mut str = String::new();
    for i in '1'..='5' {
        str.push(i);
        func(bench_group, &str);
    }

    str = "123456".to_owned();
    for i in '7'..='9' {
        str.push(i);
        func(bench_group, &str);
    }

    str = "123456789012".to_owned();
    func(bench_group, &str);

    str = "12345678901234".to_owned();
    for i in '5'..='7' {
        str.push(i);
        func(bench_group, &str);
    }

    str = "1234567890123456789".to_owned();
    func(bench_group, &str);

    str = "12345678901234567890".to_owned();
    func(bench_group, &str);
}

fn benchmark(c: &mut Criterion) {
    {
        let mut bench_group = c.benchmark_group("benchmark 64");
        benchmark_group_max_20(&mut bench_group, bench_64);
        bench_group.finish();
    }

    let mut bench_group = c.benchmark_group("benchmark 128");

    benchmark_group_max_20(&mut bench_group, bench_until_invalid_128);

    benchmark_group_max_20(&mut bench_group, bench_128);

    let mut str = "123456789012345678901".to_owned();
    bench_128(&mut bench_group, &str);

    str = "1234567890123456789012345".to_owned();
    bench_128(&mut bench_group, &str);

    str = "123456789012345678901234567890".to_owned();
    bench_128(&mut bench_group, &str);

    str = "1234567890123456789012345678901".to_owned();
    bench_128(&mut bench_group, &str);

    str = "12345678901234567890123456789012".to_owned();
    bench_128(&mut bench_group, &str);

    bench_group.finish();
}

criterion_group!(benches, benchmark);
criterion_main!(benches);
