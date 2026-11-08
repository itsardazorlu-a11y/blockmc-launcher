# BlockMC

Mod and modpack manager for Minecraft Java, powered by Modrinth. Tauri 2 + Rust + React/TypeScript.
Runs on Ubuntu/Debian Linux and Windows. License: GPL-3.0-or-later.
BlockMC does not download or launch Minecraft: it manages mods, resource packs, shaders and modpacks in isolated
instances. Point your launcher at an instance's `minecraft` folder (Open folder button) to use the files.

## Set up (once)
Ubuntu / Debian (22.04+): `./scripts/setup-ubuntu.sh`, then open a new terminal.
Windows: install (1) Node.js 20+, (2) Rust from https://rustup.rs, (3) "Desktop development with C++" from
Visual Studio Build Tools. Windows 10/11 already has WebView2.

## Run and test
    npm install
    npm run tauri dev          # start the app
    npm run check              # typecheck + Rust tests + build (or ./scripts/check.sh)
Full checklist: docs/TESTING.md

## Build installers locally
    npm run tauri build        # Ubuntu: .deb/.AppImage, Windows: .msi/.exe in src-tauri/target/release/bundle

## Release
See docs/RELEASING.md (push a tag like `v1.0.0`; GitHub Actions builds the installers).

## Layout
- crates/blockmc-core  platform-independent logic: instances, Modrinth client, modpacks, safe downloads (unit-tested)
- src-tauri            Tauri commands
- src                  React UI

Data lives in `~/.local/share/BlockMC` (Linux) or `%APPDATA%\BlockMC` (Windows).
