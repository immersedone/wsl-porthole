# WSL PortHole — Project Bible for Claude

> **This is the complete context document for WSL PortHole.**
> Drop this file into any Claude session (Claude Code, claude.ai Project, or
> VSCode extension) to resume with full context instantly.
>
> Last updated: 2026-04-02
> Original conversation: (paste URL here after sharing)

---

## 1. What is WSL PortHole?

WSL PortHole is a **Windows desktop application + background Windows Service**
that permanently solves the WSL2 port-forwarding problem:

- WSL2 gets a new dynamic IP address every time it restarts
- `netsh portproxy` rules hardcode that IP — so they break on every restart
- The current workaround is running a manual PowerShell script every time
- WSL PortHole eliminates this entirely — install once, it self-manages forever

### What it does

- Manages `netsh interface portproxy` rules between Windows, WSL2, and Docker
- Detects WSL IP changes automatically via a Hyper-V network event
- Re-applies all rules with the new IP — no scripts, no manual steps
- Exposes WSL/Docker services to the LAN (`0.0.0.0` binding)
- Routes Docker MCP servers (Windows engine) into WSL
- Auto-discovers running Docker container ports and suggests rules
- Provides a modern GUI to manage everything visually

### Display name
`WSL PortHole` — camel-case P+H for logo legibility.
Package/repo slug: `wsl-porthole`
Crate names: `wsl-porthole-core`, `wsl-porthole-service`

---

## 2. Tech Stack

| Layer | Technology | Notes |
|---|---|---|
| App shell | **Tauri v2** | ~4MB binary, native Windows, system tray, UAC elevation |
| Frontend | **React + TypeScript + Tailwind** | WebView2 renderer |
| Core logic | **Rust** (`crates/wsl-porthole-core`) | Pure library, no Tauri dep, fully testable |
| Windows Service | **Rust** (`crates/wsl-porthole-service`) | `windows-service` crate |
| Async runtime | `tokio` | Full async throughout |
| Docker API | `bollard` | Works with WSL engine (unix socket) and Windows engine (named pipe) |
| Windows APIs | `windows-rs` | netsh, firewall, event log, toast notifications |
| Config | `serde` + `serde_json` | JSON rule files, theme files |
| Build | Cargo workspace | All crates in one workspace |

### Key inspiration: WSL UI
https://github.com/octasoft-ltd/wsl-ui — same Tauri + Rust + React stack.
Borrow: `crates/` separation, 29-token theme system, startup actions pattern,
persistent status bar, filter bar, per-distro targeting.
WSL PortHole is complementary (networking) not competing (distro management).

---

## 3. Repository Structure

```
wsl-porthole/
├── CLAUDE.md                            ← this file
├── README.md
├── Cargo.toml                           ← workspace root
├── tauri.conf.json
├── package.json
├── .gitignore
│
├── crates/
│   ├── wsl-porthole-core/               ← pure Rust library
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── lib.rs
│   │       ├── ip.rs                    ← WSL IP detection
│   │       ├── rules.rs                 ← rule model + variable resolution
│   │       ├── netsh.rs                 ← netsh portproxy CRUD
│   │       ├── firewall.rs              ← Windows Defender firewall rules
│   │       ├── docker.rs                ← Docker Engine API (bollard)
│   │       ├── mcp.rs                   ← MCP server detection
│   │       ├── config.rs                ← load/save JSON config
│   │       └── import.rs                ← parse existing netsh scripts
│   │
│   └── wsl-porthole-service/            ← Windows Service
│       ├── Cargo.toml
│       └── src/
│           ├── main.rs                  ← service entry point
│           └── watcher.rs               ← Hyper-V event watcher loop
│
├── src/                                 ← React frontend (Tauri)
│   ├── main.tsx
│   ├── App.tsx
│   ├── components/
│   │   ├── RuleCard.tsx
│   │   ├── StatusBar.tsx
│   │   ├── FilterBar.tsx
│   │   ├── ThemeEditor.tsx
│   │   ├── DockerPanel.tsx
│   │   └── McpPanel.tsx
│   ├── pages/
│   │   ├── Rules.tsx
│   │   ├── Groups.tsx
│   │   ├── DockerSync.tsx
│   │   ├── McpServers.tsx
│   │   ├── LanAccess.tsx
│   │   ├── Firewall.tsx
│   │   ├── Distros.tsx
│   │   ├── StartupActions.tsx
│   │   ├── BootService.tsx
│   │   ├── WslConfig.tsx
│   │   ├── AuditLog.tsx
│   │   ├── Appearance.tsx
│   │   └── Settings.tsx
│   └── themes/
│       ├── tokens.ts
│       ├── system.json
│       ├── daylight.json
│       ├── mission-control.json
│       ├── nord.json
│       ├── dracula.json
│       └── centcom.json
│
├── docs/
│   ├── design/
│   └── scripts/
│       ├── wsl-porthole-bridge.ps1      ← bridge script (use now)
│       └── wsl-porthole-register.ps1    ← Task Scheduler setup (run ONCE)
│
└── wsl-porthole-rules.json              ← user's 13 imported rules
```

