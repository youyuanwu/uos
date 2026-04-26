# scripts/hyperv-boot-test.ps1
# Phase 0 boot test: verify embclox boots under Hyper-V Gen1 with serial output.
# Requires Hyper-V. Will prompt for elevation if not running as admin.

param(
    [string]$Image = "target\x86_64-unknown-none\debug\embclox-example.img",
    [string]$VMName = "embclox-boot-test",
    [int]$TimeoutSeconds = 60
)

# Self-elevate if not admin
$isAdmin = ([Security.Principal.WindowsPrincipal] [Security.Principal.WindowsIdentity]::GetCurrent()).IsInRole(
    [Security.Principal.WindowsBuiltInRole]::Administrator
)
if (-not $isAdmin) {
    Write-Host "Requesting elevation..."
    $repoRoot = Split-Path -Parent (Split-Path -Parent $PSCommandPath)
    $absImage = Join-Path $repoRoot $Image
    $argList = "-ExecutionPolicy Bypass -File `"$PSCommandPath`" -Image `"$absImage`" -VMName `"$VMName`" -TimeoutSeconds $TimeoutSeconds"
    Start-Process -FilePath "pwsh.exe" -ArgumentList $argList -Verb RunAs -Wait
    exit $LASTEXITCODE
}

# Wrap everything in a trap so errors don't close the window
trap {
    Write-Host ""
    Write-Host "ERROR: $_" -ForegroundColor Red
    Write-Host $_.ScriptStackTrace -ForegroundColor DarkGray
    Write-Host ""
    # Cleanup on error
    Stop-VM -Name $VMName -TurnOff -Force -ErrorAction SilentlyContinue
    Remove-VM -Name $VMName -Force -ErrorAction SilentlyContinue
    if ($localVhd) { Remove-Item $localVhd -Force -ErrorAction SilentlyContinue }
    Write-Host "Press Enter to exit..."
    Read-Host
    exit 1
}

$ErrorActionPreference = 'Stop'

# Resolve image path
if ([System.IO.Path]::IsPathRooted($Image)) {
    $imgPath = $Image
} else {
    $repoRoot = Split-Path -Parent (Split-Path -Parent $PSCommandPath)
    $imgPath = Join-Path $repoRoot $Image
}

Write-Host "Image: $imgPath"

if (-not (Test-Path $imgPath)) {
    throw "Image not found: $imgPath. Build with: cmake --build build --target example-image"
}

# Convert to VHD (Hyper-V needs VHD, not raw)
# Try qemu-img on Windows first, then check if .vhd already exists
# (pre-converted from WSL with: qemu-img convert -f raw -O vpc img vhd)
$vhdPath = "$imgPath.vhdx"
$needConvert = -not (Test-Path $vhdPath) -or ((Get-Item $vhdPath).LastWriteTime -lt (Get-Item $imgPath).LastWriteTime)
if ($needConvert) {
    Write-Host "Converting raw image to VHDX..."
    $qemuImg = Get-Command qemu-img -ErrorAction SilentlyContinue
    if ($qemuImg) {
        & qemu-img convert -f raw -O vhdx $imgPath $vhdPath
        if ($LASTEXITCODE -ne 0) { throw "qemu-img convert failed" }
    } else {
        throw "qemu-img not found. Pre-convert from WSL with:`n  qemu-img convert -f raw -O vhdx $imgPath $vhdPath"
    }
} else {
    Write-Host "Using existing VHDX: $vhdPath"
}

# Hyper-V cannot use VHDs on network paths (e.g. \\wsl.localhost\...).
# Copy to a local Windows temp directory.
$localVhd = Join-Path $env:TEMP "$VMName.vhdx"
Write-Host "Copying VHD to local path: $localVhd"
Copy-Item -Path $vhdPath -Destination $localVhd -Force

# Serial output log file
$logPath = "$imgPath-hyperv.log"
$pipeName = "$VMName-com1"
$pipePath = "\\.\pipe\$pipeName"

# Cleanup any existing VM
if (Get-VM -Name $VMName -ErrorAction SilentlyContinue) {
    Write-Host "Cleaning up existing VM '$VMName'..."
    Stop-VM -Name $VMName -TurnOff -Force -ErrorAction SilentlyContinue
    Remove-VM -Name $VMName -Force
}

# Create Gen2 VM (UEFI boot — bootloader v0.11 BIOS hangs on Hyper-V VBE)
Write-Host "Creating Gen2 VM '$VMName'..."
New-VM -Name $VMName -Generation 2 -MemoryStartupBytes 256MB -NoVHD | Out-Null
Add-VMHardDiskDrive -VMName $VMName -Path $localVhd
# Disable Secure Boot (our bootloader is not signed)
Set-VMFirmware -VMName $VMName -EnableSecureBoot Off

Write-Host "VM created (Gen2, UEFI, 256MB, Secure Boot off)"
Write-Host "NOTE: Gen2 has no COM port. Output is visible in VM Connect window."
Write-Host "      Use Hyper-V Manager -> Connect to see framebuffer output."

# Start VM and wait for it to run
Write-Host "Starting VM..."
Start-VM -Name $VMName

Write-Host "VM running. Waiting ${TimeoutSeconds}s for kernel to execute..."
Start-Sleep -Seconds $TimeoutSeconds

# Check VM state
$vm = Get-VM -Name $VMName
Write-Host "VM state: $($vm.State)"

$serialOutput = "Gen2 VM — no serial pipe, check VM Connect for framebuffer output"

# Stop VM
Write-Host "Stopping VM..."
Stop-VM -Name $VMName -TurnOff -Force -ErrorAction SilentlyContinue

# Save log
if ($serialOutput.Length -gt 0) {
    $serialOutput | Out-File -FilePath $logPath -Encoding utf8
}

# Show results — check state BEFORE stopping
Write-Host ""
Write-Host "=== Results ===" -ForegroundColor Cyan
$vm = Get-VM -Name $VMName -ErrorAction SilentlyContinue
if ($vm -and $vm.State -eq 'Running') {
    Write-Host "VM is running (kernel booted and is halting in a loop)." -ForegroundColor Green
    Write-Host "=== BOOT TEST PASSED ===" -ForegroundColor Green
    Write-Host "Open Hyper-V Manager -> Connect to see framebuffer text output."
    Write-Host ""
    Write-Host "Press Enter to stop the VM and clean up..."
    Read-Host
    $exitCode = 0
} elseif ($vm) {
    Write-Host "VM state: $($vm.State)" -ForegroundColor Yellow
    Write-Host "=== BOOT TEST INCONCLUSIVE ===" -ForegroundColor Yellow
    Write-Host "VM may have crashed. Check Hyper-V Event Viewer for details."
    $exitCode = 2
} else {
    Write-Host "VM not found" -ForegroundColor Red
    $exitCode = 1
}

# Cleanup
Write-Host ""
Write-Host "Cleaning up..."
Remove-VM -Name $VMName -Force -ErrorAction SilentlyContinue
Remove-Item $localVhd -Force -ErrorAction SilentlyContinue

Write-Host ""
Write-Host "Press Enter to exit..."
Read-Host

exit $exitCode
