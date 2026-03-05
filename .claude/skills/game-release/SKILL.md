---
name: game-release
description: Generate changelog, build release WASM, then commit and push
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

## Step 2: Build Release

1. Run `./build_wasm.sh --release` to build the production WASM
2. If the build fails, fix the issue and retry

## Step 3: Commit and Push

1. Stage all changed files with `git add -A`
2. Write a concise commit message summarizing the release
3. Commit and push to the remote
