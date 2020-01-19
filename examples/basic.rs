use revent::{hub, Subscriber};

// 1. Define your events using traits.
//
// You define your own traits for various event types.
// An object can listen to multiple event types by implementing multiple traits.
// In this example we just define a single trait for a single type.
pub trait EventHandler {
    fn event(&mut self);
}

// 2. Create an event hub using the `hub` macro.
//
// Takes care of the boilerplate code associated with the events and generates a struct (in this
// case `Hub`) that contains handles to each event channel.
hub! {
    Hub {
        // The format is `channel: dyn Trait`, where the trait is any trait and the channel is a
        // name for the event channel. You can have multiple channels of the same trait.
        // All these channels are comma-separated.
        //
        // In this example we have just 1 channel.
        basic: dyn EventHandler,
    }
}

fn main() {
    // 3. Construct a event hub.
    let mut hub = Hub::new();

    // 4. Create the derivative hub for the subscriber.
    //
    // This specifies which channels will be subscribed, which hubs can actually construct this
    // object, and which channels may be emitted into.
    hub! {
        HandlerHub: Hub {
        } subscribe MyEventHandler {
            basic,
        }
    }

    // 5. Add instances of the hub traits.
    //
    // We implement the trait for a type so it can be inserted into a channel in the hub.
    struct MyEventHandler;
    impl EventHandler for MyEventHandler {
        fn event(&mut self) {
            println!("Hello world");
        }
    }

    // 6. Implement `Subscriber` for the event handler.
    //
    // Subscriber informs the hub how to build and subscribe the type to other channels. It also
    // ensures that we don't have any recursive subscriptions.
    impl Subscriber for MyEventHandler {
        type Hub = HandlerHub;
        type Input = ();
        fn build(_: Self::Hub, _: Self::Input) -> Self {
            // We just construct the struct, no need to do anything special here in
            // this specific example.
            MyEventHandler
        }
    }

    // 7. Construct an instance inside the hub.
    //
    // We must construct an instance inside the hub because we may want to copy the `hub` inside
    // the object itself so it can emit further signals. This hub copy has only a selected amount
    // of signals enabled so we can avoid recursive event signals.
    hub.subscribe::<MyEventHandler>(());

    // 8. Emit events.
    //
    // We simply call `emit` on the topic we'd like to emit an event to. We then give a lambda that
    // takes a reference to the `dyn Trait` for that signal. This allows us to use the return type
    // or send in complex types with lifetime parameters.
    //
    // The lambda is called for every subscriber in the channel in arbitrary order.
    hub.basic.emit(|x| {
        x.event();
    });

    // 9. Remove the subscriber.
    //
    // To showcase how we can remove subscribers, returning true from the closure removes the
    // subscriber. Returning false leaves it be.
    // Removing a subscriber like this only removes it from the given channel. A subscriber may
    // have subscribed to multiple event channels.
    hub.basic.remove(|_| true);
}
