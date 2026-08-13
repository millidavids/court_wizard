---
name: game-release
description: Fold player-facing notes into the open version block on dev (default), consolidate that block (with `consolidate` argument), or promote the already-built dev tip to main (with `main` argument)
user-invocable: true
---

# Release

This skill has three modes determined by the argument:

- **No argument** → *dev mode*. Fold everything unreleased — uncommitted work plus any commits that never triggered a build — into the **open version block** at the top of `docs/CHANGELOG.md`, commit, and push to `origin/dev`. Opens the next version first if the current one has already shipped. **Triggers a full release build that publishes to Steam `staging`.**
- **`consolidate` argument** → *consolidate mode*. Rewrite the open version block into a clean, minimal set of bullets — merging overlapping entries and dropping ones that cancel each other out — then commit and push to `origin/dev`. Also triggers a build.
- **`main` argument** → *promotion mode*. Fast-forward `main` onto the already-built `dev` tip and push it, then sync the website. **Changes no files in the game repo and builds nothing** — it promotes the exact binary that has been sitting on Steam `staging`.

## The version is assigned on dev, never at promotion

`docs/CHANGELOG.md` is compiled into the binary by `include_str!` (`src/ui/manual/systems.rs:20`). Any changelog edit changes the shipped bits and therefore requires a build. That single fact drives the whole design:

- **Dev pushes build anyway**, so that is where every changelog and version edit belongs. Free.
- **Promotion must change nothing.** A file change at promotion time would invalidate the build it is trying to promote and force a rebuild — shipping a binary nobody play-tested.

So there is no `[pending]` block and no separate "lock" step. At any moment the top block of the changelog is the **open version**: already numbered, already dated, already baked into the build on `staging`. Promotion simply makes that build live.

### Which version is open

Run `git fetch --tags`, then read the version `V` from `Cargo.toml` (`grep '^version = ' Cargo.toml`). The tag decides:

- **Tag `vV` does not exist** → `V` is still open. Append bullets to the existing `## [vV] - <date>` block and refresh its date to today.
- **Tag `vV` exists** → `V` has already shipped. Open the next one:
  1. Bump the patch in `Cargo.toml` (keep bumping until you find a version with no tag).
  2. Run `cargo update -p court_wizard --offline` so `Cargo.lock` follows.
  3. Insert a fresh `## [vV+1] - <today>` block at the top of the changelog, directly below the `# Changelog` header.

If tag `vV` is missing but the top block's heading does not match `vV` — a leftover `## [pending]`, or a version mismatch — repair it in place: rename that heading to `## [vV] - <today>` and keep its bullets. Never stack two blocks for the same version.

**The date means "last build in this version", not "release day".** Every dev push refreshes it; promotion never touches it. The shipped date is therefore the date of the final dev release before promotion — normally the same day, sometimes a day or two earlier. That is deliberate: re-dating at promotion would mean a file change, a new build, and an untested binary going live.

## How CI is wired (read this before reporting anything to the user)

**`dev-release.yml` runs on any push to `dev` that touches `docs/CHANGELOG.md`** — which is dev mode and consolidate mode. It runs `cargo test` / `clippy -D warnings` / `fmt --check`, builds Windows + Linux + the signed universal macOS `.app`, and uploads all three depots as a **single** Steam build to the `staging` branch. Ordinary development pushes that don't touch the changelog don't build.

**`release.yml` runs on push to `main` and builds nothing.** It finds the `dev-release.yml` run for that exact commit, reuses *its* artifacts for the tag and GitHub Release, and asks Steam to set *its* build live on the default branch. This works because promotion fast-forwards `main` onto the built dev commit, so the SHAs match.

Three consequences worth stating plainly:

- **A dev release is not free.** Each one is a full three-platform signed build (the macOS notarization leg runs to hours). That is intended — it is how a build reaches Steam `staging` to be play-tested — but don't run dev mode for trivial changelog touch-ups expecting a no-op.
- **Promotion requires that the dev tip was actually built.** Since promotion no longer pushes a build of its own, fast-forwarding `main` onto a dev commit that `dev-release.yml` never ran on leaves `release.yml` with nothing to promote. It hard-errors — after `main` has already moved. Mode B pre-flights this locally so that never happens.
- **Promotion is not fully automatic.** Setting a build live on the default branch of a released app always sends an authorization prompt to the Steam Mobile app; there is no opt-out. CI picks the correct build id and requests the promotion, then it sits pending until the user approves on their phone. Never report a `main` release as "live" — report it as "pending your Steam Mobile confirmation".

