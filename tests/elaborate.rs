use revent::{hub, Shared, Subscriber};

pub trait X {}

hub! {
    Hub {
        x1: dyn X,
        x2: dyn X,
        x3: dyn X,
        x4: dyn X,
        x5: dyn X,
        x6: dyn X,
        x7: dyn X,
        x8: dyn X,
        x9: dyn X,
        x10: dyn X,
        x11: dyn X,
        x12: dyn X,
        x13: dyn X,
        x14: dyn X,
        x15: dyn X,
        x16: dyn X,
        x17: dyn X,
        x18: dyn X,
        x19: dyn X,
        x20: dyn X,
        x21: dyn X,
        x22: dyn X,
        x23: dyn X,
        x24: dyn X,
        x25: dyn X,
        x26: dyn X,
        x27: dyn X,
        x28: dyn X,
        x29: dyn X,
        x30: dyn X,
        x31: dyn X,
        x32: dyn X,
        x33: dyn X,
        x34: dyn X,
        x35: dyn X,
        x36: dyn X,
        x37: dyn X,
        x38: dyn X,
        x39: dyn X,
        x40: dyn X,
        x41: dyn X,
        x42: dyn X,
        x43: dyn X,
        x44: dyn X,
        x45: dyn X,
        x46: dyn X,
        x47: dyn X,
        x48: dyn X,
        x49: dyn X,
        x50: dyn X,
        x51: dyn X,
        x52: dyn X,
        x53: dyn X,
        x54: dyn X,
        x55: dyn X,
        x56: dyn X,
        x57: dyn X,
        x58: dyn X,
        x59: dyn X,
        x60: dyn X,
        x61: dyn X,
        x62: dyn X,
        x63: dyn X,
        x64: dyn X,
        x65: dyn X,
        x66: dyn X,
        x67: dyn X,
        x68: dyn X,
        x69: dyn X,
        x70: dyn X,
        x71: dyn X,
        x72: dyn X,
        x73: dyn X,
        x74: dyn X,
        x75: dyn X,
        x76: dyn X,
        x77: dyn X,
        x78: dyn X,
        x79: dyn X,
        x80: dyn X,
        x81: dyn X,
        x82: dyn X,
        x83: dyn X,
        x84: dyn X,
        x85: dyn X,
        x86: dyn X,
        x87: dyn X,
        x88: dyn X,
        x89: dyn X,
        x90: dyn X,
        x91: dyn X,
        x92: dyn X,
        x93: dyn X,
        x94: dyn X,
        x95: dyn X,
        x96: dyn X,
        x97: dyn X,
        x98: dyn X,
        x99: dyn X,
        x100: dyn X,
    }
}

