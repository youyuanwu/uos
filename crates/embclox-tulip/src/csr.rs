//! CSR (Control/Status Register) definitions for the DEC 21140/21143 Tulip.
//!
//! Registers are at 8-byte intervals from the PCI BAR0 base address.
//! Access is memory-mapped volatile 32-bit read/write.

// CSR offsets (byte offsets from BAR0)
pub const CSR0: usize = 0x00; // Bus Mode
pub const CSR1: usize = 0x08; // TX Poll Demand
pub const CSR2: usize = 0x10; // RX Poll Demand
pub const CSR3: usize = 0x18; // RX Descriptor List Base
pub const CSR4: usize = 0x20; // TX Descriptor List Base
pub const CSR5: usize = 0x28; // Status
pub const CSR6: usize = 0x30; // Operation Mode
pub const CSR7: usize = 0x38; // Interrupt Enable
pub const CSR8: usize = 0x40; // Missed Frames
pub const CSR9: usize = 0x48; // Boot ROM / Serial ROM / MII
pub const CSR11: usize = 0x58; // Timer
pub const CSR12: usize = 0x60; // SIA Status

// CSR0 bits
pub const CSR0_SWR: u32 = 1 << 0; // Software Reset

// CSR5 (Status) bits
pub const CSR5_TI: u32 = 1 << 0; // Transmit Interrupt
pub const CSR5_RI: u32 = 1 << 6; // Receive Interrupt
pub const CSR5_NIS: u32 = 1 << 16; // Normal Interrupt Summary
pub const CSR5_AIS: u32 = 1 << 15; // Abnormal Interrupt Summary

// CSR6 (Operation Mode) bits
pub const CSR6_SR: u32 = 1 << 1; // Start/Stop Receive
pub const CSR6_ST: u32 = 1 << 13; // Start/Stop Transmit

// CSR7 (Interrupt Enable) bits
pub const CSR7_TIE: u32 = 1 << 0; // Transmit Interrupt Enable
pub const CSR7_RIE: u32 = 1 << 6; // Receive Interrupt Enable
pub const CSR7_NIE: u32 = 1 << 16; // Normal Interrupt Summary Enable
pub const CSR7_AIE: u32 = 1 << 15; // Abnormal Interrupt Summary Enable

// CSR9 (Serial ROM) bits
pub const CSR9_SR: u32 = 1 << 11; // Serial ROM select
pub const CSR9_RD: u32 = 1 << 14; // Serial ROM read
pub const CSR9_DO: u32 = 1 << 3; // Serial ROM data out
pub const CSR9_DI: u32 = 1 << 2; // Serial ROM data in
pub const CSR9_SK: u32 = 1 << 0; // Serial ROM clock

/// Read a CSR register via volatile MMIO.
///
/// # Safety
/// `base` must be a valid UC-mapped MMIO base address for the Tulip device.
pub unsafe fn csr_read_mmio(base: usize, offset: usize) -> u32 {
    let ptr = (base + offset) as *const u32;
    unsafe { core::ptr::read_volatile(ptr) }
}

/// Write a CSR register via volatile MMIO.
///
/// # Safety
/// `base` must be a valid UC-mapped MMIO base address for the Tulip device.
pub unsafe fn csr_write_mmio(base: usize, offset: usize, value: u32) {
    let ptr = (base + offset) as *mut u32;
    unsafe { core::ptr::write_volatile(ptr, value) }
}

/// Read a CSR register via I/O port access.
///
/// # Safety
/// `io_base` must be a valid I/O port base address for the Tulip device.
pub unsafe fn csr_read_io(io_base: u16, offset: usize) -> u32 {
    let port = io_base + offset as u16;
    let value: u32;
    unsafe {
        core::arch::asm!("in eax, dx", in("dx") port, out("eax") value, options(nomem, nostack));
    }
    value
}

/// Write a CSR register via I/O port access.
///
/// # Safety
/// `io_base` must be a valid I/O port base address for the Tulip device.
pub unsafe fn csr_write_io(io_base: u16, offset: usize, value: u32) {
    let port = io_base + offset as u16;
    unsafe {
        core::arch::asm!("out dx, eax", in("dx") port, in("eax") value, options(nomem, nostack));
    }
}

/// Access mode for CSR registers — either MMIO or I/O ports.
#[derive(Clone, Copy)]
pub enum CsrAccess {
    /// Memory-mapped I/O at a virtual address.
    Mmio(usize),
    /// I/O port space at a base port.
    Io(u16),
}

impl CsrAccess {
    /// Read a CSR register.
    ///
    /// # Safety
    /// The base address/port must be valid for the Tulip device.
    pub unsafe fn read(&self, offset: usize) -> u32 {
        match *self {
            CsrAccess::Mmio(base) => unsafe { csr_read_mmio(base, offset) },
            CsrAccess::Io(base) => unsafe { csr_read_io(base, offset) },
        }
    }

    /// Write a CSR register.
    ///
    /// # Safety
    /// The base address/port must be valid for the Tulip device.
    pub unsafe fn write(&self, offset: usize, value: u32) {
        match *self {
            CsrAccess::Mmio(base) => unsafe { csr_write_mmio(base, offset, value) },
            CsrAccess::Io(base) => unsafe { csr_write_io(base, offset, value) },
        }
    }
}