---

## 4. Core Concepts

### 4.1 Rule Model

```json
{
  "id": "uuid-v4",
  "name": "Django API",
  "direction": "WinToWsl",
  "listenAddr": "0.0.0.0",
  "listenPort": { "Single": 8000 },
  "connectPort": { "Single": 8000 },
  "connectAddr": "${WSL_IP}",
  "distro": null,
  "lan": true,
  "enabled": true,
  "source": "Manual",
  "note": null
}
```

Port ranges: `"listenPort": { "Range": [1024, 1048] }`
Remapped ports: `"listenPort": { "Single": 8080 }, "connectPort": { "Single": 80 }`

### 4.2 Variable System

| Variable | Resolves to |
|---|---|
| `${WSL_IP}` | Current IP of the default WSL distro |
| `${WSL_IP:Ubuntu-24.04}` | IP of a specific named distro |
| `${HOST_IP}` | Windows host LAN IP (e.g. 192.168.1.42) |
| `${HOST_GW}` | WSL→Windows gateway IP (for WSL→WIN rules) |
| `${DISTRO_NAME}` | Active distro name |

### 4.3 Rule Directions

**WinToWsl** — the common case:
- Mechanism: `netsh interface portproxy add v4tov4`
- Creates matching Windows Defender inbound firewall rule atomically
- `listenAddr=0.0.0.0` = LAN visible; `127.0.0.1` = local only
- Re-applied automatically on WSL IP change by the service

**WslToWin** — reaching Windows services from WSL:
- No portproxy needed — WSL routes to Windows via Hyper-V gateway
- Mechanism: `New-NetFirewallRule -InterfaceAlias "vEthernet (WSL)"`
- Optionally writes `/etc/hosts` alias into WSL (e.g. `mcp-gw 172.22.192.1`)
- Optionally injects env vars into WSL `.bashrc`/`.profile`

### 4.4 Traffic Flow

```
LAN device (192.168.x.x)
  → Windows host 0.0.0.0:PORT        [netsh portproxy]
  → WSL2 172.22.x.x:PORT             [Hyper-V bridge]
  → Docker container 127.0.0.1:PORT   [Docker internal proxy]

WSL process
  → Windows gateway ${HOST_GW}:PORT  [Hyper-V routing]
  → Windows Firewall allow on vEthernet (WSL)
  → Docker/app on Windows engine
```

### 4.5 Windows Service

1. Registers as Windows Service (auto-start on boot)
2. Subscribes to Hyper-V VmSwitch event log — Event ID 102
3. On event: waits 3–5s for WSL IP to settle
4. `wsl hostname -I` → parse first IPv4
5. If IP changed: `netsh portproxy reset` → re-apply all enabled rules
6. Update Windows Defender firewall rules
7. Emit Windows toast: "WSL PortHole: rules updated (new IP: x.x.x.x)"
8. Fallback: 30s polling loop if event subscription unavailable

The GUI is completely separate — rules stay active with the GUI closed.

---

## 5. User's Imported Rules

Original: 38-line netsh script with hardcoded IP `172.22.207.71`.
Parsed into 13 logical rules in `wsl-porthole-rules.json`:

| # | Name | Listen | Connect | Type |
|---|---|---|---|---|
| 1 | HTTP | 80 | 80 | standard |
| 2 | HTTP alt | 8080 | 80 | remapped |
| 3 | HTTPS | 443 | 443 | standard |
| 4 | HTTPS alt | 4433 | 443 | remapped |
| 5 | SSH | 22 | 22 | standard |
| 6 | SSH alt | 28 | 22 | remapped |
| 7 | FTP alt | 218 | 21 | remapped |
| 8 | App range | 1024–1048 | 1024–1048 | range (25→1 rule) |
| 9 | App 8100 | 8100 | 8100 | standard |
| 10 | App 6001 | 6001 | 6001 | standard |
| 11 | Meilisearch | 7700 | 7700 | standard |
| 12 | Dev server | 3002 | 3002 | standard |
| 13 | Vite dev | 5173 | 5173 | standard |

