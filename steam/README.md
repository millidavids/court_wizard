# Steamworks Depot Upload

Builds reach Steamworks (App ID `4550880`) from **GitHub Actions**, gated on the `STEAM_DEPLOY` repo variable. `scripts/upload_to_steam.sh` still exists as a local fallback for when CI can't do it.

## The flow

```
/game-release            → push to dev  → dev-release.yml  → Steam `staging`
   (play-test staging)
/game-release main       → push to main → release.yml       → tag + GitHub Release
                                        → steam-promote.yml → Steam `default` (pending phone approval)
                                                            → Discord + Bluesky + Steam patch notes
```

**`dev-release.yml`** (any push to `dev` touching `docs/CHANGELOG.md`) runs the test/clippy/fmt gate, builds Windows + Linux + the signed universal macOS `.app`, and uploads **all three depots in one steamcmd invocation** — deliberately, so a release is a single Steam build id.

**`release.yml`** (push to `main`) builds nothing. It locates the `dev-release.yml` run for that exact commit and reuses its zips for the tag and GitHub Release. Because the release skill fast-forwards `main` onto the built dev commit, the SHAs match, and players get the exact bits that were on `staging`.

**`steam-promote.yml`** (`cron: '23 */6 * * *'`) owns everything after that, because all of it waits on a human tapping a phone at an unpredictable hour. Each pass advances the release one state:

```
not requested yet   -> ask Steam to set the build live on `public`
requested, not live -> wait quietly (you have not tapped yet)
live, not announced -> announce, exactly once per channel
announced           -> nothing to do
```

Three annotated git tags hold that state on the remote, so it survives across runs and cannot repeat:

| Tag | Meaning |
|---|---|
| `promoted/v<version>` | Steam accepted the promotion request; the phone prompt went out |
| `announced/v<version>` | live on `default`, and Discord + Bluesky were told |
| `steam-announced/v<version>` | the patch-notes event is up on the Steam hub |

Steam has its own marker because it is the one channel built on an undocumented endpoint, so it is the one that will eventually break. Sharing a marker would force a bad trade: either Steam could never retry (the tag is written the moment Discord succeeds), or a Steam failure would hold the tag back and Discord would re-announce every six hours. Separate markers let Steam retry until it works while the other two stay exactly-once.

"Is it live" is read from `ISteamApps/GetAppBetas` → `.response.betas.public.BuildID`, the only endpoint that reports branch assignment. Announcing on the promotion *request* would be wrong: HTTP 201 means "confirmation sent", not "players have it".

### Promotion needs your phone

Setting a build live on the **default** branch of a released app always sends an authorization prompt to the Steam Mobile app — there is no opt-out (Valve, Oct 2023). So `SetAppBuildLive` returns **HTTP 201 = pending confirmation** as its normal response. CI picks the correct build id and requests the promotion; you approve on your phone. The API notes:

- the default branch's `betakey` is **`public`**, not `default`;
- `steamid` is required whenever a released app's `betakey` is `public`.

Required secrets: `STEAM_PUBLISHER_API_KEY` (publisher Web API key with *Edit App* + *Publish*) and `STEAM_PUBLISH_STEAMID` (the SteamID64 that receives the prompt). Rollback stays a manual dashboard action — it would need the same confirmation anyway.

### Finding the build

CI stamps `r<github_run_id>` into the Steam build description, and promotion matches on that. It is deliberately not the version or the commit SHA: a re-run produces a second build with an identical version and SHA, and silently setting the older one live would be a shipping bug. Note `scripts/upload_to_steam.sh` writes a *short* SHA and no run id, so a locally-uploaded build will never match the selector — promote those by hand.

## Announcing the release

Once the build is live on `default`, `steam-promote.yml` tells three places. Each renders from the **same** changelog block, so they cannot drift:

| Channel | Content | Limit |
|---|---|---|
| Discord | the version's whole changelog block, as markdown | trimmed to 4000 chars |
| Bluesky | the `### Description` prose only | 300 graphemes, hard-fails over |
| Steam hub | the whole block as BBCode, prose leading | no observed cap; 6,977 chars posted fine |

`scripts/changelog_to_bbcode.sh` owns the BBCode. With no arguments it prints the whole top section — that form is what `release.yml` publishes as the `steam-announcement` artifact, and is the paste-by-hand fallback, so **do not change it**. The flags are for the automatic post:

```bash
scripts/changelog_to_bbcode.sh --version 1.0.38 --headline     # v1.0.38 - 2026-08-15
scripts/changelog_to_bbcode.sh --version 1.0.38 --description  # the ### Description prose
scripts/changelog_to_bbcode.sh --version 1.0.38 --body         # BBCode, prose leading, no [h2]
```

