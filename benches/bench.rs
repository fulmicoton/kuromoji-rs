use criterion::{criterion_group, criterion_main};
use criterion::Criterion;
use criterion::black_box;

use kuromoji::Tokenizer;

fn bench_tokenize(c: &mut Criterion) {
    c.bench_function("sumomomo", |b| {
        let mut tokenizer = Tokenizer::with_capacity(100);
        b.iter(|| tokenizer.tokenize("すもももももももものうち"))
    });
}

criterion_group!(benches, bench_tokenize);
criterion_main!(benches);
