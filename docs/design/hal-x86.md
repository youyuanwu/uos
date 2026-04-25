# Design: x86_64 Bare-Metal HAL

## Overview

Platform HAL for x86_64 bare-metal (QEMU + `bootloader` crate).
Provides serial, PCI, MMIO mapping, heap, timers, interrupts, and
critical sections.

## Initialization

`embclox_hal_x86::init(boot_info, Config::default())` → `Peripherals`
(serial, PCI bus, memory mapper). Called once (panics on second call).

## Modules

| Module | Purpose |
|---|---|
| `serial` | UART 16550 + `log` integration |
| `pci` | x86 I/O port PCI scanner, BAR read, bus mastering |
| `memory` | UC MMIO mapping (`MmioMapping` handle), `unmap_mmio` (unsafe) |
| `heap` | Global allocator from BSS array |
| `time` | APIC timer alarm driver (8 fixed slots) |
| `idt` | IDT init + runtime `set_handler(vector, fn)` |
| `apic` | LAPIC enable, periodic timer, EOI |
| `ioapic` | IOAPIC IRQ routing to LAPIC vectors |
| `pic` | Legacy 8259 PIC disable |
| `pit` | PIT-based TSC frequency calibration |
| `critical_section_impl` | CLI/STI for `critical-section` crate |

## MMIO mapping ownership

`map_mmio` returns `MmioMapping` (handle with vaddr + size).
`unmap_mmio(&MmioMapping)` is `unsafe` — caller must ensure no
references to the mapped memory exist. Mirrors the
`DmaAllocator::alloc_coherent`/`free_coherent` pattern.

## Future work

- `bind_interrupts!` macro (Embassy-style type-safe registration)
- Interrupt-driven UART → `embedded-io-async::Write`
- x2APIC support, ACPI platform discovery
