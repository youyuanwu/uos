use core::task::Waker;
use embassy_time_driver::Driver;

/// Minimal time driver using x86 TSC.
/// Always wakes immediately — relies on the executor re-checking time.
struct TscTimeDriver;

impl Driver for TscTimeDriver {
    fn now(&self) -> u64 {
        unsafe { core::arch::x86_64::_rdtsc() / 1000 }
    }

    fn schedule_wake(&self, _at: u64, waker: &Waker) {
        // Always wake immediately — the executor will re-poll and
        // embassy-time will re-check if the deadline has passed.
        waker.wake_by_ref();
    }
}

embassy_time_driver::time_driver_impl!(static DRIVER: TscTimeDriver = TscTimeDriver);
