# scripts/hyperv-boot-test.ps1
# Phase 0 boot test: verify embclox boots under Hyper-V Gen1 with serial output.
# Requires Hyper-V. Will prompt for elevation if not running as admin.

param(
    [string]$Image = "target\x86_64-unknown-none\debug\embclox-example.img",
    [string]$VMName = "embclox-boot-test",
    [int]$TimeoutSeconds = 15
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
$vhdPath = "$imgPath.vhd"
$needConvert = -not (Test-Path $vhdPath) -or ((Get-Item $vhdPath).LastWriteTime -lt (Get-Item $imgPath).LastWriteTime)
if ($needConvert) {
    Write-Host "Converting raw image to VHD..."
    $qemuImg = Get-Command qemu-img -ErrorAction SilentlyContinue
    if ($qemuImg) {
        qemu-img convert -f raw -O vpc -o subformat=fixed $imgPath $vhdPath
        if ($LASTEXITCODE -ne 0) { throw "qemu-img convert failed" }
    } else {
        throw "qemu-img not found. Pre-convert from WSL with:`n  qemu-img convert -f raw -O vpc $imgPath $vhdPath"
    }
} else {
    Write-Host "Using existing VHD: $vhdPath"
}

# Hyper-V cannot use VHDs on network paths (e.g. \\wsl.localhost\...).
# Copy to a local Windows temp directory.
$localVhd = Join-Path $env:TEMP "$VMName.vhd"
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

# Create Gen1 VM
Write-Host "Creating Gen1 VM '$VMName'..."
New-VM -Name $VMName -Generation 1 -MemoryStartupBytes 256MB -NoVHD | Out-Null
Add-VMHardDiskDrive -VMName $VMName -Path $localVhd
Set-VMComPort -VMName $VMName -Number 1 -Path $pipePath

Write-Host "VM created (Gen1, 256MB, COM1 -> $pipePath)"

# Start VM, then connect to pipe immediately (pipe server is created
# by Hyper-V at VM start, but early boot output arrives fast).
Write-Host "Starting VM..."
Start-VM -Name $VMName

Write-Host "Connecting to serial pipe..."
$pipe = New-Object System.IO.Pipes.NamedPipeClientStream(".", $pipeName, [System.IO.Pipes.PipeDirection]::In)
$pipe.Connect(10000)  # 10s to connect
$reader = New-Object System.IO.StreamReader($pipe)
Write-Host "Pipe connected."

# Read serial output with timeout
Write-Host "Reading serial output for ${TimeoutSeconds}s..."
$serialOutput = ""
$reader = New-Object System.IO.StreamReader($pipe)
$deadline = (Get-Date).AddSeconds($TimeoutSeconds)
try {
    while ((Get-Date) -lt $deadline) {
        if ($reader.Peek() -ge 0) {
            $line = $reader.ReadLine()
            $serialOutput += "$line`n"
            Write-Host "  $line" -ForegroundColor DarkGray
        } else {
            Start-Sleep -Milliseconds 50
        }
    }
} catch {
    Write-Host "Pipe read error: $_" -ForegroundColor Yellow
} finally {
    $reader.Close()
    $pipe.Close()
}

# Stop VM
Write-Host "Stopping VM..."
Stop-VM -Name $VMName -TurnOff -Force -ErrorAction SilentlyContinue

# Save log
if ($serialOutput.Length -gt 0) {
    $serialOutput | Out-File -FilePath $logPath -Encoding utf8
}

# Show results
Write-Host ""
Write-Host "=== Results ===" -ForegroundColor Cyan
if ($serialOutput.Trim().Length -gt 0) {
    Write-Host "=== BOOT TEST PASSED ===" -ForegroundColor Green
    Write-Host "Serial output received from Hyper-V Gen1 VM."
    Write-Host "Log saved to: $logPath"
    $exitCode = 0
} else {
    Write-Host "No serial output captured." -ForegroundColor Yellow
    Write-Host "=== BOOT TEST INCONCLUSIVE ===" -ForegroundColor Yellow
    Write-Host "Possible causes:"
    Write-Host "  - VM didn't boot (check Event Viewer > Hyper-V)"
    Write-Host "  - Named pipe timing (try increasing -TimeoutSeconds)"
    Write-Host "  - Bootloader v0.11 not compatible with Hyper-V"
    $exitCode = 2
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
