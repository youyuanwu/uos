//! Serial EEPROM (SROM) access for MAC address reading.
//!
//! The DEC 21140 stores the MAC address in a serial EEPROM accessed
//! via bit-banging through CSR9.

use crate::csr::{self, CsrAccess};

/// Read a 16-bit word from the serial EEPROM at the given address.
///
/// Returns `None` if the EEPROM does not respond within the timeout.
///
/// # Safety
/// The `csr` access must reference a valid, initialized Tulip device.
pub unsafe fn eeprom_read(csr: &CsrAccess, addr: u8) -> Option<u16> {
    unsafe { csr.write(csr::CSR9, csr::CSR9_SR) };
    eeprom_delay();

    let cmd: u32 = 0x06 | ((addr as u32) << 3);
    let cmd_bits = 9;

    for i in (0..cmd_bits).rev() {
        let bit = if (cmd >> i) & 1 != 0 { csr::CSR9_DI } else { 0 };
        let val = csr::CSR9_SR | csr::CSR9_RD | bit;

        unsafe { csr.write(csr::CSR9, val) };
        eeprom_delay();
        unsafe { csr.write(csr::CSR9, val | csr::CSR9_SK) };
        eeprom_delay();
        unsafe { csr.write(csr::CSR9, val) };
        eeprom_delay();
    }

    let mut data: u16 = 0;
    for _ in 0..16 {
        unsafe { csr.write(csr::CSR9, csr::CSR9_SR | csr::CSR9_RD) };
        eeprom_delay();
        unsafe { csr.write(csr::CSR9, csr::CSR9_SR | csr::CSR9_RD | csr::CSR9_SK) };
        eeprom_delay();

        let val = unsafe { csr.read(csr::CSR9) };
        data = (data << 1) | if val & csr::CSR9_DO != 0 { 1 } else { 0 };

        unsafe { csr.write(csr::CSR9, csr::CSR9_SR | csr::CSR9_RD) };
        eeprom_delay();
    }

    unsafe { csr.write(csr::CSR9, 0) };
    Some(data)
}

/// Read the 6-byte MAC address from the EEPROM.
///
/// # Safety
/// The `csr` access must reference a valid, initialized Tulip device.
pub unsafe fn read_mac(csr: &CsrAccess) -> Option<[u8; 6]> {
    let mut mac = [0u8; 6];

    for i in 0..3 {
        let word = unsafe { eeprom_read(csr, i as u8)? };
        mac[i * 2] = (word & 0xFF) as u8;
        mac[i * 2 + 1] = (word >> 8) as u8;
    }

    if mac == [0; 6] || mac == [0xFF; 6] {
        return None;
    }

    Some(mac)
}

/// Generate a random locally-administered MAC address.
pub fn random_mac(seed: usize) -> [u8; 6] {
    let mut h = seed as u64;
    h = h
        .wrapping_mul(6364136223846793005)
        .wrapping_add(1442695040888963407);
    let bytes = h.to_le_bytes();
    let mut mac = [bytes[0], bytes[1], bytes[2], bytes[3], bytes[4], bytes[5]];
    mac[0] = (mac[0] & 0xFC) | 0x02;
    mac
}

fn eeprom_delay() {
    for _ in 0..10 {
        core::hint::spin_loop();
    }
}
