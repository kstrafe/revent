// This example shows how you can emit events from within an event handler.
use revent::{hub, Shared, Subscriber};

// First we declare two event channels (they _can_ have the same trait but they are different to
// make the example more clear).
pub trait Event1Handler {
    fn event(&mut self);
}

pub trait Event2Handler {
    fn event(&mut self);
}

hub! {
    Hub {
        event1: dyn Event1Handler,
        event2: dyn Event2Handler,
    }
}

fn main() {
    let mut hub = Hub::new();

    // Make the handler for `event1`.
    struct MyEvent1Handler {
        hub: Hub,
    }
    impl Event1Handler for MyEvent1Handler {
        fn event(&mut self) {
            println!("Event 1");
            self.hub.event2.emit(|x| {
                x.event();
            });
        }
    }
    impl Subscriber<Hub> for MyEvent1Handler {
        type Input = ();
        fn build(mut hub: Hub, _: Self::Input) -> Self {
            // The hub which `build` is called with has all its channels deactivated,
            // meaning that you need to manually activate them here. If you use a
            // channel that is inactive you will get an error at run-time. The reason
            // for doing this is so we can detect recursions without being dependent
            // on run-time state.
            hub.event2.activate();
            Self { hub }
        }

        fn subscribe(hub: &mut Hub, shared: Shared<Self>) {
            hub.event1.subscribe(shared);
        }
    }

    // Make the handler for `event2`.
    struct MyEvent2Handler;
    impl Event2Handler for MyEvent2Handler {
        fn event(&mut self) {
            println!("Event 2");
        }
    }
    impl Subscriber<Hub> for MyEvent2Handler {
        type Input = ();
        fn build(_: Hub, _: Self::Input) -> Self {
            MyEvent2Handler
        }
        fn subscribe(hub: &mut Hub, shared: Shared<Self>) {
            hub.event2.subscribe(shared);
        }
    }

    // Subscribe an instance of each handler.
    hub.subscribe::<MyEvent1Handler>(());
    hub.subscribe::<MyEvent2Handler>(());

    // Emit to `event1`.
    hub.event1.emit(|x| {
        x.event();
    });
}
