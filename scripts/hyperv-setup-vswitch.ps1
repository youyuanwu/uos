# Create dedicated embclox-test Internal vSwitch with a known subnet
# (192.168.234.0/24, host = .1, VMs use .50, .51, ...).
# Run this once as Administrator. Idempotent.

$ErrorActionPreference = 'Stop'
$SwitchName = 'embclox-test'
$HostIp = '192.168.234.1'
$Prefix = 24

if (-not (Get-VMSwitch -Name $SwitchName -ErrorAction SilentlyContinue)) {
    Write-Host "Creating Internal vSwitch '$SwitchName'..."
    New-VMSwitch -Name $SwitchName -SwitchType Internal | Out-Null
} else {
    Write-Host "vSwitch '$SwitchName' already exists"
}

$ifAlias = "vEthernet ($SwitchName)"
$existing = Get-NetIPAddress -InterfaceAlias $ifAlias -AddressFamily IPv4 -ErrorAction SilentlyContinue
if (-not ($existing | Where-Object { $_.IPAddress -eq $HostIp })) {
    if ($existing) {
        Write-Host "Removing existing IPv4 addresses on $ifAlias"
        $existing | Remove-NetIPAddress -Confirm:$false
    }
    Write-Host "Assigning $HostIp/$Prefix to $ifAlias"
    New-NetIPAddress -InterfaceAlias $ifAlias -IPAddress $HostIp -PrefixLength $Prefix | Out-Null
} else {
    Write-Host "Host IP $HostIp/$Prefix already configured on $ifAlias"
}

Write-Host ""
Write-Host "Setup complete. Use:"
Write-Host "  scripts/hyperv-boot-test.ps1 -SwitchName '$SwitchName'"
Write-Host "VM should configure static IP 192.168.234.50/24 with gateway $HostIp"
