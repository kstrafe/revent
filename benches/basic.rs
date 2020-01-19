use criterion::{black_box, criterion_group, criterion_main, Criterion};
use revent::{hub, Shared, Subscriber};

pub trait Signal {
    fn signal(&mut self);
}

struct Handler;
impl Signal for Handler {
    fn signal(&mut self) {}
}
impl Subscriber<Hub> for Handler {
    type Input = ();
    fn build(_: Hub, _: Self::Input) -> Self {
        Self
    }
    fn subscribe(hub: &Hub, shared: Shared<Self>) {
        hub.signal.subscribe(shared);
    }
}

hub! {
    Hub {
        signal: dyn Signal,
    }
}

fn criterion_benchmark(c: &mut Criterion) {
    c.bench_function("empty signal handler", |b| {
        let hub = Hub::default();
        b.iter(|| {
            black_box(&hub).signal.emit(|x| {
                x.signal();
            });
        });
    });

    c.bench_function("single signal handler", |b| {
        let hub = Hub::default();
        hub.subscribe::<Handler>(());
        b.iter(|| {
            black_box(&hub).signal.emit(|x| {
                x.signal();
            });
        });
    });

    c.bench_function("many signal handler", |b| {
        let hub = Hub::default();
        for _ in 0..1000 {
            hub.subscribe::<Handler>(());
        }
        b.iter(|| {
            black_box(&hub).signal.emit(|x| {
                x.signal();
            });
        });
    });

    c.bench_function("many remove", |b| {
        let hub = Hub::default();
        b.iter(|| {
            for _ in 0..1000 {
                hub.subscribe::<Handler>(());
            }
            black_box(&hub).signal.filter(|_| true);
        });
    });

    c.bench_function("many subscribe", |b| {
        let mut hub = Hub::default();
        b.iter(|| {
            for _ in 0..1000 {
                hub.subscribe::<Handler>(());
            }
            hub = Hub::default();
        });
    });

    c.bench_function("adding subscribers", |b| {
        let hub = Hub::default();
        b.iter(|| {
            hub.subscribe::<Handler>(());
        });
    });
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
