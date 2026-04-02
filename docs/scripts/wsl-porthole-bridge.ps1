# Conduit-style auto-detecting portproxy script
# Run once manually or register via Task Scheduler (trigger: on WSL start)
# This is the bridge script until Conduit is installed as a Windows Service.

#Requires -RunAsAdministrator

$ErrorActionPreference = "Stop"

function Get-WslIp {
    try {
        $ip = (wsl hostname -I 2>$null).Trim().Split(" ")[0]
        if ($ip -match '^\d{1,3}(\.\d{1,3}){3}$') { return $ip }
        throw "Invalid IP returned: $ip"
    } catch {
        Write-Host "[ERROR] Could not detect WSL IP. Is WSL running?" -ForegroundColor Red
        exit 1
    }
}

function Set-PortProxy {
    param([int]$ListenPort, [int]$ConnectPort, [string]$ConnectAddress)
    netsh interface portproxy add v4tov4 `
        listenport=$ListenPort `
        listenaddress=0.0.0.0 `
        connectport=$ConnectPort `
        connectaddress=$ConnectAddress | Out-Null
}

function Set-FirewallRule {
    param([int[]]$Ports)
    $portList = $Ports -join ","
    Remove-NetFirewallRule -DisplayName "Conduit WSL Bridge" -ErrorAction SilentlyContinue
    New-NetFirewallRule `
        -DisplayName "Conduit WSL Bridge" `
        -Direction Inbound `
        -Action Allow `
        -Protocol TCP `
        -LocalPort $portList | Out-Null
    Write-Host "  Firewall rule updated for $($Ports.Count) ports" -ForegroundColor DarkGray
}

Write-Host "Conduit WSL Bridge" -ForegroundColor Cyan
Write-Host "==================" -ForegroundColor Cyan

$wslIp = Get-WslIp
Write-Host "[INFO] WSL IP detected: $wslIp" -ForegroundColor Green

Write-Host "[INFO] Resetting existing portproxy rules..." -ForegroundColor DarkGray
netsh interface portproxy reset | Out-Null

# ── Standard ports (listen == connect) ───────────────────────────────────────
$standardPorts = @(
    80, 443, 22,
    8080,    # alt HTTP  (listen 8080 → WSL 80 — see remapped below)
    8100,
    6001,
    7700,
    3002,
    5173
)

# Port ranges (collapsed from individual lines)
$rangeStart = 1024
$rangeEnd   = 1048
$rangePorts = $rangeStart..$rangeEnd

# ── Remapped ports (listenPort != connectPort) ────────────────────────────────
# Format: @{ Listen = X; Connect = Y }
$remappedPorts = @(
    @{ Listen = 8080; Connect = 80  },   # alt HTTP
    @{ Listen = 4433; Connect = 443 },   # alt HTTPS
    @{ Listen = 218;  Connect = 21  },   # alt FTP
    @{ Listen = 28;   Connect = 22  }    # alt SSH
)

Write-Host "[INFO] Adding standard port rules..." -ForegroundColor DarkGray
foreach ($port in @(80, 443, 22, 8100, 6001, 7700, 3002, 5173)) {
    Set-PortProxy -ListenPort $port -ConnectPort $port -ConnectAddress $wslIp
    Write-Host "  0.0.0.0:$port → ${wslIp}:$port" -ForegroundColor DarkGray
}

Write-Host "[INFO] Adding remapped port rules..." -ForegroundColor DarkGray
foreach ($remap in $remappedPorts) {
    Set-PortProxy -ListenPort $remap.Listen -ConnectPort $remap.Connect -ConnectAddress $wslIp
    Write-Host "  0.0.0.0:$($remap.Listen) → ${wslIp}:$($remap.Connect)  [remapped]" -ForegroundColor DarkGray
}

Write-Host "[INFO] Adding port range 1024-1048 ($($rangePorts.Count) ports)..." -ForegroundColor DarkGray
foreach ($port in $rangePorts) {
    Set-PortProxy -ListenPort $port -ConnectPort $port -ConnectAddress $wslIp
}
Write-Host "  0.0.0.0:1024-1048 → ${wslIp}:1024-1048" -ForegroundColor DarkGray

# ── Firewall ──────────────────────────────────────────────────────────────────
$allListenPorts = @(80, 443, 22, 8080, 4433, 218, 28, 8100, 6001, 7700, 3002, 5173) + $rangePorts
Set-FirewallRule -Ports $allListenPorts

# ── Summary ───────────────────────────────────────────────────────────────────
$rules = netsh interface portproxy show v4tov4
$ruleCount = ($rules | Select-String "0.0.0.0").Count
Write-Host ""
Write-Host "[DONE] $ruleCount rules active. WSL IP: $wslIp" -ForegroundColor Green
Write-Host "       Run 'netsh interface portproxy show v4tov4' to verify." -ForegroundColor DarkGray
