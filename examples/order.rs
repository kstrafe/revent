use revent::{down, Event, Notifiable};

struct Dummy(u32);

impl Notifiable for Dummy {
    fn event(&mut self, event: &dyn Event, system: &mut dyn Notifiable) {
        if let Some(number) = down::<i32>(event) {
            println!("Dummy({}): got i32: {}", self.0, number);
            self.notify(&"Response event", system);
        } else if let Some(string) = down::<&str>(event) {
            println!("Dummy({}): got string: {}", self.0, string);
        } else {
            panic!("Unexpected event");
        }
    }
}

fn main() {
    let mut this = Dummy(0);
    let mut system = Dummy(1);

    this.notify(&0i32, &mut system);
}
