```bash
cat > ~/Desktop/thelevel-rs/README.md <<'EOF'
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
let mut ch = Chain::new(&mut backend);
ch.add(&buf, &dst, 1024, 1)?;
ch.submit().wait()?;                              // borrow ends here
```

`ChipBuffer<T, CpuState>` and `ChipBuffer<T, GpuState>` are distinct
types. Reading the CPU pointer in GPU state is a compile error.
Passing a CPU-state buffer to `Chain::add` is a compile error. The
85–100% stale-read hazard measured in the C library becomes
unrepresentable.

## Backends

| backend | mechanism | status |
|---|---|---|
| `CpuBackend` | `Vec<u8>` + memcpy | works, proves the type contract |
| `WgpuBackend` | Vulkan / Metal / DX12 via wgpu | works on native GPUs |

`wgpu` is WebGPU: it enforces the web security model by default. The
handoff uses `Features::MAPPABLE_PRIMARY_BUFFERS` (native-only) so the
buffer is CPU-mappable and GPU-usable in one allocation, which is the
whole point of the model on shared-memory hardware. On a browser
target that feature does not exist and the backend does not apply —
the browser security model assumes the copy.

## Measured

N=8 chain vs N=8 per-op submits, 200 runs per batch.

| engine | chain | per-op | ratio | batches |
|---|---|---|---|---|
| i915 BLT (C, copperlist) | 18–36 µs/link | 224 µs/link | 6.25–12.31× | 4 |
| wgpu/Vulkan (Rust, HD 520) | 91 µs/link | 218 µs/link | 2.31–2.47× | 3 |

The direction is the same on both engines: chaining beats per-op
submission. The magnitude depends on the engine's per-submit cost,
which is higher on the i915 BLT ring than on Vulkan through wgpu.
The i915 range is wide because the batch-setup cost varies; the wgpu
range is tight because Vulkan's submit cost is more uniform.

A single run is a sample. The range across batches is the finding.

The i915 numbers are the C `copperlist` package; the wgpu numbers are
this crate's `examples/bench.rs`. They are different engines, different
languages, and different implementations of the same model.

## Build

```
cargo build
cargo test
cargo run --release --example bench
```

`wgpu_chain_runs_if_a_gpu_is_present` opens whatever adapter wgpu
finds. On a machine with no adapter it prints SKIP — the model is not
a GPU-specific claim.

## Fedora packaging

Packaged in Copr as `rust-thelevel-devel`, built from a vendored crate
tree because `wgpu` and its dependencies are not yet packaged in
Fedora. `dnf install` requires `--skip-broken` until those crates are
in the distribution: the RPM ships the source but cannot satisfy the
`crate(wgpu/default)` provides, because the crates are not in Fedora's
repositories.

```
sudo dnf copr enable simoesctt/rust-thelevel
sudo dnf install --skip-broken rust-thelevel-devel
```

## License

MIT. See LICENSE.
EOF
```


