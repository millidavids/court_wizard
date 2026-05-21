---
name: game-release
description: Aggregate player-facing notes on the dev branch (default), consolidate the pending changelog (with `consolidate` argument), or promote dev to main and trigger CI (with `main` argument)
user-invocable: true
---

# Release

This skill has three modes determined by the argument:

- **No argument** → *dev mode*. Aggregate the latest uncommitted changes into a single `[pending]` block at the top of `docs/CHANGELOG.md`, commit, and push to `origin/dev`. Does not bump versions, does not touch `main`, does not sync the website, does not trigger CI.
- **`consolidate` argument** → *consolidate mode*. Rewrite the existing `[pending]` block into a clean, minimal set of bullets — merging overlapping entries and dropping ones that cancel each other out — then commit and push to `origin/dev`. Does not bump versions, does not touch `main`.
- **`main` argument** → *promotion mode*. Lock the accumulated `[pending]` block to a real version, fast-forward `main` from `dev`, push `main` (which triggers CI build + Steam deploy), and sync the website.

CI (`.github/workflows/release.yml`) only runs on push to `main`, so the dev-mode and consolidate-mode loops are safe to run as often as desired without burning runner minutes.

---

## Mode A — Dev release (no argument)

### A1. Preconditions

1. Run `git rev-parse --abbrev-ref HEAD`. If not `dev`, stop and tell the user to switch.
2. Run `git status`. If there are no uncommitted changes since the last commit, stop — nothing to release.
3. Run `git diff HEAD` to see what changed.

### A2. Update the `[pending]` section in CHANGELOG.md

1. Read the top of `docs/CHANGELOG.md`.
2. Translate the diff into one or more player-facing bullets following the format rules below.
3. Locate the topmost `## [pending]` heading:
   - If it exists, append new bullets under the appropriate `### Added` / `### Changed` / `### Fixed` subsection inside that block. Create the subsection if missing. Do not duplicate bullets that are already there.
   - If it does not exist, insert a fresh block at the top, immediately below the `# Changelog` header:
     ```
     ## [pending]

     ### Added | Changed | Fixed
     - **...** — ...
     ```

Format rules for bullets (apply in all modes):
- Layman's terms only. No code references, no jargon.
- Bold the first phrase as a short summary, then describe in plain language.
- Don't spoil achievements, unlockables, or hidden content.
- Use only `### Added`, `### Changed`, `### Fixed`. Never include an `[Unreleased]` section.

### A3. Commit and push to dev

1. Stage **specific paths only** — never `git add -A` and never `git add .`. Typically:
   ```
   git add docs/CHANGELOG.md <other paths the user actually changed>
   ```
   If unsure which non-changelog files belong, ask the user before staging.
2. Commit with a short message describing the change. No `Co-Authored-By` line.
3. `git push origin dev`.

Stop here. Do not touch `main`, do not run `sync_content.sh`, do not bump the version.

---

## Mode C — Consolidate the pending block (`consolidate` argument)

Many dev releases in a row leave the `[pending]` block bloated: near-duplicate bullets, several bullets touching the same area, and — most importantly — bullets that **cancel each other out** (a bug introduced in one dev release and fixed in a later one; a feature added then reworked or removed; a value tweaked back and forth). Players who only ever see `main` never experienced the in-between states, so the changelog should describe only the **net** change since the last shipped version.

This mode rewrites the `[pending]` block in place so it reads as if written once, fresh.

### C1. Preconditions

1. Run `git rev-parse --abbrev-ref HEAD`. If not `dev`, stop and tell the user to switch.
2. Run `git status`. The working tree should be clean. If there are uncommitted changes, stop and tell the user to run `/game-release` (dev mode) first — consolidate only rewrites already-committed changelog text.
3. Confirm a `## [pending]` block exists at the top of `docs/CHANGELOG.md`. If it does not, stop — there is nothing to consolidate.

### C2. Establish ground truth

1. Run `git diff main dev --stat` (and `git diff main dev` for detail as needed) to see the **actual net code change** between the last `main` release and the current `dev` tip.
2. Use this as the source of truth: every consolidated bullet must correspond to a real net change in that diff. If a `[pending]` bullet describes something that the net diff shows was later undone, it must be dropped.

