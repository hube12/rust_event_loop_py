use crate::subscriber::{SubscriberCallback, SubscriberError, SubscriberEvent};

pub trait Subscriber<Event, Error>: Send
where
    Error: SubscriberError,
    Event: SubscriberEvent,
{
    /// Create A new [`Subscriber`] from a callback
    fn new<T: SubscriberCallback<Event> + 'static>(callback: T) -> Self;

    /// Notify the subscribers with an [`Event`]
    ///
    /// # Errors
    /// If the subscriber can not accept the event at the moment return
    /// [`Error`]
    fn notify(&self, event: Event) -> Result<(), Error>;
}
