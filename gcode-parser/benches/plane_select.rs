use criterion::{black_box, criterion_group, criterion_main, Criterion};
use gcode_parser::plane_select::plane_select;

fn plane_select_int(c: &mut Criterion) {
    c.bench_function("plane select G17", |b| {
        b.iter(|| plane_select(black_box("G17")))
    });
}

fn plane_select_float(c: &mut Criterion) {
    c.bench_function("plane select G17.1", |b| {
        b.iter(|| plane_select(black_box("G17.1")))
    });
}

criterion_group!(benches, plane_select_int, plane_select_float,);
criterion_main!(benches);
