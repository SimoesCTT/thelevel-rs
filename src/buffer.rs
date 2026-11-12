use std::marker::PhantomData;
use crate::backend::{Backend, BackendError, SyncDirection};

pub struct CpuState;
pub struct GpuState;

#[derive(Debug)]
pub enum BufferError {
    Backend(BackendError),
    InvalidState,
}

/// A buffer in CPU state. The type parameter is the contract: you
/// cannot call `accel_ptr` or hand it to an `Op` until `sync` moves
/// it to `GpuState`.
pub struct ChipBuffer<B: Backend, S = CpuState> {
    raw: B::Raw,
    bytes: usize,
    _state: PhantomData<S>,
}

impl<B: Backend> ChipBuffer<B, CpuState> {
    pub fn new(backend: &mut B, bytes: usize) -> Result<Self, BufferError> {
        let raw = backend.alloc(bytes).map_err(BufferError::Backend)?;
        Ok(Self { raw, bytes, _state: PhantomData })
    }

    /// The one handoff. Consumes the CPU-state buffer, returns the
    /// GPU-state buffer. There is no other way to cross the line.
    pub fn sync(self, backend: &mut B) -> Result<ChipBuffer<B, GpuState>, BufferError> {
        backend.sync(&self.raw, SyncDirection::CpuWrite)
            .map_err(BufferError::Backend)?;
        Ok(ChipBuffer { raw: self.raw, bytes: self.bytes, _state: PhantomData })
    }

    pub fn as_cpu(&self) -> &B::Raw { &self.raw }
    pub fn as_cpu_mut(&mut self) -> &mut B::Raw { &mut self.raw }
    pub fn bytes(&self) -> usize { self.bytes }
}

impl<B: Backend> ChipBuffer<B, GpuState> {
    /// Back to CPU state, after the accelerator has finished.
    pub fn sync_back(self, backend: &mut B) -> Result<ChipBuffer<B, CpuState>, BufferError> {
        backend.sync(&self.raw, SyncDirection::AccelWrite)
            .map_err(BufferError::Backend)?;
        Ok(ChipBuffer { raw: self.raw, bytes: self.bytes, _state: PhantomData })
    }

    pub fn as_raw(&self) -> &B::Raw { &self.raw }
    pub fn bytes(&self) -> usize { self.bytes }
}
