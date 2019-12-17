use std::any::Any;

/// A generic event. Implemented for almost all types.
///
/// A generic event is just anything that can be represented as [Any]. No further bounds are
/// applied to events, giving the maximum amount of flexibility.
pub trait Event: Any {
    /// Get the reference to this events' [Any]. Used for downcasting.
    ///
    /// Normally not used directly but rather used by [down](crate::down).
    fn as_any(&self) -> &dyn Any;
}

impl<T: Any> Event for T {
    fn as_any(&self) -> &dyn Any {
        self
    }
}
