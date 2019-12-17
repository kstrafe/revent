use criterion::{black_box, criterion_group, criterion_main, Criterion};
use revent::*;

#[derive(Debug)]
struct NumberEvent(pub i32);

// ---

struct Sink;

impl Notifiable for Sink {
    fn event(&mut self, event: &dyn Event, _: &mut dyn Notifiable) {
        if let Some(NumberEvent(number)) = down(event) {
            if *number == 0 {
                panic!("Number cannot be zero");
            }
        }
    }
}

// ---

struct Counter;

impl Notifiable for Counter {
    fn event(&mut self, event: &dyn Event, system: &mut dyn Notifiable) {
        if let Some(NumberEvent(number)) = down(event) {
            if *number != 0 {
                self.notify(&NumberEvent(number - 1), system);
            }
        }
    }
}

fn criterion_benchmark(c: &mut Criterion) {
    c.bench_function("one way event", |b| {
        let mut sink = Sink;
        b.iter(|| {
            sink.notify(&black_box(NumberEvent(1)), &mut ());
        });
    });

    c.bench_function("counter", |b| {
        let mut counter = Counter;
        b.iter(|| {
            counter.notify(&black_box(NumberEvent(1000)), &mut ());
        });
    });
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
