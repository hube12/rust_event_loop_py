use crate::{ClientHandle, Message};
use anyhow::anyhow;

use crate::ffi::{
    ffi_runtime::FFIRuntimeHandle,
    FFIError,
    FFIInner,
    FFIMessage,
    FFIRuntime,
    FFISafe,
    FFISafeMove,
};

pub struct FFIClient {
    handle:         ClientHandle,
    receiver:       tokio::sync::broadcast::Receiver<Message>,
    runtime_handle: FFIRuntimeHandle,
}

impl FFISafe for FFIClient {
}

#[allow(dead_code)]
impl FFIClient {
    #[no_mangle]
    pub extern "C" fn create_new_client(runtime: *mut FFIRuntime) -> FFIError<Self> {
        let rt = match FFIRuntime::safe(runtime) {
            Ok(rt) => rt,
            Err(error) => return error,
        };
        let (handle, receiver) = match ClientHandle::new(rt.runtime().handle()) {
            Ok(x) => x,
            Err(e) => return FFIError::from(anyhow!(e)),
        };
        FFIError::from_value(Self {
            handle,
            receiver,
            runtime_handle: rt.handle(),
        })
    }

    #[no_mangle]
    pub extern "C" fn client_receive(client: *mut Self) -> FFIError<FFIMessage> {
        let client = match Self::safe(client) {
            Ok(x) => x,
            Err(error) => return error,
        };
        let runtime = match client.runtime_handle.runtime() {
            Ok(x) => x,
            Err(error) => return error,
        };
        match runtime.block_on(client.receiver.recv()) {
            Ok(msg) => FFIError::from_value(FFIMessage(msg)),
            Err(err) => FFIError::from(anyhow!(err)),
        }
    }
    #[no_mangle]
    pub extern "C" fn client_send(
        client: *mut Self,
        mut msg: *mut FFIMessage,
    ) -> FFIError<std::os::raw::c_ulong> {
        let client = match Self::safe(client) {
            Ok(x) => x,
            Err(error) => return error,
        };
        let message = match FFIMessage::safe_move(&mut msg) {
            Ok(x) => x,
            Err(error) => return error,
        }
        .inner();
        match client.handle.send_msg(message) {
            Ok(recv_count) => FFIError::from_value(recv_count as std::os::raw::c_ulong),
            Err(err) => FFIError::from(anyhow!(err)),
        }
    }
}