All `listenAddr=0.0.0.0` (LAN visible), `connectAddr=${WSL_IP}`.

---

## 6. Full Feature List

### Rule management
- [ ] Rule CRUD (add, edit, delete, toggle)
- [ ] Port range rules (1024–1048 as one rule, expands at apply-time)
- [ ] Port remapping (listenPort ≠ connectPort)
- [ ] Variable substitution in connectAddr
- [ ] Per-distro targeting
- [ ] LAN toggle per rule (0.0.0.0 vs 127.0.0.1)
- [ ] Atomic firewall management (portproxy + firewall created/deleted together)
- [ ] Inline netsh command preview per rule
- [ ] Import from netsh script (paste .ps1, auto-parse)
- [ ] Import from JSON
- [ ] Export as JSON
- [ ] Export as netsh .ps1 script
- [ ] Rule duplication

### Auto-management (Windows Service)
- [ ] Windows Service registration (auto-start on boot)
- [ ] Hyper-V event subscription (Event ID 102)
- [ ] WSL IP change detection
- [ ] Auto-reapply all rules on IP change
- [ ] Per-distro IP tracking
- [ ] Firewall auto-sync
- [ ] Toast notification on IP change
- [ ] Fallback 30s polling loop
- [ ] Service health exposed to GUI

### Discovery
- [ ] Docker WSL engine discovery (bollard, unix socket)
- [ ] Docker Windows engine discovery (bollard, named pipe)
- [ ] Auto-suggest rules for unforwarded container ports
- [ ] Docker sync mode per rule
- [ ] MCP server detection (Windows engine containers)
- [ ] Container name → rule name mapping
- [ ] docker-compose project grouping

### WSL→Windows routing
- [ ] Firewall rule on vEthernet (WSL) per WSL→WIN rule
- [ ] /etc/hosts injection into WSL (friendly alias for gateway)
- [ ] Env var injection into WSL .bashrc/.profile
- [ ] Gateway IP auto-detection from WSL

### UI — rule list
- [ ] Direction badge (WIN→WSL / WSL→WIN)
- [ ] Distro badge
- [ ] Source badge (docker / mcp / manual / imported)
- [ ] Live health dot (green/amber/red, TCP reachability)
- [ ] Conflict indicator (port already bound by Windows process)
- [ ] LAN/local pill (globe / lock icon)
- [ ] Toggle switch per rule
- [ ] Port badge (shows remapping and ranges)
- [ ] Inline netsh command (monospace preview)
- [ ] Three-dot menu (edit, duplicate, delete, copy command, open in browser, QR code)

### UI — filter bar
- [ ] Filter by direction / scope / source / health
- [ ] Full-text search
- [ ] Active filter count badge

### UI — status bar (always visible)
- [ ] Service status dot
- [ ] Active rule count
- [ ] LAN exposure count
- [ ] Conflict count (amber if > 0)
- [ ] WSL IP (click to copy, click to force re-sync)
- [ ] Host IP (click to copy)
- [ ] Active distro name
- [ ] Last sync time

### UI — sidebar navigation
- [ ] Port rules
- [ ] Groups / profiles
- [ ] Docker sync
- [ ] MCP servers
- [ ] LAN access
- [ ] Firewall rules
- [ ] Distros (active distro selector)
- [ ] Startup actions
- [ ] Boot service (install / uninstall / restart)
- [ ] .wslconfig inspector
- [ ] Audit log
- [ ] Appearance (themes)
- [ ] Settings

### System tray
- [ ] Minimize to tray
- [ ] Tray icon with service status colour
- [ ] Context menu: Open / Groups / Sync now / Exit
- [ ] Group quick-toggle from tray

### Groups / profiles
- [ ] Named groups (e.g. "Django stack" = 8000+5432+6379)
- [ ] One-click enable/disable group
- [ ] Tray quick-toggle per group
- [ ] Per-group startup behaviour
- [ ] Import/export groups

### Startup actions (WSL UI pattern)
- [ ] Commands on WSL-start event
- [ ] Variable substitution (${DISTRO_NAME}, ${WSL_IP}, etc.)
- [ ] Action chaining with configurable delays
- [ ] Built-in: sync rules / write /etc/hosts / inject env vars
- [ ] Custom shell commands
- [ ] Target scoping (all / specific / regex)

