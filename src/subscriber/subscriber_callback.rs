use crate::subscriber::SubscriberEvent;

pub trait SubscriberCallback<Event>
where
    Self: Fn(Event) + Send + std::panic::UnwindSafe + std::panic::RefUnwindSafe + Sync,
    Event: SubscriberEvent + Sized,
{
}