## Commit message format (dev and consolidate modes)

Release commits describe **what shipped**, not the mechanics of the release. Never `lock changelog for release` or any other boilerplate that reads identically every version.

```
v<version>: <lower-case summary of what this push adds>

- <one condensed line per changelog bullet this commit introduces>
- <...>
```

- The subject names the version, then says what changed, in the user's language. Aim for under ~72 characters.
- The body lists the bullets **this commit** introduces — not the whole version block. A later dev push on the same version lists only its own additions.
- Condense each bullet to a single line; the changelog holds the full prose.
- If the push also contains code with no player-visible effect, don't invent a bullet for it — the subject line can mention it if it's the main content.
- No `Co-Authored-By` trailer, no AI/tool attribution of any kind.

Consolidate mode uses the same shape with a subject describing the cleanup, e.g. `v1.0.36: consolidate release notes`, and a body noting what was merged or dropped.

## Changelog bullet format (all modes)

- Layman's terms only. No code references, no jargon.
- Bold the first phrase as a short summary, then describe in plain language.
- Don't spoil achievements, unlockables, or hidden content.
- Use only `### Description`, `### Added`, `### Changed`, `### Fixed`. Never an
  `[Unreleased]` section.

## The `### Description` section (all modes)

Every version block opens with a `### Description` section holding **one short
paragraph of plain prose** — no bullet, no bold lead-in. It is the release's
public hook, and unlike the rest of the block it is not a list of changes:

```markdown
## [v1.0.38] - 2026-08-13

### Description
The Arcane Crystal now takes on the character of almost any spell you feed it —
grease, ice storms, war hymns, lightning rods — instead of only echoing damage.

### Added
- **...**
```

**It is published verbatim.** `scripts/post_to_bluesky.py` uses it as the body
of the Bluesky announcement, and it appears in the Discord embed along with the
rest of the block. Write it for someone who has never heard of the game.

**Hard constraint: it must fit a Bluesky post.** The limit is 300 characters
*including* the `Court Wizard v<version>` title and the `Steam · Website ·
Studio` link footer, which together cost roughly 45. That leaves about **250
characters** for the description. Longer text is truncated on a word boundary
with an ellipsis, so an over-long description ships as a cut-off sentence.

Check it before committing — the script needs no credentials to render:

```bash
python3 scripts/post_to_bluesky.py --version <version> --dry-run
```

That prints the exact post and the resolved link targets.

**Dev mode:** create the section when opening a new version; on a later push to
the same version, rewrite it so it still describes the release as a whole rather
than only the newest push.

**Consolidate mode:** rewrite it to match the consolidated block — it is a
summary of the net change, so it must be redone when the bullets are.

If a release genuinely has no player-facing hook (a build-process-only version),
still write one honest sentence; the post goes out either way.

---

## Mode A — Dev release (no argument)

### A1. Preconditions and scope

1. `git rev-parse --abbrev-ref HEAD` must be `dev`. If not, stop and tell the user to switch.
2. `git fetch --tags` — required before deciding which version is open.
3. Work out what is unreleased. It is **not** only the working tree: commits already made on `dev` that never touched `docs/CHANGELOG.md` never triggered a build, so they are unreleased too and must be swept in here.
   ```
   git status --porcelain                                   # uncommitted work
   git log --oneline $(git log -1 --format=%H -- docs/CHANGELOG.md)..HEAD   # committed but unbuilt
   ```
   - Uncommitted changes → `git diff HEAD` is the source for bullets.
   - Committed-but-unbuilt commits → their diff is the source: `git diff <last-changelog-commit>..HEAD`.
   - Both → cover both.
   - **Neither** → stop, nothing to release.

   This matters because promotion no longer creates a build of its own. A changelog-only commit here is what gets those earlier commits built and onto `staging`.

### A2. Resolve the open version

Apply *Which version is open* above. This either leaves `Cargo.toml` untouched (version still open) or bumps the patch and updates `Cargo.lock` (previous version already tagged).

### A3. Update the changelog

