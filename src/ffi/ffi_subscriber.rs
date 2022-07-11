use crate::{ffi::{
    create_heap_pointer,
    FFIArray,
    FFIError,
    FFIRuntime,
    FFISafe,
}, SEvent, SEventType};
use anyhow::anyhow;
use crate::ffi::{ffi_bytes_vec, FFINull};
use crate::subscriber::{SubscribeHandle, SubscriberCallback};

pub struct FFISubscriber(
    pub(in crate::ffi) SubscribeHandle<SEventType, Box<dyn SubscriberCallback<SEvent>>>,
);

impl FFISafe for FFISubscriber {}

#[allow(dead_code)]
impl FFISubscriber {
    #[no_mangle]
    pub extern "C" fn subscribe(
        handle: *mut Self,
        event_type: SEventType,
        callback: extern "C" fn(*mut FFISEvent),
        runtime: *mut FFIRuntime,
    ) -> FFIError<FFINull> {
        let rt = match FFIRuntime::safe(runtime) {
            Ok(rt) => rt,
            Err(error) => return error,
        };
        let handle = match FFISubscriber::safe(handle) {
            Ok(x) => x,
            Err(error) => return error,
        };
        if let Err(e) = rt.runtime().handle().block_on(handle.0.subscribe(
            event_type,
            Box::new(move |e| {
                let event = create_heap_pointer(FFISEvent::from(e));
                callback(event);
            }),
        )) {
            let typ = e.0.0;
            return FFIError::from(anyhow!(
                "Could not create a callback for this type {:?}",
                typ
            ));
        }
        FFIError::from_value(FFINull)
    }
}

#[repr(C)]
pub enum FFISEvent {
    Event1(FFIArray<std::os::raw::c_uchar>),
    Event2(FFIArray<std::os::raw::c_uchar>),
    Kill,
}

impl From<SEvent> for FFISEvent {
    fn from(event: SEvent) -> Self {
        match event {
            SEvent::Event1(s) => {
                Self::Event1(ffi_bytes_vec(s.into_bytes()))
            }
            SEvent::Event2(s) => {
                Self::Event2(ffi_bytes_vec(s.into_bytes()))
            }
            SEvent::Kill => Self::Kill,
        }
    }
}
