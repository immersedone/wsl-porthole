# WSL PortHole — Build & Release Instructions

## Prerequisites (Windows)

- **Rust** toolchain (`rustup` with `stable-x86_64-pc-windows-msvc`)
- **Node.js** 18+ and npm
- **Tauri CLI** (`cargo install tauri-cli`)
- **Visual Studio Build Tools** (C++ workload for MSVC linker)

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

## Build Steps (Windows PowerShell)

```powershell
cd C:\wsl-porthole-build

# Install frontend dependencies
npm install

# Build the Tauri app (NSIS + MSI installers)
npx tauri build
```

### Output locations

| Artifact | Path |
|---|---|
| NSIS installer (.exe) | `src-tauri/target/release/bundle/nsis/WSL PortHole_<ver>_x64-setup.exe` |
| MSI installer (.msi) | `src-tauri/target/release/bundle/msi/WSL PortHole_<ver>_x64_en-US.msi` |
| Service binary (.exe) | Build separately: `cargo build --release -p wsl-porthole-service` → `target/release/wsl-porthole-service.exe` |

## Creating a GitHub Release

After building on Windows, upload the artifacts:

```bash
# From WSL — copy built artifacts to an accessible location
VERSION="0.4.0"

# Upload to existing GitHub release
gh release upload "v${VERSION}" \
  "/mnt/c/wsl-porthole-build/src-tauri/target/release/bundle/nsis/WSL PortHole_${VERSION}_x64-setup.exe" \
  "/mnt/c/wsl-porthole-build/src-tauri/target/release/bundle/msi/WSL PortHole_${VERSION}_x64_en-US.msi" \
  "/mnt/c/wsl-porthole-build/target/release/wsl-porthole-service.exe"
```

## Full Release Checklist

1. Bump version in: `package.json`, `src-tauri/tauri.conf.json`, `src-tauri/Cargo.toml`, `crates/*/Cargo.toml`
2. Update `CLAUDE.md` version history
3. Commit, tag (`git tag v<ver>`), push (`git push origin main --tags`)
4. Create GitHub release: `gh release create v<ver> --title "v<ver> — <title>" --notes "..."`
5. Sync to Windows: `rsync ...` (see above)
6. Build on Windows: `npm install && npx tauri build && cargo build --release -p wsl-porthole-service`
7. Upload artifacts: `gh release upload v<ver> <files...>`