Selecting by `--version` rather than taking the top block matters for a `workflow_dispatch` naming an older release: `main`'s changelog has already moved on, and the top block would ship the wrong notes.

### There is no supported API for Steam events

Checked, and worth not re-checking: Valve's patch-notes docs describe the Hub Admin UI and a prompt shown after a SteamPipe upload — both need a browser. `ISteamNews` is read-only. `app_build.vdf` has no patch-notes key. The one documented automation around events points *outward* (Steam emits an RSS feed of your events; nothing ingests one).

So `scripts/steam_post_announcement.py` drives the same undocumented endpoint the event editor itself calls, with a logged-in web session. **Expect this to break** when Valve changes the editor. It is deliberately fenced: `continue-on-error`, its own marker tag, and the BBCode artifact still published for pasting by hand.

### The captured contract

Captured from the live editor on 2026-08-22. All four operations are one endpoint:

```
POST https://steamcommunity.com/gid/103582791475642999/ajaxcreateupdatedeletepartnerevents/
Cookie: sessionid=<hex>; steamLoginSecure=<steamid64>%7C%7C<access_token>
```

`103582791475642999` is the hub's clan id (`curl -sSL https://steamcommunity.com/games/<appid> | grep -o '/gid/[0-9]*'`). `sessionid` is a plain double-submit CSRF value — the same random hex in the cookie and the form body. **Never follow redirects**: a redirect turns the POST into a GET and silently drops the body.

| Operation | Flag | Fields |
|---|---|---|
| Create | `bCreate=1` | 19 — content, `event_type=12`, `tags=["patchnotes"]`, `hidden=true`, `published=false`, `jsondata` |
| Update | `bUpdate=1` | 22 — create plus `gid`, `announcement_gid`, `unlistedmode` |
| Publish | `bPublish=1` | 8 — `gid`, `announcement_gid`, visibility. **No content.** |
| Delete | `bDelete=1` | 2 — `gid` |

**Publishing is a separate request that carries no content.** Create saves a hidden, unpublished event; publish flips it live. Replaying only the create leaves a draft no feed will ever show — which the idempotency check could never see, so every run would make another one.

`event_type` is `12` (Small Update / Patch Notes); `13` is Regular Update, `14` is Major Update. `rtime32_visibility_end`, `build_id` and `build_branch` are sent as the literal string `undefined`. `jsondata` is a side blob whose localisation arrays are 32 slots wide, index 0 English; only `localized_subtitle` and `localized_summary` matter here.

Editor field limits: title 80, subtitle 120, summary 180. A changelog `### Description` runs ~200 characters, so it does **not** fit the subtitle — the subtitle is left empty, the prose opens the body, and a sentence-bounded prefix goes in the summary (which is what shows in event lists; leaving it blank makes Steam auto-generate one starting with a raw `[h3]Added[/h3]`).

Two side-effects worth knowing: publishing without artwork raises a warnings dialog and falls back to the game capsule, which is what every Court Wizard announcement already does; and **Steam moderates new posts**, so there can be a 1+ hour delay before one appears in the Steam library — and *any edit re-queues it*.

### The Steam session secret

`STEAM_WEB_REFRESH_TOKEN` is the most dangerous secret in this repo: a months-long web session for an account with publisher rights, on a public repository. Mint it with:

```bash
npm install --no-save steam-session qrcode-terminal
node scripts/steam_refresh_token.mjs      # scan the QR with the Steam Mobile app
```

It **must** be a `MobileApp`-platform token. `GenerateAccessTokenForApp` refuses `WebBrowser` tokens and has refused `SteamClient` ones since 2025-04-30 — and it reports that refusal as **HTTP 200** with an empty body and the real result in the `x-eresult` header (`15` = AccessDenied). A status-code check detects nothing and would go on to build `steamLoginSecure=<steamid>||`, a valid but entirely anonymous cookie that posts nothing. `steam_post_announcement.py` asserts on the header, on a non-empty token, and on JWT shape before using it.

Note also that Steam refresh tokens are IP-bound and GitHub runners get a fresh IP every run, so intermittent `AccessDenied` is expected. The 6-hourly cadence keeps that to a handful of attempts a day, and the `steam-announced/` marker stops them entirely once one succeeds.

To disable just this channel, delete the secret — the step is gated on it, so the workflow goes quiet rather than failing every six hours.

### Testing it without posting

```bash
python3 scripts/steam_post_announcement.py --version 1.0.38 --dry-run   # render + idempotency, no network writes
python3 scripts/steam_post_announcement.py --version 1.0.39 --draft     # create, do not publish; delete it in Steamworks
```

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
