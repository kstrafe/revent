use criterion::{black_box, criterion_group, criterion_main, Criterion};
use revent::Anchor;

mod setup {
    use revent::{Anchor, Manager, Node, Slot};
    use std::{cell::RefCell, rc::Rc};

    pub trait EventHandler {
        fn event(&mut self);
    }

    pub struct MyAnchor {
        pub basic: Slot<dyn EventHandler>,
        pub manager: Manager,
    }

    impl Anchor for MyAnchor {
        fn manager(&self) -> &Manager {
            &self.manager
        }
    }

    impl MyAnchor {
        pub fn new() -> Self {
            let manager = Manager::new();
            Self {
                basic: Slot::new("basic", &manager),
                manager,
            }
        }
    }

    pub struct MyEventHandler;
    impl EventHandler for MyEventHandler {
        fn event(&mut self) {}
    }

    impl Node<MyAnchor, ()> for MyEventHandler {
        fn register_emits(_: &MyAnchor) -> () {
            ()
        }

        fn register_listens(anchor: &mut MyAnchor, item: Rc<RefCell<Self>>) {
            anchor.basic.register(item);
        }
        const NAME: &'static str = "MyEventHandler";
    }
}

fn criterion_benchmark(c: &mut Criterion) {
    c.bench_function("empty emit", |b| {
        let mut hub = setup::MyAnchor::new();
        b.iter(|| {
            hub.basic.emit(|_| {});
        });
    });

    c.bench_function("single emit", |b| {
        let mut hub = setup::MyAnchor::new();
        hub.subscribe(|_| setup::MyEventHandler);
        b.iter(|| {
            hub.basic.emit(|x| x.event());
        });
    });

    c.bench_function("many emit", |b| {
        let mut hub = setup::MyAnchor::new();
        for _ in 0..1000 {
            hub.subscribe(|_| setup::MyEventHandler);
        }
        b.iter(|| {
            hub.basic.emit(|x| x.event());
        });
    });

    c.bench_function("subscribe and remove", |b| {
        let mut hub = setup::MyAnchor::new();
        b.iter(|| {
            let mut items = (0..1000)
                .map(|_| hub.subscribe(|_| setup::MyEventHandler))
                .collect::<Vec<_>>();
            for item in items.drain(..) {
                hub.unsubscribe(&item);
            }
        });
    });
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
