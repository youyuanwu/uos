use x86_64::instructions::port::Port;

const PIT_CH2_DATA: u16 = 0x42;
const PIT_CMD: u16 = 0x43;
const PIT_FREQ_HZ: u64 = 1_193_182;

/// Calibrate TSC frequency using the PIT channel 2.
/// Returns TSC ticks per microsecond.
pub fn calibrate_tsc_mhz() -> u64 {
    // We'll measure how many TSC ticks elapse during a known PIT interval.
    // PIT channel 2 in one-shot mode, count = 11932 ≈ 10ms at 1.193182 MHz

    let pit_count: u16 = 11932; // ~10ms
    let expected_us = (pit_count as u64 * 1_000_000) / PIT_FREQ_HZ;

    unsafe {
        // Enable PIT channel 2 gate (via port 0x61)
        let mut port61 = Port::<u8>::new(0x61);
        let val = port61.read();
        // Set bit 0 (gate), clear bit 1 (speaker)
        port61.write((val | 0x01) & !0x02);

        // PIT channel 2, lobyte/hibyte, mode 0 (one-shot), binary
        Port::<u8>::new(PIT_CMD).write(0b10110000);

        // Write count
        Port::<u8>::new(PIT_CH2_DATA).write(pit_count as u8);
        Port::<u8>::new(PIT_CH2_DATA).write((pit_count >> 8) as u8);

        // Read TSC before
        let tsc_start = core::arch::x86_64::_rdtsc();

        // Wait for PIT channel 2 output to go high (bit 5 of port 0x61)
        loop {
            if port61.read() & 0x20 != 0 {
                break;
            }
        }

        let tsc_end = core::arch::x86_64::_rdtsc();
        let tsc_elapsed = tsc_end - tsc_start;
        let tsc_per_us = tsc_elapsed / expected_us;

        log::info!(
            "TSC calibration: {} ticks in ~{}us → {} ticks/us (~{} MHz)",
            tsc_elapsed,
            expected_us,
            tsc_per_us,
            tsc_per_us
        );

        tsc_per_us
    }
}
