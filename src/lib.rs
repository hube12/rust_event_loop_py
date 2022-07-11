pub mod ffi;
pub mod subscriber;
mod events;

pub use events::{SConfig, SError, SEvent, SEventType, Sub};
use serde::{Deserialize, Serialize};
use tokio::sync::{broadcast, broadcast::error::SendError};
use tracing::{warn, trace};
use crate::subscriber::{EventHandle, SubscribeHandle, SubscriberCallback, SubscriberServer, SubscriberServerHandle};

#[no_mangle]
pub static FFI_VERSION: [u8; 5] = *b"1.0.0";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message {
    pub message: String,
}

#[derive(Debug)]
pub struct AsyncClient {
    receiver: broadcast::Receiver<Message>,
    sender: broadcast::Sender<Message>,
}

impl AsyncClient {
    pub fn new(sender: broadcast::Sender<Message>, receiver: broadcast::Receiver<Message>) -> Self {
        Self { receiver, sender }
    }

    #[allow(dead_code)]
    pub(crate) async fn run(mut self) {
        loop {
            tokio::select! {
                _=(&mut self).respond()=>{},
            }
        }
    }

    async fn respond(&mut self) {
        if let Ok(msg) = self.receiver.recv().await {
            if let Err(err) = self.sender.send(msg) {
                warn!("{:?}", err);
            }
        }
    }
}


#[derive(Debug)]
#[allow(dead_code)]
pub struct ClientHandle {
    to_client_tx: broadcast::Sender<Message>,
    client_join_handle: tokio::task::JoinHandle<()>,
}

impl ClientHandle {
    pub fn new(
        runtime: &tokio::runtime::Handle,
    ) -> anyhow::Result<(Self, broadcast::Receiver<Message>)> {
        let (from_client_tx, from_client_rx) = broadcast::channel(1024);
        let (to_client_tx, to_client_rx) = broadcast::channel(1024);
        let async_client = AsyncClient::new(from_client_tx, to_client_rx);
        let client_join_handle = runtime.spawn(async move { async_client.run().await });
        Ok((
            Self {
                to_client_tx,
                client_join_handle,
            },
            from_client_rx,
        ))
    }

    pub fn send_msg(&mut self, msg: Message) -> Result<usize, SendError<Message>> {
        self.to_client_tx.send(msg)
    }
}

#[allow(dead_code)]
pub struct Runner {
    subscriber_join_handle: tokio::task::JoinHandle<()>,
    runner_join_handle: tokio::task::JoinHandle<()>,
}

impl Runner {
    pub fn new(
        runtime: &tokio::runtime::Handle,
    ) -> anyhow::Result<(ClientHandle, Self, SubscribeHandle<SEventType, Box<dyn SubscriberCallback<SEvent>>>)> {
        let (client_handle, client_receiver) = ClientHandle::new(runtime)?;
        let sub_server = SubscriberServer::<_, _, _, Sub<SEvent>>::new(SConfig::default());
        let sub_handler = SubscriberServerHandle::new(sub_server, runtime);
        let (send_handle, subscribe_handle, subscriber_join_handle) = sub_handler.split();
        let runner_join_handle = runtime.spawn(Self::run(client_receiver, send_handle));
        Ok((client_handle, Runner {
            subscriber_join_handle,
            runner_join_handle,
        }, subscribe_handle))
    }

    #[allow(unused_assignments)]
    pub async fn run(mut client_receiver: broadcast::Receiver<Message>, send_handle: EventHandle<SEvent>) {
        loop {
            // you can handle more than one receiver by using this pattern of optional and select! short circuit
            let mut msg_client = None;
            tokio::select! {
                msg =client_receiver.recv() =>{
                    trace!("Received client message");
                    msg_client=Some(msg);
                }
            }
            if let Some(msg) = msg_client {
                match msg {
                    Ok(msg) => {
                        let err = if msg.message.starts_with("test") {
                            send_handle.send(SEvent::Event1(msg.message)).await
                        } else {
                            send_handle.send(SEvent::Event2(msg.message)).await
                        };
                        if let Err(err) = err {
                            warn!("Could not send normal Event {:?}",err);
                        }
                    }
                    Err(e) => {
                        warn!("Error wile receiving message {}",e);
                        if let Err(err) = send_handle.send(SEvent::Kill).await {
                            warn!("Could not send kill Event {:?}",err);
                        }
                    }
                }
            }
        }
    }
}