# Design: Interrupt-Driven Mode

## Overview

Interrupt-driven wakeup for the e1000 NIC and Embassy time driver.
APIC timer fires ~1ms for `embassy-time` alarms, e1000 RX interrupt
wakes the network runner, and the CPU halts when idle.

```
Executor idle (hlt)
  ← APIC timer (vector 32) → wake expired embassy-time alarms
  ← e1000 RX (vector 33)   → wake network runner via AtomicWaker
```

## Key decisions

- **Fixed alarm slots** — 8-slot array with `critical_section::Mutex`
  (not `BTreeMap` — ISR cannot allocate from heap)
- **AtomicWaker** — `receive()` registers, e1000 ISR wakes. No busy-poll.
- **ICR read-clear** — ISR reads ICR via global `AtomicUsize` to
  acknowledge the device interrupt
- **PIT calibration** — TSC ticks/μs measured via PIT channel 2 (~10ms)
- **hlt-on-idle** — `cli; enable_and_hlt` after each `poll()` cycle

## Interrupt vector assignments

| Vector | Source | Handler |
|---|---|---|
| 0-31 | CPU exceptions | Default/panic |
| 32 | APIC timer | Check alarm slots, wake expired, EOI |
| 33 | e1000 NIC (IRQ from PCI config 0x3C) | Read ICR, wake NET_WAKER, EOI |
| 39 | LAPIC spurious | No-op (no EOI) |

## Init ordering

1. `hal_x86::init()` — serial, heap, memory mapper
2. `hal_x86::idt::init()` — IDT with default handlers
3. `hal_x86::pic::disable()` — mask legacy PIC
4. Map LAPIC + IOAPIC via `memory.map_mmio()`
5. Enable LAPIC, calibrate TSC via PIT, start APIC timer
6. Register handlers (vectors 32, 33, 39)
7. PCI scan, e1000 reset + init
8. Store MMIO base for ISR, route IRQ via IOAPIC
9. Enable e1000 device interrupts
10. `sti`, start executor

## HAL modules

| Module | Purpose |
|---|---|
| `idt.rs` | IDT init + runtime `set_handler(vector, fn)` |
| `apic.rs` | LAPIC enable, timer, EOI |
| `ioapic.rs` | IRQ routing to LAPIC vectors |
| `pic.rs` | Disable legacy 8259 PIC |
| `pit.rs` | PIT-based TSC calibration |
| `time.rs` | APIC timer alarm driver (8 slots) |

## References

- [Intel SDM Vol. 3, Ch. 10](https://www.intel.com/content/www/us/en/developer/articles/technical/intel-sdm.html)
- [OSDev: APIC Timer](https://wiki.osdev.org/APIC_timer) / [IOAPIC](https://wiki.osdev.org/IOAPIC)
