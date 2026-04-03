# Building WSL PortHole

This guide covers building WSL PortHole from source, generating installers, and publishing to the Microsoft Store.

## Prerequisites

| Tool | Version | Install |
| --- | --- | --- |
| Rust | 1.75+ | `winget install Rustlang.Rustup` or [rustup.rs](https://rustup.rs/) |
| Node.js | 18+ | `winget install OpenJS.NodeJS.LTS` or [nodejs.org](https://nodejs.org/) |
| Windows | 10/11 | Required for Tauri + netsh + firewall APIs |
| WebView2 | Latest | Pre-installed on Windows 10/11. [Manual download](https://developer.microsoft.com/en-us/microsoft-edge/webview2/) |

### Optional (for MSIX / Store)

| Tool | Purpose |
| --- | --- |
| [MSIX Packaging Tool](https://learn.microsoft.com/en-us/windows/msix/packaging-tool/tool-overview) | Convert MSI to MSIX for the Store |
| [Windows SDK](https://developer.microsoft.com/en-us/windows/downloads/windows-sdk/) | `makeappx.exe`, `signtool.exe` for manual MSIX creation |
| Code-signing certificate | Required for Store submission and trusted installs |

## Quick build

```powershell
git clone https://github.com/user/wsl-porthole.git
cd wsl-porthole
npm install
npx tauri build
```

This produces three outputs:

```
src-tauri/target/release/bundle/
├── nsis/WSL PortHole_0.1.0-alpha_x64-setup.exe    # ~2.7 MB NSIS installer
└── msi/WSL PortHole_0.1.0-alpha_x64_en-US.msi     # ~4 MB MSI installer

target/release/
└── wsl-porthole-app.exe                       # ~12 MB standalone binary
```

## Building individual components

### GUI app only

```powershell
npm install
npx tauri build
```

### Windows Service only

```powershell
cargo build --release -p wsl-porthole-service
# Output: target/release/wsl-porthole-service.exe (~1 MB)
```

### Core library only (with tests)

```powershell
cargo test -p wsl-porthole-core
cargo build --release -p wsl-porthole-core
```

### Frontend only (no Tauri)

```powershell
npm install
npm run build
# Output: dist/ — static HTML/JS/CSS, can be served or opened in a browser
```

## Development mode

```powershell
npm install
npx tauri dev
```

This starts:
- Vite dev server on `http://localhost:1420` with hot reload
- Tauri native window pointing at the dev server
- Rust recompilation on save

For frontend-only development (no Rust/Tauri needed):

```powershell
npm run dev
# Open http://localhost:1420 — runs with demo data in browser mode
```

## Building from WSL

The project lives in WSL but must be compiled on the Windows side for the native binary. From within WSL:

```bash
# 1. Copy project to Windows filesystem (UNC paths are slow for cargo)
rsync -a --exclude='target' --exclude='node_modules' --exclude='dist' --exclude='.git' \
  /path/to/wsl-porthole/ /mnt/c/wsl-porthole-build/

# 2. Build via PowerShell interop
powershell.exe -NoProfile -Command "
  cd C:\wsl-porthole-build
  npm install
  npx tauri build
"

# 3. Build the service separately
powershell.exe -NoProfile -Command "
  cd C:\wsl-porthole-build
  cargo build --release -p wsl-porthole-service
"

# Installers will be in:
#   /mnt/c/wsl-porthole-build/src-tauri/target/release/bundle/nsis/
#   /mnt/c/wsl-porthole-build/src-tauri/target/release/bundle/msi/
#   /mnt/c/wsl-porthole-build/target/release/wsl-porthole-service.exe
```

## Linux / CI compilation

The Rust core library and service crate compile on Linux for development and testing:

```bash
# Install Tauri system dependencies (Ubuntu/Debian)
sudo apt-get install -y \
  libglib2.0-dev libgtk-3-dev libwebkit2gtk-4.1-dev \
  libjavascriptcoregtk-4.1-dev libsoup-3.0-dev \
  libayatana-appindicator3-dev librsvg2-dev

# Check everything compiles
cargo check               # full workspace (needs GTK libs above)
cargo check -p wsl-porthole-core -p wsl-porthole-service  # skip Tauri

# Run tests (no Windows needed — pure logic tests)
cargo test -p wsl-porthole-core    # 24 tests

# Build frontend
npm install && npm run build       # outputs to dist/
```

Note: The Tauri crate (`src-tauri/`) requires GTK/WebKit dev libraries on Linux. The `wsl-porthole-core` and `wsl-porthole-service` crates compile on any platform.

## Installer details

### NSIS installer (`*-setup.exe`)

- Recommended for end users
- Includes uninstaller in Add/Remove Programs
- Per-user or per-machine install
- ~2.7 MB

### MSI installer (`*.msi`)

- For enterprise/GPO deployment
- Compatible with `msiexec` silent install:
  ```powershell
  msiexec /i "WSL PortHole_0.1.0-alpha_x64_en-US.msi" /quiet
  ```
- ~4 MB

### Windows Service

The service binary is **not** bundled with the GUI installer. Install it separately:

```powershell
# Copy to a permanent location
Copy-Item wsl-porthole-service.exe "C:\Program Files\WSL PortHole\"

# Install and start (requires Administrator)
& "C:\Program Files\WSL PortHole\wsl-porthole-service.exe" install
sc start WslPortHole

# Verify
sc query WslPortHole
```

The service is configured for:
- Auto-start on boot
- Recovery: restart on first 3 failures (5s, 10s, 30s delays)
- Runs as Local System

## Microsoft Store publishing

### 1. Create MSIX package

The Store requires MSIX format. Convert from the MSI using the MSIX Packaging Tool:

```powershell
# Install from the Microsoft Store (free)
# Or via winget:
winget install "MSIX Packaging Tool"
```

Open the tool, select **Application package**, point it at the MSI, and follow the wizard. Key settings:

| Field | Value |
| --- | --- |
| Package name | `WslPortHole` |
| Publisher | `CN=YourName` (must match your certificate) |
| Version | `0.1.0-alpha.0` (4-part) |
| Package architecture | `x64` |

### 2. Alternatively: create MSIX manually

```powershell
# Requires Windows SDK (makeappx.exe, signtool.exe)

# Create the package layout directory
mkdir msix-layout
# Copy the installed app files into msix-layout/

# Create AppxManifest.xml (see template below)
# Then:
makeappx.exe pack /d msix-layout /p "WSLPortHole_0.1.0-alpha_x64.msix"

# Sign with your certificate
signtool.exe sign /fd SHA256 /a /f cert.pfx /p password "WSLPortHole_0.1.0-alpha_x64.msix"
```

<details>
<summary>AppxManifest.xml template</summary>

```xml
<?xml version="1.0" encoding="utf-8"?>
<Package
  xmlns="http://schemas.microsoft.com/appx/manifest/foundation/windows10"
  xmlns:uap="http://schemas.microsoft.com/appx/manifest/uap/windows10"
  xmlns:rescap="http://schemas.microsoft.com/appx/manifest/foundation/windows10/restrictedcapabilities">

  <Identity
    Name="WslPortHole"
    Publisher="CN=YourPublisherName"
    Version="0.1.0-alpha.0"
    ProcessorArchitecture="x64" />

  <Properties>
    <DisplayName>WSL PortHole</DisplayName>
    <PublisherDisplayName>Your Name</PublisherDisplayName>
    <Logo>icons\StoreLogo.png</Logo>
  </Properties>

  <Dependencies>
    <TargetDeviceFamily Name="Windows.Desktop" MinVersion="10.0.17763.0" MaxVersionTested="10.0.22621.0" />
  </Dependencies>

  <Resources>
    <Resource Language="en-us" />
  </Resources>

  <Applications>
    <Application Id="WslPortHole" Executable="wsl-porthole-app.exe" EntryPoint="Windows.FullTrustApplication">
      <uap:VisualElements
        DisplayName="WSL PortHole"
        Description="Auto-managing WSL2 port forwarding"
        Square150x150Logo="icons\Square150x150Logo.png"
        Square44x44Logo="icons\Square44x44Logo.png"
        BackgroundColor="transparent" />
    </Application>
  </Applications>

  <Capabilities>
    <rescap:Capability Name="runFullTrust" />
  </Capabilities>
</Package>
```

</details>

### 3. Register as a developer

1. Go to [Microsoft Partner Center](https://partner.microsoft.com/dashboard)
2. Pay the one-time $19 registration fee
3. Reserve the app name **"WSL PortHole"**

### 4. Submit to the Store

1. Create a new app submission in Partner Center
2. Upload the MSIX package
3. Fill in the listing:

| Field | Value |
| --- | --- |
| Category | Developer tools > Utilities |
| Description | Auto-managing WSL2 ↔ Windows port forwarding. Install once, forget forever. |
| Features | Port forwarding, Docker discovery, MCP routing, 13 themes |
| Screenshots | Use images from `docs/screenshots/` |
| Privacy policy | Required — can link to a GitHub page |
| System requirements | Windows 10 1809+, WebView2 |

4. Submit for certification review (typically 1-3 business days)

### Store requirements checklist

- [ ] MSIX package format
- [ ] Signed with a trusted certificate (or use Partner Center auto-signing)
- [ ] Passes Windows App Certification Kit (WACK) — run `appcert.exe` locally first
- [ ] Privacy policy URL
- [ ] At least 1 screenshot (1366x768 or larger)
- [ ] App description and feature list
- [ ] Age rating questionnaire completed

## Troubleshooting

### `cargo tauri build` fails with "glib-2.0 not found"

You're on Linux. Install GTK dev libraries:

```bash
sudo apt-get install -y libglib2.0-dev libgtk-3-dev libwebkit2gtk-4.1-dev \
  libjavascriptcoregtk-4.1-dev libsoup-3.0-dev libayatana-appindicator3-dev librsvg2-dev
```

### "resource file icon.ico is not in 3.00 format"

The `.ico` file is malformed. Regenerate icons or use a proper icon editor. The ICO must be a real multi-resolution ICO file, not a renamed PNG.

### `netsh` commands fail

Requires Administrator privileges. The Tauri app needs UAC elevation for netsh and firewall operations. During development, run your terminal as Administrator.

### Service won't start

```powershell
# Check the event log
Get-EventLog -LogName Application -Source WslPortHole -Newest 10

# Run in standalone mode to see errors
wsl-porthole-service.exe standalone
```

### Frontend builds but Tauri window is blank

Check that `frontendDist` in `tauri.conf.json` points to `"../dist"` and that `npm run build` completed successfully. The `dist/` folder must contain `index.html`.
