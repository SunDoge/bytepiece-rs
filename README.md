# bytepiece-rs

[![Crates.io](https://img.shields.io/crates/v/bytepiece?style=for-the-badge)](https://crates.io/crates/bytepiece)
[![docs.rs](https://img.shields.io/docsrs/bytepiece/latest?style=for-the-badge)](https://docs.rs/bytepiece)

[BytePiece]是一个Byte-based的Unigram分词器，纯Python实现，更加易读和易拓展。
本项目为[BytePiece]的Rust实现，同时也提供了[Python bindings](./bytepiece-py)。

[BytePiece] is a byte-based unigram tokenizer. It was orignally implemented in Python, which is easy to understand and extend. 
This project is a pure Rust implmementation of [BytePiece] algorithm and its [Python bindings](./bytepiece-py).


We provide python bindings. You can download prebuilt wheels in [CI artifacts](https://github.com/SunDoge/bytepiece-rs/actions/workflows/python-bindings-ci.yml).


## Benchmark

### Rust `bytepiece`

```shell
python scripts/download_model.py
cargo bench -p bytepiece
```

```
Tokenize/bytepiece_rs   time:   [37.937 µs 38.133 µs 38.340 µs]
                        change: [-2.7046% -2.2298% -1.7570%] (p = 0.00 < 0.05)
                        Performance has improved.
Found 3 outliers among 100 measurements (3.00%)
  3 (3.00%) high mild
Tokenize/bytepiece      time:   [17.504 µs 17.520 µs 17.537 µs]
                        change: [-4.5209% -4.1024% -3.7110%] (p = 0.00 < 0.05)
                        Performance has improved.
Found 5 outliers among 100 measurements (5.00%)
  2 (2.00%) high mild
  3 (3.00%) high severe

Encode/bytepiece_rs     time:   [36.565 µs 36.740 µs 37.057 µs]
                        change: [-6.4123% -5.3649% -4.4084%] (p = 0.00 < 0.05)
                        Performance has improved.
Found 9 outliers among 100 measurements (9.00%)
  1 (1.00%) low mild
  4 (4.00%) high mild
  4 (4.00%) high severe
Encode/bytepiece        time:   [19.934 µs 19.976 µs 20.028 µs]
                        change: [+1.4633% +1.9218% +2.4588%] (p = 0.00 < 0.05)
                        Performance has regressed.
Found 12 outliers among 100 measurements (12.00%)
  5 (5.00%) high mild
  7 (7.00%) high severe
```

### Python `bytepiece-py`

```shell
python scripts/download_model.py
cd bytepiece-py && maturin develop -r && cd ..
python bytepiece-py/bench.py
```

```
bytepiece:
0.7831026670028223
bytepiece-py (ours)
0.18666897300136043
rs-bytepiece
0.4513153380030417
```

[BytePiece]: https://github.com/bojone/bytepiece
