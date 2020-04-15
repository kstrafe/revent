use revent::{Channel, Node, Slot, Suspend};

// Create signal traits.
trait SignalA {
    fn signal_a(&mut self, hub: &MyHub);
}
trait SignalB {
    fn signal_b(&mut self, hub: &MyHub);
}
trait SignalC {
    fn signal_c(&mut self);
}

// Create a struct of channels and slots based on your signal traits.
#[derive(Default)]
struct MyHub {
    signal_a: Channel<dyn SignalA>,
    signal_b: Channel<dyn SignalB>, // A channel contains any number of nodes.
    signal_c: Slot<dyn SignalC>,    // A slot contains only a single node.
}

// Create trait implementors. Note that `A` implements both `SignalA` and `SignalB`.
struct A;
struct B;
struct C;

impl SignalA for A {
    fn signal_a(&mut self, hub: &MyHub) {
        println!("A::signal_a: {:?}", self as *mut _);

        self.suspend(|| {
            // Suspend here in order to not panic. `signal_b` also contains this
            hub.signal_b.emit(|x| {
                // object, so we must ensure we relinquish access to `&mut`.
                x.signal_b(hub);
            });
        });
    }
}
impl SignalB for A {
    fn signal_b(&mut self, _: &MyHub) {
        println!("A::signal_b: {:?}", self as *mut _);
    }
}
impl SignalB for B {
    fn signal_b(&mut self, hub: &MyHub) {
        println!("B::signal_b: {:?}", self as *mut _);
        hub.signal_c.emit(|x| {
            // We can also emit without suspending self. If the channel or
            // slot we emit into contains the object from which we emit, then a panic will occur.
            x.signal_c();
        });
    }
}
impl SignalC for C {
    fn signal_c(&mut self) {
        println!("C::signal_c: {:?}", self as *mut _);
    }
}

fn main() {
    // Instantiate `MyHub`.
    let mut hub = MyHub::default();

    // Insert nodes into the hub. Nodes can be cloned and used on their own using the `emit`
    // method.
    let a = Node::new(A);
    hub.signal_a.insert(0, a.clone());
    hub.signal_b.insert(0, a.clone());
    hub.signal_b.insert(0, Node::new(B));
    hub.signal_c.insert(Node::new(C));

    // Run `a` and call `signal_a`.
    a.emit(|x| {
        x.signal_a(&hub);
    });
}
