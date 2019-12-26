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
            signal: fn(i32) -> (), Subsignals {
                something: fn(str) -> (), SomethingSignals,
            },
            something: fn(str) -> (), SomethingSignals {},
        }
    }

    // We create an example subscriber struct called X.
    struct X;

    // For every signal you wish to subscribe to, you must implement its function signature.
    // Handler for `signal: fn(i32) -> (), Subsignals {}`
    impl Subscriber<i32, (), Subsignals> for X {
        fn event(&mut self, input: &i32, subsignals: &Subsignals) {
            println!("Event received, value: {}", input);
            subsignals.something("An event from an `fn(i32) -> ()` subscriber");
        }
    }

    // Tells the Hub which signals to subscribe to.
    impl Selfscribe<Sub> for X {
        fn subscribe(&mut self, this: &Hubbed<Self>, hub: &mut Sub) {
            // Subscribe itself to `signal`.
            hub.signal(this);
        }
    }

    // We create another subscriber of type Y.
    struct Y;

    impl Subscriber<str, (), SomethingSignals> for Y {
        fn event(&mut self, input: &str, _: &SomethingSignals) {
            println!("Event received, value: {}", input);
        }
    }

    impl Selfscribe<Sub> for Y {
        fn subscribe(&mut self, this: &Hubbed<Self>, hub: &mut Sub) {
            hub.something(this);
        }
    }

    // Create a hub.
    let hub = Hub::default();

    // Add X and Y to the hub.
    hub.subscribe(X);
    hub.subscribe(Y);

    // Emit the signal `signal` in the hub.
    hub.signal(&123);
}
