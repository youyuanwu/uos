# embclox-async

Tiny pure-`core` one-future runner used by the embclox example
kernels at boot time, before the embassy executor is up.

## API

```rust
pub fn block_on_with<F: Future>(fut: F, park: impl FnMut()) -> F::Output;
```

Polls `fut` with a no-op waker. On `Pending`, calls `park()` and
re-polls. Repeats until `Ready`.

The no-op waker is intentional — callers wrap this with a target-
specific `park` (e.g. `sti; hlt` on x86) so the CPU sleeps until any
interrupt fires, then unconditionally re-polls. No executor state,
no allocator, ~12 lines.

For the x86_64 wrapper, see `embclox_hal_x86::runtime::block_on_hlt`.

## Why a separate crate

`embclox-hal-x86` declares a `#[global_allocator]`, which makes
`cargo test` abort on its first allocation under std's test runtime.
Splitting `block_on_with` into a no-allocator crate keeps it
testable; four host-side unit tests cover ready-on-first-poll,
async-block threading, park-count invariants, and skip-park-on-ready.
