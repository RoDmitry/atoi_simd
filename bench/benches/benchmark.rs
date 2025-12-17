use atoi_simd::{parse, parse_prefix};
use criterion::{
    criterion_group, criterion_main, measurement::WallTime, BenchmarkGroup, BenchmarkId, Criterion,
};

fn bench_32(bench_group: &mut BenchmarkGroup<WallTime>, str: &str) {
    let len = str.len() + 1;
    let strs = [str.to_owned() + "2", str.to_owned() + "1"];
    let str_neg = "-".to_owned() + str;
    let strs_neg = [str_neg.clone() + "2", str_neg.clone() + "1"];

    bench_group.bench_with_input(BenchmarkId::new("parse u32", len), &strs, |b, val| {
        b.iter(|| {
            parse::<u32>(val[0].as_bytes()).unwrap() - parse::<u32>(val[1].as_bytes()).unwrap()
        })
    });
    bench_group.bench_with_input(BenchmarkId::new("parse i32", len), &strs, |b, val| {
        b.iter(|| {
            parse::<i32>(val[0].as_bytes()).unwrap() - parse::<i32>(val[1].as_bytes()).unwrap()
        })
    });
    bench_group.bench_with_input(
        BenchmarkId::new("parse neg i32", len),
        &strs_neg,
        |b, val| {
            b.iter(|| {
                parse::<i32>(val[0].as_bytes()).unwrap() - parse::<i32>(val[1].as_bytes()).unwrap()
            })
        },
    );

    bench_group.bench_with_input(BenchmarkId::new("str u32", len), &strs, |b, val| {
        b.iter(|| val[0].parse::<u32>().unwrap() - val[1].parse::<u32>().unwrap())
    });
    bench_group.bench_with_input(BenchmarkId::new("str i32", len), &strs, |b, val| {
        b.iter(|| val[0].parse::<i32>().unwrap() - val[1].parse::<i32>().unwrap())
    });
    bench_group.bench_with_input(BenchmarkId::new("str neg i32", len), &strs_neg, |b, val| {
        b.iter(|| val[0].parse::<i32>().unwrap() - val[1].parse::<i32>().unwrap())
    });
}

fn bench_64(bench_group: &mut BenchmarkGroup<WallTime>, str: &str) {
    let len = str.len() + 1;
    let strs = [str.to_owned() + "2", str.to_owned() + "1"];
    let str_neg = "-".to_owned() + str;
    let strs_neg = [str_neg.clone() + "2", str_neg.clone() + "1"];

    bench_group.bench_with_input(BenchmarkId::new("parse u64", len), &strs, |b, val| {
        b.iter(|| {
            parse::<u64>(val[0].as_bytes()).unwrap() - parse::<u64>(val[1].as_bytes()).unwrap()
        })
    });
    /* bench_group.bench_with_input(
        BenchmarkId::new("parse_skipped u64", len),
        str,
        |b, val| b.iter(|| parse_skipped::<u64>(val.as_bytes()).unwrap()),
    ); */
    if len < 20 {
        bench_group.bench_with_input(BenchmarkId::new("parse i64", len), &strs, |b, val| {
            b.iter(|| {
                parse::<i64>(val[0].as_bytes()).unwrap() - parse::<i64>(val[1].as_bytes()).unwrap()
            })
        });
        /* bench_group.bench_with_input(
            BenchmarkId::new("parse_skipped i64", len),
            str,
            |b, val| b.iter(|| parse_skipped::<i64>(val.as_bytes()).unwrap()),
        ); */
        bench_group.bench_with_input(
            BenchmarkId::new("parse neg i64", len),
            &strs_neg,
            |b, val| {
                b.iter(|| {
                    parse::<i64>(val[0].as_bytes()).unwrap()
                        - parse::<i64>(val[1].as_bytes()).unwrap()
                })
            },
        );
        /* bench_group.bench_with_input(
            BenchmarkId::new("parse_skipped neg i64", len),
            &strs_neg,
            |b, val| b.iter(|| parse_skipped::<i64>(val.as_bytes()).unwrap()),
        ); */
    }

    bench_group.bench_with_input(BenchmarkId::new("str u64", len), &strs, |b, val| {
        b.iter(|| val[0].parse::<u64>().unwrap() - val[1].parse::<u64>().unwrap())
    });
    if len < 20 {
        bench_group.bench_with_input(BenchmarkId::new("str i64", len), &strs, |b, val| {
            b.iter(|| val[0].parse::<i64>().unwrap() - val[1].parse::<i64>().unwrap())
        });
        bench_group.bench_with_input(BenchmarkId::new("str neg i64", len), &strs_neg, |b, val| {
            b.iter(|| val[0].parse::<i64>().unwrap() - val[1].parse::<i64>().unwrap())
        });
    }

    let zero = "0".to_owned() + str;
    let zero_strs = [zero.clone() + "2", zero.clone() + "1"];
    bench_group.bench_with_input(
        BenchmarkId::new("parse zero u64", zero.len()),
        &zero_strs,
        |b, val| {
            b.iter(|| {
                parse::<u64>(val[0].as_bytes()).unwrap() - parse::<u64>(val[1].as_bytes()).unwrap()
            })
        },
    );

    let zeroes_16 = "0000000000000000".to_owned() + str;
    let zeroes_16_strs = [zeroes_16.clone() + "2", zeroes_16.clone() + "1"];
    bench_group.bench_with_input(
        BenchmarkId::new("parse zeroes_16 u64", zeroes_16.len()),
        &zeroes_16_strs,
        |b, val| {
            b.iter(|| {
                parse::<u64>(val[0].as_bytes()).unwrap() - parse::<u64>(val[1].as_bytes()).unwrap()
            })
        },
    );
    bench_group.bench_with_input(
        BenchmarkId::new("str zeroes_16 u64", zeroes_16.len()),
        &zeroes_16_strs,
        |b, val| b.iter(|| val[0].parse::<u64>().unwrap() - val[1].parse::<u64>().unwrap()),
    );
}

