pub mod ffi;

use serde::{Deserialize, Serialize};
use tokio::sync::{broadcast, broadcast::error::SendError};
use tracing::warn;

#[no_mangle]
pub static FFI_VERSION: [u8; 5] = *b"1.0.0";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message {
    pub(crate) message: String,
}

#[derive(Debug)]
pub struct AsyncClient {
    receiver: broadcast::Receiver<Message>,
    sender:   broadcast::Sender<Message>,
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
    handle:       tokio::task::JoinHandle<()>,
}

impl ClientHandle {
    pub fn new(
        runtime: &tokio::runtime::Handle,
    ) -> anyhow::Result<(Self, broadcast::Receiver<Message>)> {
        let (from_client_tx, from_client_rx) = broadcast::channel(1024);
        let (to_client_tx, to_client_rx) = broadcast::channel(1024);
        let async_client = AsyncClient::new(from_client_tx, to_client_rx);
        let handle = runtime.spawn(async move { async_client.run().await });
        Ok((
            Self {
                to_client_tx,
                handle,
            },
            from_client_rx,
        ))
    }

    pub fn send_msg(&mut self, msg: Message) -> Result<usize, SendError<Message>> {
        self.to_client_tx.send(msg)
    }
}
