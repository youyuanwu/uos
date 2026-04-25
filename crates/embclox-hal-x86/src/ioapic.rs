use core::ptr;

/// IOAPIC default physical address (QEMU q35/pc).
pub const IOAPIC_PHYS_BASE: u64 = 0xFEC0_0000;

const IOREGSEL: usize = 0x00;
const IOWIN: usize = 0x10;

// IOAPIC registers
const IOAPICID: u8 = 0x00;
const IOAPICVER: u8 = 0x01;
const IOREDTBL_BASE: u8 = 0x10;

/// I/O APIC for routing external interrupts to LAPIC vectors.
pub struct IoApic {
    base: usize,
}

impl IoApic {
    /// Create from a UC-mapped virtual address.
    pub fn new(base: usize) -> Self {
        Self { base }
    }

    fn read(&self, reg: u8) -> u32 {
        unsafe {
            ptr::write_volatile((self.base + IOREGSEL) as *mut u32, reg as u32);
            ptr::read_volatile((self.base + IOWIN) as *const u32)
        }
    }

    fn write(&self, reg: u8, value: u32) {
        unsafe {
            ptr::write_volatile((self.base + IOREGSEL) as *mut u32, reg as u32);
            ptr::write_volatile((self.base + IOWIN) as *mut u32, value);
        }
    }

    /// Get the maximum number of redirection entries.
    pub fn max_entries(&self) -> u8 {
        ((self.read(IOAPICVER) >> 16) & 0xFF) as u8 + 1
    }

    /// Route an IRQ to a specific LAPIC vector.
    /// `irq`: IOAPIC input pin (e.g., PCI IRQ line)
    /// `vector`: IDT vector number
    /// `lapic_id`: destination LAPIC ID (usually 0 for BSP)
    pub fn enable_irq(&mut self, irq: u8, vector: u8, lapic_id: u8) {
        let reg_low = IOREDTBL_BASE + irq * 2;
        let reg_high = reg_low + 1;

        // Low 32 bits: vector, delivery mode fixed, active low, level-triggered
        // For edge-triggered (typical for PCI legacy): bits 13,15 = 0
        let low = vector as u32; // fixed delivery, edge, active high
        let high = (lapic_id as u32) << 24;

        self.write(reg_high, high);
        self.write(reg_low, low);

        log::info!(
            "IOAPIC: IRQ {} -> vector {} (LAPIC {})",
            irq,
            vector,
            lapic_id
        );
    }

    /// Mask (disable) an IRQ.
    pub fn disable_irq(&mut self, irq: u8) {
        let reg_low = IOREDTBL_BASE + irq * 2;
        let low = self.read(reg_low);
        self.write(reg_low, low | (1 << 16)); // set mask bit
    }

    /// Log IOAPIC info.
    pub fn log_info(&self) {
        log::info!(
            "IOAPIC: ID={:#x}, max_entries={}",
            self.read(IOAPICID),
            self.max_entries()
        );
    }
}
