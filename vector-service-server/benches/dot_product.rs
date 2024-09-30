use std::vec;

use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion};

use vector_service_server::{vector_dot_product_efficient, vector_dot_product_inefficient};

fn dot_product_benchmark(c: &mut Criterion) {
    let mut group = c.benchmark_group("dot_product");

    for size in [64, 256, 1024].iter() {
        group.bench_with_input(BenchmarkId::new("inefficient", size), size, |b, &size| {
            let v1 = vec![1.0; size];
            let v2 = vec![2.0; size];
            b.iter(|| vector_dot_product_inefficient(v1.clone(), v2.clone()))
        });

        group.bench_with_input(BenchmarkId::new("efficient", size), size, |b, &size| {
            let v1 = vec![1.0; size];
            let v2 = vec![2.0; size];
            b.iter(|| vector_dot_product_efficient(&v1, &v2))
        });
    }

    group.finish();
}

criterion_group!(benches, dot_product_benchmark);
criterion_main!(benches);