fn bench_128(bench_group: &mut BenchmarkGroup<WallTime>, str: &str) {
    let len = str.len() + 1;
    let strs = [str.to_owned() + "2", str.to_owned() + "1"];
    let str_neg = "-".to_owned() + str;
    let strs_neg = [str_neg.clone() + "2", str_neg.clone() + "1"];

    bench_group.bench_with_input(BenchmarkId::new("parse u128", len), &strs, |b, val| {
        b.iter(|| {
            parse::<u128>(val[0].as_bytes()).unwrap() - parse::<u128>(val[1].as_bytes()).unwrap()
        })
    });
    bench_group.bench_with_input(BenchmarkId::new("parse i128", len), &strs, |b, val| {
        b.iter(|| {
            parse::<i128>(val[0].as_bytes()).unwrap() - parse::<i128>(val[1].as_bytes()).unwrap()
        })
    });
    bench_group.bench_with_input(
        BenchmarkId::new("parse neg i128", len),
        &strs_neg,
        |b, val| {
            b.iter(|| {
                parse::<i128>(val[0].as_bytes()).unwrap()
                    - parse::<i128>(val[1].as_bytes()).unwrap()
            })
        },
    );

    bench_group.bench_with_input(BenchmarkId::new("str u128", len), &strs, |b, val| {
        b.iter(|| val[0].parse::<u128>().unwrap() - val[1].parse::<u128>().unwrap())
    });
    bench_group.bench_with_input(BenchmarkId::new("str i128", len), &strs, |b, val| {
        b.iter(|| val[0].parse::<i128>().unwrap() - val[1].parse::<i128>().unwrap())
    });
    bench_group.bench_with_input(
        BenchmarkId::new("str neg i128", len),
        &strs_neg,
        |b, val| b.iter(|| val[0].parse::<i128>().unwrap() - val[1].parse::<i128>().unwrap()),
    );

    let zero = "0".to_owned() + str;
    let zero_strs = [zero.clone() + "2", zero.clone() + "1"];
    bench_group.bench_with_input(
        BenchmarkId::new("parse zero u128", zero.len()),
        &zero_strs,
        |b, val| {
            b.iter(|| {
                parse::<u128>(val[0].as_bytes()).unwrap()
                    - parse::<u128>(val[1].as_bytes()).unwrap()
            })
        },
    );

    let zeroes_32 = "00000000000000000000000000000000".to_owned() + str;
    let zeroes_32_strs = [zeroes_32.clone() + "2", zeroes_32.clone() + "1"];
    bench_group.bench_with_input(
        BenchmarkId::new("parse zeroes_32 u128", zeroes_32.len()),
        &zeroes_32_strs,
        |b, val| {
            b.iter(|| {
                parse::<u128>(val[0].as_bytes()).unwrap()
                    - parse::<u128>(val[1].as_bytes()).unwrap()
            })
        },
    );
    bench_group.bench_with_input(
        BenchmarkId::new("str zeroes_32 u128", zeroes_32.len()),
        &zeroes_32_strs,
        |b, val| b.iter(|| val[0].parse::<u128>().unwrap() - val[1].parse::<u128>().unwrap()),
    );
}