### QR code / LAN URL
- [ ] QR code for any LAN-exposed rule
- [ ] Copy URL button
- [ ] Auto-updates when host IP changes

### Conflict detection
- [ ] Scan listen ports vs Windows TCP listeners
- [ ] Warn before applying conflicting rule
- [ ] Identify owning Windows process
- [ ] Offer to kill process or change port

### Health checks
- [ ] Per-rule TCP connect check (60s interval)
- [ ] Green/amber/red status dots
- [ ] Manual re-check from three-dot menu

### .wslconfig inspector
- [ ] Read/edit networkingMode, memory, CPU, swap, DNS, autoProxy
- [ ] Warn on mirrored + VPN combination
- [ ] Warn on mirrored + Windows Server
- [ ] Apply changes (restart WSL)

### Audit log
- [ ] Timestamped: rule changes, IP changes, service events, conflicts
- [ ] Filter by date and event type
- [ ] Export as text

### Theme system (29 tokens)
- [ ] 13 built-in themes (see §7)
- [ ] Custom theme editor with live preview
- [ ] Export/import as .wph-theme.json

### Keyboard navigation
- [ ] Arrow keys in rule list
- [ ] Space to toggle, Enter to edit, Delete to remove
- [ ] Ctrl+N add, Ctrl+F search, Ctrl+S sync

---

## 7. Theme System

29 CSS token variables per theme. JSON files, shareable as single files.

### Built-in themes

| Theme | Category |
|---|---|
| System | Auto (follows OS) — default |
| Daylight | Light |
| Obsidian Light | Light |
| Mission Control | Dark (cyan on dark) |
| Obsidian | Dark |
| Nord | Dark |
| Dracula | Dark |
| Monokai | Dark |
| GitHub Dark | Dark |
| Solarized Dark | Dark |
| CENTCOM | Dark (dark olive + amber, tactical ops-room) |
| High Contrast | Accessibility |
| High Contrast Light | Accessibility |

### CENTCOM token reference

```json
{
  "--bg-primary":     "#1a1a0d",
  "--bg-secondary":   "#252510",
  "--bg-tertiary":    "#0f0f08",
  "--accent":         "#EF9F27",
  "--accent-dim":     "#BA7517",
  "--text-primary":   "#EF9F27",
  "--text-secondary": "#B4A882",
  "--status-ok":      "#8fbd3a",
  "--status-warn":    "#EF9F27",
  "--status-err":     "#E24B4A",
  "--border":         "#3a3a00"
}
```

---

## 8. Development Roadmap

### Phase 1 — Core Rust library (`crates/wsl-porthole-core`)
- [ ] `ip.rs` — detect_wsl_ip(), detect_wsl_ip_for(distro), detect_host_ip(), detect_host_gateway()
- [ ] `rules.rs` — Rule struct, Direction, PortSpec, Source, resolve_addr(), expand_ports()
- [ ] `config.rs` — load_rules(path), save_rules(path, rules)
- [ ] `netsh.rs` — apply_rule(), remove_rule(), reset_all(), list_active()
- [ ] `firewall.rs` — add_inbound_rule(), remove_rule(), add_wsl_interface_rule()
- [ ] `import.rs` — parse_netsh_script(text) -> Vec<Rule>
- [ ] `docker.rs` — list_wsl_containers(), list_windows_containers(), container_ports(id)
- [ ] `mcp.rs` — detect_mcp_servers()
- [ ] Unit tests for all modules

### Phase 2 — Windows Service (`crates/wsl-porthole-service`)
- [ ] Windows Service scaffolding (windows-service crate)
- [ ] Service install/uninstall/start/stop
- [ ] watcher.rs — Hyper-V VmSwitch Event ID 102 subscription
- [ ] IP change detection + rule reapplication
- [ ] Firewall sync
- [ ] Toast notifications
- [ ] Fallback 30s polling
- [ ] Status IPC (pipe/socket for GUI to query)

### Phase 3 — Tauri app + basic rule list
- [ ] Tauri v2 + React + TypeScript + Tailwind scaffold
- [ ] Tauri commands wrapping wsl-porthole-core
- [ ] Rule list, toggle, add/edit/delete
- [ ] Status bar (service, IP, rule count)
- [ ] Sidebar stub navigation
- [ ] System theme

### Phase 4 — Import + service integration
- [ ] Import from netsh script (paste dialog + auto-parse preview)
- [ ] Import/export JSON
- [ ] Export as netsh .ps1
- [ ] Boot service page (install/uninstall/start/stop from GUI)
- [ ] Live service status in status bar
- [ ] Sync now button

