## Hyper-V
Grant the local user to be able to start stop vm without admin rights.
```ps1
Add-LocalGroupMember -Group "Hyper-V Administrators" -Member "domain\name"
Get-LocalGroupMember -Group "Hyper-V Administrators"
Start-VM "MyVM"
Stop-VM "MyVM"
```

For Hyper-V test networking (vSwitch setup, why we don't use Default
Switch, ICS ARP pollution issue): see [HyperV-Testing.md](HyperV-Testing.md).