use self::cpu::KvmCpu;
use self::ffi::{
    kvm_check_version, kvm_create_vcpu, kvm_create_vm, kvm_get_vcpu_mmap_size, kvm_max_vcpus,
    kvm_set_user_memory_region,
};
use super::{HypervisorError, MemoryAddr, Platform, Ram};
use libc::{mmap, open, MAP_FAILED, MAP_PRIVATE, O_RDWR, PROT_READ, PROT_WRITE};
use std::os::fd::{AsRawFd, FromRawFd, OwnedFd};
use std::ptr::null_mut;
use std::sync::Arc;
use thiserror::Error;

mod cpu;
mod ffi;
mod regs;
mod run;

/// Implementation of [`Platform`] using KVM.
///
/// Fields in this struct need to drop in a correct order (e.g. vm must be dropped before ram).
pub struct Kvm {
    vcpu_mmap_size: usize,
    vm: OwnedFd,
    ram: Arc<Ram>,
    kvm: OwnedFd,
}

impl Kvm {
    pub fn new(cpu: usize, ram: Arc<Ram>) -> Result<Self, HypervisorError> {
        use std::io::Error;

        // Open KVM device.
        let kvm = unsafe { open(c"/dev/kvm".as_ptr(), O_RDWR) };

        if kvm < 0 {
            return Err(HypervisorError::OpenKvmFailed(Error::last_os_error()));
        }

        // Check KVM version.
        let kvm = unsafe { OwnedFd::from_raw_fd(kvm) };
        let mut compat = false;

        match unsafe { kvm_check_version(kvm.as_raw_fd(), &mut compat) } {
            0 if !compat => {
                return Err(HypervisorError::KvmVersionMismatched);
            }
            0 => {}
            v => {
                return Err(HypervisorError::GetKvmVersionFailed(
                    Error::from_raw_os_error(v),
                ))
            }
        }

        // Check max CPU.
        let mut max = 0;

        match unsafe { kvm_max_vcpus(kvm.as_raw_fd(), &mut max) } {
            0 => {}
            v => {
                return Err(HypervisorError::GetMaxCpuFailed(Error::from_raw_os_error(
                    v,
                )));
            }
        }

        if max < cpu {
            return Err(HypervisorError::MaxCpuTooLow);
        }

        // Get size of CPU context.
        let vcpu_mmap_size = match unsafe { kvm_get_vcpu_mmap_size(kvm.as_raw_fd()) } {
            size @ 0.. => size as usize,
            _ => return Err(HypervisorError::GetMmapSizeFailed(Error::last_os_error())),
        };

        // Create a VM.
        let mut vm = -1;

        match unsafe { kvm_create_vm(kvm.as_raw_fd(), &mut vm) } {
            0 => {}
            v => return Err(HypervisorError::CreateVmFailed(Error::from_raw_os_error(v))),
        }

        // Set RAM.
        let vm = unsafe { OwnedFd::from_raw_fd(vm) };
        let slot = 0;
        let addr = ram.vm_addr().try_into().unwrap();
        let len = ram.len().try_into().unwrap();
        let mem = ram.host_addr().cast();

        match unsafe { kvm_set_user_memory_region(vm.as_raw_fd(), slot, addr, len, mem) } {
            0 => {}
            v => return Err(HypervisorError::MapRamFailed(Error::from_raw_os_error(v))),
        }

        Ok(Self {
            vcpu_mmap_size,
            vm,
            ram,
            kvm,
        })
    }
}

impl Platform for Kvm {
    type Cpu<'a> = KvmCpu<'a>;
    type CpuErr = KvmCpuError;

    fn create_cpu(&self, id: usize) -> Result<Self::Cpu<'_>, Self::CpuErr> {
        use std::io::Error;

        // Create vCPU.
        let id = id.try_into().unwrap();
        let mut vcpu = -1;
        let vcpu = match unsafe { kvm_create_vcpu(self.vm.as_raw_fd(), id, &mut vcpu) } {
            0 => unsafe { OwnedFd::from_raw_fd(vcpu) },
            v => return Err(KvmCpuError::CreateVcpuFailed(Error::from_raw_os_error(v))),
        };

        // Get kvm_run.
        let cx = unsafe {
            mmap(
                null_mut(),
                self.vcpu_mmap_size,
                PROT_READ | PROT_WRITE,
                MAP_PRIVATE,
                vcpu.as_raw_fd(),
                0,
            )
        };

        if cx == MAP_FAILED {
            return Err(KvmCpuError::GetKvmRunFailed(Error::last_os_error()));
        }

        Ok(unsafe { KvmCpu::new(vcpu, cx.cast(), self.vcpu_mmap_size) })
    }
}

/// Implementation of [`Platform::CpuErr`].
#[derive(Debug, Error)]
pub enum KvmCpuError {
    #[error("failed to create vcpu")]
    CreateVcpuFailed(#[source] std::io::Error),

    #[error("couldn't get a pointer to kvm_run")]
    GetKvmRunFailed(#[source] std::io::Error),
}
