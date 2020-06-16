use criterion::{black_box, criterion_group, criterion_main, Criterion, Throughput};
use gcode_parser::{GcodeProgram, ParseInput};

static PROGRAM: &'static str = r#"G54
G21
G0 x0 y0 z0
F500
M3 s5000
G1 x20
G1 x10 y20 z2
G1 z-5
G1 x0 y0 z0
g0 z0
g0 z500
m5
m2"#;

fn program(c: &mut Criterion) {
    let mut group = c.benchmark_group("programs");

    group.throughput(Throughput::Bytes(PROGRAM.len() as u64));

    group.bench_function("basic program", |b| {
        b.iter(|| GcodeProgram::from_str(black_box(PROGRAM)))
    });
}

criterion_group!(benches, program);
criterion_main!(benches);
