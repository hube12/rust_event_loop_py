use std::{sync::Arc, time::Duration};

use anyhow::anyhow;
use tokio::runtime::Runtime;

use crate::ffi::{ffi_error::FFIError, FFISafe};

pub struct FFIRuntime {
    inner: Arc<Runtime>,
}

impl FFISafe for FFIRuntime {
}

impl FFIRuntime {
    pub(crate) fn new(inner: Runtime) -> Self {
        Self {
            inner: Arc::new(inner),
        }
    }

    pub(crate) fn runtime(&self) -> Arc<Runtime> {
        self.inner.clone()
    }

    pub(crate) fn handle(&self) -> FFIRuntimeHandle {
        FFIRuntimeHandle {
            inner: Arc::downgrade(&self.inner),
        }
    }

    #[no_mangle]
    #[must_use]
    pub extern "C" fn create_runtime() -> FFIError<Self> {
        let rt = tokio::runtime::Builder::new_multi_thread()
            .thread_keep_alive(Duration::from_secs(1000))
            .thread_name("Custom thread")
            .worker_threads(4)
            .enable_all()
            .build();
        match rt {
            Ok(rt) => {
                let runtime = Self::new(rt);
                FFIError::from_value(runtime)
            },
            Err(error) => FFIError::from(anyhow!(error)),
        }
    }
}

#[derive(Clone, Debug)]
pub(crate) struct FFIRuntimeHandle {
    inner: std::sync::Weak<Runtime>,
}

impl FFIRuntimeHandle {
    pub(crate) fn runtime<T>(&self) -> Result<Arc<Runtime>, FFIError<T>> {
        self.inner.upgrade().ok_or_else(|| {
            FFIError::from(anyhow::Error::msg(
                "Runtime could not be acquired, it was already dropped",
            ))
        })
    }
}
