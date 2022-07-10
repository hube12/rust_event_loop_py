use crate::ffi::{create_heap_pointer, destroy_cstring, destroy_pointer, ffi_cstring, FFINull};
use std::{any::Any, ffi::c_void};

#[repr(C)]
pub struct FFIError<T> {
    pub(crate) obj:   *mut T,
    pub(crate) error: *mut std::os::raw::c_char,
}

/// Dangerous function as you don't know if the pointer is allocated or not on
/// the heap
impl<T> From<Result<*mut T, anyhow::Error>> for FFIError<T> {
    fn from(res: Result<*mut T, anyhow::Error>) -> Self {
        match res {
            Ok(res) => Self::from(res),
            Err(error) => Self::from(error),
        }
    }
}

/// Dangerous function as you don't know if the pointer is allocated or not on
/// the heap
impl<T> From<*mut T> for FFIError<T> {
    fn from(obj: *mut T) -> Self {
        Self {
            obj,
            error: std::ptr::null::<T>() as *mut _,
        }
    }
}

impl From<(&str, &'static str)> for FFIError<std::os::raw::c_char> {
    fn from((msg, default): (&str, &'static str)) -> Self {
        ffi_cstring(msg, default)
    }
}

impl<T> From<anyhow::Error> for FFIError<T> {
    fn from(error: anyhow::Error) -> Self {
        let error_string = format!("{:#?}", error);
        let error_string = std::ffi::CString::new(error_string).unwrap_or_else(|_| {
            std::ffi::CString::new("Error could not be translated to a valid CString")
                .expect("Can not fail")
        });
        let error = error_string.into_raw();
        Self {
            obj: std::ptr::null_mut(),
            error,
        }
    }
}
#[no_mangle]
/// # Safety
/// FFI safeties
pub unsafe extern "C" fn destroy_error(error: FFIError<std::os::raw::c_void>) {
    FFIError::destroy(error);
}

impl<T: 'static> FFIError<T> {
    pub fn from_value(value: T) -> Self {
        if value.type_id() == FFINull.type_id() {
            // we don't allocated in this case
            Self::from(std::ptr::null_mut())
        } else {
            Self::from(create_heap_pointer(value))
        }
    }
    /// # Safety
    /// Same as `destroy_cstring` and `destroy_pointer`
    pub unsafe fn destroy(self) {
        if self.obj.is_null() {
            if !self.error.is_null() {
                destroy_cstring(self.error);
            }
        } else {
            destroy_pointer(self.obj.cast::<c_void>());
        }
    }
}
