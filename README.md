# bytepiece-rs

[![Crates.io](https://img.shields.io/crates/v/bytepiece?style=for-the-badge)](https://crates.io/crates/bytepiece)
[![docs.rs](https://img.shields.io/docsrs/bytepiece/latest?style=for-the-badge)](https://docs.rs/bytepiece)

[BytePiece]是一个Byte-based的Unigram分词器，纯Python实现，更加易读和易拓展。
本项目为[BytePiece]的Rust实现，同时也提供了[Python bindings](./bytepiece-py)。

[BytePiece] is a byte-based unigram tokenizer. It was orignally implemented in Python, which is easy to understand and extend. 
This project is a pure Rust implmementation of [BytePiece] algorithm and its [Python bindings](./bytepiece-py).


We provide python bindings. You can download prebuilt wheels in [CI artifacts](https://github.com/SunDoge/bytepiece-rs/actions/workflows/python-bindings-ci.yml).


## WebAssembly

已提供浏览器与 Node.js 绑定，以及 TypeScript 声明。构建和使用示例见 [bytepiece-wasm](./bytepiece-wasm/README.md)。

```sh
rustup target add wasm32-unknown-unknown
cargo install wasm-bindgen-cli --version 0.2.128 --locked
scripts/build_wasm.sh
node scripts/test_wasm.mjs
```

## Compatibility and validation

算法对照 [原版 BytePiece 0.6.3](https://github.com/bojone/bytepiece)，使用自然对数计算 unigram 分数和采样概率，保留连续换行的分段规则。Python / Wasm 的字符串入口执行 NFC 归一化；Rust 的借用切片 API 和字节入口保留原始字节，调用方需要时应先归一化字符串。

模型加载支持 `OwnedTokenizer::from_path` 和 `OwnedTokenizer::from_slice`。模型需包含全部 256 个单字节 token、非空 token、正计数和唯一的 ID（从 3 起，0/1/2 保留给 PAD/BOS/EOS）；无效模型返回错误，避免不可达分词路径。Python 绑定保留 Python 3.9+ 的稳定 ABI，并将 Python 包的最低版本声明与其对齐。

```sh
cargo test --workspace --locked
cargo clippy --workspace --all-targets --locked -- -D warnings
```

`scripts/check_upstream.py` 对原版与本项目的 Python 绑定做差分验证，再通过 `node scripts/test_wasm.mjs --upstream` 验证相同输入的 Wasm 结果；环境配置见 [Wasm 文档](./bytepiece-wasm/README.md)。

## Benchmark

### Rust `bytepiece`

```shell
python scripts/download_model.py
cargo bench -p bytepiece --bench tokenize --locked
```

2026-09-07，本机 Linux x86_64 / Rust 1.98.1，原版 80k 模型和仓库中英混合段落，`alpha = -1`。比较起始提交 `643665c` 与本次完整改动（包含依赖升级及 thin LTO）。每项预热 3 秒、测量 5 秒、100 个样本；表中为 Criterion 样本均值：

| 操作 | 修改前 | 修改后 | 耗时降低 |
| --- | ---: | ---: | ---: |
| tokenize | 19.64 µs | 16.28 µs | 17.1% |
| encode | 20.95 µs | 16.29 µs | 22.2% |

两项均达到 Criterion 的显著性判定（p < 0.05）。共享机器存在调度噪声，结果用于本次同机比较。优化包括直接从最优路径输出 ID、复用每次调用内的分段缓冲区、用字节扫描代替正则分段、减少路径写入，以及分离确定性和随机采样循环。

复测时，在修改前运行 `cargo bench -p bytepiece --bench tokenize -- --save-baseline before`，再在修改后运行 `cargo bench -p bytepiece --bench tokenize -- --baseline before`；需保留同一个 `target/criterion` 目录。第三方 `bytepiece_rs` 的对照项也随依赖版本升级，以上表格只比较本项目自身。

### Python `bytepiece-py`

以下为项目原有的历史数据，本次性能比较见上面的 Rust 表格。

```shell
python scripts/download_model.py
cd bytepiece-py && maturin develop -r && cd ..
python bytepiece-py/examples/bench.py
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