#[test]
#[should_panic(
    expected = "Recursion detected: [\"x1\", \"x100\", \"x99\", \"x98\", \"x97\", \"x96\", \"x95\", \"x94\", \"x93\", \"x92\", \"x91\", \"x90\", \"x89\", \"x88\", \"x87\", \"x86\", \"x85\", \"x84\", \"x83\", \"x82\", \"x81\", \"x80\", \"x79\", \"x78\", \"x77\", \"x76\", \"x75\", \"x74\", \"x73\", \"x72\", \"x71\", \"x70\", \"x69\", \"x68\", \"x67\", \"x66\", \"x65\", \"x64\", \"x63\", \"x62\", \"x61\", \"x60\", \"x59\", \"x58\", \"x57\", \"x56\", \"x55\", \"x54\", \"x53\", \"x52\", \"x51\", \"x50\", \"x49\", \"x48\", \"x47\", \"x46\", \"x45\", \"x44\", \"x43\", \"x42\", \"x41\", \"x40\", \"x39\", \"x38\", \"x37\", \"x36\", \"x35\", \"x34\", \"x33\", \"x32\", \"x31\", \"x30\", \"x29\", \"x28\", \"x27\", \"x26\", \"x25\", \"x24\", \"x23\", \"x22\", \"x21\", \"x20\", \"x19\", \"x18\", \"x17\", \"x16\", \"x15\", \"x14\", \"x13\", \"x12\", \"x11\", \"x10\", \"x9\", \"x8\", \"x7\", \"x6\", \"x5\", \"x4\", \"x3\", \"x2\"]"
)]
fn elaborate() {
    let hub = Hub::new();

    hub.subscribe::<X1>(());
    hub.subscribe::<X2>(());
    hub.subscribe::<X3>(());
    hub.subscribe::<X4>(());
    hub.subscribe::<X5>(());
    hub.subscribe::<X6>(());
    hub.subscribe::<X7>(());
    hub.subscribe::<X8>(());
    hub.subscribe::<X9>(());
    hub.subscribe::<X10>(());
    hub.subscribe::<X11>(());
    hub.subscribe::<X12>(());
    hub.subscribe::<X13>(());
    hub.subscribe::<X14>(());
    hub.subscribe::<X15>(());
    hub.subscribe::<X16>(());
    hub.subscribe::<X17>(());
    hub.subscribe::<X18>(());
    hub.subscribe::<X19>(());
    hub.subscribe::<X20>(());
    hub.subscribe::<X21>(());
    hub.subscribe::<X22>(());
    hub.subscribe::<X23>(());
    hub.subscribe::<X24>(());
    hub.subscribe::<X25>(());
    hub.subscribe::<X26>(());
    hub.subscribe::<X27>(());
    hub.subscribe::<X28>(());
    hub.subscribe::<X29>(());
    hub.subscribe::<X30>(());
    hub.subscribe::<X31>(());
    hub.subscribe::<X32>(());
    hub.subscribe::<X33>(());
    hub.subscribe::<X34>(());
    hub.subscribe::<X35>(());
    hub.subscribe::<X36>(());
    hub.subscribe::<X37>(());
    hub.subscribe::<X38>(());
    hub.subscribe::<X39>(());
    hub.subscribe::<X40>(());
    hub.subscribe::<X41>(());
    hub.subscribe::<X42>(());
    hub.subscribe::<X43>(());
    hub.subscribe::<X44>(());
    hub.subscribe::<X45>(());
    hub.subscribe::<X46>(());
    hub.subscribe::<X47>(());
    hub.subscribe::<X48>(());
    hub.subscribe::<X49>(());
    hub.subscribe::<X50>(());
    hub.subscribe::<X51>(());
    hub.subscribe::<X52>(());
    hub.subscribe::<X53>(());
    hub.subscribe::<X54>(());
    hub.subscribe::<X55>(());
    hub.subscribe::<X56>(());
    hub.subscribe::<X57>(());
    hub.subscribe::<X58>(());
    hub.subscribe::<X59>(());
    hub.subscribe::<X60>(());
    hub.subscribe::<X61>(());
    hub.subscribe::<X62>(());
    hub.subscribe::<X63>(());
    hub.subscribe::<X64>(());
    hub.subscribe::<X65>(());
    hub.subscribe::<X66>(());
    hub.subscribe::<X67>(());
    hub.subscribe::<X68>(());
    hub.subscribe::<X69>(());
    hub.subscribe::<X70>(());
    hub.subscribe::<X71>(());
    hub.subscribe::<X72>(());
    hub.subscribe::<X73>(());
    hub.subscribe::<X74>(());
    hub.subscribe::<X75>(());
    hub.subscribe::<X76>(());
    hub.subscribe::<X77>(());
    hub.subscribe::<X78>(());
    hub.subscribe::<X79>(());
    hub.subscribe::<X80>(());
    hub.subscribe::<X81>(());
    hub.subscribe::<X82>(());
    hub.subscribe::<X83>(());
    hub.subscribe::<X84>(());
    hub.subscribe::<X85>(());
    hub.subscribe::<X86>(());
    hub.subscribe::<X87>(());
    hub.subscribe::<X88>(());
    hub.subscribe::<X89>(());
    hub.subscribe::<X90>(());
    hub.subscribe::<X91>(());
    hub.subscribe::<X92>(());
    hub.subscribe::<X93>(());
    hub.subscribe::<X94>(());
    hub.subscribe::<X95>(());
    hub.subscribe::<X96>(());
    hub.subscribe::<X97>(());
    hub.subscribe::<X98>(());
    hub.subscribe::<X99>(());
    hub.subscribe::<X100>(());
}

