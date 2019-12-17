use crate::{Event, Notifiable};

/// [Notifiable] that ignores all notifications. Useful as a root system.
pub struct Ignore;

impl Notifiable for Ignore {
    fn event(&mut self, _: &dyn Event, _: &mut dyn Notifiable) {}
}
