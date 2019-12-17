use crate::{Notifiable, Notifier, TypedBinarySystem};

/// A guard for acquiring a notifier and restoring the notifier into a tree after dropping.
pub struct NotifierGuard<'a, 'b, T: Notifiable, N: Notifiable, F: FnMut(&mut T) -> &mut Notifier<N>>
{
    pub(crate) accessor: F,
    pub(crate) split: Option<N>,
    pub(crate) system: TypedBinarySystem<'a, 'b, T>,
}

impl<'a, 'b, T: Notifiable, N: Notifiable, F: FnMut(&mut T) -> &mut Notifier<N>>
    NotifierGuard<'a, 'b, T, N, F>
{
    /// Split the notifier guard into the concrete notifier and its system.
    pub fn split(&mut self) -> (&mut N, &mut dyn Notifiable) {
        (self.split.as_mut().unwrap(), &mut self.system)
    }
}

impl<'a, 'b, T: Notifiable, N: Notifiable, F: FnMut(&mut T) -> &mut Notifier<N>> Drop
    for NotifierGuard<'a, 'b, T, N, F>
{
    fn drop(&mut self) {
        let Self {
            accessor,
            split,
            system,
        } = self;

        let this = &mut (system.0).0;

        accessor(this).set(split.take().unwrap());
    }
}
