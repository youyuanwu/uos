//! DEC 21140 "Tulip" Ethernet driver for bare-metal x86_64.
//!
//! Supports the DEC 21140 (PCI 0x1011:0x0009) and DEC 21143 (PCI 0x1011:0x0019)
//! Tulip family NICs. Used on Hyper-V Gen1 (legacy network adapter) and QEMU
//! (`-device tulip`).

#![no_std]

pub mod csr;
pub mod desc;
pub mod device;
pub mod eeprom;

pub use device::TulipDevice;
