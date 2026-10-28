# Releasing ScoreCraft

> **Last reviewed:** 2026-10-28 · **Last updated:** 2026-10-28 · **Change:** major (first version, plan before the first release) · **Target:** enScore 3.7 (2026)

The release process follows the Crafting Apps' playbook: version bump on `main`, a `release`
branch that CI builds, signs and publishes, and checksums for every file.

## Cutting a release

1. `cargo xtask version <x.y.z>` bumps every crate, app metadata and `CHANGELOG.md`.
2. Update `ROADMAP.md`'s progress log and the parity docs' status lines if numbers moved.
3. Merge to `main`, then fast-forward `release` to that commit. `release.yml` builds every target.
4. Check the draft GitHub release: every artefact present, `SHA256SUMS.txt` matches, notes written
   for musicians (what changed, what to watch out for).
5. Publish. Mobile store submissions follow from the same commit.

## What gets built

| Platform | Artefacts | Signing |
|---|---|---|
| macOS | `scorecraft-<ver>-macos-universal.dmg`, CLI zip | Developer ID, notarized, stapled |
| Windows | x64 and arm64 `.msi` and portable `.zip` | Authenticode |
| Linux | AppImage (x86_64, aarch64), Flatpak bundle, `.deb`, `.rpm`, tarball | checksums; AppImage zsync |
| Android | `.apk` (sideload) and `.aab` (store) | release keystore |
| iOS / iPadOS | TestFlight / App Store build | Apple distribution certificate |
| Web | `scorecraft-web-<ver>.zip` static site | checksums |
| Sync server | static Linux binaries, OCI image | checksums, image digest |

## Secrets

Signing keys and store credentials live only in CI secrets of the `release` environment, never in
the repository or logs. Rotations are recorded in [`decisions.md`](decisions.md).

## Checks

- `cargo xtask ci` green on the release commit, corpus tests included.
- Installers install and launch on a clean VM per platform; the reader opens a public-domain score
  and turns a page.
- No feature flag enables network access by default.

## Revision history

| Date | Change | Summary |
|---|---|---|
| 2026-10-28 | major | First version |
