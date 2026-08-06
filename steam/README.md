# Steamworks Depot Upload

Builds reach Steamworks (App ID `4550880`) from **GitHub Actions**, gated on the `STEAM_DEPLOY` repo variable. `scripts/upload_to_steam.sh` still exists as a local fallback for when CI can't do it.

## The flow

```
/game-release            → push to dev  → dev-release.yml  → Steam `staging`
   (play-test staging)
/game-release main       → push to main → release.yml       → Steam `default` (pending phone approval)
```

**`dev-release.yml`** (any push to `dev` touching `docs/CHANGELOG.md`) runs the test/clippy/fmt gate, builds Windows + Linux + the signed universal macOS `.app`, and uploads **all three depots in one steamcmd invocation** — deliberately, so a release is a single Steam build id.

**`release.yml`** (push to `main`) builds nothing. It locates the `dev-release.yml` run for that exact commit, reuses its zips for the tag and GitHub Release, and calls `ISteamApps/SetAppBuildLive` to set *that same build* live on the default branch. Because the release skill fast-forwards `main` onto the built dev commit, the SHAs match, and players get the exact bits that were on `staging`.

### Promotion needs your phone

Setting a build live on the **default** branch of a released app always sends an authorization prompt to the Steam Mobile app — there is no opt-out (Valve, Oct 2023). So `SetAppBuildLive` returns **HTTP 201 = pending confirmation** as its normal response. CI picks the correct build id and requests the promotion; you approve on your phone. The API notes:

- the default branch's `betakey` is **`public`**, not `default`;
- `steamid` is required whenever a released app's `betakey` is `public`.

Required secrets: `STEAM_PUBLISHER_API_KEY` (publisher Web API key with *Edit App* + *Publish*) and `STEAM_PUBLISH_STEAMID` (the SteamID64 that receives the prompt). Rollback stays a manual dashboard action — it would need the same confirmation anyway.

### Finding the build

CI stamps `r<github_run_id>` into the Steam build description, and promotion matches on that. It is deliberately not the version or the commit SHA: a re-run produces a second build with an identical version and SHA, and silently setting the older one live would be a shipping bug. Note `scripts/upload_to_steam.sh` writes a *short* SHA and no run id, so a locally-uploaded build will never match the selector — promote those by hand.

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

Normally you don't — `/game-release main` requests it and you approve the Steam Mobile prompt (see *The flow* above).

By hand, for a locally-uploaded build or a rollback: Steamworks dashboard → App `4550880` → Builds → select the build → set live on `default`. Same phone confirmation applies.

## Files in this directory

- `app_build_4550880.vdf` — template; the upload script substitutes `$VERSION`, `$SHA`, `$CONTENTROOT`, `$BUILDOUTPUT`, `$DEPOT_LINES`.
- `depot_windows.vdf`, `depot_linux.vdf`, `depot_macos.vdf` — per-depot file mappings.
- `app_build_4550880.generated.vdf` — produced by the upload script (gitignored).

## Prerequisites

- `steamcmd` on PATH (`brew install steamcmd` on macOS, or download from Valve on Linux).
- For WSL2 cross-compiles, the toolchains are installed via `rustup target add x86_64-pc-windows-gnu x86_64-unknown-linux-gnu` plus `gcc-mingw-w64-x86-64` for the Windows linker.
- For local (unsigned) macOS bundles: a Mac with `cargo` and both Apple targets
  (`rustup target add aarch64-apple-darwin x86_64-apple-darwin`). Signed builds come only
  from the `macos-release.yml` workflow, which needs the `APPLE_CERT_P12_BASE64`,
  `APPLE_CERT_PASSWORD`, and `APPSTORE_API_KEY_JSON` repo secrets.
