# Steam Release Plan

## Business & Operations

### Steamworks Partner Account
- Go to partner.steamgames.com
- Pay the $100 Steam Direct fee (refundable after $1,000 revenue)
- Complete tax/banking paperwork (W-9 or W-8BEN, bank account for payouts)

### Create the App & Get an App ID
- ~~In the Steamworks dashboard, create a new application~~
- ~~You'll receive an App ID (e.g., 2345678) — needed for all code integration~~
- **DONE** — App ID is `4550880`

### Store Page Setup
- Title: Court Wizard
- Short description (< 300 chars)
- Detailed description (HTML supported)
- Screenshots: at least 5, 1920x1080 recommended
- Genre tags: Strategy, Real-Time Strategy, Tower Defense, Fantasy
- System requirements: Windows 10+, Linux
- Content rating questionnaire

### Store Page Assets

| Asset              | Size          | Required             |
|--------------------|---------------|----------------------|
| Header Capsule     | 460x215       | Yes                  |
| Small Capsule      | 231x87        | Yes                  |
| Main Capsule       | 616x353       | Yes                  |
| Hero Capsule       | 374x448       | Yes                  |
| Library Capsule    | 600x900       | Yes                  |
| Library Hero       | 3840x1240     | Yes                  |
| Community Icon     | 32x32         | Yes                  |
| Client Icon        | 16x16–256x256 ICO | Yes             |
| Screenshots (5+)   | 1920x1080     | Yes                  |
| Trailer            | 1920x1080 MP4 | Strongly recommended |
| Page Background    | 1438x810      | Optional             |

### Steamworks Feature Configuration
- Achievements: Define all 45 existing achievements with icons (64x64 locked + unlocked)
- Cloud Saves: Enable and set quota (~1MB is plenty)
- Stats (optional): Track total games played, levels completed, etc.
- Rich Presence (optional): Show game state in friends list

### Build Depots
- Create depots for Windows and Linux
- Configure launch options for each platform

### Pricing
- Set price in Steamworks dashboard (regional pricing auto-suggested)

### Review & Release
- Submit for Steam review (1-5 business days)
- Steam checks store page, build runs, basic content
- After approval, set release date and publish

---

## Technical Integration

### Dependencies
- ~~Add `steamworks = "0.12"` to Cargo.toml~~
- ~~Check if `bevy-steamworks` supports Bevy 0.18.1~~
- **DONE** — Using `bevy-steamworks = "0.15"` in `Cargo.toml`

### New Files

All created and working:

| File                       | Purpose                                           | Status |
|----------------------------|---------------------------------------------------|--------|
| `src/steam/mod.rs`         | Module definition, re-export SteamPlugin           | DONE   |
| `src/steam/plugin.rs`      | Plugin registration, graceful init/fallback         | DONE   |
| `src/steam/achievements.rs`| Listens for AchievementUnlockedMessage, syncs to Steam | DONE |
| `src/steam/cloud_save.rs`  | Steam Cloud restore on startup, sync on state changes | DONE |
| `src/steam/constants.rs`   | App ID (4550880), 45 achievement API name mappings  | DONE   |
| `steam_appid.txt`          | Dev-only file (gitignored) with App ID             | DONE   |

### Files Modified

| File                                    | Change                                              | Status |
|-----------------------------------------|-----------------------------------------------------|--------|
| `Cargo.toml`                            | Added `bevy-steamworks = "0.15"`                     | DONE   |
| `src/main.rs`                           | Added `mod steam;` and `SteamPlugin` (before DefaultPlugins) | DONE |
| `.gitignore`                            | Added `steam_appid.txt`                              | DONE   |
| `build_native.sh`                       | Copies steam_api64.dll / libsteam_api.so alongside binary | DONE |
| `.github/workflows/release.yml`         | Add Steam depot upload step via steamcmd             | DONE   |
| `.github/workflows/build.yml`           | Reusable matrix build for Windows/Linux/macOS         | DONE   |
| `steam/app_build_4550880.vdf` + depots  | Steamworks build/depot scripts (placeholder IDs)      | DONE   |
| `steam/README.md`                       | One-time setup instructions for depot IDs + secrets   | DONE   |

### Steam Plugin Architecture

**DONE** — Implemented as described:

- `SteamworksPlugin::init_app(APP_ID)` called at startup
- On success: registers achievement sync, cloud save restore/sync systems
- On failure: logs warning, game continues without Steam features
- `bevy-steamworks` handles `run_callbacks()` automatically

### Achievement Syncing

**DONE** — All 45 `AchievementId` variants mapped to `ACH_*` Steam API name strings in `src/steam/constants.rs`. The `sync_achievements_to_steam` system listens for `AchievementUnlockedMessage` during `InGame` state, checks if already unlocked on Steam, sets + stores stats.

Each achievement still needs a 64x64 unlocked icon and a 64x64 locked icon (grayscale/dimmed). These are uploaded through the Steamworks dashboard, not bundled with the game.

### Steam Cloud Saves

**DONE** — Implemented with a restore-on-startup + sync-on-state-change approach:

- `restore_save_from_steam_cloud` runs at `Startup` — pulls cloud save to local only if no local save exists (new device scenario)
- `sync_save_to_steam_cloud` runs on `OnEnter(MainMenu)` and `OnEnter(MetaGame)` — writes local save to Steam Cloud
- Uses `saves_v2.json` (shared constant with `config::storage`)
- Graceful: checks `is_cloud_enabled_for_account()` and `is_cloud_enabled_for_app()` before any operation
- Local filesystem save system unchanged — saves still work without Steam

### Build Pipeline Updates

**DONE** — `build_native.sh` finds and copies Steam redistributable DLLs/SOs from the `steamworks-sys` build output. CI now builds Windows/Linux/macOS via `.github/workflows/build.yml` and uploads to Steamworks via `steamcmd` from the `steam-deploy` job in `release.yml`. Builds publish to the `staging` Steam branch; promote manually. See `steam/README.md` for the one-time depot ID + secret setup.

### Steam Overlay

Bevy + Steam Overlay generally works out of the box on native. Test that:
- Shift+Tab opens the overlay
- The overlay doesn't break input handling
- The CRT shader doesn't interfere with overlay rendering

---

## Priority Order

1. ~~Create Steamworks account & App ID~~ — **DONE** (App ID: 4550880)
2. ~~Add steamworks crate + SteamPlugin with graceful fallback~~ — **DONE**
3. ~~Wire up achievement syncing~~ — **DONE** (45 achievements mapped)
4. ~~Add Steam Cloud saves~~ — **DONE**
5. ~~Update build script for Steam DLLs~~ — **DONE**
6. ~~Update CI/CD for depot uploads~~ — **DONE** (gated on `STEAM_DEPLOY` repo variable until depot IDs + secrets are filled in)
7. Create store page assets (can be done in parallel)
8. Create achievement icons (64x64 locked + unlocked for each of 45 achievements)
9. Test Steam Overlay compatibility
10. Submit for review
