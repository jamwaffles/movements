use std::{any::Any, cell::UnsafeCell, sync::Arc};

use crate::native_thread as imp;

/// Inner representation for JoinHandle
pub(crate) struct JoinInner<T> {
    pub(crate) native: Option<imp::Thread>,
    // thread: Thread,
    pub(crate) packet: Packet<T>,
}

impl<T> JoinInner<T> {
    fn join(&mut self) -> Result<T> {
        self.native.take().unwrap().join();
        unsafe { (*self.packet.0.get()).take().unwrap() }
    }
}

pub struct JoinHandle<T>(pub(crate) JoinInner<T>);

unsafe impl<T> Send for JoinHandle<T> {}
unsafe impl<T> Sync for JoinHandle<T> {}

impl<T> JoinHandle<T> {
    pub fn join(mut self) -> Result<T> {
        self.0.join()
    }
}

pub type Result<T> = std::result::Result<T, Box<dyn Any + Send + 'static>>;

pub(crate) struct Packet<T>(pub Arc<UnsafeCell<Option<Result<T>>>>);

unsafe impl<T: Send> Send for Packet<T> {}
unsafe impl<T: Sync> Sync for Packet<T> {}
