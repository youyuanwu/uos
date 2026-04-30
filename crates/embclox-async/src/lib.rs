//! Tiny one-future runner with a caller-supplied "park" function.
//!
//! Used by the embclox example kernels to drive synchronous boot-time
//! initialization that wants `async fn` ergonomics but cannot run the
//! full embassy executor yet (e.g. before any task can be spawned).
//!
//! The host-side [`block_on_with`] is pure `core`: no allocator, no
//! target-specific instructions. Kernels wrap it with a target-specific
//! halt — see `embclox_hal_x86::runtime::block_on_hlt` for the
//! `sti; hlt` version used on x86_64.

#![no_std]

use core::future::Future;
use core::pin::Pin;
use core::task::{Context, Poll, RawWaker, RawWakerVTable, Waker};

static NOOP_VTABLE: RawWakerVTable = RawWakerVTable::new(
    |_| RawWaker::new(core::ptr::null(), &NOOP_VTABLE),
    |_| {},
    |_| {},
    |_| {},
);

/// Run a single future to completion, calling `park` between polls.
///
/// On each iteration:
/// 1. Poll the future with a no-op waker.
/// 2. If `Ready`, return the value.
/// 3. Otherwise, call `park` (typically `sti; hlt` on x86) and loop.
///
/// The no-op waker is intentional. Callers don't need to register a
/// real waker because `park` is expected to block until *some*
/// hardware event occurs (interrupt, DMA completion, etc.) and we
/// then unconditionally re-poll on the next loop iteration. This
/// keeps the implementation 12 lines with no executor state.
///
/// # Caller contract
///
/// - `park` must return whenever any event that might change the
///   future's readiness occurs. On x86 this means: at minimum, a
///   periodic timer interrupt is configured so `park` doesn't sleep
///   forever waiting for an event the future is polling for.
/// - The future itself **must not** call any blocking primitive
///   (e.g., `hlt`). It must return [`Poll::Pending`] when not ready
///   and let `block_on_with` drive the halt. Otherwise the caller's
///   `park` may be skipped and an interrupt could be lost.
pub fn block_on_with<F: Future>(mut fut: F, mut park: impl FnMut()) -> F::Output {
    // Safety: we never move `fut` after pinning it here.
    let mut fut = unsafe { Pin::new_unchecked(&mut fut) };
    let raw = RawWaker::new(core::ptr::null(), &NOOP_VTABLE);
    // Safety: NOOP_VTABLE callbacks are all no-ops; the raw pointer is
    // never dereferenced.
    let waker = unsafe { Waker::from_raw(raw) };
    let mut cx = Context::from_waker(&waker);
    loop {
        if let Poll::Ready(r) = fut.as_mut().poll(&mut cx) {
            return r;
        }
        park();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use core::cell::Cell;

    /// Park function that does nothing — adequate for futures that
    /// become Ready on subsequent polls without needing an external
    /// event.
    fn noop_park() {}

    #[test]
    fn returns_immediately_for_ready_future() {
        assert_eq!(block_on_with(async { 42u32 }, noop_park), 42);
    }

    #[test]
    fn threads_results_through_async_block() {
        let result: Result<u32, &'static str> = block_on_with(
            async {
                let a = async { 10u32 }.await;
                let b = async { 32u32 }.await;
                Ok(a + b)
            },
            noop_park,
        );
        assert_eq!(result, Ok(42));
    }

    /// A future that returns Pending the first N polls and then Ready.
    /// Verifies the runner re-polls after `park`.
    struct PendingNTimes {
        remaining: u32,
    }

    impl Future for PendingNTimes {
        type Output = u32;
        fn poll(mut self: Pin<&mut Self>, _: &mut Context<'_>) -> Poll<Self::Output> {
            if self.remaining == 0 {
                Poll::Ready(7)
            } else {
                self.remaining -= 1;
                Poll::Pending
            }
        }
    }

    #[test]
    fn calls_park_between_polls() {
        let park_count = Cell::new(0u32);
        let result = block_on_with(PendingNTimes { remaining: 3 }, || {
            park_count.set(park_count.get() + 1)
        });
        assert_eq!(result, 7);
        // Pending 3 times → 3 parks, then Ready on the 4th poll.
        assert_eq!(park_count.get(), 3);
    }

    #[test]
    fn ready_on_first_poll_skips_park() {
        let park_count = Cell::new(0u32);
        let result = block_on_with(async { 100u32 }, || park_count.set(park_count.get() + 1));
        assert_eq!(result, 100);
        assert_eq!(park_count.get(), 0);
    }
}
