use crate::config::boot_env;
use obconf::BootEnv;

/// Main entry point for interrupt.
///
/// This will be called by an inline assembly.
///
/// See `trap` function on the PS4 for a reference.
pub extern "C" fn interrupt_handler(frame: &mut TrapFrame) {
    match frame.num {
        TrapNo::Breakpoint => match boot_env() {
            BootEnv::Vm(vm) => super::vm::interrupt_handler(vm, frame),
        },
    }
}

/// Predefined interrupt vector number.
#[allow(dead_code)] // Used by inline assembly.
#[repr(u32)]
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum TrapNo {
    Breakpoint = 3, // T_BPTFLT
}

/// Contains states of the interupted program.
#[repr(C)]
pub struct TrapFrame {
    pub rdi: usize,  // tf_rdi
    pub rsi: usize,  // tf_rsi
    pub rdx: usize,  // tf_rdx
    pub rcx: usize,  // tf_rcx
    pub r8: usize,   // tf_r8
    pub r9: usize,   // tf_r9
    pub rax: usize,  // tf_rax
    pub rbx: usize,  // tf_rbx
    pub rbp: usize,  // tf_rbp
    pub r10: usize,  // tf_r10
    pub r11: usize,  // tf_r11
    pub r12: usize,  // tf_r12
    pub r13: usize,  // tf_r13
    pub r14: usize,  // tf_r14
    pub r15: usize,  // tf_r15
    pub num: TrapNo, // tf_trapno
    pub fs: u16,     // tf_fs
    pub gs: u16,     // tf_gs
}
