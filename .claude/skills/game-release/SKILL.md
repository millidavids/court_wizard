---
name: game-release
description: Generate changelog, then commit and push (CI builds release binaries)
user-invocable: true
---

# Release

Perform a full release of the game. Execute these steps in order, stopping if any step fails.

## Step 1: Generate Changelog

1. Run `git diff` to see all uncommitted changes (staged + unstaged)
2. Analyze the diff to understand what changed from the player's perspective
3. Read the top of `CHANGELOG.md` to match the existing format and version style
4. Get the current version from `Cargo.toml`
5. Add a new entry at the top of the changelog (below the header) with today's date and the current version
6. Follow these rules:
   - Use layman's terms — no code references, no technical jargon
   - Categorize changes under `### Added`, `### Changed`, and/or `### Fixed` as appropriate
   - Bold the first phrase of each bullet as a short summary, then describe in plain language
   - Don't spoil achievements, unlockables, or hidden content
   - Don't add an `[Unreleased]` section

## Step 2: Commit and Push

1. Stage all changed files with `git add -A`
2. Write a concise commit message summarizing the release
3. Commit and push to the remote

## Step 3: Sync and Deploy Website Changelog

The marketing site at `../court_wizard_website` mirrors `CHANGELOG.md`, `CREDITS.md`, and `INSTRUCTIONS.md` into its `content/` directory and renders them at runtime. After the game commit lands, publish the updated content to the website:

1. From the website repo root, run `./scripts/sync_content.sh` to copy the three markdown files from the game repo into `content/`.
2. Check `git status` in the website repo. If `content/` has changes:
   - Stage them with `git add content/`
   - Commit with a message like `Sync content from game vX.Y.Z`
   - Push to the remote (Cloudflare Pages will auto-deploy on push to `main`)
3. If `content/` has no changes, skip the commit — the website is already in sync.

Notes:
- Only the `content/` directory should change in this step. If `sync_content.sh` reports errors (e.g., the game repo can't be found at `../court_wizard`), stop and surface the error to the user instead of attempting the deploy.
- Never modify unrelated website files during this step.
