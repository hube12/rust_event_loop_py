use std::{ffi::CString, os::raw::c_void};

use crate::ffi::{FFIArray, FFIError};

pub fn create_heap_pointer<T>(x: T) -> *mut T {
    Box::into_raw(Box::new(x))
}

pub fn create_void_heap_pointer<T>(x: T) -> *mut c_void {
    Box::into_raw(Box::new(x)).cast::<c_void>()
}

/// Trait that allow to get a safe mutable reference from a pointer
///
/// # Safety
///
/// The `safe` function will return an Error if a null pointer is passed instead
/// of a valid one.
/// Valid is defined as follows :
///    * The pointer must be properly aligned.
///
///    * It must be "dereferencable" in the sense defined in [the module
///      documentation].
///
///    * The pointer must point to an initialized instance of `T`.
///
///    * You must enforce Rust's aliasing rules, since the returned lifetime
///      `'a` is arbitrarily chosen and does not necessarily reflect the actual
///      lifetime of the data. In particular, for the duration of this lifetime,
///      the memory the pointer points to must not get accessed (read or
///      written) through any other pointer.
pub(crate) trait FFISafe<Input = Self> {
    fn safe<'a, T>(unsafe_ref: *mut Input) -> Result<&'a mut Input, FFIError<T>> {
        match unsafe { unsafe_ref.as_mut() } {
            None => Err(FFIError::from(anyhow::Error::msg(format!(
                "{} should be initialized",
                std::any::type_name::<T>()
            )))),
            Some(rt) => Ok(rt),
        }
    }
}

/// Trait that allow the removal of the content of the pointer and transfer the
/// ownership This should be used for temporary object as it allows the user to
/// do thing in a less error prone way
///
/// # Safety
///
/// The `safe_move` function will put a null pointer inside the passed one :
///    * If the pointer was already a null one then an Error is returned.
///
///    * If the pointer is dangling, then we can not do anything about it (this
///      **should** never happen as long as objects comes from this FFI)
///
///    * If the pointer is not readable then we can not do anything about it
///
///    * If the pointer is not aligned we can not do anything about it either
///
///    * The pointer must point to heap memory and not stack memory
///     (this means no coercion, only Box::leak/into_raw)
///
/// # Warning
///
/// In the code you might see the pattern where we take a pointer to a FFI
/// pointer and simply null that one, which means that technically the FFI
/// pointer is not null, this is done for simplicity sake, however you have to
/// on the other FFI boundary to null that pointer yourself since it has been
/// freed once moved.
pub(crate) trait FFISafeMove<Input = Self> {
    fn safe_move<T>(unsafe_ref: &mut *mut Input) -> Result<Input, FFIError<T>> {
        if unsafe_ref.is_null() {
            return Err(FFIError::from(anyhow::Error::msg(format!(
                "{} should be initialized",
                std::any::type_name::<T>()
            ))));
        }

        let mut ptr = std::ptr::null_mut();
        std::mem::swap(unsafe_ref, &mut ptr); // swap the previous pointer with null
        Ok(unsafe { *Box::from_raw(ptr) }) // make a copy
        // drop ptr
    }
}

pub(crate) trait FFIInner {
    type Output;
    /// Move the inner content out of the object and replace it with a null
    /// pointer
    fn inner(self) -> Self::Output;
}

pub(crate) trait FFIBorrowInner {
    type Output;
    /// Borrow the inner content of the object
    fn borrow_inner(&mut self) -> &mut Self::Output;
}

pub(crate) fn ffi_cstring(msg: &str, err_msg: &'static str) -> FFIError<std::os::raw::c_char> {
    match CString::new(msg.as_bytes()) {
        Ok(cstr) => FFIError::from(cstr.into_raw()),
        Err(_) => FFIError::from(anyhow::Error::msg(err_msg)),
    }
}

pub(crate) fn ffi_string(msg: &str) -> FFIArray<std::os::raw::c_uchar> {
    FFIArray::from(msg.as_bytes())
}

pub(crate) fn ffi_bytes_array(msg: &[u8]) -> FFIArray<std::os::raw::c_uchar> {
    FFIArray::from(msg)
}

pub(crate) fn ffi_bytes_vec(msg: Vec<u8>) -> FFIArray<std::os::raw::c_uchar> {
    FFIArray::from(msg)
}

/// # Safety
/// We check that the pointer is not null before destroying it
#[no_mangle]
pub unsafe extern "C" fn destroy_pointer(pointer: *mut std::ffi::c_void) {
    if !pointer.is_null() {
        let _ = Box::from_raw(pointer);
    }
}

/// # Safety
/// We check that the pointer is not null before destroying it
#[no_mangle]
pub unsafe extern "C" fn destroy_cstring(pointer: *mut std::os::raw::c_char) {
    if !pointer.is_null() {
        let _ = CString::from_raw(pointer);
    }
}

/// This function can not be specialized directly so it needs to be
/// implemented per structure
///
/// # Safety
/// This should be called only for an allocated array via the previous trait
/// We do not guarantee any safety, this will deadly fail if the pointer is
/// tampered with
#[no_mangle]
pub unsafe extern "C" fn destroy_array(ptr_arr: *mut std::ffi::c_void) {
    if !ptr_arr.is_null() {
        let arr = (ptr_arr as *mut FFIArray<c_void>).read();
        let _ = Vec::from_raw_parts(arr.ptr, arr.len, arr.capacity);
    }
    destroy_pointer(ptr_arr);
}


pub struct FFITriple<T, TT, TTT> {
    pub(crate) first: Option<T>,
    pub(crate) second: Option<TT>,
    pub(crate) third: Option<TTT>,
}

impl<T, TT, TTT> FFISafe for FFITriple<T, TT, TTT> {}

impl<T, TT, TTT> From<(T, TT, TTT)> for FFITriple<T, TT, TTT> {
    fn from((a, b, c): (T, TT, TTT)) -> Self {
        Self {
            first: Some(a),
            second: Some(b),
            third: Some(c),
        }
    }
}