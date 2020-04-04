use revent::{Anchor, Manager, Named, Slot, Subscriber};
use std::{cell::RefCell, rc::Rc};

// 1. Define your events using traits.
//
// You define your own traits for various event types.
// An object can listen to multiple event types by implementing multiple traits.
// In this example we just define a single trait for a single type.
pub trait EventHandler {
    fn event(&mut self);
}

// 2. Create an event node - a collection of event slots.
#[derive(Debug)]
struct Hub {
    basic: Slot<dyn EventHandler>,
    manager: Rc<RefCell<Manager>>,
}

// 2a. The node only needs to implement `manager`.
impl Anchor for Hub {
    fn manager(&self) -> &Rc<RefCell<Manager>> {
        &self.manager
    }
}

// 2b. Construct the new node. A manager must be supplied to all slots.
impl Hub {
    fn new() -> Self {
        let manager = Rc::new(RefCell::new(Manager::new()));
        Self {
            basic: Slot::new("basic", manager.clone()),
            manager,
        }
    }
}

fn main() {
    // 3. Construct a new node.
    let mut hub = Hub::new();

    // 4. Add instances of the hub traits.
    //
    // We implement the trait for a type so it can be inserted into a channel in the hub.
    struct MyEventHandler;
    impl EventHandler for MyEventHandler {
        fn event(&mut self) {
            println!("MyEventHandler: Hello world");
        }
    }

    // 5. Implement `Subscriber` for the event handler.
    //
    // Subscriber informs the hub how to build and subscribe the type to slots. It also
    // ensures that we don't have any recursive subscriptions.
    impl Subscriber<Hub> for MyEventHandler {
        type Emitter = revent::Null;
        fn register(node: &mut Hub, item: Rc<RefCell<Self>>) {
            // Tells the hub node which slots to listen to.
            // node.basic.register(item.clone());
            node.basic.register(item);
            // node.basic.clone();
        }
    }

    impl Named for MyEventHandler {
        const NAME: &'static str = "MyEventHandler";
    }

    // 6. Construct an instance inside the hub.
    let item = hub.subscribe(|_| MyEventHandler);

    // 7. Emit events.
    //
    // We simply call `emit` on the slot we'd like to emit an event to. We then give a lambda that
    // takes a reference to the `dyn Trait` for that signal handler. This allows us to use the return type
    // or send in complex types with lifetime parameters.
    //
    // The lambda is called for every subscriber in the slot in subscription order.
    hub.basic.emit(|x| {
        x.event();
    });

    println!("{:#?}", hub);

    // 8. Remove the subscriber.
    //
    // To showcase how we can remove subscribers, just insert the item returned from `subscribe`.
    // This uses the item's `register` method to figure out which slots to unsubscribe from.
    hub.unsubscribe(&item);
}
