use criterion::{black_box, criterion_group, criterion_main, Criterion};
use revent::{hub, node, Subscriber};

fn criterion_benchmark(c: &mut Criterion) {
    c.bench_function("empty emit", |b| {
        pub trait A {}
        hub! {
            X {
                a: A,
            }
        }
        let mut x = X::new();
        b.iter(|| {
            x.a.emit(|_| {});
        });
    });

    c.bench_function("single emit", |b| {
        pub trait A {}
        hub! {
            X {
                a: A,
            }
        }

        node! {
            X {
                a: A,
            } => Node(Handler) {
            }
        }

        struct Handler;
        impl A for Handler {}
        impl Subscriber for Handler {
            type Input = ();
            fn build(_: Self::Node, _: Self::Input) -> Self {
                Self
            }
        }

        let mut x = X::new();
        x.subscribe::<Handler>(());
        b.iter(|| {
            x.a.emit(|_| {});
        });
    });

    c.bench_function("many emit", |b| {
        pub trait A {}
        hub! {
            X {
                a: A,
            }
        }

        node! {
            X {
                a: A,
            } => Node(Handler) {
            }
        }

        struct Handler;
        impl A for Handler {}
        impl Subscriber for Handler {
            type Input = ();
            fn build(_: Self::Node, _: Self::Input) -> Self {
                Self
            }
        }

        let mut x = X::new();
        for _ in 0..1000 {
            x.subscribe::<Handler>(());
        }

        b.iter(|| {
            x.a.emit(|_| {});
        });
    });

    c.bench_function("many remove", |b| {
        pub trait A {}
        hub! {
            X {
                a: A,
            }
        }

        node! {
            X {
                a: A,
            } => Node(Handler) {
            }
        }

        struct Handler;
        impl A for Handler {}
        impl Subscriber for Handler {
            type Input = ();
            fn build(_: Self::Node, _: Self::Input) -> Self {
                Self
            }
        }

        let mut x = X::new();
        for _ in 0..1000 {
            x.subscribe::<Handler>(());
        }

        b.iter(|| {
            x.a.remove(|_| false);
        });
    });

    c.bench_function("subscribe and remove", |b| {
        pub trait A {}
        hub! {
            X {
                a: A,
            }
        }

        node! {
            X {
                a: A,
            } => Node(Handler) {
            }
        }

        struct Handler;
        impl A for Handler {}
        impl Subscriber for Handler {
            type Input = ();
            fn build(_: Self::Node, _: Self::Input) -> Self {
                Self
            }
        }

        let mut x = X::new();

        b.iter(|| {
            x.subscribe::<Handler>(());
            x.a.remove(|_| true);
        });
    });
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
