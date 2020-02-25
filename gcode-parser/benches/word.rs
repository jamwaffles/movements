use criterion::{black_box, criterion_group, criterion_main, Criterion};
use gcode_parser::word::word;

fn word_g1(c: &mut Criterion) {
    c.bench_function("word G1", |b| {
        b.iter(|| word::<String, ()>('G')(black_box("G1")))
    });
}

fn word_f32(c: &mut Criterion) {
    c.bench_function("word f32", |b| {
        b.iter(|| word::<f32, ()>('X')(black_box("X12.5678")))
    });
}

fn word_u32(c: &mut Criterion) {
    c.bench_function("word u32", |b| {
        b.iter(|| word::<u32, ()>('P')(black_box("P99")))
    });
}

fn word_whitespace(c: &mut Criterion) {
    c.bench_function("word whitespace", |b| {
        b.iter(|| word::<String, ()>('M')(black_box("M  199")))
    });
}

criterion_group!(benches, word_g1, word_f32, word_u32, word_whitespace);
criterion_main!(benches);
