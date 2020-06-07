use criterion::{black_box, criterion_group, criterion_main, Criterion};
#[cfg(feature = "asynchronous")]
use revent::asynchronous::Mailer;
use revent::{Channel, Node, Slot, Suspend};

fn criterion_benchmark(c: &mut Criterion) {
    #[cfg(feature = "asynchronous")]
    c.bench_function("mailer blocking", |b| {
        let mailer = Mailer::unbounded();
        let mailbox = mailer.mailbox();

        b.iter(|| {
            mailer.send(());
            black_box(mailbox.recv());
        });
    });

    #[cfg(feature = "asynchronous")]
    c.bench_function("mailer empty", |b| {
        let mut channel = Mailer::unbounded();

        b.iter(|| {
            channel.send(());
            black_box(&mut channel);
        });
    });

    #[cfg(feature = "asynchronous")]
    c.bench_function("mailer one", |b| {
        let mut channel = Mailer::unbounded();

        let _mailbox = channel.mailbox();

        b.iter(|| {
            channel.send(());
            black_box(&mut channel);
        });
    });

    c.bench_function("emit", |b| {
        trait Trait {
            fn function(&mut self);
        }

        impl Trait for () {
            fn function(&mut self) {
                black_box(self);
            }
        }

        let mut channel: Channel<dyn Trait> = Channel::new();

        channel.insert(0, Node::new(()));

        b.iter(|| {
            channel.emit(|x| x.function());
            black_box(&mut channel);
        });
    });

    c.bench_function("recursion", |b| {
        trait Trait {
            fn function(&mut self, hub: &Hub);
        }
        trait Reset {
            fn reset(&mut self);
        }

        #[derive(Default)]
        struct Hub {
            channel: Channel<dyn Trait>,
            reset: Slot<dyn Reset>,
        }

        let mut hub = Hub::default();

        struct Subscriber {
            value: usize,
        };

        impl Trait for Subscriber {
            fn function(&mut self, hub: &Hub) {
                if self.value == 0 {
                    return;
                }

                self.value -= 1;

                self.suspend(|| {
                    hub.channel.emit(|item| {
                        item.function(hub);
                    });
                });

                black_box(self.value);
            }
        }

        impl Reset for Subscriber {
            fn reset(&mut self) {
                self.value = 1000;
            }
        }

        let x = Node::new(Subscriber { value: 1000 });
        hub.channel.insert(0, x.clone());
        hub.reset.insert(x);

        b.iter(|| {
            hub.channel.emit(|x| {
                x.function(&hub);
            });
            hub.reset.emit(|x| x.reset());

            black_box(&mut hub);
        });
    });

    c.bench_function("node access", |b| {
        let node = Node::new(());

        b.iter(|| {
            node.emit(|x| {
                black_box(x);
            });
        });
    });

    c.bench_function("node suspend", |b| {
        let node = Node::new(());

        b.iter(|| {
            node.emit(|x| {
                x.suspend(|| {
                    black_box(&node);
                });
            });
        });
    });
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
