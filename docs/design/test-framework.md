# Test Framework

## Overview

Two testing modes for bare-metal code running inside QEMU:

1. **Integration tests** — boot QEMU image, verify from host (TCP
   probing, log scanning). End-to-end validation.

2. **Unit tests** — `#[test]` functions compiled into a QEMU image.
   Guest runs all tests, prints results to serial, exits via
   `isa-debug-exit`. Tests e1000 driver internals and HAL modules
   against real hardware.

**Rule of thumb**: assertions about internal state → unit test.
Assertions requiring an external observer → integration test.

## Test runner (`scripts/qemu-test.sh`)

Shared script that boots a QEMU image and checks pass/fail:

```sh
qemu-test.sh <image> [--probe tcp:PORT:STRING] [--log-match PATTERN] [--timeout SECS]
              [--qemu-args "EXTRA_ARGS"]
```

- Adds `-device isa-debug-exit,iobase=0xf4,iosize=0x04` automatically
- Remaps QEMU exit codes: guest 0 → host 0, guest 1 → host 1
- Serial output written to `<image-dir>/<image-name>-qemu.log`
- Only added to test invocations, not the dev `qemu` CMake target

## Unit tests

Uses `#[test_suite]` proc macro from `embclox-test-macros`:

```rust
#[embclox_test_macros::test_suite(name = "e1000_smoke")]
mod tests {
    use super::*;

    #[test]
    fn status_link_up() {
        let status = ctx().regs.read_reg(STAT);
        assert!(status & 0x2 != 0);
    }
}
pub use tests::suite;
```

The macro collects `#[test]` functions, strips the attribute, and
generates `suite() -> (&str, &[TestCase])`. Main calls `run_suite()`
for each suite.

**Abort on failure**: no unwinding in `no_std`. Panic handler prints
the failure and exits QEMU immediately. All suites currently run in a
single binary/QEMU boot and must be order-independent.

## Integration tests

The TCP echo example (`examples/`) serves as the integration test.
`qemu-test.sh --probe tcp:5555:hello-embclox` boots the image, waits
for the guest, and verifies TCP echo round-trip.

## Running tests

```sh
cmake -B build && ctest --test-dir build    # both tests
ctest --test-dir build -R unit              # unit only
ctest --test-dir build -R integration       # integration only
```

## Project layout

```
scripts/qemu-test.sh             # shared QEMU runner
qemu-tests/unit/                 # unit test binary (no_std)
├── src/main.rs                  # boot, HAL init, run suites, exit
├── src/harness.rs               # TestCase, run_suite(), qemu_exit()
└── src/suites/e1000_smoke.rs    # e1000 smoke tests (3 tests)
crates/embclox-test-macros/      # #[test_suite] proc macro
CMakeLists.txt                   # ctest integration (unit + integration)
```

## Adding a new test suite

1. Create `qemu-tests/unit/src/suites/my_suite.rs`
2. Use `#[embclox_test_macros::test_suite(name = "my_suite")]`
3. Add `pub mod my_suite;` to `suites/mod.rs`
4. Call `run_suite()` with `my_suite::tests::suite()` in `main.rs`

## Test context patterns

Test functions are `fn()` — no arguments. How to pass context depends
on what the test needs:

- **No context needed**: HAL subsystems like `PciBus` (zero-sized) can
  be constructed inline. Heap allocator is global after HAL init.
- **Static context**: e1000 tests need MMIO addresses and DMA offsets
  from device setup. These are stored in a `static mut Option<Ctx>`
  initialized once by main before suites run.

### Why e1000 tests use a static

`E1000Device::new()` allocates ~1MB of DMA memory (4 bulk allocations)
and configures TX/RX rings in device registers. The driver has no
`Drop` impl or `deinit()` method — creating a device per test would
leak DMA memory and exhaust the heap. The `map_mmio` call also can't
be repeated for the same physical address (page already mapped).

To enable per-test init/teardown, the driver would need a `Drop` impl
that frees DMA regions and resets the device. This is a future
improvement — for now, e1000 device state is initialized once and
shared across all tests in the suite.
