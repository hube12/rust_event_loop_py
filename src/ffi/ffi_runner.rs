use anyhow::anyhow;

use crate::ffi::{FFIClientHandle, FFIError, FFIRuntime, FFISafe, FFISubscriber, FFITriple};
use crate::{ Runner};

pub struct FFIRunner(Runner);

impl FFISafe for FFIRunner {}

#[allow(dead_code)]
impl FFIRunner {
    #[no_mangle]
    pub extern "C" fn create_new_runner(
        runtime: *mut FFIRuntime,
    ) -> FFIError<FFITriple<FFIClientHandle, Self, FFISubscriber>> {
        let rt = match FFIRuntime::safe(runtime) {
            Ok(rt) => rt,
            Err(error) => return error,
        };

        match Runner::new(
            rt.runtime().handle(),
        ) {
            Ok((client_handle, runner, subscribe_handle)) => {
                let client = FFIClientHandle {
                    handle: client_handle,
                };
                let runner = Self(runner);
                let subscriber = FFISubscriber(subscribe_handle);
                FFIError::from_value(FFITriple::from((client, runner, subscriber)))
            }
            Err(error) => FFIError::from(anyhow!(error)),
        }
    }

    #[no_mangle]
    pub extern "C" fn runner_get_client(
        tuple: *mut FFITriple<FFIClientHandle, Self, FFISubscriber>,
    ) -> FFIError<FFIClientHandle> {
        let tuple = match FFITriple::safe(tuple) {
            Ok(tuple) => tuple,
            Err(error) => return error,
        };
        match tuple.first.take() {
            None => FFIError::from(anyhow::anyhow!(
                "First element of the tuple was already removed"
            )),
            Some(first) => FFIError::from_value(first),
        }
    }

    #[no_mangle]
    pub extern "C" fn runner_get_runner(
        tuple: *mut FFITriple<FFIClientHandle, Self, FFISubscriber>,
    ) -> FFIError<Self> {
        let tuple = match FFITriple::safe(tuple) {
            Ok(tuple) => tuple,
            Err(error) => return error,
        };
        match tuple.second.take() {
            None => FFIError::from(anyhow::anyhow!(
                "First element of the tuple was already removed"
            )),
            Some(first) => FFIError::from_value(first),
        }
    }

    #[no_mangle]
    pub extern "C" fn runner_get_subscriber(
        tuple: *mut FFITriple<FFIClientHandle, Self, FFISubscriber>,
    ) -> FFIError<FFISubscriber> {
        let tuple = match FFITriple::safe(tuple) {
            Ok(tuple) => tuple,
            Err(error) => return error,
        };
        match tuple.third.take() {
            None => FFIError::from(anyhow::anyhow!(
                "Third element of the tuple was already removed"
            )),
            Some(second) => FFIError::from_value(second),
        }
    }
}