### C3. Rewrite the pending block

Read the entire `[pending]` block, then rebuild it applying these rules:

- **Merge** — collapse multiple bullets about the same feature or area into a single bullet describing its final state.
- **Supersede** — when a later bullet reworks or replaces an earlier one, keep only the final result, described in plain present-tense language. Drop the intermediate history.
- **Cancel** — if a change and its reversal both happened entirely within `[pending]` (e.g. a bug introduced then fixed, a feature added then removed), drop **both** bullets. The net effect is zero, so players see nothing.
- **Keep** — distinct, still-true changes remain as their own bullets.
- Re-sort the survivors into `### Added` / `### Changed` / `### Fixed`. Drop any subsection left empty.
- Apply the same format rules as dev mode (layman's terms, bold lead-in, no spoilers).

The rewritten block must still be headed `## [pending]` — consolidate never assigns a version.

If consolidation would empty the block entirely (everything cancelled out), leave a single `## [pending]` heading with no bullets rather than deleting it, and tell the user.

### C4. Commit and push to dev

1. Stage the changelog only: `git add docs/CHANGELOG.md`.
2. Commit: `git commit -m "Consolidate pending changelog"`.
3. `git push origin dev`.

Stop here. Do not touch `main`, do not run `sync_content.sh`, do not bump the version.

---

## Mode B — Promotion to main (`main` argument)

### B1. Preconditions

1. `git rev-parse --abbrev-ref HEAD` must be `dev`. If not, stop.
2. `git status` must be clean. If there are uncommitted changes, stop and ask the user to run `/game-release` (dev mode) first to fold them into `[pending]`.
3. Fetch tags: `git fetch --tags`.

### B2. Lock the `[pending]` block to a version

1. Read the version from `Cargo.toml` (`grep '^version = ' Cargo.toml`).
2. Verify no tag `v<version>` already exists (`git rev-parse v<version>` should fail). If it does, bump the patch in `Cargo.toml` until you find a free version, run `cargo update -p court_wizard --offline`, and use that version.
3. In `docs/CHANGELOG.md`, rename the top heading from `## [pending]` to `## [v<version>] - <today's date in YYYY-MM-DD>`.
   - If there is no `[pending]` block (nothing accumulated since the last main release), generate a fresh versioned block from the dev-vs-main diff using the same format rules.

### B3. Commit the lock to dev, then merge into main

1. Stage specific paths: `git add docs/CHANGELOG.md Cargo.toml Cargo.lock` (include `Cargo.lock` only if it changed).
2. Commit on `dev`: `git commit -m "v<version>: lock changelog for release"`.
3. Push `dev`: `git push origin dev`.
4. Switch to main: `git switch main`.
5. Pull latest: `git pull --ff-only`.
6. Fast-forward merge dev: `git merge --ff-only dev`. If this fails (main has commits dev doesn't), stop and surface the error — do not force-push, do not rebase silently.
7. Push main: `git push origin main`. This triggers `.github/workflows/release.yml` which builds all platforms, attaches zips to a GitHub Release, and uploads to Steam `staging`.
8. Switch back to dev: `git switch dev`.

### B4. Sync website

1. From `../court_wizard_website` run `./scripts/sync_content.sh`.
2. If `content/` has changes, stage `content/` only, commit with `Sync content from game v<version>`, push.
3. If no changes, skip — already in sync.

If `sync_content.sh` errors out (e.g., game repo not found at `../court_wizard`), stop and surface the error.

### B5. Report

Tell the user:
- The new version that just shipped to `main`.
- The CI run URL (`gh run list --workflow=release.yml --limit 1`) so they can monitor the build.
- Whether the website sync produced a commit.

---

## Hard rules (all modes)

- **Never use `git add -A` or `git add .`** — stage only the files the user actually changed. The `steamworks_achievements.csv` file is intentionally deleted locally; sweeping deletions into a commit would be wrong.
- **Never use `git reset`** in any form.
- **Never use `git checkout` to revert working-tree changes.** Use `git switch` for branch changes only.
- **Never include a `Co-Authored-By` trailer.**
- **Never push to main directly.** The promotion flow merges `dev` into `main` via fast-forward only.
- **Never force-push.** If a fast-forward merge to `main` is impossible, stop and ask the user.
