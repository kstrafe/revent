use crate::event::Event;

mod ignore;
pub use ignore::Ignore;
mod binary;
pub use binary::TypedBinarySystem;

/// Main trait of this crate to implement on structures.
pub trait Notifiable {
    /// Handle the event for this structure. Call [Notifiable::notify] instead of this.
    ///
    /// What you should do: In this method delegate the event down to all fields that are
    /// [Notifiable] and perform any internal changes to the structure to reflect the event.
    fn event(&mut self, event: &dyn Event, system: &mut dyn Notifiable);

    /// Notify this structure and the system about an event.
    ///
    /// Calls [Notifiable::event] on both the current object and the system.
    fn notify(&mut self, event: &dyn Event, system: &mut dyn Notifiable)
    where
        Self: Sized,
    {
        self.event(event, system);
        let this: &mut dyn Notifiable = self;
        system.event(event, this);
    }
}
