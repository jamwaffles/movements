use criterion::{black_box, criterion_group, criterion_main, Criterion};
use gcode_parser::coord::coord;

fn comment_eol(c: &mut Criterion) {
    c.bench_function("parens comment", |b| {
        b.iter(|| coord::<()>(black_box("( hello my name is bobby beans )")))
    });
}

fn comment_parens(c: &mut Criterion) {
    c.bench_function("eol comment", |b| {
        b.iter(|| coord::<()>(black_box("; hello my name is bobby beans")))
    });
}

criterion_group!(benches, comment_eol, comment_parens);
criterion_main!(benches);
