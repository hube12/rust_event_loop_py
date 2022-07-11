pub trait SubscriberConfig: Default + Send {
    /// Number of subscriber to be spawned at once
    fn subscriber_count(&self) -> usize;

    /// Channel size for the Subscriber
    fn channel_size(&self) -> usize;
}