1. Read the top of `docs/CHANGELOG.md`.
2. Translate the diffs identified in A1 into player-facing bullets — covering the committed-but-unbuilt work as well as the working tree.
3. Append them to the open `## [v<version>]` block under the appropriate `### Added` / `### Changed` / `### Fixed` subsection, creating the subsection if missing. Do not duplicate bullets already present.
4. Refresh the block's date to today.

If the version was just opened in A2, the block is new and contains only these bullets.

**If the change has no player-facing effect at all** — developer docs, CI config, the release skill itself, tooling that isn't compiled into the game — there is no honest bullet to write. Do not invent one, and do not edit the changelog. Commit and push the change to `dev` as an ordinary commit instead: it triggers no build, and A1 sweeps it into the next real release automatically. Tell the user that is what happened and why. Forcing a build here would spend hours of CI to produce a byte-identical binary and would put a second build on `staging` for no reason.

### A4. Commit and push to dev

1. Stage **specific paths only** — never `git add -A` and never `git add .`:
   ```
   git add docs/CHANGELOG.md <other paths the user actually changed>
   ```
   Add `Cargo.toml Cargo.lock` as well when A2 opened a new version. If unsure which non-changelog files belong, ask the user before staging.
2. Commit using the message format above.
3. `git push origin dev`.

### A5. Report what CI is doing

Tell the user:
- which version this landed in, and whether it opened a new one;
- that `dev-release.yml` is running — test/clippy/fmt gate, then Windows + Linux + signed macOS, then a single Steam upload to **`staging`** — and that the macOS notarization leg is the slow one;
- the run URL (`gh run list --workflow=dev-release.yml --limit 1`, otherwise the Actions tab);
- that they should play-test from `staging`, and that **this exact build is what `/game-release main` will promote** if it's the last dev release before promotion.

If the changelog ended up unchanged (every bullet already present), say so — no build was triggered, and a manual `dev-release.yml` dispatch is needed if they wanted one.

Stop here. Do not touch `main`, do not run `sync_content.sh`.

---

## Mode C — Consolidate the open version block (`consolidate` argument)

Many dev releases in a row leave the open block bloated: near-duplicate bullets, several bullets touching the same area, and — most importantly — bullets that **cancel each other out** (a bug introduced in one dev release and fixed in a later one; a feature added then reworked or removed; a value tweaked back and forth). Players who only ever see `main` never experienced the in-between states, so the changelog should describe only the **net** change since the last shipped version.

This mode rewrites the block in place so it reads as if written once, fresh.

### C1. Preconditions

1. `git rev-parse --abbrev-ref HEAD` must be `dev`. If not, stop.
2. `git status` must be clean. If there are uncommitted changes, stop and tell the user to run `/game-release` (dev mode) first — consolidate only rewrites already-committed changelog text.
3. `git fetch --tags`, then confirm the top block is an untagged `## [v<version>]` matching `Cargo.toml`. If its tag already exists, that version has shipped — stop, there is nothing open to consolidate.

### C2. Establish ground truth

1. `git diff main dev --stat` (and `git diff main dev` for detail) shows the **actual net code change** between the last promoted release and the current dev tip.
2. Every consolidated bullet must correspond to a real net change in that diff. If a bullet describes something the net diff shows was later undone, drop it.

### C3. Rewrite the block

- **Merge** — collapse multiple bullets about the same feature or area into one describing its final state.
- **Supersede** — when a later bullet reworks or replaces an earlier one, keep only the final result in plain present-tense language. Drop the intermediate history.
- **Cancel** — if a change and its reversal both happened inside this version (bug introduced then fixed, feature added then removed), drop **both**. Net effect zero, so players see nothing.
- **Keep** — distinct, still-true changes remain as their own bullets.
- Re-sort survivors into `### Added` / `### Changed` / `### Fixed`. Drop any subsection left empty.

Keep the `## [v<version>]` heading exactly as it is and refresh its date to today. Consolidate never changes the version number.

If consolidation would empty the block entirely (everything cancelled out), leave the heading with no bullets and tell the user — promoting an empty version block is almost certainly not what they want.

### C4. Commit and push to dev

1. `git add docs/CHANGELOG.md`.
2. Commit using the message format above.
3. `git push origin dev`.

Stop here. Do not touch `main`, do not run `sync_content.sh`.

---

## Mode B — Promotion to main (`main` argument)

