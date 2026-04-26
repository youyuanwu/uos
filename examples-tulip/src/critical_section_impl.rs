//! Critical section implementation using x86 interrupt disable/enable.

struct X86CriticalSection;
critical_section::set_impl!(X86CriticalSection);

unsafe impl critical_section::Impl for X86CriticalSection {
    unsafe fn acquire() -> bool {
        let flags: u64;
        unsafe { core::arch::asm!("pushfq; pop {}", out(reg) flags, options(nomem, nostack)) };
        let was_enabled = flags & (1 << 9) != 0;
        unsafe { core::arch::asm!("cli", options(nomem, nostack)) };
        was_enabled
    }

    unsafe fn release(was_enabled: bool) {
        if was_enabled {
            unsafe { core::arch::asm!("sti", options(nomem, nostack)) };
        }
    }
}
