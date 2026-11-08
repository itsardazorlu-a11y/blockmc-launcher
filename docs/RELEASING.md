# Releasing BlockMC

1. Make sure everything passes: `./scripts/check.sh` (Linux) or `scripts\check.ps1` (Windows). Run through docs/TESTING.md.
2. Set the version in `package.json`, `src-tauri/tauri.conf.json` and `src-tauri/Cargo.toml` (same number in all three) and add a line to CHANGELOG.md.
3. Commit and push to `main`. Wait for the green check on the Actions tab (workflow `ci`).
4. Create the tag. It must start with `v` and match the version:
       git tag v1.0.1
       git push origin v1.0.1
5. The `release` workflow builds the Ubuntu `.deb` and `.AppImage` and the Windows `.msi` and `-setup.exe` and attaches them to a DRAFT release (about 15-25 minutes).
6. Download the installers from the draft and test them (docs/TESTING.md, section 4).
7. On GitHub open Releases, open the draft, edit the notes, and press "Publish release".

If a build fails: open the failed run, expand the red step, fix the code, commit, then move the tag:
    git tag -d v1.0.1
    git push origin :refs/tags/v1.0.1
    git tag v1.0.1
    git push origin v1.0.1
