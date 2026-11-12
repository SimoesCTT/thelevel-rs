use thelevel::backends::cpu::CpuBackend;
use thelevel::backends::wgpu_backend::WgpuBackend;
use thelevel::{Backend, ChipBuffer, Op, Chain};

#[test]
fn cpu_chain_produces_one_submit_and_one_wait() {
    let mut b = CpuBackend::open().unwrap();
    let a = ChipBuffer::new(&mut b, 4096).unwrap();
    let a = a.sync(&mut b).unwrap();
    let c = ChipBuffer::new(&mut b, 4096).unwrap();
    let c = c.sync(&mut b).unwrap();
    let mut chain = Chain::new(&mut b);
    chain.add(&a, &c, 1024, 1).unwrap();
    chain.submit().wait().unwrap();
}

#[test]
fn wgpu_chain_runs_if_a_gpu_is_present() {
    // This is the "any GPU" test. wgpu picks Vulkan on this machine,
    // which is the i915 through the Vulkan driver. If no adapter,
    // skip: the model is not a GPU-specific claim.
    let mut b = match WgpuBackend::open() {
        Ok(b) => b,
        Err(_) => { eprintln!("SKIP: no wgpu adapter"); return; }
    };
    let a = ChipBuffer::new(&mut b, 4096).unwrap();
    let a = a.sync(&mut b).unwrap();
    let c = ChipBuffer::new(&mut b, 4096).unwrap();
    let c = c.sync(&mut b).unwrap();
    let mut chain = Chain::new(&mut b);
    chain.add(&a, &c, 1024, 1).unwrap();
    chain.submit().wait().unwrap();
}
