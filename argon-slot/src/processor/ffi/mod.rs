use std::ffi::c_void;

mod r#async;
pub use r#async::CAsyncSlotProcessor;

mod sync;
pub use sync::CSyncSlotProcessor;

#[repr(C)]
pub struct CEvent {
    pub data: *mut c_void,
}

unsafe impl Send for CEvent {}
unsafe impl Sync for CEvent {}

#[repr(C)]
pub struct CData {
    pub data: *mut c_void,
}

unsafe impl Send for CData {}
unsafe impl Sync for CData {}

#[repr(C)]
pub struct CAction {
    pub data: *mut c_void,
}

unsafe impl Send for CAction {}
unsafe impl Sync for CAction {}

#[repr(C)]
pub struct CActions(pub *mut CAction, pub usize, pub usize);

impl From<Vec<CAction>> for CActions {
    fn from(value: Vec<CAction>) -> Self {
        let (ptr, size, capacity) = value.into_raw_parts();
        Self(ptr, size, capacity)
    }
}
