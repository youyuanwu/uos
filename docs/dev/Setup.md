## Hyper-V
Grant the local user to be able to start stop vm without admin rights.
```ps1
Add-LocalGroupMember -Group "Hyper-V Administrators" -Member "domain\name"
Get-LocalGroupMember -Group "Hyper-V Administrators"
Start-VM "MyVM"
Stop-VM "MyVM"
```