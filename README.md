# WSL PortHole

> Auto-managing WSL2 ↔ Windows ↔ Docker port forwarding. Install once, forget forever.

WSL PortHole is a Windows desktop app + background service that permanently solves the WSL2 port-forwarding problem. WSL2 gets a new IP on every restart, breaking all `netsh portproxy` rules — WSL PortHole detects changes automatically, re-applies rules, manages firewall entries, discovers Docker services, and routes MCP servers between layers.

![Tauri v2](https://img.shields.io/badge/Tauri-v2-blue)
![Rust](https://img.shields.io/badge/Rust-1.94-orange)
![Vue 3](https://img.shields.io/badge/Vue-3-green)
![License](https://img.shields.io/badge/License-MIT-yellow)

![WSL PortHole — Port Rules](docs/screenshots/01-rules.png)

## The problem

Every time WSL2 restarts, it gets a new dynamic IP. Your carefully configured `netsh portproxy` rules break. The usual fix is a PowerShell script you run manually every time. WSL PortHole eliminates this entirely.

## Features

### Core
- **Zero-maintenance** — Windows Service detects WSL IP changes and re-applies rules automatically
- **Bi-directional** — WIN→WSL (netsh portproxy) and WSL→WIN (firewall rules on vEthernet)
- **Per-distro targeting** — rules bound to specific WSL distributions or Docker containers
- **Variable system** — `${WSL_IP}`, `${HOST_IP}`, `${HOST_GW}`, `${WSL_IP:DistroName}`
- **Port ranges** — forward 1024–1048 as a single rule, expanded at apply-time
- **Port remapping** — listen on 8080, forward to WSL:80

### Discovery
- **Docker-aware** — auto-discovers exposed ports from containers (WSL + Windows engines)
- **MCP server routing** — detects Model Context Protocol servers in Docker, creates WSL→WIN firewall rules
- **LAN exposure** — bind to `0.0.0.0` for network-wide access, copy URL, QR code for mobile

### Management
- **Rule groups** — bundle rules into profiles, one-click toggle, startup behavior
- **Export/import bundles** — transfer rules + groups between machines as a single JSON file
- **Import from netsh** — paste a PowerShell script, auto-parse into rules
- **Startup actions** — run commands on WSL start with configurable delay chaining

### System
- **Auto-updates** — checks GitHub releases, downloads and installs with one click
- **Windows Service** — one-click install with UAC elevation, auto-start on boot
- **WSL distro management** — list distros, aliases, status, resource stats
- **.wslconfig inspector** — read/edit WSL settings with mirrored-mode warnings
- **Audit log** — timestamped event log with filter and export
- **13 themes** — Mission Control, CENTCOM, Nord, Dracula, Monokai, and more
- **~4 MB installer** — Tauri v2, no Electron bloat

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

Grab the latest from the [Releases](../../releases) page:

| File | Description |
| --- | --- |
| `WSL.PortHole_x.x.x_x64-setup.exe` | NSIS installer (recommended) |
| `WSL.PortHole_x.x.x_x64_en-US.msi` | MSI installer (enterprise/GPO) |

The app checks for updates automatically and can install them with one click from the Updates page.

### Build from source

See [docs/BUILD.md](./docs/BUILD.md) for full build instructions.

```powershell
git clone https://github.com/immersedone/wsl-porthole.git
cd wsl-porthole
npm install
npx tauri build
```

## Windows Service

The service runs independently of the GUI — rules stay active with the app closed.

**Install from the GUI:** Open the app → Boot Service → Install Service. A UAC prompt will appear for admin privileges.

**Manual install:**
```powershell
# Build the service
cargo build --release -p wsl-porthole-service

# Install (run as Administrator)
sc create WslPortHole binPath="path\to\wsl-porthole-service.exe" start=auto
sc start WslPortHole
```

The service:
1. Detects WSL IP changes (30s polling + Hyper-V event subscription)
2. Waits 5s for the IP to settle
3. Re-applies all enabled portproxy + firewall rules
4. Sends a Windows toast notification

## Transferring between machines

Export your entire setup from Port Rules → Export Bundle. This creates a JSON file containing all rules and groups. On the target machine, use Import → Bundle (JSON) → choose merge or replace.

## Tech stack

| Layer | Technology |
| --- | --- |
| App shell | Tauri v2 (~4 MB installer) |
| Frontend | Vue 3 + TypeScript + Tailwind CSS |
| Core logic | Rust (`wsl-porthole-core` crate) |
| Windows Service | Rust (`wsl-porthole-service` crate) |
| Docker API | bollard (WSL unix socket + Windows named pipe) |
| Windows APIs | netsh, PowerShell (firewall), windows-service |
| Config | JSON files in `%APPDATA%\WSL PortHole\` |

## Project structure

```text
wsl-porthole/
├── crates/
│   ├── wsl-porthole-core/       # Pure Rust library (ip, rules, netsh, firewall, docker, mcp)
│   └── wsl-porthole-service/    # Windows Service (watcher, IPC, toast notifications)
├── src-tauri/                   # Tauri v2 app shell (25+ commands bridging Rust → Vue)
├── src/                         # Vue 3 frontend (14 pages, 7 components, 13 themes)
└── docs/                        # Build instructions, bridge scripts
```

## License

MIT

## Docs

- [BUILD.md](./docs/BUILD.md) — build from source, Windows compilation, release process
- [CLAUDE.md](./CLAUDE.md) — full design context, architecture, decisions, and roadmap
