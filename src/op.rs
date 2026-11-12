use crate::backend::{Backend, BackendError};
use crate::buffer::{ChipBuffer, GpuState};

#[derive(Debug)]
pub enum OpError { Backend(BackendError) }

/// A completion handle. The borrow of the buffers it touches ends
/// when the handle is consumed by `wait`. That is the stability
/// guarantee: waiting on op N does not wait on N+1, because the
/// handle for N is a distinct value with a distinct lifetime.
pub struct OpHandle<'a, B: Backend> {
    backend: &'a mut B,
    handle: u64,
}

pub struct Op<'a, B: Backend> {
    handle: OpHandle<'a, B>,
}

impl<'a, B: Backend> Op<'a, B> {
    /// Submit one copy. Both buffers must be in GPU state; the type
    /// system enforces it.
    pub fn copy(
        backend: &'a mut B,
        src: &ChipBuffer<B, GpuState>,
        dst: &ChipBuffer<B, GpuState>,
        width: u32, height: u32,
    ) -> Result<Self, OpError> {
        let h = backend.submit_copy(src.as_raw(), dst.as_raw(), width, height)
            .map_err(OpError::Backend)?;
        Ok(Self { handle: OpHandle { backend, handle: h } })
    }

    pub fn wait(self) -> Result<(), OpError> {
        self.handle.backend.wait(self.handle.handle).map_err(OpError::Backend)
    }
}
