use crate::{
    ffi::{ffi_bytes_vec, FFIArray, FFIError, FFIInner, FFISafe, FFISafeMove},
    Message,
};
use anyhow::anyhow;
use std::ffi::CStr;

pub struct FFIMessage(pub(crate) Message);

impl FFISafeMove for FFIMessage {
}

impl FFISafe for FFIMessage {
}


impl FFIInner for FFIMessage {
    type Output = Message;

    fn inner(self) -> Self::Output {
        self.0
    }
}


impl FFIMessage {
    /// # Safety
    /// Pointer should be NUL terminated and respect `CStr::from_ptr`
    /// preconditions
    #[no_mangle]
    pub unsafe extern "C" fn message_from_cstring(
        ptr: *mut std::os::raw::c_char,
    ) -> FFIError<Self> {
        if ptr.is_null() {
            return FFIError::from(anyhow::Error::msg("Null pointer provided as an argument"));
        }

        match CStr::from_ptr(ptr).to_str() {
            Ok(s) => FFIError::from_value(FFIMessage(Message {
                message: String::from(s),
            })),
            Err(error) => FFIError::from(anyhow!(error)),
        }
    }

    #[no_mangle]
    #[must_use]
    pub extern "C" fn message_as_cstring(
        ptr: *mut Self,
    ) -> FFIError<FFIArray<std::os::raw::c_uchar>> {
        let message = match Self::safe(ptr) {
            Ok(rt) => rt,
            Err(error) => return error,
        };

        FFIError::from_value(ffi_bytes_vec(message.0.message.clone().into_bytes()))
    }
}
