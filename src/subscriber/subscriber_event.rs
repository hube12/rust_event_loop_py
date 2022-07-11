use crate::subscriber::SubscriberEventType;
use std::fmt::Debug;

pub trait SubscriberEvent: Clone + Send + Sync + Sized + Debug {
    type Type: SubscriberEventType;

    /// Whether to kill the Subscriber event loop depending of that event
    fn should_kill(&self) -> bool;

    /// Returns the type of that event
    fn get_type(&self) -> Self::Type;
}
