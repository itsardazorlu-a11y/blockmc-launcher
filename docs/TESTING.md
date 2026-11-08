# Testing BlockMC before a release

## 1. Automated checks (2 minutes after the first build)
Linux: `./scripts/check.sh`   Windows: `powershell -ExecutionPolicy Bypass -File scripts\check.ps1`
This runs the TypeScript check, the Rust unit tests, the frontend build and a full Rust compile of the app.

## 2. Run the app
`npm run tauri dev` (first build takes several minutes).

## 3. Manual checklist (needs internet)
Instances
- [ ] New instance: the version list loads, the newest version is selected, typing in the filter narrows it, the loader buttons switch, Create is disabled with an empty name.
- [ ] Create a Fabric 1.20.1 instance. It appears in the Library. Rename it. Duplicate it. Delete the copy (a confirm dialog appears).
- [ ] Open folder opens your file manager on `.../instances/<name>/minecraft`.

Mods
- [ ] Open the instance, Mods tab: results load. Change sort to "Download count": the order changes. Go to page 2 and page 3: results change. Change "Show" to 50.
- [ ] Install "Fabric API". The Installed tab lists it and the jar exists in the folder `mods`.
- [ ] Install a mod that needs Fabric API (for example Sodium or Lithium): it installs, and the message mentions dependencies if any were added.
- [ ] Remove Fabric API while another mod needs it: a confirm dialog warns you. Cancel keeps it.
- [ ] Check for updates works (it may say everything is up to date).
- [ ] Resource packs and Shader packs tabs install a .zip into `resourcepacks` / `shaderpacks`.
- [ ] Vanilla instance: the Mods tab shows a clear message that vanilla cannot use mods.

Modpacks
- [ ] Discover: results show, sort and the version / loader filters work, numbered pages work.
- [ ] Install a small modpack. A new instance appears in the Library containing the pack's mods and configs.
- [ ] Turn off your internet and install another pack: you get an error and no half-installed instance is left in the Library.

Safety
- [ ] No tokens or personal data are stored: the app has no login at all.
- [ ] Mods in one instance never appear in another.

Look and feel
- [ ] Light and dark themes both look right (Settings). Dropdowns are readable in both.
- [ ] Resize the window to its smallest size: nothing overlaps.

## 4. Test the installer itself
Download the installers from the draft GitHub release and install them on a clean machine or VM:
- Ubuntu: `sudo apt install ./BlockMC_1.0.0_amd64.deb` then start "BlockMC" from the menu.
- Windows: run the `.msi` or `-setup.exe`. Windows SmartScreen warns about unsigned apps: choose "More info" then "Run anyway".
Repeat the first three items of the manual checklist. Only then publish the release.
