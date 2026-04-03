# WSL PortHole

> Auto-managing WSL2 ↔ Windows port forwarding. Install once, forget forever.

WSL PortHole is a Windows desktop app + background service that permanently solves the WSL2 port-forwarding problem. WSL2 gets a new IP every restart, breaking all your `netsh portproxy` rules. WSL PortHole detects the change and re-applies everything automatically — no scripts, no manual steps.

![Tauri v2](https://img.shields.io/badge/Tauri-v2-blue)
![Rust](https://img.shields.io/badge/Rust-1.94-orange)
![Vue 3](https://img.shields.io/badge/Vue-3-green)
![License](https://img.shields.io/badge/License-MIT-yellow)

![WSL PortHole — Port Rules](docs/screenshots/01-rules.png)

## Features

- **Zero-maintenance** — Windows Service fires on WSL network events, re-applies rules automatically
- **Bi-directional** — WIN→WSL (netsh portproxy) and WSL→WIN (firewall rules)
- **Docker-aware** — auto-discovers exposed ports from running containers (WSL + Windows engines)
- **MCP server routing** — detects and routes Docker MCP servers into WSL
- **LAN exposure** — bind rules to `0.0.0.0` for network-wide access with copy URL + QR code
- **Port ranges** — forward 1024–1048 as a single rule, expanded at apply-time
- **Port remapping** — listen on 8080, forward to WSL:80
- **Variable system** — `${WSL_IP}`, `${HOST_IP}`, `${HOST_GW}`, `${WSL_IP:DistroName}`
- **Import/Export** — paste a netsh script, auto-parse into rules; export as JSON
- **Rule groups** — bundle rules into profiles, toggle with one click
- **Startup actions** — run commands on WSL start with configurable delay chaining
- **13 themes** — Mission Control, CENTCOM, Nord, Dracula, Monokai, and more
- **.wslconfig inspector** — read/edit WSL settings with mirrored-mode warnings
- **Audit log** — timestamped event log with filter and export
- **~3 MB installer** — Tauri v2, no Electron bloat

## Screenshots

| Port Rules | Docker Sync | Appearance |
| --- | --- | --- |
| ![Rules](docs/screenshots/01-rules.png) | ![Docker](docs/screenshots/03-docker.png) | ![Appearance](docs/screenshots/12-appearance.png) |

| CENTCOM Theme | Nord Theme | Daylight Theme |
| --- | --- | --- |
| ![CENTCOM](docs/screenshots/14-theme-centcom.png) | ![Nord](docs/screenshots/15-theme-nord.png) | ![Daylight](docs/screenshots/16-theme-daylight.png) |

<details>
<summary>All pages</summary>

| Page | Screenshot |
| --- | --- |
| Port Rules | ![Port Rules](docs/screenshots/01-rules.png) |
| Groups | ![Groups](docs/screenshots/02-groups.png) |
| Docker Sync | ![Docker Sync](docs/screenshots/03-docker.png) |
| MCP Servers | ![MCP Servers](docs/screenshots/04-mcp.png) |
| LAN Access | ![LAN Access](docs/screenshots/05-lan.png) |
| Firewall | ![Firewall](docs/screenshots/06-firewall.png) |
| Distros | ![Distros](docs/screenshots/07-distros.png) |
| Startup Actions | ![Startup Actions](docs/screenshots/08-startup.png) |
| Boot Service | ![Boot Service](docs/screenshots/09-service.png) |
| .wslconfig | ![.wslconfig](docs/screenshots/10-wslconfig.png) |
| Audit Log | ![Audit Log](docs/screenshots/11-audit.png) |
| Appearance | ![Appearance](docs/screenshots/12-appearance.png) |
| Settings | ![Settings](docs/screenshots/13-settings.png) |

</details>

## Install

### Download (recommended)

Grab the latest release from the [Releases](../../releases) page:

| File | Description |
| --- | --- |
| `WSL PortHole_x.x.x_x64-setup.exe` | NSIS installer (recommended) |
| `WSL PortHole_x.x.x_x64_en-US.msi` | MSI installer (enterprise/GPO) |

### Build from source

**Prerequisites:** [Rust](https://rustup.rs/), [Node.js](https://nodejs.org/) (v18+), Windows 10/11

```powershell
git clone https://github.com/user/wsl-porthole.git
cd wsl-porthole
npm install
npx tauri build
```

Output:

- `src-tauri/target/release/bundle/nsis/` — NSIS installer
- `src-tauri/target/release/bundle/msi/` — MSI installer
- `target/release/wsl-porthole-service.exe` — Windows Service (standalone)

## Windows Service

The service runs independently of the GUI — rules stay active with the app closed.

```powershell
# Build the service
cargo build --release -p wsl-porthole-service

# Install (run as Administrator)
wsl-porthole-service.exe install
sc start WslPortHole

# Other commands
wsl-porthole-service.exe status
wsl-porthole-service.exe standalone   # run in foreground for debugging
wsl-porthole-service.exe uninstall
```

The service:

1. Detects WSL IP changes (30s polling, Hyper-V event subscription planned)
2. Waits 5s for the IP to settle
3. Re-applies all enabled portproxy + firewall rules
4. Sends a Windows toast notification

## Quick start (bridge script — use before app is built)

If you just need port forwarding working right now:

```powershell
# Run once as admin to register the auto-start task:
.\docs\scripts\wsl-porthole-register.ps1

# Done. Rules auto-update on every WSL restart.
# Or run manually:
.\docs\scripts\wsl-porthole-bridge.ps1
```

## Tech stack

| Layer | Technology |
| --- | --- |
| App shell | Tauri v2 (~3 MB installer) |
| Frontend | Vue 3 + TypeScript + Tailwind |
| Core logic | Rust (`wsl-porthole-core` crate, 24 tests) |
| Windows Service | Rust (`wsl-porthole-service` crate) |
| Docker API | bollard (WSL unix socket + Windows named pipe) |
| Windows APIs | netsh, PowerShell (firewall), windows-service |

## Project structure

```text
wsl-porthole/
├── crates/
│   ├── wsl-porthole-core/       # Pure Rust library (ip, rules, netsh, firewall, docker, mcp, import, config)
│   └── wsl-porthole-service/    # Windows Service (watcher, IPC, toast notifications)
├── src-tauri/                   # Tauri v2 app shell (20 commands bridging Rust → Vue)
├── src/                         # Vue 3 frontend (13 pages, 5 components, 13 themes)
└── docs/scripts/                # Bridge PowerShell scripts (use now)
```

## License

MIT

## Docs

- [BUILDING.md](./docs/BUILDING.md) — build from source, generate installers, publish to the Microsoft Store
- [CLAUDE.md](./CLAUDE.md) — full design context, architecture, and roadmap
