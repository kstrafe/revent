// Shows how we can use [remove](revent::Signal::remove) to replace objects with new ones for use
// in state transitions.
//
// This example shows a transition from `A` to `B`.
//
// Note: This transition applies only to the signal `update`. If `A` subscribes to any other
// signal, the code inside main will _not_ remove `A` from said signal.
// Transitions are mainly intended for single-listener nodes.
use revent::{hub, node, Subscriber};

hub! {
    Hub {
        update: Update,
    }
}

pub trait Update {
    fn update(&mut self) -> Option<Box<dyn Fn(&mut Hub)>>;
}

// ---

node! {
    Hub {
        update: Update,
    } => NodeA(A) {
    }
}
struct A;

impl Update for A {
    fn update(&mut self) -> Option<Box<dyn Fn(&mut Hub)>> {
        println!("Update: A");
        Some(Box::new(|hub| {
            hub.subscribe::<B>(());
        }))
    }
}

impl Subscriber for A {
    type Input = ();
    fn build(_: Self::Node, _: Self::Input) -> Self {
        println!("build: A");
        Self
    }
}

impl Drop for A {
    fn drop(&mut self) {
        println!("Drop: A");
    }
}

// ---

node! {
    Hub {
        update: Update,
    } => NodeB(B) {
    }
}

struct B;

impl Update for B {
    fn update(&mut self) -> Option<Box<dyn Fn(&mut Hub)>> {
        println!("Update: B");
        None
    }
}

impl Subscriber for B {
    type Input = ();
    fn build(_: Self::Node, _: Self::Input) -> Self {
        println!("build: B");
        Self
    }
}

impl Drop for B {
    fn drop(&mut self) {
        println!("Drop: B");
    }
}

// ---

fn main() {
    let mut hub = Hub::new();

    // Begin by adding `A` to the hub.
    hub.subscribe::<A>(());

    // Store transition functions inside this vector.
    let mut transitions: Vec<Box<dyn Fn(&mut Hub)>> = vec![];

    // An `Update` object can return Option<...>, if it is Some(...), then the intent is to
    // transition, so the object is removed. The function inside the `Some(...)` is then put into a
    // list so that it can spawn a new object later.
    //
    // If the return value is `None`, then we decide to keep the object.
    hub.update().remove(|x| {
        let transition = x.update();
        let remove = transition.is_some();
        if let Some(item) = transition {
            transitions.push(item);
        }
        remove
    });

    // Create the new items.
    for item in transitions.drain(..) {
        item(&mut hub);
    }

    // The hub should now contain our new items, this emit will cause `B` to be invoked. `A` has
    // been destroyed.
    hub.update().emit(|x| {
        x.update();
    });
}
