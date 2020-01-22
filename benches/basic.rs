use criterion::{black_box, criterion_group, criterion_main, Criterion};
use revent::{shared, Topic};

fn criterion_benchmark(c: &mut Criterion) {
    c.bench_function("empty emit", |b| {
        let mut topic: Topic<i32> = Topic::new();
        b.iter(|| {
            black_box(&mut topic).emit(|x| {
                assert!(*x > 0);
            });
        });
    });

    c.bench_function("adding subscribers", |b| {
        let mut topic = Topic::new();
        b.iter(|| {
            topic.insert(shared(123));
        });
    });

    c.bench_function("single emit", |b| {
        let mut topic = Topic::new();
        topic.insert(shared(123));
        b.iter(|| {
            black_box(&mut topic).emit(|x| {
                assert!(*x > 0);
            });
        });
    });

    c.bench_function("many emit", |b| {
        let mut topic = Topic::new();
        for val in 1..1000 {
            topic.insert(shared(val));
        }
        b.iter(|| {
            black_box(&mut topic).emit(|x| {
                assert!(*x > 0);
            });
        });
    });
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
