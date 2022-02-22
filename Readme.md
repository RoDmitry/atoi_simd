# Rust fast `&str` to `i64` parser (x86_64 simd, sse2)

Modified [this](https://rust-malaysia.github.io/code/2020/07/11/faster-integer-parsing.html) version to support various string length and negative values

#### You need to define the `target-cpu` for it to build with optimized performance.
For example `-e RUSTFLAGS="-C target-cpu=native"`
