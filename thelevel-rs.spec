%global crate thelevel
%global debug_package %{nil}

Name:           rust-thelevel-devel
Version:        0.1.0
Release:        1%{?dist}
Summary:        CPU/GPU shared-memory model in Rust, enforced by the type system

License:        MIT
URL:            https://github.com/SimoesCTT/thelevel-rs
Source0:        %{url}/archive/v%{version}/thelevel-rs-%{version}.tar.gz
Source1:        thelevel-rs-%{version}-vendor.tar.xz

BuildRequires:  cargo-rpm-macros >= 26
BuildRequires:  rust
BuildRequires:  rust-std-static

%description
The CPU/GPU shared-memory model in Rust: a buffer both the CPU and
the accelerator can touch, one explicit handoff, an operation with a
completion handle, and a chain submitted once. The handoff is a
type-level transition: ChipBuffer<T, CpuState> and
ChipBuffer<T, GpuState> are distinct types, sync is the only way
across, and Op::copy takes only GPU-state buffers.

Backends: CpuBackend (proves the type contract without a GPU) and
WgpuBackend (Vulkan/Metal/DX12 via wgpu, native only).

This package builds from a vendored crate tree because wgpu and its
dependencies are not yet packaged in Fedora.

%prep
%autosetup -n thelevel-rs-%{version}
tar -xf %{SOURCE1}
%cargo_prep -v vendor

%build
%cargo_vendor_manifest
%cargo_build

%install
%cargo_install

%check
%cargo_test

%files
%license LICENSE
%license cargo-vendor.txt
%doc README.md
%{_datadir}/cargo/registry/thelevel-%{version}

%changelog
* Fri Nov 13 2026 Americo Simoes <amexsimoes@gmail.com> - 0.1.0-1
- Initial package, vendored crate tree.
