#[derive(Debug, Clone, Default)]
#[repr(C)]
pub(super) struct IRetFrame {
    pub(super) ip: usize,
    pub(super) cs: usize,
    pub(super) flags: usize,

    // ----
    // The following will only be present if interrupt is raised from another
    // privilege ring. Otherwise, they are undefined values.
    // ----
    pub(super) sp: usize,
    pub(super) ss: usize,
}

// TODO: More segment registers?
#[derive(Debug, Clone, Default)]
#[repr(C)]
pub(super) struct Registers {
    #[cfg(target_arch = "x86_64")]
    pub(super) r15: usize,
    #[cfg(target_arch = "x86_64")]
    pub(super) r14: usize,
    #[cfg(target_arch = "x86_64")]
    pub(super) r13: usize,
    #[cfg(target_arch = "x86_64")]
    pub(super) r12: usize,
    #[cfg(target_arch = "x86_64")]
    pub(super) r11: usize,
    #[cfg(target_arch = "x86_64")]
    pub(super) r10: usize,
    #[cfg(target_arch = "x86_64")]
    pub(super) r9: usize,
    #[cfg(target_arch = "x86_64")]
    pub(super) r8: usize,
    pub(super) bp: usize,
    pub(super) di: usize,
    pub(super) si: usize,
    pub(super) dx: usize,
    pub(super) cx: usize,
    pub(super) bx: usize,
    pub(super) ax: usize,
}

#[derive(Debug, Clone, Default)]
#[repr(C)]
pub struct InterruptStackFrame {
    /// Will be undefined if there is no error code
    pub(super) error_code: usize,
    pub(super) registers: Registers,
    pub(super) iret: IRetFrame,
}

#[macro_export]
macro_rules! wrap_interrupt {
    (error code) => {
        concat!(
            // Move eax into code's place, put code in last instead (to be
            // compatible with InterruptStack)
            "xchg (%esp), %eax\n",
        )
    };
    (no error code) => {
        concat!(
            // Clear eax so exit code is zero
            "push %eax\n",
            "xor %eax, %eax\n",
        )
    };
    ($name: ident, $interrupt: literal, $error_code: expr, $postfix: expr) => {
        #[unsafe(naked)]
        pub unsafe extern "C" fn $name() {
            core::arch::naked_asm!(
                concat!(
                    $error_code,
                    "push %ebx\n",
                    "push %ecx\n",
                    "push %edx\n",
                    "push %esi\n",
                    "push %edi\n",
                    "push %ebp\n",
                    "push %eax\n",
                    "mov %esp, %edx\n",
                    "mov $", $interrupt, ", %ecx\n",
                    "call {interrupt_handler}\n",
                    "pop %eax\n",
                    "pop %ebp\n",
                    "pop %edi\n",
                    "pop %esi\n",
                    "pop %edx\n",
                    "pop %ecx\n",
                    "pop %ebx\n",
                    "pop %eax\n",
                    "iret\n",
                ),
                interrupt_handler = sym interrupt_handler,
                options(att_syntax),
            );
        }
    };
}

pub(super) use wrap_interrupt;