struct X1;
impl X for X1 {}
impl Subscriber<Hub> for X1 {
    type Input = ();
    fn build(mut hub: Hub, _: Self::Input) -> Self {
        hub.x100.activate();
        Self
    }
    fn subscribe(hub: &Hub, shared: Shared<Self>) {
        hub.x1.subscribe(shared);
    }
}
struct X2;
impl X for X2 {}
impl Subscriber<Hub> for X2 {
    type Input = ();
    fn build(mut hub: Hub, _: Self::Input) -> Self {
        hub.x1.activate();
        Self
    }
    fn subscribe(hub: &Hub, shared: Shared<Self>) {
        hub.x2.subscribe(shared);
    }
}
struct X3;
impl X for X3 {}
impl Subscriber<Hub> for X3 {
    type Input = ();
    fn build(mut hub: Hub, _: Self::Input) -> Self {
        hub.x2.activate();
        Self
    }
    fn subscribe(hub: &Hub, shared: Shared<Self>) {
        hub.x3.subscribe(shared);
    }
}
struct X4;
impl X for X4 {}
impl Subscriber<Hub> for X4 {
    type Input = ();
    fn build(mut hub: Hub, _: Self::Input) -> Self {
        hub.x3.activate();
        Self
    }
    fn subscribe(hub: &Hub, shared: Shared<Self>) {
        hub.x4.subscribe(shared);
    }
}
struct X5;
impl X for X5 {}
impl Subscriber<Hub> for X5 {
    type Input = ();
    fn build(mut hub: Hub, _: Self::Input) -> Self {
        hub.x4.activate();
        Self
    }
    fn subscribe(hub: &Hub, shared: Shared<Self>) {
        hub.x5.subscribe(shared);
    }
}
struct X6;
impl X for X6 {}
impl Subscriber<Hub> for X6 {
    type Input = ();
    fn build(mut hub: Hub, _: Self::Input) -> Self {
        hub.x5.activate();
        Self
    }
    fn subscribe(hub: &Hub, shared: Shared<Self>) {
        hub.x6.subscribe(shared);
    }
}
struct X7;
impl X for X7 {}
impl Subscriber<Hub> for X7 {
    type Input = ();
    fn build(mut hub: Hub, _: Self::Input) -> Self {
        hub.x6.activate();
        Self
    }
    fn subscribe(hub: &Hub, shared: Shared<Self>) {
        hub.x7.subscribe(shared);
    }
}
struct X8;
impl X for X8 {}
impl Subscriber<Hub> for X8 {
    type Input = ();
    fn build(mut hub: Hub, _: Self::Input) -> Self {
        hub.x7.activate();
        Self
    }
    fn subscribe(hub: &Hub, shared: Shared<Self>) {
        hub.x8.subscribe(shared);
    }
}
struct X9;
impl X for X9 {}
impl Subscriber<Hub> for X9 {
    type Input = ();
    fn build(mut hub: Hub, _: Self::Input) -> Self {
        hub.x8.activate();
        Self
    }
    fn subscribe(hub: &Hub, shared: Shared<Self>) {
        hub.x9.subscribe(shared);
    }
}
struct X10;
impl X for X10 {}
impl Subscriber<Hub> for X10 {
    type Input = ();
    fn build(mut hub: Hub, _: Self::Input) -> Self {
        hub.x9.activate();
        Self
    }
    fn subscribe(hub: &Hub, shared: Shared<Self>) {
        hub.x10.subscribe(shared);
    }
}
struct X11;
impl X for X11 {}
impl Subscriber<Hub> for X11 {
    type Input = ();
    fn build(mut hub: Hub, _: Self::Input) -> Self {
        hub.x10.activate();
        Self
    }
    fn subscribe(hub: &Hub, shared: Shared<Self>) {
        hub.x11.subscribe(shared);
    }
}
struct X12;
impl X for X12 {}
impl Subscriber<Hub> for X12 {
    type Input = ();
    fn build(mut hub: Hub, _: Self::Input) -> Self {
        hub.x11.activate();
        Self
    }
    fn subscribe(hub: &Hub, shared: Shared<Self>) {
        hub.x12.subscribe(shared);
    }
}
struct X13;
impl X for X13 {}
impl Subscriber<Hub> for X13 {
    type Input = ();
    fn build(mut hub: Hub, _: Self::Input) -> Self {
        hub.x12.activate();
        Self
    }
    fn subscribe(hub: &Hub, shared: Shared<Self>) {
        hub.x13.subscribe(shared);
    }
}
struct X14;
impl X for X14 {}
impl Subscriber<Hub> for X14 {
    type Input = ();
    fn build(mut hub: Hub, _: Self::Input) -> Self {
        hub.x13.activate();
        Self
    }
    fn subscribe(hub: &Hub, shared: Shared<Self>) {
        hub.x14.subscribe(shared);
    }
}
struct X15;
impl X for X15 {}
impl Subscriber<Hub> for X15 {
    type Input = ();
    fn build(mut hub: Hub, _: Self::Input) -> Self {
        hub.x14.activate();
        Self
    }
    fn subscribe(hub: &Hub, shared: Shared<Self>) {
        hub.x15.subscribe(shared);
    }
}
struct X16;
impl X for X16 {}
impl Subscriber<Hub> for X16 {
    type Input = ();
    fn build(mut hub: Hub, _: Self::Input) -> Self {
        hub.x15.activate();
        Self
    }
    fn subscribe(hub: &Hub, shared: Shared<Self>) {
        hub.x16.subscribe(shared);
    }
}
struct X17;
impl X for X17 {}
impl Subscriber<Hub> for X17 {
    type Input = ();
    fn build(mut hub: Hub, _: Self::Input) -> Self {
        hub.x16.activate();
        Self
    }
    fn subscribe(hub: &Hub, shared: Shared<Self>) {
        hub.x17.subscribe(shared);
    }
}
struct X18;
impl X for X18 {}
impl Subscriber<Hub> for X18 {
    type Input = ();
    fn build(mut hub: Hub, _: Self::Input) -> Self {
        hub.x17.activate();
        Self
    }
    fn subscribe(hub: &Hub, shared: Shared<Self>) {
        hub.x18.subscribe(shared);
    }
}
struct X19;
impl X for X19 {}
impl Subscriber<Hub> for X19 {
    type Input = ();
    fn build(mut hub: Hub, _: Self::Input) -> Self {
        hub.x18.activate();
        Self
    }
    fn subscribe(hub: &Hub, shared: Shared<Self>) {
        hub.x19.subscribe(shared);
    }
}
struct X20;
impl X for X20 {}
impl Subscriber<Hub> for X20 {
    type Input = ();
    fn build(mut hub: Hub, _: Self::Input) -> Self {
        hub.x19.activate();
        Self
    }
    fn subscribe(hub: &Hub, shared: Shared<Self>) {
        hub.x20.subscribe(shared);
    }
}
struct X21;
impl X for X21 {}
impl Subscriber<Hub> for X21 {
    type Input = ();
    fn build(mut hub: Hub, _: Self::Input) -> Self {
        hub.x20.activate();
        Self
    }
    fn subscribe(hub: &Hub, shared: Shared<Self>) {
        hub.x21.subscribe(shared);
    }
}
struct X22;
impl X for X22 {}
impl Subscriber<Hub> for X22 {
    type Input = ();
    fn build(mut hub: Hub, _: Self::Input) -> Self {
        hub.x21.activate();
        Self
    }
    fn subscribe(hub: &Hub, shared: Shared<Self>) {
        hub.x22.subscribe(shared);
    }
}
struct X23;
impl X for X23 {}
impl Subscriber<Hub> for X23 {
    type Input = ();
    fn build(mut hub: Hub, _: Self::Input) -> Self {
        hub.x22.activate();
        Self
    }
    fn subscribe(hub: &Hub, shared: Shared<Self>) {
        hub.x23.subscribe(shared);
    }
}
struct X24;
impl X for X24 {}
impl Subscriber<Hub> for X24 {
    type Input = ();
    fn build(mut hub: Hub, _: Self::Input) -> Self {
        hub.x23.activate();
        Self
    }
    fn subscribe(hub: &Hub, shared: Shared<Self>) {
        hub.x24.subscribe(shared);
    }
}
struct X25;
impl X for X25 {}
impl Subscriber<Hub> for X25 {
    type Input = ();
    fn build(mut hub: Hub, _: Self::Input) -> Self {
        hub.x24.activate();
        Self
    }
    fn subscribe(hub: &Hub, shared: Shared<Self>) {
        hub.x25.subscribe(shared);
    }
}
struct X26;
impl X for X26 {}
impl Subscriber<Hub> for X26 {
    type Input = ();
    fn build(mut hub: Hub, _: Self::Input) -> Self {
        hub.x25.activate();
        Self
    }
    fn subscribe(hub: &Hub, shared: Shared<Self>) {
        hub.x26.subscribe(shared);
    }
}
struct X27;
impl X for X27 {}
impl Subscriber<Hub> for X27 {
    type Input = ();
    fn build(mut hub: Hub, _: Self::Input) -> Self {
        hub.x26.activate();
        Self
    }
    fn subscribe(hub: &Hub, shared: Shared<Self>) {
        hub.x27.subscribe(shared);
    }
}
struct X28;
impl X for X28 {}
impl Subscriber<Hub> for X28 {
    type Input = ();
    fn build(mut hub: Hub, _: Self::Input) -> Self {
        hub.x27.activate();
        Self
    }
    fn subscribe(hub: &Hub, shared: Shared<Self>) {
        hub.x28.subscribe(shared);
    }
}
struct X29;
impl X for X29 {}
impl Subscriber<Hub> for X29 {
    type Input = ();
    fn build(mut hub: Hub, _: Self::Input) -> Self {
        hub.x28.activate();
        Self
    }
    fn subscribe(hub: &Hub, shared: Shared<Self>) {
        hub.x29.subscribe(shared);
    }
}
struct X30;
impl X for X30 {}
impl Subscriber<Hub> for X30 {
    type Input = ();
    fn build(mut hub: Hub, _: Self::Input) -> Self {
        hub.x29.activate();
        Self
    }
    fn subscribe(hub: &Hub, shared: Shared<Self>) {
        hub.x30.subscribe(shared);
    }
}
struct X31;
impl X for X31 {}
impl Subscriber<Hub> for X31 {
    type Input = ();
    fn build(mut hub: Hub, _: Self::Input) -> Self {
        hub.x30.activate();
        Self
    }
    fn subscribe(hub: &Hub, shared: Shared<Self>) {
        hub.x31.subscribe(shared);
    }
}
struct X32;
impl X for X32 {}
impl Subscriber<Hub> for X32 {
    type Input = ();
    fn build(mut hub: Hub, _: Self::Input) -> Self {
        hub.x31.activate();
        Self
    }
    fn subscribe(hub: &Hub, shared: Shared<Self>) {
        hub.x32.subscribe(shared);
    }
}
struct X33;
impl X for X33 {}
impl Subscriber<Hub> for X33 {
    type Input = ();
    fn build(mut hub: Hub, _: Self::Input) -> Self {
        hub.x32.activate();
        Self
    }
    fn subscribe(hub: &Hub, shared: Shared<Self>) {
        hub.x33.subscribe(shared);
    }
}
struct X34;
impl X for X34 {}
impl Subscriber<Hub> for X34 {
    type Input = ();
    fn build(mut hub: Hub, _: Self::Input) -> Self {
        hub.x33.activate();
        Self
    }
    fn subscribe(hub: &Hub, shared: Shared<Self>) {
        hub.x34.subscribe(shared);
    }
}
struct X35;
impl X for X35 {}
impl Subscriber<Hub> for X35 {
    type Input = ();
    fn build(mut hub: Hub, _: Self::Input) -> Self {
        hub.x34.activate();
        Self
    }
    fn subscribe(hub: &Hub, shared: Shared<Self>) {
        hub.x35.subscribe(shared);
    }
}
struct X36;
impl X for X36 {}
impl Subscriber<Hub> for X36 {
    type Input = ();
    fn build(mut hub: Hub, _: Self::Input) -> Self {
        hub.x35.activate();
        Self
    }
    fn subscribe(hub: &Hub, shared: Shared<Self>) {
        hub.x36.subscribe(shared);
    }
}
struct X37;
impl X for X37 {}
impl Subscriber<Hub> for X37 {
    type Input = ();
    fn build(mut hub: Hub, _: Self::Input) -> Self {
        hub.x36.activate();
        Self
    }
    fn subscribe(hub: &Hub, shared: Shared<Self>) {
        hub.x37.subscribe(shared);
    }
}
struct X38;
impl X for X38 {}
impl Subscriber<Hub> for X38 {
    type Input = ();
    fn build(mut hub: Hub, _: Self::Input) -> Self {
        hub.x37.activate();
        Self
    }
    fn subscribe(hub: &Hub, shared: Shared<Self>) {
        hub.x38.subscribe(shared);
    }
}
struct X39;
impl X for X39 {}
impl Subscriber<Hub> for X39 {
    type Input = ();
    fn build(mut hub: Hub, _: Self::Input) -> Self {
        hub.x38.activate();
        Self
    }
    fn subscribe(hub: &Hub, shared: Shared<Self>) {
        hub.x39.subscribe(shared);
    }
}
struct X40;
impl X for X40 {}
impl Subscriber<Hub> for X40 {
    type Input = ();
    fn build(mut hub: Hub, _: Self::Input) -> Self {
        hub.x39.activate();
        Self
    }
    fn subscribe(hub: &Hub, shared: Shared<Self>) {
        hub.x40.subscribe(shared);
    }
}
struct X41;
impl X for X41 {}
impl Subscriber<Hub> for X41 {
    type Input = ();
    fn build(mut hub: Hub, _: Self::Input) -> Self {
        hub.x40.activate();
        Self
    }
    fn subscribe(hub: &Hub, shared: Shared<Self>) {
        hub.x41.subscribe(shared);
    }
}
struct X42;
impl X for X42 {}
impl Subscriber<Hub> for X42 {
    type Input = ();
    fn build(mut hub: Hub, _: Self::Input) -> Self {
        hub.x41.activate();
        Self
    }
    fn subscribe(hub: &Hub, shared: Shared<Self>) {
        hub.x42.subscribe(shared);
    }
}
struct X43;
impl X for X43 {}
impl Subscriber<Hub> for X43 {
    type Input = ();
    fn build(mut hub: Hub, _: Self::Input) -> Self {
        hub.x42.activate();
        Self
    }
    fn subscribe(hub: &Hub, shared: Shared<Self>) {
        hub.x43.subscribe(shared);
    }
}
struct X44;
impl X for X44 {}
impl Subscriber<Hub> for X44 {
    type Input = ();
    fn build(mut hub: Hub, _: Self::Input) -> Self {
        hub.x43.activate();
        Self
    }
    fn subscribe(hub: &Hub, shared: Shared<Self>) {
        hub.x44.subscribe(shared);
    }
}
struct X45;
impl X for X45 {}
impl Subscriber<Hub> for X45 {
    type Input = ();
    fn build(mut hub: Hub, _: Self::Input) -> Self {
        hub.x44.activate();
        Self
    }
    fn subscribe(hub: &Hub, shared: Shared<Self>) {
        hub.x45.subscribe(shared);
    }
}
struct X46;
impl X for X46 {}
impl Subscriber<Hub> for X46 {
    type Input = ();
    fn build(mut hub: Hub, _: Self::Input) -> Self {
        hub.x45.activate();
        Self
    }
    fn subscribe(hub: &Hub, shared: Shared<Self>) {
        hub.x46.subscribe(shared);
    }
}
struct X47;
impl X for X47 {}
impl Subscriber<Hub> for X47 {
    type Input = ();
    fn build(mut hub: Hub, _: Self::Input) -> Self {
        hub.x46.activate();
        Self
    }
    fn subscribe(hub: &Hub, shared: Shared<Self>) {
        hub.x47.subscribe(shared);
    }
}
struct X48;
impl X for X48 {}
impl Subscriber<Hub> for X48 {
    type Input = ();
    fn build(mut hub: Hub, _: Self::Input) -> Self {
        hub.x47.activate();
        Self
    }
    fn subscribe(hub: &Hub, shared: Shared<Self>) {
        hub.x48.subscribe(shared);
    }
}
struct X49;
impl X for X49 {}
impl Subscriber<Hub> for X49 {
    type Input = ();
    fn build(mut hub: Hub, _: Self::Input) -> Self {
        hub.x48.activate();
        Self
    }
    fn subscribe(hub: &Hub, shared: Shared<Self>) {
        hub.x49.subscribe(shared);
    }
}
struct X50;
impl X for X50 {}
impl Subscriber<Hub> for X50 {
    type Input = ();
    fn build(mut hub: Hub, _: Self::Input) -> Self {
        hub.x49.activate();
        Self
    }
    fn subscribe(hub: &Hub, shared: Shared<Self>) {
        hub.x50.subscribe(shared);
    }
}
struct X51;
impl X for X51 {}
impl Subscriber<Hub> for X51 {
    type Input = ();
    fn build(mut hub: Hub, _: Self::Input) -> Self {
        hub.x50.activate();
        Self
    }
    fn subscribe(hub: &Hub, shared: Shared<Self>) {
        hub.x51.subscribe(shared);
    }
}
struct X52;
impl X for X52 {}
impl Subscriber<Hub> for X52 {
    type Input = ();
    fn build(mut hub: Hub, _: Self::Input) -> Self {
        hub.x51.activate();
        Self
    }
    fn subscribe(hub: &Hub, shared: Shared<Self>) {
        hub.x52.subscribe(shared);
    }
}
struct X53;
impl X for X53 {}
impl Subscriber<Hub> for X53 {
    type Input = ();
    fn build(mut hub: Hub, _: Self::Input) -> Self {
        hub.x52.activate();
        Self
    }
    fn subscribe(hub: &Hub, shared: Shared<Self>) {
        hub.x53.subscribe(shared);
    }
}
struct X54;
impl X for X54 {}
impl Subscriber<Hub> for X54 {
    type Input = ();
    fn build(mut hub: Hub, _: Self::Input) -> Self {
        hub.x53.activate();
        Self
    }
    fn subscribe(hub: &Hub, shared: Shared<Self>) {
        hub.x54.subscribe(shared);
    }
}
struct X55;
impl X for X55 {}
impl Subscriber<Hub> for X55 {
    type Input = ();
    fn build(mut hub: Hub, _: Self::Input) -> Self {
        hub.x54.activate();
        Self
    }
    fn subscribe(hub: &Hub, shared: Shared<Self>) {
        hub.x55.subscribe(shared);
    }
}
struct X56;
impl X for X56 {}
impl Subscriber<Hub> for X56 {
    type Input = ();
    fn build(mut hub: Hub, _: Self::Input) -> Self {
        hub.x55.activate();
        Self
    }
    fn subscribe(hub: &Hub, shared: Shared<Self>) {
        hub.x56.subscribe(shared);
    }
}
struct X57;
impl X for X57 {}
impl Subscriber<Hub> for X57 {
    type Input = ();
    fn build(mut hub: Hub, _: Self::Input) -> Self {
        hub.x56.activate();
        Self
    }
    fn subscribe(hub: &Hub, shared: Shared<Self>) {
        hub.x57.subscribe(shared);
    }
}
struct X58;
impl X for X58 {}
impl Subscriber<Hub> for X58 {
    type Input = ();
    fn build(mut hub: Hub, _: Self::Input) -> Self {
        hub.x57.activate();
        Self
    }
    fn subscribe(hub: &Hub, shared: Shared<Self>) {
        hub.x58.subscribe(shared);
    }
}
struct X59;
impl X for X59 {}
impl Subscriber<Hub> for X59 {
    type Input = ();
    fn build(mut hub: Hub, _: Self::Input) -> Self {
        hub.x58.activate();
        Self
    }
    fn subscribe(hub: &Hub, shared: Shared<Self>) {
        hub.x59.subscribe(shared);
    }
}
struct X60;
impl X for X60 {}
impl Subscriber<Hub> for X60 {
    type Input = ();
    fn build(mut hub: Hub, _: Self::Input) -> Self {
        hub.x59.activate();
        Self
    }
    fn subscribe(hub: &Hub, shared: Shared<Self>) {
        hub.x60.subscribe(shared);
    }
}
struct X61;
impl X for X61 {}
impl Subscriber<Hub> for X61 {
    type Input = ();
    fn build(mut hub: Hub, _: Self::Input) -> Self {
        hub.x60.activate();
        Self
    }
    fn subscribe(hub: &Hub, shared: Shared<Self>) {
        hub.x61.subscribe(shared);
    }
}
struct X62;
impl X for X62 {}
impl Subscriber<Hub> for X62 {
    type Input = ();
    fn build(mut hub: Hub, _: Self::Input) -> Self {
        hub.x61.activate();
        Self
    }
    fn subscribe(hub: &Hub, shared: Shared<Self>) {
        hub.x62.subscribe(shared);
    }
}
struct X63;
impl X for X63 {}
impl Subscriber<Hub> for X63 {
    type Input = ();
    fn build(mut hub: Hub, _: Self::Input) -> Self {
        hub.x62.activate();
        Self
    }
    fn subscribe(hub: &Hub, shared: Shared<Self>) {
        hub.x63.subscribe(shared);
    }
}
struct X64;
impl X for X64 {}
impl Subscriber<Hub> for X64 {
    type Input = ();
    fn build(mut hub: Hub, _: Self::Input) -> Self {
        hub.x63.activate();
        Self
    }
    fn subscribe(hub: &Hub, shared: Shared<Self>) {
        hub.x64.subscribe(shared);
    }
}
struct X65;
impl X for X65 {}
impl Subscriber<Hub> for X65 {
    type Input = ();
    fn build(mut hub: Hub, _: Self::Input) -> Self {
        hub.x64.activate();
        Self
    }
    fn subscribe(hub: &Hub, shared: Shared<Self>) {
        hub.x65.subscribe(shared);
    }
}
struct X66;
impl X for X66 {}
impl Subscriber<Hub> for X66 {
    type Input = ();
    fn build(mut hub: Hub, _: Self::Input) -> Self {
        hub.x65.activate();
        Self
    }
    fn subscribe(hub: &Hub, shared: Shared<Self>) {
        hub.x66.subscribe(shared);
    }
}
struct X67;
impl X for X67 {}
impl Subscriber<Hub> for X67 {
    type Input = ();
    fn build(mut hub: Hub, _: Self::Input) -> Self {
        hub.x66.activate();
        Self
    }
    fn subscribe(hub: &Hub, shared: Shared<Self>) {
        hub.x67.subscribe(shared);
    }
}
struct X68;
impl X for X68 {}
impl Subscriber<Hub> for X68 {
    type Input = ();
    fn build(mut hub: Hub, _: Self::Input) -> Self {
        hub.x67.activate();
        Self
    }
    fn subscribe(hub: &Hub, shared: Shared<Self>) {
        hub.x68.subscribe(shared);
    }
}
struct X69;
impl X for X69 {}
impl Subscriber<Hub> for X69 {
    type Input = ();
    fn build(mut hub: Hub, _: Self::Input) -> Self {
        hub.x68.activate();
        Self
    }
    fn subscribe(hub: &Hub, shared: Shared<Self>) {
        hub.x69.subscribe(shared);
    }
}
struct X70;
impl X for X70 {}
impl Subscriber<Hub> for X70 {
    type Input = ();
    fn build(mut hub: Hub, _: Self::Input) -> Self {
        hub.x69.activate();
        Self
    }
    fn subscribe(hub: &Hub, shared: Shared<Self>) {
        hub.x70.subscribe(shared);
    }
}
struct X71;
impl X for X71 {}
impl Subscriber<Hub> for X71 {
    type Input = ();
    fn build(mut hub: Hub, _: Self::Input) -> Self {
        hub.x70.activate();
        Self
    }
    fn subscribe(hub: &Hub, shared: Shared<Self>) {
        hub.x71.subscribe(shared);
    }
}
struct X72;
impl X for X72 {}
impl Subscriber<Hub> for X72 {
    type Input = ();
    fn build(mut hub: Hub, _: Self::Input) -> Self {
        hub.x71.activate();
        Self
    }
    fn subscribe(hub: &Hub, shared: Shared<Self>) {
        hub.x72.subscribe(shared);
    }
}
struct X73;
impl X for X73 {}
impl Subscriber<Hub> for X73 {
    type Input = ();
    fn build(mut hub: Hub, _: Self::Input) -> Self {
        hub.x72.activate();
        Self
    }
    fn subscribe(hub: &Hub, shared: Shared<Self>) {
        hub.x73.subscribe(shared);
    }
}
struct X74;
impl X for X74 {}
impl Subscriber<Hub> for X74 {
    type Input = ();
    fn build(mut hub: Hub, _: Self::Input) -> Self {
        hub.x73.activate();
        Self
    }
    fn subscribe(hub: &Hub, shared: Shared<Self>) {
        hub.x74.subscribe(shared);
    }
}
struct X75;
impl X for X75 {}
impl Subscriber<Hub> for X75 {
    type Input = ();
    fn build(mut hub: Hub, _: Self::Input) -> Self {
        hub.x74.activate();
        Self
    }
    fn subscribe(hub: &Hub, shared: Shared<Self>) {
        hub.x75.subscribe(shared);
    }
}
struct X76;
impl X for X76 {}
impl Subscriber<Hub> for X76 {
    type Input = ();
    fn build(mut hub: Hub, _: Self::Input) -> Self {
        hub.x75.activate();
        Self
    }
    fn subscribe(hub: &Hub, shared: Shared<Self>) {
        hub.x76.subscribe(shared);
    }
}
struct X77;
impl X for X77 {}
impl Subscriber<Hub> for X77 {
    type Input = ();
    fn build(mut hub: Hub, _: Self::Input) -> Self {
        hub.x76.activate();
        Self
    }
    fn subscribe(hub: &Hub, shared: Shared<Self>) {
        hub.x77.subscribe(shared);
    }
}
struct X78;
impl X for X78 {}
impl Subscriber<Hub> for X78 {
    type Input = ();
    fn build(mut hub: Hub, _: Self::Input) -> Self {
        hub.x77.activate();
        Self
    }
    fn subscribe(hub: &Hub, shared: Shared<Self>) {
        hub.x78.subscribe(shared);
    }
}
struct X79;
impl X for X79 {}
impl Subscriber<Hub> for X79 {
    type Input = ();
    fn build(mut hub: Hub, _: Self::Input) -> Self {
        hub.x78.activate();
        Self
    }
    fn subscribe(hub: &Hub, shared: Shared<Self>) {
        hub.x79.subscribe(shared);
    }
}
struct X80;
impl X for X80 {}
impl Subscriber<Hub> for X80 {
    type Input = ();
    fn build(mut hub: Hub, _: Self::Input) -> Self {
        hub.x79.activate();
        Self
    }
    fn subscribe(hub: &Hub, shared: Shared<Self>) {
        hub.x80.subscribe(shared);
    }
}
struct X81;
impl X for X81 {}
impl Subscriber<Hub> for X81 {
    type Input = ();
    fn build(mut hub: Hub, _: Self::Input) -> Self {
        hub.x80.activate();
        Self
    }
    fn subscribe(hub: &Hub, shared: Shared<Self>) {
        hub.x81.subscribe(shared);
    }
}
struct X82;
impl X for X82 {}
impl Subscriber<Hub> for X82 {
    type Input = ();
    fn build(mut hub: Hub, _: Self::Input) -> Self {
        hub.x81.activate();
        Self
    }
    fn subscribe(hub: &Hub, shared: Shared<Self>) {
        hub.x82.subscribe(shared);
    }
}
struct X83;
impl X for X83 {}
impl Subscriber<Hub> for X83 {
    type Input = ();
    fn build(mut hub: Hub, _: Self::Input) -> Self {
        hub.x82.activate();
        Self
    }
    fn subscribe(hub: &Hub, shared: Shared<Self>) {
        hub.x83.subscribe(shared);
    }
}
struct X84;
impl X for X84 {}
impl Subscriber<Hub> for X84 {
    type Input = ();
    fn build(mut hub: Hub, _: Self::Input) -> Self {
        hub.x83.activate();
        Self
    }
    fn subscribe(hub: &Hub, shared: Shared<Self>) {
        hub.x84.subscribe(shared);
    }
}
struct X85;
impl X for X85 {}
impl Subscriber<Hub> for X85 {
    type Input = ();
    fn build(mut hub: Hub, _: Self::Input) -> Self {
        hub.x84.activate();
        Self
    }
    fn subscribe(hub: &Hub, shared: Shared<Self>) {
        hub.x85.subscribe(shared);
    }
}
struct X86;
impl X for X86 {}
impl Subscriber<Hub> for X86 {
    type Input = ();
    fn build(mut hub: Hub, _: Self::Input) -> Self {
        hub.x85.activate();
        Self
    }
    fn subscribe(hub: &Hub, shared: Shared<Self>) {
        hub.x86.subscribe(shared);
    }
}
struct X87;
impl X for X87 {}
impl Subscriber<Hub> for X87 {
    type Input = ();
    fn build(mut hub: Hub, _: Self::Input) -> Self {
        hub.x86.activate();
        Self
    }
    fn subscribe(hub: &Hub, shared: Shared<Self>) {
        hub.x87.subscribe(shared);
    }
}
struct X88;
impl X for X88 {}
impl Subscriber<Hub> for X88 {
    type Input = ();
    fn build(mut hub: Hub, _: Self::Input) -> Self {
        hub.x87.activate();
        Self
    }
    fn subscribe(hub: &Hub, shared: Shared<Self>) {
        hub.x88.subscribe(shared);
    }
}
struct X89;
impl X for X89 {}
impl Subscriber<Hub> for X89 {
    type Input = ();
    fn build(mut hub: Hub, _: Self::Input) -> Self {
        hub.x88.activate();
        Self
    }
    fn subscribe(hub: &Hub, shared: Shared<Self>) {
        hub.x89.subscribe(shared);
    }
}
struct X90;
impl X for X90 {}
impl Subscriber<Hub> for X90 {
    type Input = ();
    fn build(mut hub: Hub, _: Self::Input) -> Self {
        hub.x89.activate();
        Self
    }
    fn subscribe(hub: &Hub, shared: Shared<Self>) {
        hub.x90.subscribe(shared);
    }
}
struct X91;
impl X for X91 {}
impl Subscriber<Hub> for X91 {
    type Input = ();
    fn build(mut hub: Hub, _: Self::Input) -> Self {
        hub.x90.activate();
        Self
    }
    fn subscribe(hub: &Hub, shared: Shared<Self>) {
        hub.x91.subscribe(shared);
    }
}
struct X92;
impl X for X92 {}
impl Subscriber<Hub> for X92 {
    type Input = ();
    fn build(mut hub: Hub, _: Self::Input) -> Self {
        hub.x91.activate();
        Self
    }
    fn subscribe(hub: &Hub, shared: Shared<Self>) {
        hub.x92.subscribe(shared);
    }
}
struct X93;
impl X for X93 {}
impl Subscriber<Hub> for X93 {
    type Input = ();
    fn build(mut hub: Hub, _: Self::Input) -> Self {
        hub.x92.activate();
        Self
    }
    fn subscribe(hub: &Hub, shared: Shared<Self>) {
        hub.x93.subscribe(shared);
    }
}
struct X94;
impl X for X94 {}
impl Subscriber<Hub> for X94 {
    type Input = ();
    fn build(mut hub: Hub, _: Self::Input) -> Self {
        hub.x93.activate();
        Self
    }
    fn subscribe(hub: &Hub, shared: Shared<Self>) {
        hub.x94.subscribe(shared);
    }
}
struct X95;
impl X for X95 {}
impl Subscriber<Hub> for X95 {
    type Input = ();
    fn build(mut hub: Hub, _: Self::Input) -> Self {
        hub.x94.activate();
        Self
    }
    fn subscribe(hub: &Hub, shared: Shared<Self>) {
        hub.x95.subscribe(shared);
    }
}
struct X96;
impl X for X96 {}
impl Subscriber<Hub> for X96 {
    type Input = ();
    fn build(mut hub: Hub, _: Self::Input) -> Self {
        hub.x95.activate();
        Self
    }
    fn subscribe(hub: &Hub, shared: Shared<Self>) {
        hub.x96.subscribe(shared);
    }
}
struct X97;
impl X for X97 {}
impl Subscriber<Hub> for X97 {
    type Input = ();
    fn build(mut hub: Hub, _: Self::Input) -> Self {
        hub.x96.activate();
        Self
    }
    fn subscribe(hub: &Hub, shared: Shared<Self>) {
        hub.x97.subscribe(shared);
    }
}
struct X98;
impl X for X98 {}
impl Subscriber<Hub> for X98 {
    type Input = ();
    fn build(mut hub: Hub, _: Self::Input) -> Self {
        hub.x97.activate();
        Self
    }
    fn subscribe(hub: &Hub, shared: Shared<Self>) {
        hub.x98.subscribe(shared);
    }
}
struct X99;
impl X for X99 {}
impl Subscriber<Hub> for X99 {
    type Input = ();
    fn build(mut hub: Hub, _: Self::Input) -> Self {
        hub.x98.activate();
        Self
    }
    fn subscribe(hub: &Hub, shared: Shared<Self>) {
        hub.x99.subscribe(shared);
    }
}
struct X100;
impl X for X100 {}
impl Subscriber<Hub> for X100 {
    type Input = ();
    fn build(mut hub: Hub, _: Self::Input) -> Self {
        hub.x99.activate();
        Self
    }
    fn subscribe(hub: &Hub, shared: Shared<Self>) {
        hub.x100.subscribe(shared);
    }
}
