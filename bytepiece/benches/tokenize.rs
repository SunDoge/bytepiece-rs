use criterion::{criterion_group, criterion_main, Criterion};

const TEXT: &'static str = r#"
BytePiece是一个Byte-based的Unigram分词器，纯Python实现，更加易读和易拓展。
由于采用了新的训练算法，所以压缩率通常比现有Tokenizer更高，同时支持多进程加速训练。
此外，它直接操作文本的UTF-8 Bytes，几乎不进行任何的预处理，所以更加纯粹和语言无关。
"#;

fn bench_bytepiece_rs(c: &mut Criterion, text: &str) {
    use bytepiece_rs::Tokenizer;
    let tokenizer = Tokenizer::load_from("../bytepiece_80k.model");
    c.bench_function("bytepiece_rs tokenize", |b| {
        b.iter(|| tokenizer.tokenize(text, -1.0))
    });
}

fn bench_bytepiece(c: &mut Criterion, text: &str) {
    use bytepiece::prelude::*;
    let tokenizer = OwnedTokenizer::from_path("../bytepiece_80k.model").unwrap();
    c.bench_function("bytepiece tokenize", |b| {
        b.iter(|| tokenizer.tokenize(text, -1.0))
    });
}

fn criterion_benchmark(c: &mut Criterion) {
    bench_bytepiece_rs(c, TEXT);
    bench_bytepiece(c, TEXT);
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
