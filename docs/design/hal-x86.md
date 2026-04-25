# Design: x86_64 Bare-Metal HAL

## Overview

Platform HAL for x86_64 bare-metal (QEMU + `bootloader` crate).
Provides serial, PCI, MMIO mapping, heap, timers, interrupts, and
critical sections. Does NOT implement driver-specific traits — those
live in application code using HAL primitives.

```
┌──────────────────────────────────────────────┐
│  Application                                 │
│  (implements e1000::DmaAllocator via HAL)    │
├──────────────┬───────────────────────────────┤
│  crates/e1000│  crates/hal-x86              │
│  (no platform│  (no driver knowledge)        │
│   deps)      │                               │
└──────────────┴───────────────────────────────┘
```

## Initialization

`hal_x86::init(boot_info, Config::default())` → `Peripherals`
(serial, PCI bus, memory mapper). Called once (panics on second call).
Init order: serial → heap → memory mapper.

## Modules

| Module | Purpose |
|---|---|
| `serial` | UART 16550 + `log` integration + `serial_print!` macros |
| `pci` | x86 I/O port PCI scanner (`find_device`, `read_bar`, `enable_bus_mastering`) |
| `memory` | UC MMIO mapping (advancing cursor), `phys_offset`/`kernel_offset` accessors |
| `heap` | Global allocator from BSS array |
| `time` | APIC timer alarm driver (8 fixed slots, `critical_section::Mutex`) |
| `idt` | IDT init + runtime `set_handler(vector, fn)` |
| `apic` | LAPIC enable, periodic timer, EOI |
| `ioapic` | IOAPIC IRQ routing to LAPIC vectors |
| `pic` | Legacy 8259 PIC disable |
| `pit` | PIT-based TSC frequency calibration |
| `critical_section_impl` | CLI/STI for `critical-section` crate |

## Embedded trait alignment

| Trait crate | Status |
|---|---|
| `embedded-io-async` | Used by example (TCP socket) — consumer only |
| `embedded-io` | Deferred — no consumer for blocking serial `Write` |
| `embedded-hal` | N/A — x86 has no GPIO/SPI/I2C |

## Future work

- `bind_interrupts!` macro (Embassy-style type-safe registration)
- Interrupt-driven UART → `embedded-io-async::Write`
- x2APIC support, ACPI platform discovery

## References

- [Embassy](https://embassy.dev) / [embassy-stm32](https://github.com/embassy-rs/embassy/tree/main/embassy-stm32)
