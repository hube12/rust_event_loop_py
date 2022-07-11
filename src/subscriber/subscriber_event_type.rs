use std::{fmt::Debug, hash::Hash};

pub trait SubscriberEventType: Hash + Eq + Send + Sync + Debug + Clone {}
