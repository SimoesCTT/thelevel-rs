# thelevel — the model, in Rust, on any native GPU

The C libraries (`the-level`, `copperlist`, `reach`) **describe** a
contract for CPU/GPU shared memory. This crate **enforces** it: the
handoff is a type transition the compiler checks, and the chain's
completion handle is a borrow.

The model, independent of any GPU:
- a buffer both the CPU and the accelerator can touch,
- one explicit handoff between CPU state and GPU state,
- an operation with a completion handle,
- a chain of operations, one submit, one wait.

## The type-level contract

```rust
let buf = ChipBuffer::new(&mut backend, 4096)?;   // CpuState
let buf = buf.sync(&mut backend)?;                // GpuState — the only way across
let op  = Op::copy(&mut backend, &buf, &dst, 1024, 1)?;
op.wait()?;                                       // borrow ends here

ChipBuffer<T, CpuState> and ChipBuffer<T, GpuState> are distinct
types. Reading the CPU pointer in GPU state is a compile error.
Passing a CPU-state buffer to Op::copy is a compile error. The
85–100% stale-read hazard measured in the C library becomes
unrepresentable.

Backends
backend	mechanism	status
CpuBackend	Vec<u8> + memcpy	works, proves the type contract
WgpuBackend	Vulkan / Metal / DX12 via wgpu	works on native GPUs that share memory
wgpu is WebGPU: it enforces the web security model by default. The
handoff uses Features::MAPPABLE_PRIMARY_BUFFERS (native-only) so the
buffer is CPU-mappable and GPU-usable in one allocation, which is the
whole point of the model on shared-memory hardware. On a browser
target that feature does not exist and the backend does not apply —
the browser security model assumes the copy.

The i915 BLT backend is not yet ported. The C version measures
6.25×–12.31× per link at N=8; the wgpu backend has not been measured,
and the win may differ, because Vulkan's submit already batches.

Build
text
cargo build
cargo test
wgpu_chain_runs_if_a_gpu_is_present opens whatever adapter wgpu
finds. On a machine with no adapter it prints SKIP — the model is not
a GPU-specific claim.

License
MIT. See LICENSE.

## Fedora packaging

Not yet packaged. `rust-wgpu-devel` is not in Fedora 43, and `wgpu`
carries a large `crate(...)` dependency tree. Until that tree is
packaged, the crate builds and tests locally but has no RPM. This is
a dependency-availability problem, not a spec problem.

The other three packages (`the-level`, `copperlist`, `reach`) are in
Copr at simoesctt/the-level, simoesctt/copperlist, simoesctt/reach.
