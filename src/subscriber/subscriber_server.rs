use crate::subscriber::{
    subscriber_error::SubscriberError,
    Subscriber,
    SubscriberCallback,
    SubscriberConfig,
    SubscriberEvent,
    SubscriberEventType,
    Subscribers,
};
use std::{
    borrow::BorrowMut,
    collections::{HashMap, VecDeque},
    fmt::Debug,
};
use tokio::sync::mpsc::Receiver;
use tracing::{info, warn};


#[derive(Debug)]
pub struct SubscriberServer<Type, Event, Config, Sub> {
    store: HashMap<Type, VecDeque<Event>>,
    config: Config,
    subscribers: HashMap<Type, Subscribers<Sub>>,
}

impl<Type, Event, Config, Sub> Default for SubscriberServer<Type, Event, Config, Sub>
    where
        Config: SubscriberConfig,
{
    fn default() -> Self {
        Self::new(Config::default())
    }
}

impl<Type, Event, Config, Sub> SubscriberServer<Type, Event, Config, Sub>
    where
        Config: SubscriberConfig,
{
    pub fn new(config: Config) -> Self {
        Self {
            store: HashMap::with_capacity(16 + 2 + 1 + 1),
            config,
            subscribers: HashMap::with_capacity(10),
        }
    }
}

impl<Type, Event, Config, Sub> SubscriberServer<Type, Event, Config, Sub>
    where
        Event: SubscriberEvent<Type=Type>,
        Config: SubscriberConfig,
        Type: SubscriberEventType,
{
    pub async fn run<Error, Callback>(
        mut self,
        mut recv_event: Receiver<Event>,
        mut recv_subscribe: Receiver<(Type, Callback)>,
    ) where
        Error: SubscriberError,
        Sub: Subscriber<Event, Error>,
        Callback: SubscriberCallback<Event> + 'static,
    {
        loop {
            // Do subscription
            tokio::select! {
                Some((event_type, callback)) = recv_subscribe.recv() =>{
                    self.borrow_mut().subscribe(event_type, callback);
                }
                Some(event) = recv_event.recv() => {
                    if event.should_kill() {
                        info!("Killing Subscriber server");
                        return;
                    }
                    self.borrow_mut().send(event);
                }
                else => {
                    break;
                }
            }
        }
    }
}

impl<Type, Event, Config, Sub> SubscriberServer<Type, Event, Config, Sub>
    where
        Type: SubscriberEventType,
        Event: SubscriberEvent<Type=Type>,
        Config: SubscriberConfig,
{
    fn send<Error>(&mut self, event: Event)
        where
            Error: SubscriberError,
            Sub: Subscriber<Event, Error> ,
    {
        let event_type = event.get_type();
        if let Some(subscribers) = self.subscribers.get(&event_type) {
            if let Some(err) = subscribers.notify(event) {
                warn!("Could not notify subscribers {:?}",err);
            }
        } else {
            let vec = self.store
                .entry(event_type)
                .or_insert_with(|| VecDeque::with_capacity(self.config.channel_size()));
            while vec.len() > 10 {
                let _dropped = vec.pop_front();
            }
            vec.push_back(event);
        }

    }
}

impl<Type, Event, Config, Sub> SubscriberServer<Type, Event, Config, Sub>
    where
        Type: SubscriberEventType,
        Event: SubscriberEvent,
        Config: SubscriberConfig,
{
    fn subscribe<Callback, Error>(&mut self, event_type: Type, callback: Callback)
        where
            Error: SubscriberError,
            Sub: Subscriber<Event, Error>,
            Callback: SubscriberCallback<Event> + 'static,
    {
        let sub = Subscriber::new(callback);
        if let Some(store) = self.store.get_mut(&event_type) {
            while let Some(event) = store.pop_front() {
                if let Err(err) = Sub::notify(&sub, event) {
                    warn!("Could not notify subscriber {:?}",err);
                }
            }
        }
        self.subscribers
            .entry(event_type)
            .or_insert_with(Subscribers::default)
            .push(sub);
    }
}
