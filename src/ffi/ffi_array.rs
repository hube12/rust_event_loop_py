use std::mem::ManuallyDrop;

#[repr(C)]
pub struct FFIArray<T> {
    pub(crate) ptr:      *mut T,
    pub(crate) len:      usize,
    pub(crate) capacity: usize,
}

impl<T> From<Vec<T>> for FFIArray<T> {
    fn from(vec: Vec<T>) -> Self {
        let (ptr, len, capacity) = {
            let mut me = ManuallyDrop::new(vec);
            (me.as_mut_ptr(), me.len(), me.capacity())
        };
        Self { ptr, len, capacity }
    }
}

impl<T: Clone> From<&[T]> for FFIArray<T> {
    fn from(vec: &[T]) -> Self {
        Self::from(Vec::from(vec))
    }
}
