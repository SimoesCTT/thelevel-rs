//! The wgpu backend. This is the "any GPU" backend: wgpu runs on
//! Vulkan (AMD, Intel, NVIDIA), Metal (Apple), DX12 (Windows), and
//! WebGPU (browsers).
//!
//! The model is the same as i915's. What changes is the mechanism:
//! a buffer is a wgpu::Buffer, the handoff is map_async + poll +
//! unmap, an operation is a command encoder copy_buffer_to_buffer,
//! the completion handle is a SubmissionIndex, the chain is one
//! command encoder with N copies and one submit.

use std::sync::Arc;
use crate::backend::{Backend, BackendError, SyncDirection};

pub struct WgpuBackend {
    device: Arc<wgpu::Device>,
    queue:  Arc<wgpu::Queue>,
    _instance: wgpu::Instance,
    _adapter: wgpu::Adapter,
    next_id: u64,
    buffers: std::collections::HashMap<u64, wgpu::Buffer>,
    submissions: std::collections::HashMap<u64, wgpu::SubmissionIndex>,
}

impl Backend for WgpuBackend {
    type Raw = u64;

    fn open() -> Result<Self, BackendError> {
        let instance = wgpu::Instance::new(&wgpu::InstanceDescriptor {
            backends: wgpu::Backends::all(),
            ..Default::default()
        });
        let adapter = pollster::block_on(instance.request_adapter(
            &wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::HighPerformance,
                compatible_surface: None,
                force_fallback_adapter: false,
            },
        )).ok_or(BackendError::NoDevice)?;
        let (device, queue) = pollster::block_on(adapter.request_device(
            &wgpu::DeviceDescriptor {
                label: Some("thelevel"),
                required_features: wgpu::Features::empty(),
                required_limits: wgpu::Limits::downlevel_defaults(),
                ..Default::default()
            },
            None,
        )).map_err(|e| BackendError::Io(
            std::io::Error::new(std::io::ErrorKind::Other, e.to_string())))?;
        Ok(Self {
            device: Arc::new(device),
            queue:  Arc::new(queue),
            _instance: instance,
            _adapter: adapter,
            next_id: 1,
            buffers: Default::default(),
            submissions: Default::default(),
        })
    }

    fn alloc(&mut self, bytes: usize) -> Result<u64, BackendError> {
        let id = self.next_id;
        self.next_id += 1;
        let buf = self.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("thelevel-chipmem"),
            size: bytes as u64,
            usage: wgpu::BufferUsages::MAP_READ
                 | wgpu::BufferUsages::MAP_WRITE
                 | wgpu::BufferUsages::COPY_SRC
                 | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        self.buffers.insert(id, buf);
        Ok(id)
    }

    fn sync(&mut self, raw: &u64, to: SyncDirection) -> Result<(), BackendError> {
        let buf = self.buffers.get(raw).ok_or(BackendError::Unsupported)?;
        let mode = match to {
            SyncDirection::CpuWrite  => wgpu::MapMode::Write,
            SyncDirection::AccelWrite => wgpu::MapMode::Read,
        };
        let (tx, rx) = std::sync::mpsc::channel();
        buf.slice(..).map_async(mode, move |res| { let _ = tx.send(res); });
        self.device.poll(wgpu::Maintain::Wait);
        rx.recv().map_err(|_| BackendError::Unsupported)?
            .map_err(|_| BackendError::Unsupported)?;
        buf.unmap();
        Ok(())
    }

    fn submit_copy(
        &mut self, src: &u64, dst: &u64, width: u32, _height: u32,
    ) -> Result<u64, BackendError> {
        let s = self.buffers.get(src).ok_or(BackendError::Unsupported)?;
        let d = self.buffers.get(dst).ok_or(BackendError::Unsupported)?;
        let size = (width as u64) * 4;
        let mut enc = self.device.create_command_encoder(
            &wgpu::CommandEncoderDescriptor { label: Some("thelevel-op") });
        enc.copy_buffer_to_buffer(s, 0, d, 0, size);
        let idx = self.queue.submit(std::iter::once(enc.finish()));
        let id = self.next_id;
        self.next_id += 1;
        self.submissions.insert(id, idx);
        Ok(id)
    }

    fn wait(&mut self, handle: u64) -> Result<(), BackendError> {
        if self.submissions.contains_key(&handle) {
            self.device.poll(wgpu::Maintain::Wait);
            self.submissions.remove(&handle);
            Ok(())
        } else {
            Err(BackendError::Unsupported)
        }
    }
}
