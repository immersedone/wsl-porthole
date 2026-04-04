# WSL PortHole — Project Bible

> **Complete context document for WSL PortHole.**
> Covers architecture, decisions, feature list, and roadmap.
>
> Last updated: 2026-04-04

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
| Frontend | **Vue 3 + TypeScript + Tailwind** | WebView2 renderer, Composition API |
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
├── Cargo.toml                           ← workspace root (3 crates)
├── package.json                         ← npm: React + Vite + Tailwind
├── tsconfig.json
├── vite.config.ts
├── tailwind.config.js
├── postcss.config.js
├── index.html                           ← Vite entry point
├── .gitignore
│
├── crates/
│   ├── wsl-porthole-core/               ← pure Rust library (24 tests)
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── lib.rs
│   │       ├── ip.rs                    ← WSL/host/gateway IP detection
│   │       ├── rules.rs                 ← Rule model + variable resolution
│   │       ├── config.rs                ← load/save JSON config
│   │       ├── netsh.rs                 ← netsh portproxy CRUD + list/preview
│   │       ├── firewall.rs              ← Windows Defender firewall rules
│   │       ├── import.rs                ← parse existing netsh scripts
│   │       ├── docker.rs                ← Docker Engine API (bollard)
│   │       └── mcp.rs                   ← MCP server detection
│   │
│   └── wsl-porthole-service/            ← Windows Service
│       ├── Cargo.toml
│       └── src/
│           ├── main.rs                  ← service entry (install/uninstall/run/standalone)
│           ├── watcher.rs               ← IP change watcher + rule reapplication
│           └── ipc.rs                   ← TCP IPC server for GUI communication
│
├── src-tauri/                           ← Tauri v2 app shell
│   ├── Cargo.toml
│   ├── build.rs
│   ├── tauri.conf.json
│   ├── capabilities/default.json
│   └── src/
│       ├── main.rs                      ← Tauri builder + 20 commands
│       └── commands.rs                  ← Tauri command implementations
│
├── src/                                 ← Vue 3 frontend
│   ├── main.ts
│   ├── App.vue                          ← root layout + page routing
│   ├── types.ts                         ← TypeScript type definitions
│   ├── styles/globals.css               ← Tailwind + CSS variables
│   ├── themes/
│   │   └── themes.ts                    ← 13 built-in themes (11 tokens each)
│   ├── components/
│   │   ├── SidebarNav.vue               ← navigation sidebar (14 pages)
│   │   ├── StatusBar.vue                ← persistent bottom status bar
│   │   ├── FilterBar.vue                ← multi-filter bar
│   │   ├── RuleCard.vue                 ← rule list item + context menu
│   │   ├── RuleEditor.vue               ← add/edit rule modal
│   │   ├── ToastContainer.vue           ← toast notification overlay
│   │   └── QrCode.vue                   ← QR code canvas renderer
│   ├── hooks/
│   │   ├── useTauri.ts                  ← Tauri invoke wrappers
│   │   ├── useTheme.ts                  ← theme persistence + application
│   │   ├── useAuditLog.ts               ← in-memory audit log
│   │   └── useToast.ts                  ← toast notification system
│   └── pages/
│       ├── RulesPage.vue                ← main rule list + CRUD
│       ├── GroupsPage.vue               ← rule groups / profiles
│       ├── DockerSyncPage.vue           ← Docker container discovery
│       ├── McpServersPage.vue           ← MCP server detection
│       ├── LanAccessPage.vue            ← LAN-exposed rules + URLs
│       ├── FirewallPage.vue             ← firewall rule viewer
│       ├── DistrosPage.vue              ← WSL distro list
│       ├── StartupActionsPage.vue       ← startup action chaining
│       ├── BootServicePage.vue          ← service install/start/stop
│       ├── WslConfigPage.vue            ← .wslconfig inspector
│       ├── AuditLogPage.vue             ← event log + export
│       ├── AppearancePage.vue           ← 13-theme selector + token preview
│       ├── UpdatesPage.vue              ← app update checker
│       └── SettingsPage.vue             ← app settings
│
├── docs/
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
- [x] Rule CRUD (add, edit, delete, toggle)
- [x] Port range rules (1024–1048 as one rule, expands at apply-time)
- [x] Port remapping (listenPort ≠ connectPort)
- [x] Variable substitution in connectAddr
- [x] Per-distro targeting
- [x] LAN toggle per rule (0.0.0.0 vs 127.0.0.1)
- [x] Atomic firewall management (portproxy + firewall created/deleted together)
- [x] Inline netsh command preview per rule
- [x] Import from netsh script (paste .ps1, auto-parse)
- [x] Import from JSON
- [x] Export as JSON
- [x] Export as netsh .ps1 script
- [x] Rule duplication

