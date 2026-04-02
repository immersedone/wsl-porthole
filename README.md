# WSL PortHole

> Auto-managing WSL2 ↔ Windows port forwarding. Install once, forget forever.

WSL PortHole is a Tauri desktop app + Windows Service that keeps your `netsh portproxy`
rules in sync with WSL2's dynamic IP. When WSL restarts and gets a new IP,
Conduit detects it and re-applies all your rules automatically — no scripts,
no manual steps.

## Features

- **Zero-maintenance** — runs as a Windows Service, fires on WSL network events
- **Bi-directional** — WIN→WSL (portproxy) and WSL→WIN (firewall rules)
- **Docker-aware** — auto-discovers exposed ports from running containers
- **MCP server routing** — routes Docker MCP servers on the Windows engine into WSL
- **LAN exposure** — bind rules to `0.0.0.0` for network-wide access
- **Port ranges** — forward 1024–1048 as a single rule
- **Port remapping** — listen on 8080, forward to WSL:80
- **Variable rules** — `${WSL_IP}` resolves at apply-time, never stale
- **Import** — paste your existing netsh script, rules are auto-parsed
- **17+ themes** — including CENTCOM yellow, Nord, Dracula, system auto

## Stack

Tauri v2 · Rust · React · TypeScript · `bollard` · `windows-rs` · `tokio`

## Status

🚧 In development. Bridge scripts available in `docs/scripts/` for immediate use.

## Quick start (bridge script — use before app is ready)

```powershell
# 1. Copy conduit-portproxy.ps1 to your machine
# 2. Run once as admin to register the auto-start task:
.\docs\scripts\conduit-register-task.ps1
# Done. Rules will now auto-update on every WSL restart.
```

## Docs

See [CLAUDE.md](./CLAUDE.md) for full design context and architecture.
