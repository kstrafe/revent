use crate::{Event, Notifiable};

struct BinarySystem<'a, 'b>((&'a mut dyn Notifiable, &'b mut dyn Notifiable));

impl<'a, 'b> Notifiable for BinarySystem<'a, 'b> {
    fn event(&mut self, event: &dyn Event, system: &mut dyn Notifiable) {
        (self.0)
            .0
            .event(event, &mut BinarySystem((system, (self.0).1)));
        (self.0)
            .1
            .event(event, &mut BinarySystem((system, (self.0).0)));
    }
}

pub struct TypedBinarySystem<'a, 'b, T: Notifiable>(pub (&'a mut T, &'b mut dyn Notifiable));

impl<'a, 'b, T: Notifiable> Notifiable for TypedBinarySystem<'a, 'b, T> {
    fn event(&mut self, event: &dyn Event, system: &mut dyn Notifiable) {
        (self.0)
            .0
            .event(event, &mut BinarySystem((system, (self.0).1)));
        (self.0)
            .1
            .event(event, &mut BinarySystem((system, (self.0).0)));
    }
}
