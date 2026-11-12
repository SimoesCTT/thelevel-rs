//! thelevel — a Rust contract for CPU/GPU shared memory.
//!
//! The model, independent of any GPU:
//!   - a buffer both the CPU and the accelerator can touch,
//!   - one explicit handoff between CPU state and GPU state,
//!   - an operation with a completion handle,
//!   - a chain of operations submitted once.
//!
//! The model is enforced by the type system where it can be:
//! `ChipBuffer<T, CpuState>` and `ChipBuffer<T, GpuState>` are
//! distinct types, and `sync` is the only transition between them.
//! Reading the CPU pointer in GPU state is a compile error.
//!
//! Backends implement `Backend`. The first is the i915 BLT ring,
//! which is `unsafe` in one module. A `wgpu` backend is a stub that
//! returns `Unsupported` until it is written.

pub mod buffer;
pub mod backend;
pub mod op;
pub mod chain;
pub mod backends;

pub use buffer::{ChipBuffer, CpuState, GpuState, BufferError};
pub use backend::{Backend, BackendError};
pub use op::{Op, OpHandle, OpError};
pub use chain::{Chain, ChainHandle, ChainError};
