use clean_slate::const_generics_test;
use criterion::{black_box, criterion_group, criterion_main, Criterion};

fn integer(c: &mut Criterion) {
    c.bench_function("G4", |b| {
        b.iter(|| const_generics_test::recognise_word::<'G', 4>(black_box("g 04")))
    });
}

fn decimal(c: &mut Criterion) {
    c.bench_function("G17.1", |b| {
        b.iter(|| {
            const_generics_test::recognise_word_decimal::<'G', 17, 1>(black_box("g 017  .   1"))
        })
    });
}

fn literal(c: &mut Criterion) {
    c.bench_function("Literal", |b| {
        b.iter(|| const_generics_test::literal::<'P'>(black_box("P 1234.0056789")))
    });
}

criterion_group!(benches, integer, decimal, literal);
criterion_main!(benches);
