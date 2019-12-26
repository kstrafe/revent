use criterion::{criterion_group, criterion_main, Criterion};
use revent::*;

fn criterion_benchmark(c: &mut Criterion) {
    c.bench_function("basic signal", |b| {
        hub! {
            mod x;
            Hub(Sub) {
                signal: fn(()) -> (), Subsignals {},
            }
        }

        struct X;

        impl Subscriber<(), (), Subsignals> for X {
            fn event(&mut self, _: &(), _: &Subsignals) {}
        }

        impl Selfscribe<Sub> for X {
            fn subscribe(&mut self, this: &Hubbed<Self>, hub: &mut Sub) {
                hub.signal(this);
            }
        }

        let hub = Hub::default();
        hub.subscribe(X);

        b.iter(|| {
            hub.signal(&());
        });
    });

    c.bench_function("many subscribers to single signal", |b| {
        hub! {
            mod x;
            Hub(Sub) {
                signal: fn(()) -> (), Subsignals {},
            }
        }

        struct X;

        impl Subscriber<(), (), Subsignals> for X {
            fn event(&mut self, _: &(), _: &Subsignals) {}
        }

        impl Selfscribe<Sub> for X {
            fn subscribe(&mut self, this: &Hubbed<Self>, hub: &mut Sub) {
                hub.signal(this);
            }
        }

        let hub = Hub::default();

        for _ in 0..1000 {
            hub.subscribe(X);
        }

        b.iter(|| {
            hub.signal(&());
        });
    });

    c.bench_function("deep signal chain", |b| {
        hub! {
            mod x;
            Hub(Sub) {
                signal1: fn(()) -> (), Subsignals1 {
                    signal2: fn(()) -> (), Subsignals2,
                },
                signal2: fn(()) -> (), Subsignals2 {
                    signal3: fn(()) -> (), Subsignals3,
                },
                signal3: fn(()) -> (), Subsignals3 { },
            }
        }

        struct X1;
        struct X2;
        struct X3;

        impl Subscriber<(), (), Subsignals1> for X1 {
            fn event(&mut self, _: &(), signals: &Subsignals1) {
                signals.signal2(&());
            }
        }

        impl Selfscribe<Sub> for X1 {
            fn subscribe(&mut self, this: &Hubbed<Self>, hub: &mut Sub) {
                hub.signal1(this);
            }
        }

        impl Subscriber<(), (), Subsignals2> for X2 {
            fn event(&mut self, _: &(), signals: &Subsignals2) {
                signals.signal3(&());
            }
        }

        impl Selfscribe<Sub> for X2 {
            fn subscribe(&mut self, this: &Hubbed<Self>, hub: &mut Sub) {
                hub.signal2(this);
            }
        }

        impl Subscriber<(), (), Subsignals3> for X3 {
            fn event(&mut self, _: &(), _: &Subsignals3) {}
        }

        impl Selfscribe<Sub> for X3 {
            fn subscribe(&mut self, this: &Hubbed<Self>, hub: &mut Sub) {
                hub.signal3(this);
            }
        }

        let hub = Hub::default();

        hub.subscribe(X1);
        hub.subscribe(X2);
        hub.subscribe(X3);

        b.iter(|| {
            hub.signal1(&());
        });
    });
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
