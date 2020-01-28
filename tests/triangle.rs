use revent::{hub, node, Subscriber};

pub trait A {
    fn a(&mut self, from: &str);
}
pub trait B {
    fn b(&mut self, from: &str);
}
pub trait C {
    fn c(&mut self, from: &str);
}

hub! {
    Hub {
        a: A,
        b: B,
        c: C,
    }
}

struct X {
    node: NodeX,
}
node! {
    Hub {
        a: A,
    } => NodeX(X) {
        b: B,
        c: C,
    }
}
impl Subscriber for X {
    type Input = ();
    fn build(node: Self::Node, _: Self::Input) -> Self {
        Self { node }
    }
}
impl A for X {
    fn a(&mut self, from: &str) {
        println!("a: from {}", from);
        self.node.b().emit(|x| x.b("A"));
        self.node.c().emit(|x| x.c("A"));
    }
}

struct Y {
    node: NodeY,
}
node! {
    Hub {
        b: B,
    } => NodeY(Y) {
        c: C,
    }
}
impl Subscriber for Y {
    type Input = ();
    fn build(node: Self::Node, _: Self::Input) -> Self {
        Self { node }
    }
}
impl B for Y {
    fn b(&mut self, from: &str) {
        println!("b: from {}", from);
        self.node.c().emit(|x| x.c("B"));
    }
}

struct Z;
node! {
    Hub {
        c: C,
    } => NodeZ(Z) {
    }
}
impl Subscriber for Z {
    type Input = ();
    fn build(_: Self::Node, _: Self::Input) -> Self {
        Self
    }
}
impl C for Z {
    fn c(&mut self, from: &str) {
        println!("c: from {}", from);
    }
}

#[test]
fn triangle() {
    let mut hub = Hub::new();
    hub.subscribe::<X>(());
    hub.subscribe::<Y>(());
    hub.subscribe::<Z>(());

    hub.a().emit(|x| x.a("MAIN"));
}