fn bench_prefix_32(bench_group: &mut BenchmarkGroup<WallTime>, str: &str) {
    let len = str.len() + 1;
    let postfix = "s1111111111111111111";
    let strs = [
        str.to_owned() + "2" + postfix,
        str.to_owned() + "1" + postfix,
    ];
    let str_neg = "-".to_owned() + str;
    let strs_neg = [
        str_neg.clone() + "2" + postfix,
        str_neg.clone() + "1" + postfix,
    ];

    bench_group.bench_with_input(
        BenchmarkId::new("parse_prefix u32", len),
        &strs,
        |b, val| {
            b.iter(|| {
                parse_prefix::<u32>(val[0].as_bytes()).unwrap().0
                    - parse_prefix::<u32>(val[1].as_bytes()).unwrap().0
            })
        },
    );
    bench_group.bench_with_input(
        BenchmarkId::new("parse_prefix i32", len),
        &strs,
        |b, val| {
            b.iter(|| {
                parse_prefix::<i32>(val[0].as_bytes()).unwrap().0
                    - parse_prefix::<i32>(val[1].as_bytes()).unwrap().0
            })
        },
    );
    bench_group.bench_with_input(
        BenchmarkId::new("parse_prefix neg i32", len),
        &strs_neg,
        |b, val| {
            b.iter(|| {
                parse_prefix::<i32>(val[0].as_bytes()).unwrap().0
                    - parse_prefix::<i32>(val[1].as_bytes()).unwrap().0
            })
        },
    );
}

fn bench_prefix_64(bench_group: &mut BenchmarkGroup<WallTime>, str: &str) {
    let len = str.len() + 1;
    let postfix = "s1111111111111111111";
    let strs = [
        str.to_owned() + "2" + postfix,
        str.to_owned() + "1" + postfix,
    ];
    let str_neg = "-".to_owned() + str;
    let strs_neg = [
        str_neg.clone() + "2" + postfix,
        str_neg.clone() + "1" + postfix,
    ];

    bench_group.bench_with_input(
        BenchmarkId::new("parse_prefix u64", len),
        &strs,
        |b, val| {
            b.iter(|| {
                parse_prefix::<u64>(val[0].as_bytes()).unwrap().0
                    - parse_prefix::<u64>(val[1].as_bytes()).unwrap().0
            })
        },
    );
    if len < 20 {
        bench_group.bench_with_input(
            BenchmarkId::new("parse_prefix i64", len),
            &strs,
            |b, val| {
                b.iter(|| {
                    parse_prefix::<i64>(val[0].as_bytes()).unwrap().0
                        - parse_prefix::<i64>(val[1].as_bytes()).unwrap().0
                })
            },
        );
        bench_group.bench_with_input(
            BenchmarkId::new("parse_prefix neg i64", len),
            &strs_neg,
            |b, val| {
                b.iter(|| {
                    parse_prefix::<i64>(val[0].as_bytes()).unwrap().0
                        - parse_prefix::<i64>(val[1].as_bytes()).unwrap().0
                })
            },
        );
    }
}

fn bench_prefix_128(bench_group: &mut BenchmarkGroup<WallTime>, str: &str) {
    let len = str.len() + 1;
    let postfix = "s111111111111111111111111111111111111111";
    let strs = [
        str.to_owned() + "2" + postfix,
        str.to_owned() + "1" + postfix,
    ];
    let str_neg = "-".to_owned() + str;
    let strs_neg = [
        str_neg.clone() + "2" + postfix,
        str_neg.clone() + "1" + postfix,
    ];

    bench_group.bench_with_input(
        BenchmarkId::new("parse_prefix u128", len),
        &strs,
        |b, val| {
            b.iter(|| {
                parse_prefix::<u128>(val[0].as_bytes()).unwrap().0
                    - parse_prefix::<u128>(val[1].as_bytes()).unwrap().0
            })
        },
    );
    bench_group.bench_with_input(
        BenchmarkId::new("parse_prefix i128", len),
        &strs,
        |b, val| {
            b.iter(|| {
                parse_prefix::<i128>(val[0].as_bytes()).unwrap().0
                    - parse_prefix::<i128>(val[1].as_bytes()).unwrap().0
            })
        },
    );
    bench_group.bench_with_input(
        BenchmarkId::new("parse_prefix neg i128", len),
        &strs_neg,
        |b, val| {
            b.iter(|| {
                parse_prefix::<i128>(val[0].as_bytes()).unwrap().0
                    - parse_prefix::<i128>(val[1].as_bytes()).unwrap().0
            })
        },
    );
}

