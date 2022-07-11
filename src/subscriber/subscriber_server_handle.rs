use std::fmt::Debug;
use tokio::sync::mpsc::{channel, error::SendError, Receiver, Sender};

use tokio::task::JoinHandle;

use crate::{
    subscriber::{
        Subscriber,
        SubscriberCallback,
        SubscriberConfig,
        SubscriberError,
        SubscriberEvent,
        SubscriberEventType,
    },
};
use crate::subscriber::SubscriberServer;

pub struct SubscriberServerHandle<Type, Event, Callback>
where
    Event: SubscriberEvent,
    Type: SubscriberEventType,
    Callback: SubscriberCallback<Event>,
{
    pub send_event:     EventHandle<Event>,
    pub send_subscribe: SubscribeHandle<Type, Callback>,
    join_handle:        JoinHandle<()>,
}

pub struct SubscribeHandle<Type, Callback> {
    pub send_subscribe: Sender<(Type, Callback)>,
}

pub struct EventHandle<Event> {
    pub send_event: Sender<Event>,
}

impl<Type, Callback> SubscribeHandle<Type, Callback> {
    pub async fn subscribe(
        &self,
        event_type: Type,
        callback: Callback,
    ) -> Result<(), SendError<(Type, Callback)>> {
        self.send_subscribe.send((event_type, callback)).await
    }
}

impl<Event: Debug> EventHandle<Event> {
    pub async fn send(&self, event: Event) -> Result<(), SendError<Event>> {
        self.send_event.send(event).await
    }
}

impl<Type, Event, Callback> SubscriberServerHandle<Type, Event, Callback>
where
    Event: SubscriberEvent<Type = Type> + 'static,
    Type: SubscriberEventType + 'static,
    Callback: SubscriberCallback<Event> + 'static,
{
    pub fn new<Config, Sub, Error>(
        server: SubscriberServer<Type, Event, Config, Sub>,
        rt: &tokio::runtime::Handle,
    ) -> Self
    where
        Config: SubscriberConfig + 'static,
        Error: SubscriberError,
        Sub: Subscriber<Event, Error> + 'static,
        Receiver<Callback>: Send + 'static,
        Receiver<Event>: Send + 'static,
    {
        let (send_event, recv_event) = channel(10);
        let (send_subscribe, recv_subscribe) = channel(10);
        let join_handle = rt.spawn(async move { server.run(recv_event, recv_subscribe).await });
        Self {
            send_event: EventHandle { send_event },
            send_subscribe: SubscribeHandle { send_subscribe },
            join_handle,
        }
    }

    #[must_use]
    pub fn split(
        self,
    ) -> (
        EventHandle<Event>,
        SubscribeHandle<Type, Callback>,
        JoinHandle<()>,
    ) {
        (self.send_event, self.send_subscribe, self.join_handle)
    }

    /// This does not stop the process, you need to kill it first
    pub fn stop_handle(&self) {
        self.join_handle.abort();
    }

    pub async fn send(&mut self, event: Event) -> Result<(), SendError<Event>> {
        self.send_event.send(event).await
    }

    pub async fn subscribe(
        &self,
        event_type: Type,
        callback: Callback,
    ) -> Result<(), SendError<(Type, Callback)>> {
        self.send_subscribe.subscribe(event_type, callback).await
    }
}
