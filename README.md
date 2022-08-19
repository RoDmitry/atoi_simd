# Rust fast `&str` to `i64` parser (x86_64 SIMD, SSE4.1)

[![Crate](https://img.shields.io/crates/v/atoi_simd.svg)](https://crates.io/crates/atoi_simd)
[![API](https://docs.rs/atoi_simd/badge.svg)](https://docs.rs/atoi_simd)

Modified [this](https://github.com/pickfire/parseint) version (from [article](https://rust-malaysia.github.io/code/2020/07/11/faster-integer-parsing.html)) to support various strings of different lengths and negative values.

Must be used when you are certain that the string contains only digits.
If you pass not only digits, it will give you the wrong output (not error).

Max string length is 16 numbers (17 with sign).

It needs the `target-feature` or `target-cpu` flags for it to build with optimized performance. By default the `target-feature` is set in ./.cargo/config.toml

Also you can use one of the following environment variables:

-   `RUSTFLAGS="-C target-feature=+sse2,+sse3,+sse4.1,+ssse3"`

-   `RUSTFLAGS="-C target-cpu=native"`

For Windows PowerShell you can set it with `$Env:RUSTFLAGS='-C target-feature=+sse2,+sse3,+sse4.1,+ssse3'`

## Examples

```
assert_eq!(atoi_simd::parse("0").unwrap(), 0_u64);
assert_eq!(atoi_simd::parse("1234").unwrap(), 1234_u64);

assert_eq!(atoi_simd::parse_i64("2345").unwrap(), 2345_i64);
assert_eq!(atoi_simd::parse_i64("-2345").unwrap(), -2345_i64);
```

## Benchmarks

### What was noticed

-   It's around <b>7 times faster than the standard parse</b> (for long string, Rust 1.60)
-   The performance is constant (the same) for strings of different lengths

You can run `cargo bench` on your machine.

### Results

<details open><summary><b>Rust 1.63</b>, Windows 10, Intel i7 9700K, "target-feature" set</summary>

This one became faster on Rust 1.63:

```
long string std u64                  1234567890123456
                        time:   [9.0293 ns 9.0843 ns 9.1661 ns]
                        change: [-0.6548% +0.8424% +2.3425%] (p = 0.29 > 0.05)
                        No change in performance detected.
Found 8 outliers among 100 measurements (8.00%)
  2 (2.00%) high mild
  6 (6.00%) high severe
```

This one became even slover. I reran it (with rebuild) multiple times - same result:

```
long string negative std i64         -1234567890123456
                        time:   [17.554 ns 17.607 ns 17.667 ns]
                        change: [-1.6112% -0.2132% +1.5620%] (p = 0.80 > 0.05)
                        No change in performance detected.
Found 6 outliers among 100 measurements (6.00%)
  3 (3.00%) high mild
  3 (3.00%) high severe
```

```
long string u64                      1234567890123456
                        time:   [1.9273 ns 1.9346 ns 1.9424 ns]
                        change: [-2.3999% -0.4986% +1.2253%] (p = 0.62 > 0.05)
                        No change in performance detected.
Found 5 outliers among 100 measurements (5.00%)
  2 (2.00%) high mild
  3 (3.00%) high severe
```

```
long string i64                      1234567890123456
                        time:   [2.3258 ns 2.3357 ns 2.3468 ns]
                        change: [-2.1695% -0.4296% +1.3102%] (p = 0.65 > 0.05)
                        No change in performance detected.
Found 6 outliers among 100 measurements (6.00%)
  1 (1.00%) high mild
  5 (5.00%) high severe
```

```
long string negative i64             -1234567890123456
                        time:   [2.5319 ns 2.5439 ns 2.5607 ns]
                        change: [-2.0344% -0.3167% +1.5650%] (p = 0.75 > 0.05)
                        No change in performance detected.
Found 8 outliers among 100 measurements (8.00%)
  1 (1.00%) high mild
  7 (7.00%) high severe
```

```
short string std u64                       1
                        time:   [2.3305 ns 2.3462 ns 2.3656 ns]
                        change: [-4.1262% -1.9850% +0.2412%] (p = 0.07 > 0.05)
                        No change in performance detected.
Found 7 outliers among 100 measurements (7.00%)
  1 (1.00%) high mild
  6 (6.00%) high severe
```

```
short string negative std i64              -1
                        time:   [3.7983 ns 3.8177 ns 3.8402 ns]
                        change: [-1.4979% -0.0694% +1.5137%] (p = 0.94 > 0.05)
                        No change in performance detected.
Found 9 outliers among 100 measurements (9.00%)
  5 (5.00%) high mild
  4 (4.00%) high severe
```

```
short string u64                           1
                        time:   [2.0024 ns 2.0097 ns 2.0184 ns]
                        change: [-3.4351% -1.3017% +0.5198%] (p = 0.22 > 0.05)
                        No change in performance detected.
Found 3 outliers among 100 measurements (3.00%)
  1 (1.00%) high mild
  2 (2.00%) high severe
```

```
short string i64                           1
                        time:   [2.4245 ns 2.4356 ns 2.4499 ns]
                        change: [-2.9298% -1.3203% +0.3535%] (p = 0.12 > 0.05)
                        No change in performance detected.
Found 9 outliers among 100 measurements (9.00%)
  3 (3.00%) high mild
  6 (6.00%) high severe
```

```
short string negative i64                  -1
                        time:   [2.5191 ns 2.5233 ns 2.5285 ns]
                        change: [-2.8014% -0.9235% +0.7916%] (p = 0.35 > 0.05)
                        No change in performance detected.
Found 8 outliers among 100 measurements (8.00%)
  2 (2.00%) high mild
  6 (6.00%) high severe
```

### Bonus 15 chars benchmarks:

```
15 chars string std u64              123456789012345
                        time:   [8.4146 ns 8.4352 ns 8.4604 ns]
                        change: [-2.5855% -1.0348% +0.5767%] (p = 0.21 > 0.05)
                        No change in performance detected.
Found 7 outliers among 100 measurements (7.00%)
  2 (2.00%) high mild
  5 (5.00%) high severe
```

```
15 chars string negative std i64     -123456789012345
                        time:   [10.268 ns 10.331 ns 10.415 ns]
                        change: [-0.7653% +0.9929% +2.7733%] (p = 0.30 > 0.05)
                        No change in performance detected.
Found 13 outliers among 100 measurements (13.00%)
  7 (7.00%) high mild
  6 (6.00%) high severe
```

```
15 chars string u64                  123456789012345
                        time:   [1.8990 ns 1.9042 ns 1.9103 ns]
                        change: [-1.8510% -0.3256% +0.9332%] (p = 0.70 > 0.05)
                        No change in performance detected.
Found 6 outliers among 100 measurements (6.00%)
  2 (2.00%) high mild
  4 (4.00%) high severe
```

```
15 chars string i64                  123456789012345
                        time:   [2.3780 ns 2.3831 ns 2.3892 ns]
                        change: [-2.1490% -0.6463% +0.8095%] (p = 0.41 > 0.05)
                        No change in performance detected.
Found 9 outliers among 100 measurements (9.00%)
  5 (5.00%) high mild
  4 (4.00%) high severe
```

```
15 chars string negative i64         -123456789012345
                        time:   [2.5323 ns 2.5445 ns 2.5589 ns]
                        change: [-2.8686% -0.9755% +0.9693%] (p = 0.34 > 0.05)
                        No change in performance detected.
Found 7 outliers among 100 measurements (7.00%)
  2 (2.00%) high mild
  5 (5.00%) high severe
```

</details>

<details><summary><b>Rust 1.60</b>, Windows 10, Intel i7 9700K, "target-feature" set</summary>

```
long string std u64                  1234567890123456
                        time:   [15.136 ns 15.172 ns 15.220 ns]
                        change: [-1.0266% +1.4318% +4.7776%] (p = 0.42 > 0.05)
                        No change in performance detected.
Found 14 outliers among 100 measurements (14.00%)
  1 (1.00%) low severe
  1 (1.00%) low mild
  3 (3.00%) high mild
  9 (9.00%) high severe
```

When parsing to `i64` (standard `.parse::<i64>()`) it's somehow faster rather then `u64` (`.parse::<u64>()`)

```
long string negative std i64         -1234567890123456
                        time:   [12.451 ns 12.468 ns 12.489 ns]
                        change: [-2.8201% -1.8197% -0.9578%] (p = 0.00 < 0.05)
                        Change within noise threshold.
Found 15 outliers among 100 measurements (15.00%)
  2 (2.00%) low mild
  5 (5.00%) high mild
  8 (8.00%) high severe
```

```
long string u64                      1234567890123456
                        time:   [2.1173 ns 2.1212 ns 2.1254 ns]
                        change: [-1.7643% -0.7705% +0.0464%] (p = 0.11 > 0.05)
                        No change in performance detected.
Found 3 outliers among 100 measurements (3.00%)
  3 (3.00%) high severe
```

```
long string i64                      1234567890123456
                        time:   [2.0971 ns 2.1018 ns 2.1083 ns]
                        change: [-1.4917% -0.3822% +0.4114%] (p = 0.53 > 0.05)
                        No change in performance detected.
Found 16 outliers among 100 measurements (16.00%)
  3 (3.00%) low mild
  5 (5.00%) high mild
  8 (8.00%) high severe
```

```
long string negative i64             -1234567890123456
                        time:   [2.1659 ns 2.1689 ns 2.1729 ns]
                        change: [-1.8464% -0.6673% +0.2406%] (p = 0.25 > 0.05)
                        No change in performance detected.
Found 12 outliers among 100 measurements (12.00%)
  4 (4.00%) low mild
  1 (1.00%) high mild
  7 (7.00%) high severe
```

```
short string std u64                       1
                        time:   [2.7282 ns 2.7315 ns 2.7355 ns]
                        change: [-0.3423% +0.5560% +1.4297%] (p = 0.25 > 0.05)
                        No change in performance detected.
Found 16 outliers among 100 measurements (16.00%)
  6 (6.00%) high mild
  10 (10.00%) high severe
```

```
short string negative std i64              -1
                        time:   [3.4122 ns 3.4210 ns 3.4304 ns]
                        change: [-0.4427% +0.2415% +1.0592%] (p = 0.57 > 0.05)
                        No change in performance detected.
Found 4 outliers among 100 measurements (4.00%)
  1 (1.00%) high mild
  3 (3.00%) high severe
```

```
short string u64                           1
                        time:   [2.0971 ns 2.0989 ns 2.1014 ns]
                        change: [-0.4568% +0.1569% +0.7932%] (p = 0.63 > 0.05)
                        No change in performance detected.
Found 16 outliers among 100 measurements (16.00%)
  2 (2.00%) low mild
  2 (2.00%) high mild
  12 (12.00%) high severe
```

This one must be a little lower, around 2.3 ns

```
short string i64                           1
                        time:   [2.6629 ns 2.6704 ns 2.6789 ns]
                        change: [-0.2341% +0.4340% +0.9879%] (p = 0.19 > 0.05)
                        No change in performance detected.
Found 6 outliers among 100 measurements (6.00%)
  2 (2.00%) high mild
  4 (4.00%) high severe
```

```
short string negative i64                  -1
                        time:   [2.3049 ns 2.3077 ns 2.3115 ns]
                        change: [-0.8049% -0.1058% +0.5989%] (p = 0.79 > 0.05)
                        No change in performance detected.
Found 16 outliers among 100 measurements (16.00%)
  5 (5.00%) low mild
  3 (3.00%) high mild
  8 (8.00%) high severe
```

### Bonus 15 chars benchmarks:

```
15 chars string std u64              123456789012345
                        time:   [14.314 ns 14.347 ns 14.386 ns]
                        change: [+0.5781% +1.5775% +3.0108%] (p = 0.00 < 0.05)
                        Change within noise threshold.
Found 10 outliers among 100 measurements (10.00%)
  7 (7.00%) high mild
  3 (3.00%) high severe
```

```
15 chars string negative std i64     -123456789012345
                        time:   [11.797 ns 11.869 ns 11.952 ns]
                        change: [-2.0623% -0.8216% +0.4470%] (p = 0.21 > 0.05)
                        No change in performance detected.
Found 11 outliers among 100 measurements (11.00%)
  3 (3.00%) high mild
  8 (8.00%) high severe
```

```
15 chars string u64                  123456789012345
                        time:   [1.8545 ns 1.8559 ns 1.8576 ns]
                        change: [-1.0279% -0.3076% +0.3114%] (p = 0.40 > 0.05)
                        No change in performance detected.
Found 16 outliers among 100 measurements (16.00%)
  3 (3.00%) low mild
  4 (4.00%) high mild
  9 (9.00%) high severe
```

```
15 chars string i64                  123456789012345
                        time:   [2.3638 ns 2.3734 ns 2.3825 ns]
                        change: [-1.8528% -0.7356% +0.2488%] (p = 0.17 > 0.05)
                        No change in performance detected.
Found 4 outliers among 100 measurements (4.00%)
  3 (3.00%) high mild
  1 (1.00%) high severe
```

```
15 chars string negative i64         -123456789012345
                        time:   [2.3077 ns 2.3109 ns 2.3152 ns]
                        change: [-1.9844% -1.2570% -0.5860%] (p = 0.00 < 0.05)
                        Change within noise threshold.
Found 15 outliers among 100 measurements (15.00%)
  3 (3.00%) low mild
  2 (2.00%) high mild
  10 (10.00%) high severe
```

</details>