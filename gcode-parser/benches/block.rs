use criterion::{black_box, criterion_group, criterion_main, Criterion};
use gcode_parser::{block::block, ParseInput};

fn block_simple(c: &mut Criterion) {
    c.bench_function("block simple", |b| {
        b.iter(|| block(black_box(ParseInput::new("G1 X10.3 Y20.1 Z30.2"))))
    });
}

fn block_delete(c: &mut Criterion) {
    c.bench_function("block delete", |b| {
        b.iter(|| block(black_box(ParseInput::new("/ G1 X10.3 Y20.1 Z30.2"))))
    });
}

fn block_line_number(c: &mut Criterion) {
    c.bench_function("block line number", |b| {
        b.iter(|| block(black_box(ParseInput::new("N1234 G1 X10.3 Y20.1 Z30.2"))))
    });
}

fn block_line_number_and_delete(c: &mut Criterion) {
    c.bench_function("block line number and delete", |b| {
        b.iter(|| block(black_box(ParseInput::new("/ N1234 G1 X10.3 Y20.1 Z30.2"))))
    });
}

criterion_group!(
    benches,
    block_simple,
    block_delete,
    block_line_number,
    block_line_number_and_delete
);
criterion_main!(benches);
