use core::cell::UnsafeCell;
use core::task::Context;
use embassy_net_driver::{Capabilities, HardwareAddress, LinkState};
use embassy_sync::waitqueue::AtomicWaker;

use crate::dma_alloc::BootDmaAllocator;
use crate::mmio_regs::MmioRegs;

type Dev = embclox_e1000::E1000Device<MmioRegs, BootDmaAllocator>;

/// Waker for the network runner task — signaled from the e1000 ISR.
pub static NET_WAKER: AtomicWaker = AtomicWaker::new();

/// Embassy network driver adapter for the e1000 NIC.
///
/// Wraps `E1000Device` and implements `embassy_net_driver::Driver`.
/// Uses `UnsafeCell` because `Driver::receive()` returns both RX and TX
/// tokens from `&mut self`, requiring split access to the inner device.
///
/// # Safety
/// Single-core only. The ISR must only touch `NET_WAKER` (AtomicWaker),
/// never the device itself.
pub struct E1000Embassy {
    device: UnsafeCell<Dev>,
    mac: [u8; 6],
}

// Safety: single-core. ISR only touches AtomicWaker (not device).
// Device access via UnsafeCell is only from executor thread.
unsafe impl Send for E1000Embassy {}

impl E1000Embassy {
    /// Create a new Embassy adapter wrapping an e1000 device.
    pub fn new(device: Dev, mac: [u8; 6]) -> Self {
        Self {
            device: UnsafeCell::new(device),
            mac,
        }
    }

    #[allow(clippy::mut_from_ref)]
    fn dev_mut(&self) -> &mut Dev {
        unsafe { &mut *self.device.get() }
    }
}

impl embassy_net_driver::Driver for E1000Embassy {
    type RxToken<'a>
        = RxToken<'a>
    where
        Self: 'a;
    type TxToken<'a>
        = TxToken<'a>
    where
        Self: 'a;

    fn receive(&mut self, cx: &mut Context) -> Option<(Self::RxToken<'_>, Self::TxToken<'_>)> {
        let (rx, tx) = self.dev_mut().split();
        if rx.has_rx_packet() && tx.has_tx_space() {
            return Some((RxToken { parent: self }, TxToken { parent: self }));
        }
        // Store waker for interrupt-driven wake instead of busy-poll
        NET_WAKER.register(cx.waker());
        None
    }

    fn transmit(&mut self, cx: &mut Context) -> Option<Self::TxToken<'_>> {
        let (_, tx) = self.dev_mut().split();
        if tx.has_tx_space() {
            return Some(TxToken { parent: self });
        }
        NET_WAKER.register(cx.waker());
        None
    }

    fn link_state(&mut self, cx: &mut Context) -> LinkState {
        NET_WAKER.register(cx.waker());
        LinkState::Up
    }

    fn capabilities(&self) -> Capabilities {
        let mut caps = Capabilities::default();
        caps.max_transmission_unit = 1514;
        caps
    }

    fn hardware_address(&self) -> HardwareAddress {
        HardwareAddress::Ethernet(self.mac)
    }
}

/// Embassy RX token — consumes one received packet via `recv_with`.
pub struct RxToken<'a> {
    parent: &'a E1000Embassy,
}

impl<'a> embassy_net_driver::RxToken for RxToken<'a> {
    fn consume<R, F: FnOnce(&mut [u8]) -> R>(self, f: F) -> R {
        let (mut rx, _) = self.parent.dev_mut().split();
        rx.recv_with(f).expect("packet was ready in receive()")
    }
}

/// Embassy TX token — transmits one packet via `transmit_with`.
pub struct TxToken<'a> {
    parent: &'a E1000Embassy,
}

impl<'a> embassy_net_driver::TxToken for TxToken<'a> {
    fn consume<R, F: FnOnce(&mut [u8]) -> R>(self, len: usize, f: F) -> R {
        let (_, mut tx) = self.parent.dev_mut().split();
        tx.transmit_with(len, f).expect("tx space was available")
    }
}
