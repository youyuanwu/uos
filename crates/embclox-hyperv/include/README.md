# Hyper-V Header Files

Linux kernel headers used as reference for generating Rust bindings via bindgen.

Source: Linux v7.0 (`/include/linux/`)

## Files

| File | Origin | Description |
|------|--------|-------------|
| `hyperv_net_bindgen.h` | Simplified | Stripped version of `drivers/net/hyperv/hyperv_net.h` for bindgen. Removes all Linux dependencies (`sk_buff`, `net_device`, `spinlock`, XDP, etc.) while keeping NVSP/RNDIS wire-format structs, protocol constants, and message type enums. Adds freestanding typedefs and RNDIS/OID constants. |
| `rndis.h` | Direct copy | `include/linux/rndis.h` — RNDIS protocol constants (message types, status codes, OIDs, packet filter bits). Self-contained, no Linux dependencies. |
| `hyperv_vmbus.h` | Simplified | VMBus wire-format structs extracted from `include/linux/hyperv.h`. Contains `vmpacket_descriptor`, `vmtransfer_page_range`, `vmtransfer_page_packet_header`, `vmdata_gpa_direct`, and `enum vmbus_packet_type`. |

## Updating

When updating from a newer kernel version:

1. Copy the new `rndis.h` directly (it has no Linux dependencies)
2. Re-run the extraction for `hyperv_net_bindgen.h` and `hyperv_vmbus.h`
3. Verify with `gcc -fsyntax-only -std=c11 <file>`
