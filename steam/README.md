# Steamworks Depot Upload

CI uploads a build to Steamworks (App ID `4550880`) on every tagged release using the `game-ci/steam-deploy@v3` action. Builds publish to the **`staging`** Steam branch — promote to `default` (live) manually from the Steamworks dashboard when ready.

The action generates the `app_build` and depot VDF files internally, so this directory only contains setup notes.

## Depot layout

| Depot ID | Platform              | Stage path                         |
|----------|-----------------------|------------------------------------|
| 4550882  | Windows (x86_64)      | `steam-content/windows/court_wizard/` |
| 4550883  | Linux (x86_64)        | `steam-content/linux/court_wizard/`   |
| 4550884  | macOS (Apple Silicon) | `steam-content/macos/court_wizard/`   |

`firstDepotIdOverride: 4550882` is set on the action so depot1/2/3 map to those IDs in order.

## One-time setup

Until these steps are done, the `steam-deploy` job in `.github/workflows/release.yml` is gated off by the `STEAM_DEPLOY` repo variable.

### 1. Builder account

On the Steamworks partner site, create a dedicated builder sub-account (e.g. via a "robots" group) with permission scoped to App `4550880` only. Disable the Steam Mobile Authenticator on this account — CI auth requires email-based Steam Guard so the sentry inside `config.vdf` is portable. Mobile authenticator forces per-login confirmation that can't be automated.

### 2. Generate `config.vdf` locally

```bash
rm -f ~/Steam/config/config.vdf
steamcmd +login <builder_username> +quit
# enter password, then the email Steam Guard code
steamcmd +login <builder_username> +quit
# second run should NOT prompt — sentry was saved
base64 -w 0 ~/Steam/config/config.vdf
```

### 3. Add GitHub secrets and variable

Repo → Settings → Secrets and variables → Actions:

- Secret `STEAM_USER` — builder username
- Secret `STEAM_CONFIG_VDF` — base64 output from step 2
- Variable `STEAM_DEPLOY` — `true` to enable the deploy job

(No `STEAM_PASS` needed — the action authenticates via the sentry in `configVdf`.)

## What CI does on each release

1. `build` job produces three platform zips via `package.sh`.
2. `release` job tags the commit, creates a GitHub Release, and attaches the zips.
3. `steam-deploy` job downloads the zips, unpacks each into `steam-content/<platform>/`, and invokes `game-ci/steam-deploy@v3` to push the build to the `staging` branch.

## Promoting staging to live

Steamworks dashboard → App `4550880` → Builds → select the new build → set live on `default`.