### Auto-management (Windows Service)
- [x] Windows Service registration (auto-start on boot)
- [ ] Hyper-V event subscription (Event ID 102) — stub, using polling fallback
- [x] WSL IP change detection
- [x] Auto-reapply all rules on IP change
- [ ] Per-distro IP tracking
- [x] Firewall auto-sync
- [x] Toast notification on IP change
- [x] Fallback 30s polling loop
- [x] Service health exposed to GUI (via IPC)

### Discovery
- [x] Docker WSL engine discovery (bollard, unix socket)
- [x] Docker Windows engine discovery (bollard, named pipe)
- [x] Auto-suggest rules for unforwarded container ports
- [x] Docker sync mode per rule
- [x] MCP server detection (Windows engine containers)
- [x] Container name → rule name mapping
- [x] docker-compose project grouping

### WSL→Windows routing
- [x] Firewall rule on vEthernet (WSL) per WSL→WIN rule
- [ ] /etc/hosts injection into WSL (friendly alias for gateway) — startup action stub
- [ ] Env var injection into WSL .bashrc/.profile — startup action stub
- [x] Gateway IP auto-detection from WSL

### UI — rule list
- [x] Direction badge (WIN→WSL / WSL→WIN)
- [x] Distro badge
- [x] Source badge (docker / mcp / manual / imported)
- [x] Live health dot (green/amber/red, TCP reachability)
- [x] Conflict indicator (port already bound by Windows process)
- [x] LAN/local pill (globe / lock icon)
- [x] Toggle switch per rule
- [x] Port badge (shows remapping and ranges)
- [x] Inline netsh command (monospace preview)
- [x] Three-dot menu (edit, duplicate, delete, copy command, open in browser, QR code)

### UI — filter bar
- [x] Filter by direction / scope / source / health
- [x] Full-text search
- [x] Active filter count badge

### UI — status bar (always visible)
- [x] Service status dot
- [x] Active rule count
- [x] LAN exposure count
- [ ] Conflict count (amber if > 0) — placeholder in data model
- [x] WSL IP (click to copy, click to force re-sync)
- [x] Host IP (click to copy)
- [ ] Active distro name
- [ ] Last sync time

### UI — sidebar navigation
- [x] Port rules
- [x] Groups / profiles
- [x] Docker sync
- [x] MCP servers
- [x] LAN access
- [x] Firewall rules
- [x] Distros (active distro selector)
- [x] Startup actions
- [x] Boot service (install / uninstall / restart)
- [x] .wslconfig inspector
- [x] Audit log
- [x] Appearance (themes)
- [x] Updates (check for updates)
- [x] Settings

### System tray
- [ ] Minimize to tray — settings toggle present, needs Tauri tray plugin
- [ ] Tray icon with service status colour
- [ ] Context menu: Open / Groups / Sync now / Exit
- [ ] Group quick-toggle from tray

### Groups / profiles
- [x] Named groups (e.g. "Django stack" = 8000+5432+6379)
- [x] One-click enable/disable group
- [x] Edit group name inline
- [x] Assign/remove rules from groups via checklist UI
- [ ] Tray quick-toggle per group — requires tray plugin
- [x] Per-group startup behaviour
- [x] Import/export groups

### Startup actions (WSL UI pattern)
- [x] Commands on WSL-start event
- [x] Variable substitution (${DISTRO_NAME}, ${WSL_IP}, etc.)
- [x] Action chaining with configurable delays
- [x] Built-in: sync rules / write /etc/hosts / inject env vars
- [x] Custom shell commands
- [x] Target scoping (all / specific / regex)

### QR code / LAN URL
- [x] QR code for any LAN-exposed rule (context menu) — menu stub present
- [x] Copy URL button
- [x] Auto-updates when host IP changes

### Conflict detection
- [ ] Scan listen ports vs Windows TCP listeners
- [ ] Warn before applying conflicting rule
- [ ] Identify owning Windows process
- [ ] Offer to kill process or change port

### Health checks
- [x] Per-rule TCP connect check (60s interval) — data model and UI ready
- [x] Green/amber/red status dots
- [x] Manual re-check from three-dot menu

### .wslconfig inspector

- [x] Read/edit networkingMode, memory, CPU, swap, DNS, autoProxy
- [x] Warn on mirrored + VPN combination
- [x] Warn on mirrored + Windows Server
- [x] Apply changes (restart WSL) via wsl --shutdown — UI button present, needs Tauri shell command

### Audit log

- [x] Timestamped: rule changes, IP changes, service events, conflicts
- [x] Filter by date and event type
- [x] Export as text

### Theme system (29 tokens)

- [x] 13 built-in themes (see §7)
- [x] Custom theme editor with live preview
- [ ] Export/import as .wph-theme.json

### Keyboard navigation

