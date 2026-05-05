# Steamworks Depot Upload

CI uploads a build to Steamworks (App ID `4550880`) on every tagged release. The build is published to the **`staging`** Steam branch — promote to `default` (live) manually from the Steamworks dashboard when ready.

## One-time setup

Until these steps are done, the `steam-deploy` job in `.github/workflows/release.yml` is gated off by the `STEAM_DEPLOY` repo variable.

### 1. Create the depots in Steamworks

In the Steamworks partner dashboard, under App `4550880`, create three depots:

- Windows (x86_64)
- Linux (x86_64)
- macOS (Apple Silicon)

Note the numeric depot IDs Steam assigns.

### 2. Fill in the depot IDs

Replace the `WIN_DEPOT_ID`, `LINUX_DEPOT_ID`, `MAC_DEPOT_ID` placeholders in:

- `steam/app_build_4550880.vdf`
- `steam/depot_windows.vdf`
- `steam/depot_linux.vdf`
- `steam/depot_macos.vdf`

### 3. Create a builder sub-account

On the Steamworks partner site, create a dedicated builder sub-account with permission scoped to App `4550880` only. Do **not** use your primary partner account.

### 4. Generate `config.vdf` locally (Steam Guard)

GitHub Actions can't complete a Steam Guard email prompt, so we hand it a pre-authorized sentry file:

```bash
mkdir -p $HOME/Steam/config
steamcmd +login <builder_username> +quit
# Steam Guard will email a code; enter it.
# Re-run once more to confirm the sentry was saved without prompting:
steamcmd +login <builder_username> +quit

# Encode the resulting sentry file for GitHub Secrets:
base64 -w 0 ~/Steam/config/config.vdf
```

### 5. Add GitHub secrets and variable

Repo → Settings → Secrets and variables → Actions:

- Secret `STEAM_USER` — builder username
- Secret `STEAM_PASS` — builder password
- Secret `STEAM_CONFIG_VDF` — base64 output from step 4
- Variable `STEAM_DEPLOY` — `true` to enable the deploy job

## What CI does on each release

1. `build` job produces three platform zips via `package.sh`.
2. `release` job tags the commit, creates a GitHub Release, and attaches the zips.
3. `steam-deploy` job downloads the zips, unpacks each into `steam-content/<platform>/`, restores the Steam Guard sentry, rewrites `$VERSION` / `$SHA` in `app_build_4550880.vdf`, and runs `steamcmd +run_app_build` to push the build to the `staging` branch.

## Promoting staging to live

Steamworks dashboard → App `4550880` → Builds → select the new build → set live on `default`.
