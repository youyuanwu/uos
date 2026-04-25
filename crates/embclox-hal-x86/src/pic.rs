use x86_64::instructions::port::Port;

/// Disable the legacy 8259 PIC by masking all IRQs.
/// Must be called before enabling the APIC to prevent spurious
/// legacy interrupts.
pub fn disable() {
    unsafe {
        // Remap PIC to vectors 0x20-0x2F to avoid exception overlap,
        // then mask all IRQs.
        // ICW1: init + ICW4 needed
        Port::<u8>::new(0x20).write(0x11);
        Port::<u8>::new(0xA0).write(0x11);
        // ICW2: vector offset
        Port::<u8>::new(0x21).write(0x20); // master: vector 0x20
        Port::<u8>::new(0xA1).write(0x28); // slave: vector 0x28
                                           // ICW3: master/slave wiring
        Port::<u8>::new(0x21).write(0x04); // slave on IRQ2
        Port::<u8>::new(0xA1).write(0x02); // cascade identity
                                           // ICW4: 8086 mode
        Port::<u8>::new(0x21).write(0x01);
        Port::<u8>::new(0xA1).write(0x01);
        // Mask all IRQs
        Port::<u8>::new(0x21).write(0xFF);
        Port::<u8>::new(0xA1).write(0xFF);
    }
    log::info!("Legacy PIC disabled (all IRQs masked)");
}
