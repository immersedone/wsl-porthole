# Register Conduit bridge script as a Task Scheduler task
# Triggers on: user logon + WSL network interface coming up
# Run this ONCE with admin privileges - never touch again.

#Requires -RunAsAdministrator

$scriptPath = "$env:USERPROFILE\conduit-portproxy.ps1"
$taskName   = "Conduit WSL Port Bridge"

# Copy the bridge script to user profile if not already there
if (-not (Test-Path $scriptPath)) {
    Copy-Item "$PSScriptRoot\conduit-portproxy.ps1" $scriptPath
}

$action  = New-ScheduledTaskAction `
    -Execute "powershell.exe" `
    -Argument "-NonInteractive -WindowStyle Hidden -ExecutionPolicy Bypass -File `"$scriptPath`""

# Two triggers: logon (covers WSL auto-start) + network event (covers WSL restart mid-session)
$triggerLogon   = New-ScheduledTaskTrigger -AtLogOn
$triggerNetwork = New-CimInstance -Namespace "Root/Microsoft/Windows/TaskScheduler" `
    -ClassName "MSFT_TaskEventTrigger" `
    -ClientOnly `
    -Property @{
        Enabled       = $true
        Subscription  = '<QueryList><Query Id="0"><Select Path="Microsoft-Windows-Hyper-V-VmSwitch-Operational">*[System[EventID=102]]</Select></Query></QueryList>'
        Delay         = "PT5S"   # 5 second delay to let WSL IP settle
    }

$settings = New-ScheduledTaskSettingsSet `
    -ExecutionTimeLimit (New-TimeSpan -Minutes 2) `
    -MultipleInstances IgnoreNew `
    -RunOnlyIfNetworkAvailable

$principal = New-ScheduledTaskPrincipal `
    -UserId $env:USERNAME `
    -LogonType Interactive `
    -RunLevel Highest

Unregister-ScheduledTask -TaskName $taskName -Confirm:$false -ErrorAction SilentlyContinue

Register-ScheduledTask `
    -TaskName  $taskName `
    -Action    $action `
    -Trigger   @($triggerLogon, $triggerNetwork) `
    -Settings  $settings `
    -Principal $principal `
    -Description "Auto-updates WSL portproxy rules when WSL IP changes. Managed by Conduit." | Out-Null

Write-Host "[DONE] Task '$taskName' registered." -ForegroundColor Green
Write-Host "       It will run automatically on logon and WSL network events." -ForegroundColor DarkGray
Write-Host "       To run now: Start-ScheduledTask -TaskName '$taskName'" -ForegroundColor DarkGray
