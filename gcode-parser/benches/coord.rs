use criterion::{black_box, criterion_group, criterion_main, Criterion};
use gcode_parser::coord::coord;

fn coord_full(c: &mut Criterion) {
    c.bench_function("coord full", |b| b.iter(|| coord::<()>(black_box("X1.435356 Y2.435356 Z3.435356 A4.435356 B5.435356 C6.435356 U7.435356 V8.435356 W9.678444"))));
}

fn coord_single(c: &mut Criterion) {
    c.bench_function("coord single", |b| {
        b.iter(|| coord::<()>(black_box("X12.5678")))
    });
}

fn coord_xyz(c: &mut Criterion) {
    c.bench_function("coord xyz", |b| {
        b.iter(|| coord::<()>(black_box("X12.5678 Y3.4656 Z5.5555")))
    });
}

fn coord_out_of_order(c: &mut Criterion) {
    c.bench_function("coord out of order", |b| {
        b.iter(|| coord::<()>(black_box("C6.435356 Z3.435356 A4.435356 X1.435356 U7.435356 V8.435356 W9.678444 Y2.435356 B5.435356")))
    });
}

criterion_group!(
    benches,
    coord_full,
    coord_single,
    coord_xyz,
    coord_out_of_order
);
criterion_main!(benches);
