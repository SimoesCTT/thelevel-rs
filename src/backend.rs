use std::fmt;

#[derive(Debug)]
pub enum BackendError {
    Unsupported,
    Io(std::io::Error),
    NoDevice,
    OutOfMemory,
}

impl fmt::Display for BackendError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            BackendError::Unsupported => write!(f, "backend does not support this operation"),
            BackendError::Io(e) => write!(f, "io: {e}"),
            BackendError::NoDevice => write!(f, "no device"),
            BackendError::OutOfMemory => write!(f, "out of memory"),
        }
    }
}
impl std::error::Error for BackendError {}
impl From<std::io::Error> for BackendError {
    fn from(e: std::io::Error) -> Self { BackendError::Io(e) }
}

pub trait Backend {
    type Raw: Clone;

    fn open() -> Result<Self, BackendError> where Self: Sized;
    fn alloc(&mut self, bytes: usize) -> Result<Self::Raw, BackendError>;
    fn sync(&mut self, raw: &Self::Raw, to: SyncDirection) -> Result<(), BackendError>;
    fn submit_copy(
        &mut self, src: &Self::Raw, dst: &Self::Raw, width: u32, height: u32,
    ) -> Result<u64, BackendError>;
    fn wait(&mut self, handle: u64) -> Result<(), BackendError>;
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SyncDirection { CpuWrite, AccelWrite }
