# Changelog

## 1.0.1
- Required dependencies are now installed with every mod, including dependencies that name an exact version, and the install message lists them.
- New "Fix missing dependencies" button scans installed mods and installs anything they need (for example Fabric API).
- Installing a mod that declares itself incompatible with an installed mod is blocked with a clear message.
- A missing dependency error now names the dependency instead of showing an id.

## 1.0.0
- Instances: create with the full Minecraft version list (newest to 1.8) and a loader picker; rename, duplicate, delete, open folder.
- Modrinth mods, resource packs and shader packs: search with sorting, paging and page size; install with required dependencies; update; remove with a warning.
- Discover page for modpacks with sorting, version and loader filters and numbered pages; one-click .mrpack install into a new instance.
- Downloads are SHA-512 verified and only accepted from trusted hosts.
- Light and dark themes, settings page.
- Ubuntu (.deb, .AppImage) and Windows (.msi, .exe) installers built by GitHub Actions.