### Phase 5 — Docker discovery
- [ ] Docker panel (WSL engine containers + exposed ports)
- [ ] Windows engine Docker panel (MCP servers)
- [ ] Add rule from container
- [ ] Allow in WSL (WSL→WIN firewall rule)
- [ ] Auto-refresh 30s

### Phase 6 — Filter, search, conflict, health
- [ ] Filter bar
- [ ] Conflict detection + resolution
- [ ] Health check dots (TCP)
- [ ] QR code generator

### Phase 7 — Advanced features
- [ ] Groups / profiles
- [ ] Startup actions + chaining
- [ ] /etc/hosts + env var injection
- [ ] Audit log
- [ ] .wslconfig inspector
- [ ] Keyboard navigation

### Phase 8 — Theme system + tray + polish
- [ ] Full 13-theme system + 29 token editor
- [ ] Custom theme editor with live preview
- [ ] System tray
- [ ] Multi-distro support
- [ ] Installer (NSIS/MSI via Tauri bundler)
- [ ] Auto-updater

---

## 9. Key Decisions Log

| Decision | Choice | Rationale |
|---|---|---|
| App framework | Tauri v2 | ~4MB binary, native APIs, no Electron |
| Language | Rust + React/TS | Rust for Windows APIs; React for UI speed |
| Core lib | Separate crate | Testable without Tauri; shared by GUI + service |
| Service trigger | Hyper-V Event ID 102 | Event-driven, no polling overhead |
| IP variable | ${WSL_IP} in config | Never hardcode IPs |
| Rule storage | JSON | Simple, portable, no database |
| Theme system | 29 CSS tokens as JSON | WSL UI proved this; shareable single files |
| Bi-directional | netsh (WIN→WSL) + firewall only (WSL→WIN) | Different OS mechanisms |
| Port ranges | Single rule, expand at apply-time | 38 lines → 13 logical rules |
| Firewall | Atomic with portproxy | Silent half-rules cause confusion |
| Docker | bollard for both unix + named pipe | One library for both engines |

---

## 10. Known Gotchas

- `netsh portproxy` is **TCP only** — no UDP support. Document clearly.
- **Mirrored mode + full-tunnel VPN** (GlobalProtect, Cisco, OpenVPN TAP) = broken. Detect and warn.
- **Mirrored mode on Windows Server 2025** silently falls back to NAT.
- **Port 5432** commonly conflicts with local Windows PostgreSQL.
- **Admin rights required** for netsh + firewall — Tauri UAC elevation needed.
- **WSL IP takes 3–5s** to settle after WSL starts — add configurable delay in service.
- **Multiple distros** have different IPs — track each separately.
- **Docker Windows engine** uses `npipe:////./pipe/docker_engine`; WSL engine uses unix socket.
- **netsh portproxy rules persist** across reboots — only connectaddress breaks. Never reset_all unnecessarily.
- **IPv6** — start with v4tov4 only; v6 support is future work.

---

## 11. Design References

| Project | URL |
|---|---|
| WSL UI | https://github.com/octasoft-ltd/wsl-ui |
| PortProxyGooey | https://github.com/STaRDoGG/PortProxyGooey |
| wsl2-auto-portproxy | https://github.com/HobaiRiku/wsl2-auto-portproxy |
| WSL networking docs | https://learn.microsoft.com/en-us/windows/wsl/networking |

---

## 12. Bridge Scripts (use today, before app is built)

`docs/scripts/wsl-porthole-bridge.ps1`
Auto-detects WSL IP, applies all 13 rules. Run manually after WSL restart.

`docs/scripts/wsl-porthole-register.ps1`
Run **once** with admin rights. Registers Task Scheduler entry:
- Trigger 1: At logon
- Trigger 2: Hyper-V network event + 5s delay

After running once, rules auto-update on every WSL restart forever.

---

## 13. Teleport Instructions

### Claude Code (recommended)
```bash
cd wsl-porthole
claude
# CLAUDE.md is read automatically. Say:
# "Start Phase 1. Implement ip.rs in wsl-porthole-core."
```

### claude.ai Projects
1. Projects → New project → "WSL PortHole"
2. Project instructions → paste this CLAUDE.md
3. Attach wsl-porthole-rules.json
4. Every conversation starts with full context

### New chat
Paste this file, then describe what to build next.

---

## 14. Session History

Original design session: 2026-04-02, claude.ai
Conversation URL: (paste here)
