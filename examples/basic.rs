use revent::{hub, Shared, Subscriber};

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
    let hub = Hub::new();

    // 4. Add instances of the hub traits.
    //
    // We implement the trait for a type so it can be inserted into a channel in the hub.
    struct MyEventHandler;
    impl EventHandler for MyEventHandler {
        fn event(&mut self) {
            println!("Hello world");
        }
    }

    // 5. Implement `Subscriber` for the event handler.
    //
    // Subscriber informs the hub how to build and subscribe the type to other channels. It also
    // ensures that we don't have any recursive subscriptions.
    impl Subscriber<Hub> for MyEventHandler {
        type Input = ();
        fn build(_: Hub, _: Self::Input) -> Self {
            // We just construct the struct, no need to do anything special here in
            // this specific example.
            MyEventHandler
        }

        fn subscribe(hub: &Hub, shared: Shared<Self>) {
            // Here we inform which channels we'd like to subscribe to.
            hub.basic.subscribe(shared);
        }
    }

    // 6. Construct an instance inside the hub.
    //
    // We must construct an instance inside the hub because we may want to copy the `hub` inside
    // the object itself so it can emit further signals. This hub copy has only a selected amount
    // of signals enabled so we can avoid recursive event signals.
    hub.subscribe::<MyEventHandler>(());

    // 7. Emit events.
    //
    // We simply call `emit` on the topic we'd like to emit an event to. We then give a lambda that
    // takes a reference to the `dyn Trait` for that signal. This allows us to use the return type
    // or send in complex types with lifetime parameters.
    //
    // The lambda is called for every subscriber in the channel in arbitrary order.
    hub.basic.emit(|x| {
        x.event();
    });
}
