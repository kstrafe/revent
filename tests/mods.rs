mod other {
    use revent::hub;

    pub trait A {}
    hub! {
        X {
            a: A,
        }
    }
}

mod name {
    use super::other::*;
    use revent::node;

    node! {
        X {
            a: A,
        } => Node(NodeStruct) {
        }
    }

    struct NodeStruct;

    impl A for NodeStruct {}
}

use other::X;

#[test]
fn from_other_module() {
    let _ = X::new();
}