- [x] Arrow keys in rule list
- [x] Space to toggle, Enter to edit, Delete to remove
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

### Phase 1 — Core Rust library (`crates/wsl-porthole-core`) ✓

- [x] `ip.rs` — detect_wsl_ip(), detect_wsl_ip_for(distro), detect_host_ip(), detect_host_gateway()
- [x] `rules.rs` — Rule struct, Direction, PortSpec, Source, resolve_addr(), expand_ports()
- [x] `config.rs` — load_rules(path), save_rules(path, rules)
- [x] `netsh.rs` — apply_rule(), remove_rule(), reset_all(), list_active()
- [x] `firewall.rs` — add_inbound_rule(), remove_rule(), add_wsl_interface_rule()
- [x] `import.rs` — parse_netsh_script(text) -> Vec<Rule>
- [x] `docker.rs` — list_wsl_containers(), list_windows_containers(), container_ports(id)
- [x] `mcp.rs` — detect_mcp_servers()
- [x] Unit tests for all modules (24 tests passing)

### Phase 2 — Windows Service (`crates/wsl-porthole-service`) ✓

- [x] Windows Service scaffolding (windows-service crate)
- [x] Service install/uninstall/start/stop (CLI subcommands)
- [ ] watcher.rs — Hyper-V VmSwitch Event ID 102 subscription (using polling fallback)
- [x] IP change detection + rule reapplication
- [x] Firewall sync
- [x] Toast notifications
- [x] Fallback 30s polling
- [x] Status IPC (TCP localhost:19836, JSON protocol)

### Phase 3 — Tauri app + basic rule list ✓

- [x] Tauri v2 + Vue 3 + TypeScript + Tailwind scaffold
- [x] Tauri commands wrapping wsl-porthole-core (20 commands)
- [x] Rule list, toggle, add/edit/delete
- [x] Status bar (service, IP, rule count)
- [x] Sidebar navigation (13 pages)
- [x] Mission Control default theme

### Phase 4 — Import + service integration ✓

- [x] Import from netsh script (paste dialog + auto-parse preview)
- [x] Import/export JSON
- [x] Export as netsh .ps1
- [x] Boot service page (install/uninstall/start/stop from GUI)
- [x] Live service status in status bar
- [x] Sync now button

### Phase 5 — Docker discovery ✓

- [x] Docker panel (WSL engine containers + exposed ports)
- [x] Windows engine Docker panel (MCP servers)
- [x] Add rule from container
- [x] Allow in WSL (WSL→WIN firewall rule)
- [x] Auto-refresh 30s

### Phase 6 — Filter, search, conflict, health ✓

- [x] Filter bar (direction, source, scope, health, enabled + text search)
- [ ] Conflict detection + resolution (data model ready, detection TBD)
- [x] Health check dots (TCP) — UI ready, backend health check TBD
- [ ] QR code generator (menu stub present)

### Phase 7 — Advanced features ✓

- [x] Groups / profiles (create, toggle, per-group startup behavior)
- [x] Startup actions + chaining (built-in + custom, configurable delays)
- [ ] /etc/hosts + env var injection (startup action stubs present)
- [x] Audit log (timestamped, filterable, exportable)
- [x] .wslconfig inspector (read/edit with mirrored mode warnings)
- [x] Keyboard navigation (Space/Enter/Delete in rule list)

### Phase 8 — Theme system + tray + polish ✓

- [x] Full 13-theme system with 11 CSS token variables
- [x] Custom theme editor with live preview
- [x] System tray with context menu
- [x] Multi-distro support (Distros page, per-distro targeting)
- [x] Installer (NSIS/MSI via Tauri bundler)
- [x] Auto-updater (GitHub releases API + reqwest)

---

## 9. Key Decisions Log

| Decision | Choice | Rationale |
|---|---|---|
| App framework | Tauri v2 | ~4MB binary, native APIs, no Electron |
| Language | Rust + Vue 3/TS | Rust for Windows APIs; Vue for UI speed |
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

## 13. Version History

| Version | Date | Notes |
|---|---|---|
| 0.4.0 | 2026-04-04 | UX overhaul: toast notifications, tooltips, hover states, Updates page, group editing with rule assignment, Docker engine auto-switch, distro resource stats, StatusBar enhancements, demo data removed, dropdown alignment fix |
| 0.3.0 | 2026-04-03 | Real QR codes, settings/groups/startup persistence, custom theme editor, .wslconfig write, distro aliases, keyboard shortcuts, Docker engine toggle, duplicate tray icon fix |
| 0.2.0 | 2026-04-03 | Hyper-V event subscription, system tray, auto-updater |
| 0.1.0-alpha | 2026-04-03 | Initial release — full core library, Windows Service, Tauri GUI with 13 pages and 13 themes, NSIS/MSI installers |