fn benchmark_group_max_10(
    bench_group: &mut BenchmarkGroup<WallTime>,
    func: fn(bench_group: &mut BenchmarkGroup<WallTime>, str: &str),
) {
    let mut str = String::new();
    func(bench_group, &str);

    for i in '1'..='9' {
        str.push(i);
        func(bench_group, &str);
    }
}

fn benchmark_group_max_20(
    bench_group: &mut BenchmarkGroup<WallTime>,
    func: fn(bench_group: &mut BenchmarkGroup<WallTime>, str: &str),
) {
    let mut str = String::new();
    func(bench_group, &str);

    for i in '1'..='4' {
        str.push(i);
        func(bench_group, &str);
    }

    if cfg!(feature = "full") {
        for i in '5'..='9' {
            str.push(i);
            func(bench_group, &str);
        }

        for i in '0'..='6' {
            str.push(i);
            func(bench_group, &str);
        }
    } else {
        str = "12345678901234".to_owned();
        for i in '5'..='6' {
            str.push(i);
            func(bench_group, &str);
        }
    }

    str = "123456789012345678".to_owned();
    func(bench_group, &str);

    str = "1234567890123456789".to_owned();
    func(bench_group, &str);
}

fn benchmark(c: &mut Criterion) {
    {
        let mut bench_group = c.benchmark_group("benchmark 32");
        benchmark_group_max_10(&mut bench_group, bench_32);
        benchmark_group_max_10(&mut bench_group, bench_prefix_32);
        bench_group.finish();
    }
    {
        let mut bench_group = c.benchmark_group("benchmark 64");
        benchmark_group_max_20(&mut bench_group, bench_64);
        benchmark_group_max_20(&mut bench_group, bench_prefix_64);
        bench_group.finish();
    }

    let mut bench_group = c.benchmark_group("benchmark 128");

    benchmark_group_max_20(&mut bench_group, bench_128);
    benchmark_group_max_20(&mut bench_group, bench_prefix_128);

    let mut str = "12345678901234567890".to_owned();
    bench_128(&mut bench_group, &str);
    bench_prefix_128(&mut bench_group, &str);

    str = "123456789012345678901234".to_owned();
    bench_128(&mut bench_group, &str);
    bench_prefix_128(&mut bench_group, &str);

    str = "12345678901234567890123456789".to_owned();
    bench_128(&mut bench_group, &str);
    bench_prefix_128(&mut bench_group, &str);

    str = "123456789012345678901234567890".to_owned();
    bench_128(&mut bench_group, &str);
    bench_prefix_128(&mut bench_group, &str);

    str = "1234567890123456789012345678901".to_owned();
    bench_128(&mut bench_group, &str);
    bench_prefix_128(&mut bench_group, &str);

    str = "12345678901234567890123456789012".to_owned();
    bench_128(&mut bench_group, &str);
    bench_prefix_128(&mut bench_group, &str);

    str = "123456789012345678901234567890123".to_owned();
    bench_128(&mut bench_group, &str);
    bench_prefix_128(&mut bench_group, &str);

    str = "12345678901234567890123456789012345".to_owned();
    bench_128(&mut bench_group, &str);
    bench_prefix_128(&mut bench_group, &str);

    str = "1234567890123456789012345678901234567".to_owned();
    bench_128(&mut bench_group, &str);
    bench_prefix_128(&mut bench_group, &str);

    str = "12345678901234567890123456789012345678".to_owned();
    bench_128(&mut bench_group, &str);
    bench_prefix_128(&mut bench_group, &str);

    bench_group.finish();
}

criterion_group!(benches, benchmark);
criterion_main!(benches);