Promotion is a pure fast-forward. **It edits no files, creates no commit in the game repo, and bumps nothing.** If you find yourself wanting to change a file here, the flow has gone wrong — stop and fold that change into a dev release instead.

### B1. Preconditions

Every one of these is a hard stop.

1. `git rev-parse --abbrev-ref HEAD` must be `dev`.
2. `git status` must be clean. If not, tell the user to run `/game-release` (dev mode) first — those changes need to be in the build being promoted.
3. `git fetch --tags`.
4. Read `V` from `Cargo.toml`. Tag `vV` must **not** already exist — if it does, this version has already been promoted.
5. The top block of `docs/CHANGELOG.md` must be `## [vV] - <date>` matching `V`. If it is `## [pending]` or names a different version, the changelog does not match what is about to ship — run `/game-release` (dev mode) to repair it, which also produces the build.
6. `git rev-parse dev origin/dev` must match. If local `dev` is ahead, push it first (via dev mode) — an unpushed commit has no build.
7. **The dev tip must have a `dev-release.yml` run.** This is the check that replaces the old build-at-promotion step:
   ```
   gh run list --workflow=dev-release.yml --commit "$(git rev-parse HEAD)" \
     --json databaseId,status,conclusion,url
   ```
   - **Empty array** → stop. The tip was pushed without touching the changelog, so nothing built it. Tell the user to run `/game-release` to fold their changes in.
   - **`conclusion: failure`/`cancelled`** → stop and surface it. Promoting would fail in CI anyway.
   - **`status: in_progress`** → allowed, but tell the user: `release.yml` will wait for it (it polls up to ~5h20m), and they will not have play-tested this build.
   - **`conclusion: success`** → ideal. This is the play-tested build on `staging` that is about to go live.

### B2. Fast-forward main and push

1. `git switch main`
2. `git pull --ff-only`
3. `git merge --ff-only dev`. If this fails (main has commits dev doesn't), stop and surface the error — do not force-push, do not rebase silently.
4. Verify the SHAs match: `git rev-parse main dev`.
5. `git push origin main`. This triggers `.github/workflows/release.yml`, which builds nothing — it locates the dev build for this SHA, tags it, publishes the GitHub Release from that build's zips, and asks Steam to set that build live on the default branch.
6. `git switch dev`.

### B3. Sync website

1. From `../court_wizard_website` run `./scripts/sync_content.sh`.
2. If `content/` has changes, stage `content/` only, commit with `Sync content from game v<version>`, push.
3. If no changes, skip — already in sync.

If `sync_content.sh` errors out (e.g., game repo not found at `../court_wizard`), stop and surface the error.

### B4. Report

Be precise about what has and has not happened — the release is **not** live at this point.

Tell the user:
- the version now on `main`, and that promotion changed no files;
- that **one** run is in flight — `release.yml` — plus the dev build it is reusing if that is still running. Give the URL (`gh run list --limit 5`, otherwise the Actions tab);
- **that promotion finishes only after they approve the prompt in their Steam Mobile app.** Setting a build live on the default branch of a released app always requires that confirmation — CI picks the right build id and requests it, nothing more. Do not describe the release as live, shipped, or out;
- that the Steamworks announcement BBCode is on the `release.yml` run's summary page (and as the `steam-announcement` artifact), ready to paste into Steamworks → Court Wizard → Hub Admin → create event → *Small Update / Patch Notes*;
- whether the website sync produced a commit.

If `release.yml` fails at the promotion step, the retry is *Actions → Release → Run workflow* with **`force: true`** — the tag already exists by then, so a plain re-run would skip everything.

---

## Hard rules (all modes)

- **Never use `git add -A` or `git add .`** — stage only the files the user actually changed. The `steamworks_achievements.csv` file is intentionally deleted locally; sweeping deletions into a commit would be wrong.
- **Never use `git reset`** in any form.
- **Never use `git checkout` to revert working-tree changes.** Use `git switch` for branch changes only.
- **Never include a `Co-Authored-By` trailer** or any other AI/tool attribution.
- **Never push to main directly.** Promotion fast-forwards `dev` into `main` only.
- **Never force-push.** If a fast-forward merge to `main` is impossible, stop and ask the user.
- **Never edit a file in promotion mode.** Version numbers and changelog text are settled on `dev`, before the build.
