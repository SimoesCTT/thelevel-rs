//! A CPU backend. No GPU. It exists to prove the type-level contract
//! compiles: a buffer in CpuState cannot be passed to Op::copy, and
//! only `sync` moves it to GpuState. The copy is memcpy.
use crate::backend::{Backend, BackendError, SyncDirection};

pub struct CpuBackend {
    next_id: u64,
    bufs: std::collections::HashMap<u64, Vec<u8>>,
}

impl Backend for CpuBackend {
    type Raw = u64;

    fn open() -> Result<Self, BackendError> {
        Ok(Self { next_id: 1, bufs: Default::default() })
    }

    fn alloc(&mut self, bytes: usize) -> Result<u64, BackendError> {
        let id = self.next_id;
        self.next_id += 1;
        self.bufs.insert(id, vec![0u8; bytes]);
        Ok(id)
    }

    fn sync(&mut self, _: &u64, _: SyncDirection) -> Result<(), BackendError> {
        Ok(())
    }

    fn submit_copy(
        &mut self, src: &u64, dst: &u64, width: u32, _height: u32,
    ) -> Result<u64, BackendError> {
        let s = self.bufs.get(src).ok_or(BackendError::Unsupported)?.clone();
        let d = self.bufs.get_mut(dst).ok_or(BackendError::Unsupported)?;
        let n = (width as usize) * 4;
        d[..n].copy_from_slice(&s[..n]);
        let id = self.next_id;
        self.next_id += 1;
        Ok(id)
    }

    fn wait(&mut self, _: u64) -> Result<(), BackendError> { Ok(()) }
}
