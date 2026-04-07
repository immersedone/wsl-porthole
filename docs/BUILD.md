# WSL PortHole — Build & Release Instructions

> Verified 2026-04-08 on Windows 11 + WSL2 (Ubuntu 24.04)

## Prerequisites (Windows side)

Install all four via `winget` from a WSL terminal:

```bash
# Rust (MSVC target — installs to "C:\Program Files\Rust stable MSVC X.XX")
powershell.exe -Command "winget install Rustlang.Rust.MSVC --accept-source-agreements --accept-package-agreements"

# Node.js LTS (installs to "C:\Program Files\nodejs")
powershell.exe -Command "winget install OpenJS.NodeJS.LTS --accept-source-agreements --accept-package-agreements"

# NSIS (installs to "C:\Program Files (x86)\NSIS")
powershell.exe -Command "winget install NSIS.NSIS --accept-source-agreements --accept-package-agreements"

# VS Build Tools with C++ workload (REQUIRED — provides link.exe for MSVC target)
powershell.exe -Command "winget install Microsoft.VisualStudio.2022.BuildTools --accept-source-agreements --accept-package-agreements --override '--passive --wait --add Microsoft.VisualStudio.Workload.VCTools --includeRecommended'"
```

### PATH configuration

Rust from winget installs to a versioned folder, not `~/.cargo/bin`. The build
commands below use `cmd /c vcvars64.bat && ...` to set up the full MSVC
environment (includes `link.exe`), then prepend Rust/Node/NSIS.

```
RUST_PATH = C:\Program Files\Rust stable MSVC 1.94\bin   ← adjust version
NODE_PATH = C:\Program Files\nodejs
NSIS_PATH = C:\Program Files (x86)\NSIS
VCVARS    = C:\Program Files (x86)\Microsoft Visual Studio\2022\BuildTools\VC\Auxiliary\Build\vcvars64.bat
```

### WSL-side tools

```bash
# gh CLI (for release uploads) — no sudo needed
mkdir -p ~/.local/bin
curl -sL https://github.com/cli/cli/releases/latest/download/gh_*_linux_amd64.tar.gz | tar xz -C /tmp
cp /tmp/gh_*/bin/gh ~/.local/bin/gh
~/.local/bin/gh auth login
```

## Build Directory

The project is developed in WSL at `/var/www/vhosts/wsl-porthole` and synced
to `C:\wsl-porthole-build` for Windows compilation.

### Sync from WSL to Windows

```bash
rsync -av --delete \
  --exclude='node_modules' \
  --exclude='target' \
  --exclude='dist' \
  --exclude='.git' \
  /var/www/vhosts/wsl-porthole/ /mnt/c/wsl-porthole-build/
```

## Build Steps

### Option A: From WSL (recommended)

All commands run via `powershell.exe`, using `cmd /c vcvars64.bat` to load the
MSVC linker environment. Adjust the Rust version path if yours differs.

```bash
VCVARS='"C:\Program Files (x86)\Microsoft Visual Studio\2022\BuildTools\VC\Auxiliary\Build\vcvars64.bat"'
PATHS='C:\Program Files\Rust stable MSVC 1.94\bin;C:\Program Files\nodejs;C:\Program Files (x86)\NSIS'

# Install frontend dependencies
powershell.exe -Command "cmd /c '${VCVARS} >nul 2>&1 && set PATH=${PATHS};%PATH% && cd /d C:\wsl-porthole-build && npm install'"

# Build Tauri app (NSIS + MSI installers)
powershell.exe -Command "cmd /c '${VCVARS} >nul 2>&1 && set PATH=${PATHS};%PATH% && cd /d C:\wsl-porthole-build && npx tauri build'"

# Build service binary separately
powershell.exe -Command "cmd /c '${VCVARS} >nul 2>&1 && set PATH=${PATHS};%PATH% && cd /d C:\wsl-porthole-build && cargo build --release -p wsl-porthole-service'"
```

### Option B: From Windows PowerShell / Terminal

Open a **Developer Command Prompt for VS 2022** (or run `vcvars64.bat` first):

```powershell
cd C:\wsl-porthole-build
npm install
npx tauri build
cargo build --release -p wsl-porthole-service
```

### Output locations

| Artifact | Path |
|---|---|
| NSIS installer (.exe) | `src-tauri/target/release/bundle/nsis/WSL PortHole_<ver>_x64-setup.exe` |
| MSI installer (.msi) | `src-tauri/target/release/bundle/msi/WSL PortHole_<ver>_x64_en-US.msi` |
| Service binary (.exe) | `target/release/wsl-porthole-service.exe` |

## Creating a GitHub Release

```bash
VERSION="0.6.1"

# Create draft release
~/.local/bin/gh release create "v${VERSION}" \
  -t "v${VERSION} — <title>" -d -F /path/to/notes.md

# Upload artifacts from Windows build dir
~/.local/bin/gh release upload "v${VERSION}" \
  "/mnt/c/wsl-porthole-build/src-tauri/target/release/bundle/nsis/WSL PortHole_${VERSION}_x64-setup.exe" \
  "/mnt/c/wsl-porthole-build/src-tauri/target/release/bundle/msi/WSL PortHole_${VERSION}_x64_en-US.msi" \
  "/mnt/c/wsl-porthole-build/target/release/wsl-porthole-service.exe"

# Publish (remove draft flag)
~/.local/bin/gh release edit "v${VERSION}" --draft=false
```

## Full Release Checklist

1. Bump version in: `package.json`, `src-tauri/tauri.conf.json`, `src-tauri/Cargo.toml`, `crates/*/Cargo.toml`
2. Update `CLAUDE.md` version history
3. Commit, tag (`git tag v<ver>`), push (`git push origin main --tags`)
4. Create GitHub release draft: `gh release create v<ver> -d ...`
5. Sync to Windows: `rsync ...` (see above)
6. Build on Windows: `npm install && npx tauri build && cargo build --release -p wsl-porthole-service`
7. Upload artifacts: `gh release upload v<ver> <files...>`
8. Publish: `gh release edit v<ver> --draft=false`

## Troubleshooting

| Error | Fix |
|---|---|
| `linker link.exe not found` | Install VS Build Tools with C++ workload, use `vcvars64.bat` |
| `rustc not recognized` | Rust MSI installs to `C:\Program Files\Rust stable MSVC X.XX\bin` — add to PATH |
| `makensis not found` | Install NSIS via winget, add `C:\Program Files (x86)\NSIS` to PATH |
| `frontendDist "../dist" doesn't exist` | Run `npm run build` first (or let `npx tauri build` handle it via `beforeBuildCommand`) |
