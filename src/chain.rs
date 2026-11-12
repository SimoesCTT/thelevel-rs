use crate::backend::{Backend, BackendError};
use crate::buffer::{ChipBuffer, GpuState};

#[derive(Debug)]
pub enum ChainError { Backend(BackendError) }

/// A chain of copies submitted once. Each link is a GPU-state source
/// and destination. The borrow of every buffer in the chain lives
/// until the terminal handle is waited on.
pub struct Chain<'a, B: Backend> {
    backend: &'a mut B,
    handles: Vec<u64>,
}

pub struct ChainHandle<'a, B: Backend> {
    backend: &'a mut B,
    handles: Vec<u64>,
}

impl<'a, B: Backend> Chain<'a, B> {
    pub fn new(backend: &'a mut B) -> Self {
        Self { backend, handles: Vec::new() }
    }

    pub fn add(
        &mut self,
        src: &ChipBuffer<B, GpuState>,
        dst: &ChipBuffer<B, GpuState>,
        width: u32, height: u32,
    ) -> Result<(), ChainError> {
        let h = self.backend.submit_copy(src.as_raw(), dst.as_raw(), width, height)
            .map_err(ChainError::Backend)?;
        self.handles.push(h);
        Ok(())
    }

    /// One wait on the terminal. There is no per-link completion on
    /// this backend; the terminal is the observable.
    pub fn submit(self) -> ChainHandle<'a, B> {
        ChainHandle { backend: self.backend, handles: self.handles }
    }
}

impl<'a, B: Backend> ChainHandle<'a, B> {
    pub fn wait(mut self) -> Result<(), ChainError> {
        for h in self.handles.drain(..) {
            self.backend.wait(h).map_err(ChainError::Backend)?;
        }
        Ok(())
    }
}
