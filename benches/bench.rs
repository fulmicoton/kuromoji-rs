#![feature(test)]

extern crate kuromoji;
extern crate test;

use test::Bencher;
use kuromoji::Tokenizer;

#[bench]
fn test_tokenize(b: &mut Bencher) {
    let mut tokenizer = Tokenizer::new();
    b.iter(|| {
        let tokens = tokenizer.tokenize("すもももももももものうち");
    })
}