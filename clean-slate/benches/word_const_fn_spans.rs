use clean_slate::const_generics_spanned;
use criterion::{black_box, criterion_group, criterion_main, Criterion};

fn integer(c: &mut Criterion) {
    c.bench_function("Spanned G4", |b| {
        b.iter(|| const_generics_spanned::recognise_word::<'G', 4>(black_box("g 04".into())))
    });
}

fn decimal(c: &mut Criterion) {
    c.bench_function("Spanned G17.1", |b| {
        b.iter(|| {
            const_generics_spanned::recognise_word_decimal::<'G', 17, 1>(black_box(
                "g 017  .   1".into(),
            ))
        })
    });
}

fn literal(c: &mut Criterion) {
    c.bench_function("Spanned literal", |b| {
        b.iter(|| const_generics_spanned::literal::<'P'>(black_box("P 1234.0056789".into())))
    });
}

criterion_group!(benches, integer, decimal, literal);
criterion_main!(benches);
