# Hyper-V Header Files

Protocol headers for generating Rust bindings via bindgen.

Source: [microsoft/mu_msvm](https://github.com/microsoft/mu_msvm) — Microsoft's UEFI VM firmware (Project Mu).
License: BSD-2-Clause-Patent

## Files

| File | Origin | Description |
|------|--------|-------------|
| `msvm_compat.h` | Local | Compatibility shim providing freestanding type definitions (`UINT8`/`UINT16`/`UINT32`/`UINT64`/`BOOLEAN`) so mu_msvm headers compile without UEFI build infrastructure. |
| `nvspprotocol.h` | mu_msvm | `MsvmPkg/NetvscDxe/nvspprotocol.h` — NVSP protocol message types, status codes, and all wire-format structs for VMBus network channel communication. |
| `rndis_msvm.h` | mu_msvm | `MsvmPkg/NetvscDxe/rndis.h` — RNDIS (Remote NDIS) protocol: message types, status codes, request/response structs, OIDs. |
| `VmbusPacketFormat.h` | mu_msvm | `MsvmPkg/Include/Vmbus/VmbusPacketFormat.h` — VMBus ring buffer packet descriptors: `VMPACKET_DESCRIPTOR`, `VMTRANSFER_PAGE_RANGE`, `VMTRANSFER_PAGE_PACKET_HEADER`, `VMBUS_PACKET_TYPE` enum. |

## Updating

When updating from a newer mu_msvm release:

1. Download updated headers from `microsoft/mu_msvm`
2. Replace `#include "AllowNamelessAggregate.h"` and `#include "StaticAssert1.h"` with `#include "msvm_compat.h"`
3. Verify with `gcc -ffreestanding -nostdinc -fsyntax-only -std=c11 <file>`
