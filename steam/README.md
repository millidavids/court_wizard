# Steamworks Depot Upload

Builds are uploaded to Steamworks (App ID `4550880`) **manually from a local machine** using `scripts/upload_to_steam.sh`. Builds publish to the **`staging`** Steam branch — promote to `default` (live) manually from the Steamworks dashboard when ready.

The CI-based upload step (`game-ci/steam-deploy@v3` in `release.yml`) still exists but is gated off via the `STEAM_DEPLOY` repo variable. We hit two issues with the automated flow that made the local approach simpler for now:

1. Steam's anti-fraud locks builder sub-accounts after repeated logins from different GitHub-runner IPs (the configVdf sentry mechanism is fragile against IP variation).
2. Mobile-authenticator accounts can't be used headlessly.

If we ever want to revive CI-side uploads, switch to TOTP-per-release (a `workflow_dispatch` input that takes a fresh email Steam Guard code each time).

## Depot layout

| Depot ID | Platform                             | Stage path                            |
|----------|--------------------------------------|---------------------------------------|
| 4550882  | Windows (x86_64)                     | `steam-content/windows/court_wizard/` |
| 4550883  | Linux (x86_64)                       | `steam-content/linux/court_wizard/`   |
| 4550881  | macOS (universal, signed `.app`)     | `steam-content/macos/court_wizard/`   |

Depot `4550884` (the old unsigned Apple Silicon build) is retired — do not upload to it.
The macOS depot content is `Court Wizard.app` (plus `README.txt` and the install-root
`controller_config/` mirror), produced as a signed/notarized universal bundle.

## How to release

`scripts/upload_to_steam.sh` uploads whichever platform zips it finds at the repo root for the current `Cargo.toml` version. You can run it from any host that has those zips and a working `steamcmd`. Two typical patterns:

**macOS is special:** the shipped macOS build must be the **signed, notarized** universal
`Court Wizard.app` produced by the `macos-release.yml` GitHub Actions workflow. release.yml
calls it as a nested job (`macos-release`) once per release, built off main alongside the
Windows/Linux legs; it can also be run by hand via *Actions → macOS Release → Run workflow*.
If the CI Steam upload fails, download its `court_wizard-v<version>-macos-universal.zip`
artifact (also attached to the GitHub release) and upload manually. A locally built
`./scripts/package.sh macos` zip has the same layout but is **UNSIGNED** — fine for local
testing, not for shipping.

**Pre-ship smoke test (first signed .app on real hardware):** the bundle's Info.plist declares
`NSHighResolutionCapable`, which the old bare binary never did — on a Retina Mac, verify
sharpness, cursor alignment under the CRT barrel effect, and framerate before promoting to
`default`, and confirm the Steam overlay (Shift+Tab) works (the entitlements in
`packaging/macos/entitlements.plist` exist exactly for that).

### Option A — All three platforms from one machine (requires the zips to all be present)

If you've gathered all three zips on the same machine (e.g. WSL2 with the signed macOS zip
downloaded from the workflow artifact):

```bash
./scripts/package.sh windows
./scripts/package.sh linux
# download court_wizard-v<version>-macos-universal.zip from the macos-release.yml run
./scripts/upload_to_steam.sh <your_steam_username>
```

### Option B — Upload each platform from its own host

On WSL2 (cross-compiles Windows + Linux):

```bash
./scripts/package.sh windows
./scripts/package.sh linux
./scripts/upload_to_steam.sh <your_steam_username>
```

From any machine with the signed macOS artifact zip at the repo root:

```bash
./scripts/upload_to_steam.sh <your_steam_username>
```

Each invocation uploads only the depots whose zips are present, so this works fine — Steam tracks them all under the same App ID and branch. Approve the Steam Guard prompt on your phone if asked. The first run on a given machine will trigger Steam Guard; subsequent runs reuse the cached session.

### Promote to live

Steamworks dashboard → App `4550880` → Builds → select the new build → set live on `default`.

## Files in this directory

- `app_build_4550880.vdf` — template; the upload script substitutes `$VERSION`, `$SHA`, `$CONTENTROOT`, `$BUILDOUTPUT`, `$DEPOT_LINES`.
- `depot_windows.vdf`, `depot_linux.vdf`, `depot_macos.vdf` — per-depot file mappings.
- `app_build_4550880.generated.vdf` — produced by the upload script (gitignored).

## Playtest (app 4820340)

The CI playtest deploy job was removed from `release.yml` — the Playtest is disabled in
Steamworks. If it ever comes back, note its depot IDs were 4820342 (windows), 4820343 (macOS),
4820344 (linux), assigned by `firstDepotIdOverride` increment in that order.

## Prerequisites

- `steamcmd` on PATH (`brew install steamcmd` on macOS, or download from Valve on Linux).
- For WSL2 cross-compiles, the toolchains are installed via `rustup target add x86_64-pc-windows-gnu x86_64-unknown-linux-gnu` plus `gcc-mingw-w64-x86-64` for the Windows linker.
- For local (unsigned) macOS bundles: a Mac with `cargo` and both Apple targets
  (`rustup target add aarch64-apple-darwin x86_64-apple-darwin`). Signed builds come only
  from the `macos-release.yml` workflow, which needs the `APPLE_CERT_P12_BASE64`,
  `APPLE_CERT_PASSWORD`, and `APPSTORE_API_KEY_JSON` repo secrets.
