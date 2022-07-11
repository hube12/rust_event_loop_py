use crate::{ClientHandle};
use anyhow::anyhow;

use crate::ffi::{
    FFIError,
    FFIInner,
    FFIMessage,
    FFISafe,
    FFISafeMove,
};

pub struct FFIClientHandle {
    pub handle:         ClientHandle,
}

impl FFISafe for FFIClientHandle {
}

#[allow(dead_code)]
impl FFIClientHandle {
    #[no_mangle]
    pub extern "C" fn client_handle_send(
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
