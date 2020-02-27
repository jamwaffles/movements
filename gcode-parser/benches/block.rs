use criterion::{black_box, criterion_group, criterion_main, Criterion};
use gcode_parser::block::block;

// fn block_simple(c: &mut Criterion) {
//     c.bench_function("block simple", |b| b.iter(|| block(black_box("G1"))));
// }

fn block_delete(c: &mut Criterion) {
    c.bench_function("block delete", |b| b.iter(|| block(black_box("/ G1 X1"))));
}

fn block_line_number(c: &mut Criterion) {
    c.bench_function("block line number", |b| {
        b.iter(|| block(black_box("N1234 G1 X1")))
    });
}

fn block_line_number_and_delete(c: &mut Criterion) {
    c.bench_function("block line number and delete", |b| {
        b.iter(|| block(black_box("/ N1234 G1 X1")))
    });
}

criterion_group!(
    benches,
    /*block_g1, */ block_delete,
    block_line_number,
    block_line_number_and_delete
);
criterion_main!(benches);
