use criterion::{black_box, criterion_group, criterion_main, Criterion};
use revent::*;
use std::any::TypeId;

struct EmptyEvent;

impl Event for EmptyEvent {}

struct PanicEvent;

impl Event for PanicEvent {}

// ---

struct Uncached {}

impl Notifiable for Uncached {
    fn notify(&mut self, event: &dyn Event, _: &mut EventStore) {
        if event.type_id() == TypeId::of::<PanicEvent>() {
            panic!();
        }
    }
}

// ---

struct Cached {
    store: Option<EventStore>,
}

impl Notifiable for Cached {
    fn notify(&mut self, event: &dyn Event, _: &mut EventStore) {
        if event.type_id() == TypeId::of::<PanicEvent>() {
            panic!();
        }
    }

    fn take_storage(&mut self) -> EventStore {
        self.store.take().unwrap()
    }

    fn set_storage(&mut self, store: EventStore) {
        self.store = Some(store);
    }
}

// ---

fn criterion_benchmark(c: &mut Criterion) {
    c.bench_function("default eventstore settings", |b| {
        let mut uncached = Uncached {};

        b.iter(|| {
            uncached.with_notify(|_, store| {
                store.emit(black_box(EmptyEvent {}));
            });
        });
    });

    c.bench_function("cached eventstore", |b| {
        let mut cached = Cached {
            store: Some(EventStore::default()),
        };

        b.iter(|| {
            cached.with_notify(|_, store| {
                store.emit(black_box(EmptyEvent {}));
            });
        });
    });

    c.bench_function("default eventstore settings 1000 events", |b| {
        let mut uncached = Uncached {};

        b.iter(|| {
            uncached.with_notify(|_, store| {
                for _ in 0..1000 {
                    store.emit(black_box(EmptyEvent {}));
                }
            });
        });
    });

    c.bench_function("cached eventstore 1000 events", |b| {
        let mut cached = Cached {
            store: Some(EventStore::default()),
        };

        b.iter(|| {
            cached.with_notify(|_, store| {
                for _ in 0..1000 {
                    store.emit(black_box(EmptyEvent {}));
                }
            });
        });
    });
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
