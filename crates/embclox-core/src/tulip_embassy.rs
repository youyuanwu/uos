use core::cell::UnsafeCell;
use core::task::Context;
use embassy_net_driver::{Capabilities, HardwareAddress, LinkState};
use embassy_sync::waitqueue::AtomicWaker;
use embclox_dma::DmaAllocator;

/// Waker for the Tulip network runner task — signaled from ISR.
pub static TULIP_WAKER: AtomicWaker = AtomicWaker::new();

/// Embassy network driver adapter for the DEC 21140/21143 Tulip NIC.
///
/// Wraps `TulipDevice` and implements `embassy_net_driver::Driver`.
///
/// # Safety
/// Single-core only. The ISR must only touch `TULIP_WAKER` (AtomicWaker),
/// never the device itself.
pub struct TulipEmbassy<D: DmaAllocator> {
    device: UnsafeCell<embclox_tulip::TulipDevice<D>>,
    mac: [u8; 6],
}

// Safety: single-core. ISR only touches AtomicWaker (not device).
unsafe impl<D: DmaAllocator> Send for TulipEmbassy<D> {}

impl<D: DmaAllocator> TulipEmbassy<D> {
    pub fn new(device: embclox_tulip::TulipDevice<D>, mac: [u8; 6]) -> Self {
        Self {
            device: UnsafeCell::new(device),
            mac,
        }
    }

    #[allow(clippy::mut_from_ref)]
    fn dev_mut(&self) -> &mut embclox_tulip::TulipDevice<D> {
        unsafe { &mut *self.device.get() }
    }
}

impl<D: DmaAllocator> embassy_net_driver::Driver for TulipEmbassy<D> {
    type RxToken<'a>
        = RxToken<'a, D>
    where
        Self: 'a;
    type TxToken<'a>
        = TxToken<'a, D>
    where
        Self: 'a;

    fn receive(&mut self, cx: &mut Context) -> Option<(Self::RxToken<'_>, Self::TxToken<'_>)> {
        let dev = self.dev_mut();
        if dev.has_rx_packet() && dev.has_tx_space() {
            return Some((RxToken { parent: self }, TxToken { parent: self }));
        }
        TULIP_WAKER.register(cx.waker());
        None
    }

    fn transmit(&mut self, cx: &mut Context) -> Option<Self::TxToken<'_>> {
        if self.dev_mut().has_tx_space() {
            return Some(TxToken { parent: self });
        }
        TULIP_WAKER.register(cx.waker());
        None
    }

    fn link_state(&mut self, cx: &mut Context) -> LinkState {
        TULIP_WAKER.register(cx.waker());
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

pub struct RxToken<'a, D: DmaAllocator> {
    parent: &'a TulipEmbassy<D>,
}

impl<'a, D: DmaAllocator> embassy_net_driver::RxToken for RxToken<'a, D> {
    fn consume<R, F: FnOnce(&mut [u8]) -> R>(self, f: F) -> R {
        self.parent
            .dev_mut()
            .recv_with(f)
            .expect("packet was ready in receive()")
    }
}

pub struct TxToken<'a, D: DmaAllocator> {
    parent: &'a TulipEmbassy<D>,
}

impl<'a, D: DmaAllocator> embassy_net_driver::TxToken for TxToken<'a, D> {
    fn consume<R, F: FnOnce(&mut [u8]) -> R>(self, len: usize, f: F) -> R {
        self.parent
            .dev_mut()
            .transmit_with(len, f)
            .expect("tx space was available")
    }
}
