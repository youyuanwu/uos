# examples-hyperv

Bare-metal kernel demonstrating Hyper-V VMBus + NetVSC + embassy-net
TCP echo on **local Hyper-V Gen1** and **Azure Gen1** VMs. Boots via
Limine BIOS+UEFI.

Two boot configurations:

| Image | Config | cmdline | Used for |
|-------|--------|---------|----------|
| `build/hyperv.iso` | `limine.conf` | `net=static` (default) | Local Hyper-V on `embclox-test` Internal vSwitch |
| `build/hyperv.vhd` | `limine-azure.conf` | `net=dhcp` | Azure Gen1 deployment |

## Build & run

```bash
# ISO for local Hyper-V or QEMU smoke test:
cmake --build build --target hyperv-image      # -> build/hyperv.iso

# VHD for Azure deployment:
cmake --build build --target hyperv-vhd        # -> build/hyperv.vhd
```

**CI (basic boot smoke test on QEMU — no VMBus):**

```bash
ctest --test-dir build -R hyperv-boot
```

**Local Hyper-V Gen1 (full NetVSC + TCP echo):**

```powershell
.\scripts\hyperv-setup-vswitch.ps1   # one-time, as Administrator
powershell.exe -ExecutionPolicy Bypass -File scripts/hyperv-boot-test.ps1 \
    -Iso build/hyperv.iso
```

**Azure Gen1 deployment:** see [`tests/infra/README.md`](../tests/infra/README.md).

## What this example shows

- Limine boot with synthetic device discovery (CPUID Hyper-V detection)
- VMBus channel setup, NetVSC NVSP+RNDIS handshake (driven by
  `embclox-hyperv::init`, internally async via `block_on_hlt`)
- SynIC SINT2 ISR + APIC periodic timer interleaved
- Embassy executor with `hlt`-on-idle (`runtime::run_executor`)
- TCP echo on port 1234

## Source layout

- `src/main.rs` — Limine boot, Hyper-V detection, IDT, embassy executor
- `limine.conf` / `limine-azure.conf` — boot menu entries
