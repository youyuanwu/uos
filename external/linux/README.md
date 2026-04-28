# Linux Kernel Headers (Reference Only)

Copied from Linux v7.0 for reference. These are **not** used for building —
the project uses headers from [microsoft/mu_msvm](https://github.com/microsoft/mu_msvm)
(BSD-2-Clause-Patent) in `crates/embclox-hyperv/include/` instead.

## Files

| File | Source path | Description |
|------|------------|-------------|
| `hyperv_net.h` | `drivers/net/hyperv/hyperv_net.h` | NetVSC driver: NVSP/RNDIS structs, driver state, offload types |
| `rndis.h` | `include/linux/rndis.h` | RNDIS protocol constants: message types, status codes, OIDs |
| `hyperv.h` | `include/linux/hyperv.h` | VMBus: packet descriptors, channel structs, hypercall types |

## License

These files are licensed under GPL-2.0-only. See each file header for details.
