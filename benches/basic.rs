use criterion::{black_box, criterion_group, criterion_main, Criterion};
use revent::Node;

mod setup {
    use revent::{Manager, Named, Node, Slot, Subscriber};
    use std::{cell::RefCell, rc::Rc};

    pub trait EventHandler {
        fn event(&mut self);
    }

    pub struct Hub {
        pub basic: Slot<dyn EventHandler>,
        pub manager: Rc<RefCell<Manager>>,
    }

    impl Node for Hub {
        fn manager(&self) -> &Rc<RefCell<Manager>> {
            &self.manager
        }
    }

    impl Hub {
        pub fn new() -> Self {
            let manager = Rc::new(RefCell::new(Manager::new()));
            Self {
                basic: Slot::new("basic", manager.clone()),
                manager,
            }
        }
    }

    pub struct MyEventHandler;
    impl EventHandler for MyEventHandler {
        fn event(&mut self) {}
    }

    impl Subscriber<Hub> for MyEventHandler {
        type Input = ();
        type Outputs = revent::Null;
        fn create(_: Self::Input, _: Self::Outputs) -> Self {
            MyEventHandler
        }

        fn register(node: &mut Hub, item: Rc<RefCell<Self>>) {
            node.basic.register(item);
        }
    }

    impl Named for MyEventHandler {
        const NAME: &'static str = "MyEventHandler";
    }
}

fn criterion_benchmark(c: &mut Criterion) {
    c.bench_function("empty emit", |b| {
        let mut hub = setup::Anchor::new();
        b.iter(|| {
            hub.basic.emit(|_| {});
        });
    });

    c.bench_function("single emit", |b| {
        let mut hub = setup::Anchor::new();
        hub.subscribe::<setup::MyEventHandler>(());
        b.iter(|| {
            hub.basic.emit(|x| x.event());
        });
    });

    c.bench_function("many emit", |b| {
        let mut hub = setup::Anchor::new();
        for _ in 0..1000 {
            hub.subscribe::<setup::MyEventHandler>(());
        }
        b.iter(|| {
            hub.basic.emit(|x| x.event());
        });
    });

    c.bench_function("subscribe and remove", |b| {
        let mut hub = setup::Anchor::new();
        b.iter(|| {
            let mut items = (0..1000)
                .map(|_| hub.subscribe::<setup::MyEventHandler>(()))
                .collect::<Vec<_>>();
            for item in items.drain(..) {
                hub.unsubscribe(&item);
            }
        });
    });
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
