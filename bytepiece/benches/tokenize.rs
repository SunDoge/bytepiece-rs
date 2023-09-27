use std::path::PathBuf;

use bytepiece::Tokenize;
use criterion::{criterion_group, criterion_main, Criterion};

const TEXT: &'static str = r#"
BytePiece是一个Byte-based的Unigram分词器，纯Python实现，更加易读和易拓展。
由于采用了新的训练算法，所以压缩率通常比现有Tokenizer更高，同时支持多进程加速训练。
此外，它直接操作文本的UTF-8 Bytes，几乎不进行任何的预处理，所以更加纯粹和语言无关。
"#;

const MODEL_PATH: &str = "../models/bytepiece_80k.model";

fn get_model_path(path: &str) -> PathBuf {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    dbg!(&root.display());
    root.join(path)
}

fn bench_tokenize(c: &mut Criterion, text: &str) {
    let model_path = get_model_path(MODEL_PATH);
    let t1 = bytepiece_rs::Tokenizer::load_from(model_path.to_str().unwrap());
    let t2 = bytepiece::tokenizer::OwnedTokenizer::from_path(&model_path).unwrap();

    let mut group = c.benchmark_group("Tokenize");
    group.bench_function("bytepiece_rs", |b| {
        b.iter(|| t1.tokenize(text, -1.0, false))
    });
    group.bench_function("bytepiece", |b| b.iter(|| t2.tokenize(&text, -1.0)));
    group.finish();

    let mut group = c.benchmark_group("Encode");
    group.bench_function("bytepiece_rs", |b| {
        b.iter(|| t1.encode(text, false, false, -1.0, false))
    });
    group.bench_function("bytepiece", |b| {
        b.iter(|| t2.encode(text, false, false, -1.0))
    });
    group.finish();
}

fn criterion_benchmark(c: &mut Criterion) {
    bench_tokenize(c, TEXT);
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
