use revent::{hub, Hubbed, Selfscribe, Subscriber};

fn main() {
    // Create a hub.
    hub! {
        // Module for holding helper structs and other things, a user does not need to
        // care about its contents.
        mod x;
        // Hub and Sub. The Hub is the top-level signal emitter; from the `Hub` you emit
        // signals. The `Sub` is similar to the `Hub` but allows one to subscribe to
        // signals, and is used only in the `Selfscribe` trait.
        Hub(Sub) {
            // A signal is just a function with the ability to
            // propagate further signals to `Subsignals`. You can decide which names
            // to use here.
            // Note that in this case `Subsignals` contains no further signals. See
            // the `examples/subsignals.rs` file for such an example.
            signal: fn(i32) -> (), Subsignals {},
        }
    }

    // We create an example subscriber struct called X.
    struct X;

    // For every signal you wish to subscribe to, you must implement its function signature.
    // Handler for `signal: fn(i32) -> (), Subsignals {}`
    impl Subscriber<i32, (), Subsignals> for X {
        fn event(&mut self, input: &i32, _: &Subsignals) {
            println!("Event received, value: {}", input);
        }
    }

    // Tells the Hub which signals to subscribe to.
    impl Selfscribe<Sub> for X {
        fn subscribe(&mut self, this: &Hubbed<Self>, hub: &mut Sub) {
            // Subscribe itself to `signal`.
            hub.signal(this);
        }
    }

    // Create a hub.
    let hub = Hub::default();

    // Add X to the hub.
    hub.subscribe(X);

    // Emit the signal `signal` in the hub.
    hub.signal(&123);
}
