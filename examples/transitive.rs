// This example shows how you can emit events from within an event handler.
use revent::{hub, Subscriber};

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
    hub! {
        HandlerHub1: Hub {
            event2: dyn Event2Handler,
        } subscribe MyEvent1Handler {
            event1,
        }
    }
    struct MyEvent1Handler {
        hub: HandlerHub1,
    }
    impl Event1Handler for MyEvent1Handler {
        fn event(&mut self) {
            println!("Event 1");
            self.hub.event2.emit(|x| {
                x.event();
            });
        }
    }
    impl Subscriber for MyEvent1Handler {
        type Hub = HandlerHub1;
        type Input = ();
        fn build(hub: Self::Hub, _: Self::Input) -> Self {
            Self { hub }
        }
    }

    // Make the handler for `event2`.
    hub! {
        HandlerHub2: Hub {
        } subscribe MyEvent2Handler {
            event2,
        }
    }
    struct MyEvent2Handler;
    impl Event2Handler for MyEvent2Handler {
        fn event(&mut self) {
            println!("Event 2");
        }
    }
    impl Subscriber for MyEvent2Handler {
        type Hub = HandlerHub2;
        type Input = ();
        fn build(_: Self::Hub, _: Self::Input) -> Self {
            MyEvent2Handler
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
