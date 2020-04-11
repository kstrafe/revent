use criterion::{black_box, criterion_group, criterion_main, Criterion};
use revent::Anchor;

mod setup {
    use revent::{
        feed::{Feed, Feedee, Feeder},
        Anchor, Manager, Node,
    };
    use std::{cell::RefCell, rc::Rc};

    #[derive(Clone)]
    pub enum Message {
        Vector(Vec<u8>),
        Nothing,
    }

    impl Message {
        pub fn get_vector(&self) -> &Vec<u8> {
            match self {
                Message::Vector(vec) => vec,
                _ => panic!(),
            }
        }
    }

    pub struct MyAnchor {
        pub feed: Feed<Message>,
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
                feed: Feed::new("feed", &manager, 1),
                manager,
            }
        }
    }

    // ---

    pub struct FeedeeEmitter {
        pub feed: Feedee<Message>,
    }

    pub struct FeedeeHandler {
        pub emits: FeedeeEmitter,
    }

    impl Node<MyAnchor, FeedeeEmitter> for FeedeeHandler {
        fn register_emits(anchor: &MyAnchor) -> FeedeeEmitter {
            FeedeeEmitter {
                feed: anchor.feed.feedee(),
            }
        }

        fn register_listens(_: &mut MyAnchor, _: Rc<RefCell<Self>>) {}
        const NAME: &'static str = "FeedeeHandler";
    }

    // ---

    pub struct FeederEmitter {
        pub feed: Feeder<Message>,
    }

    pub struct FeederHandler {
        pub emits: FeederEmitter,
    }

    impl Node<MyAnchor, FeederEmitter> for FeederHandler {
        fn register_emits(anchor: &MyAnchor) -> FeederEmitter {
            FeederEmitter {
                feed: anchor.feed.feeder(),
            }
        }

        fn register_listens(_: &mut MyAnchor, _: Rc<RefCell<Self>>) {}
        const NAME: &'static str = "FeederHandler";
    }
}

fn criterion_benchmark(c: &mut Criterion) {
    c.bench_function("cloning big data to feedees", |b| {
        let mut hub = setup::MyAnchor::new();
        let items = (0..3)
            .map(|_| hub.subscribe(|emits| setup::FeedeeHandler { emits }))
            .collect::<Vec<_>>();
        let feeder = hub.subscribe(|emits| setup::FeederHandler { emits });
        b.iter(|| {
            feeder
                .borrow_mut()
                .emits
                .feed
                .feed(black_box(setup::Message::Vector(vec![0; 8294400])));

            for item in items.iter() {
                assert_eq!(
                    8294400,
                    item.borrow_mut()
                        .emits
                        .feed
                        .pop()
                        .unwrap()
                        .get_vector()
                        .capacity()
                );
            }
        });
    });

    c.bench_function("small feedees", |b| {
        let mut hub = setup::MyAnchor::new();
        let items = (0..10_000)
            .map(|_| hub.subscribe(|emits| setup::FeedeeHandler { emits }))
            .collect::<Vec<_>>();
        let feeder = hub.subscribe(|emits| setup::FeederHandler { emits });
        b.iter(|| {
            feeder
                .borrow_mut()
                .emits
                .feed
                .feed(black_box(setup::Message::Nothing));

            for item in items.iter() {
                assert!(matches!(
                    item.borrow_mut().emits.feed.pop().unwrap(),
                    setup::Message::Nothing
                ));
            }
        });
    });
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
