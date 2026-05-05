# Steamworks Depot Upload

Builds are uploaded to Steamworks (App ID `4550880`) **manually from a local machine** using `scripts/upload_to_steam.sh`. Builds publish to the **`staging`** Steam branch — promote to `default` (live) manually from the Steamworks dashboard when ready.

The CI-based upload step (`game-ci/steam-deploy@v3` in `release.yml`) still exists but is gated off via the `STEAM_DEPLOY` repo variable. We hit two issues with the automated flow that made the local approach simpler for now:

1. Steam's anti-fraud locks builder sub-accounts after repeated logins from different GitHub-runner IPs (the configVdf sentry mechanism is fragile against IP variation).
2. Mobile-authenticator accounts can't be used headlessly.

If we ever want to revive CI-side uploads, switch to TOTP-per-release (a `workflow_dispatch` input that takes a fresh email Steam Guard code each time).

## Depot layout

| Depot ID | Platform              | Stage path                            |
|----------|-----------------------|---------------------------------------|
| 4550882  | Windows (x86_64)      | `steam-content/windows/court_wizard/` |
| 4550883  | Linux (x86_64)        | `steam-content/linux/court_wizard/`   |
| 4550884  | macOS (Apple Silicon) | `steam-content/macos/court_wizard/`   |

## How to release

`scripts/upload_to_steam.sh` uploads whichever platform zips it finds at the repo root for the current `Cargo.toml` version. You can run it from any host that has those zips and a working `steamcmd`. Two typical patterns:

### Option A — All three platforms from one machine (requires the zips to all be present)

If you've gathered all three zips on the same machine (e.g. WSL2 with the macOS zip copied over):

```bash
./scripts/package.sh windows
./scripts/package.sh linux
# bring court_wizard-v<version>-macos-apple-silicon.zip over from the Mac
./scripts/upload_to_steam.sh <your_steam_username>
```

### Option B — Upload each platform from its own host

On WSL2 (cross-compiles Windows + Linux):

```bash
./scripts/package.sh windows
./scripts/package.sh linux
./scripts/upload_to_steam.sh <your_steam_username>
```

On the MacBook Pro (builds macOS natively):

```bash
./scripts/package.sh macos
./scripts/upload_to_steam.sh <your_steam_username>
```

Each invocation uploads only the depots whose zips are present, so this works fine — Steam tracks them all under the same App ID and branch. Approve the Steam Guard prompt on your phone if asked. The first run on a given machine will trigger Steam Guard; subsequent runs reuse the cached session.

### Promote to live

Steamworks dashboard → App `4550880` → Builds → select the new build → set live on `default`.

## Files in this directory

- `app_build_4550880.vdf` — template; the upload script substitutes `$VERSION`, `$SHA`, `$CONTENTROOT`, `$BUILDOUTPUT`, `$DEPOT_LINES`.
- `depot_windows.vdf`, `depot_linux.vdf`, `depot_macos.vdf` — per-depot file mappings.
- `app_build_4550880.generated.vdf` — produced by the upload script (gitignored).

## Prerequisites

- `steamcmd` on PATH (`brew install steamcmd` on macOS, or download from Valve on Linux).
- For WSL2 cross-compiles, the toolchains are installed via `rustup target add x86_64-pc-windows-gnu x86_64-unknown-linux-gnu` plus `gcc-mingw-w64-x86-64` for the Windows linker.
- For macOS builds: just a Mac with `cargo` and the `aarch64-apple-darwin` target.
