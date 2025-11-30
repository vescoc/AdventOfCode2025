use std::hint;

use criterion::{criterion_group, criterion_main, Criterion};

use day01 as day;

const INPUT: &str = include_str!("../../input");

pub fn criterion_benchmark(c: &mut Criterion) {
    let mut group = c.benchmark_group("day01");
    group.bench_function("part 1", |b| {
        b.iter(|| hint::black_box(day::part_1(hint::black_box(INPUT))));
    });
    group.bench_function("part 2", |b| {
        b.iter(|| hint::black_box(day::part_2(hint::black_box(INPUT))));
    });
    group.finish();
}
    
criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
