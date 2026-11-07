# BlockMC

Modern Minecraft Java launcher. Tauri 2 + Rust + React/TypeScript. License: GPL-3.0-or-later
(compatible with reusing PolyMC's GPL-3.0 code as reference).

## Run
    npm install
    npm run tauri dev

## Test
    cargo test -p blockmc-core

## Layout
- crates/blockmc-core  platform-independent logic (instances now; installs, Java, launch later)
- src-tauri            thin Tauri command layer
- src                  React UI, design tokens in src/styles/tokens.css

## Releases (Linux and Windows)
Push a tag like `v0.1.0`; `.github/workflows/release.yml` builds a draft GitHub release with a Linux
.deb/.AppImage and a Windows installer (.msi/.exe).

## Status: beta (mod manager)
Working: instances, Modrinth search for mods / resource packs / shader packs filtered to the instance's
Minecraft version and loader, install with required dependencies, SHA-512 verified downloads from the
Modrinth CDN only, remove, update check and update, open instance folder.
Not built: downloading or launching Minecraft, accounts, Java, CurseForge, modpacks, website, backups.
Point your launcher (or the game's own folder setting) at an instance's `minecraft` folder to use the files.
