#!/usr/bin/env bash
# Render the newest docs/CHANGELOG.md section as Steam BBCode, ready to paste
# into a Steamworks event (Steamworks → app → Hub Admin → create event, type
# "Small Update / Patch Notes").
#
# Steamworks has no supported Web API for creating events or announcements, so
# this generates the body rather than posting it. release.yml writes the output
# to the run's job summary and uploads it as an artifact.
#
# Usage: scripts/changelog_to_bbcode.sh [path/to/CHANGELOG.md]
#
# Input (the topmost `## [...]` block):
#     ## [v1.0.35] - 2026-08-05
#
#     ### Fixed
#     - **Lead-in** — plain-language prose.
#
# Output:
#     [h2]v1.0.35 — 2026-08-05[/h2]
#     [h3]Fixed[/h3]
#     [list]
#     [*][b]Lead-in[/b] — plain-language prose.
#     [/list]

set -euo pipefail

CHANGELOG="${1:-docs/CHANGELOG.md}"

if [ ! -f "$CHANGELOG" ]; then
    echo "error: changelog not found: $CHANGELOG" >&2
    exit 1
fi

# Everything from the first `## [` heading up to (not including) the next one.
# Matches the extraction release.yml already uses for the GitHub Release body
# and the Discord embed, so all three stay in step.
section=$(awk '
    /^## \[/ { if (seen) exit; seen = 1 }
    seen     { print }
' "$CHANGELOG")

if [ -z "$section" ]; then
    echo "error: no '## [' section found in $CHANGELOG" >&2
    exit 1
fi

printf '%s\n' "$section" | awk '
    # Close any open [list] before starting a new block.
    function close_list() {
        if (in_list) { print "[/list]"; in_list = 0 }
    }

    # `## [v1.0.35] - 2026-08-05` -> `[h2]v1.0.35 — 2026-08-05[/h2]`
    # Also tolerates a heading with no date (e.g. an unlocked `## [pending]`).
    /^## \[/ {
        line = $0
        sub(/^## \[/, "", line)
        if (match(line, /\] - /)) {
            version = substr(line, 1, RSTART - 1)
            date = substr(line, RSTART + 4)
            printf "[h2]%s — %s[/h2]\n", version, date
        } else {
            sub(/\].*$/, "", line)
            printf "[h2]%s[/h2]\n", line
        }
        next
    }

    # `### Fixed` -> `[h3]Fixed[/h3]`
    /^### / {
        close_list()
        heading = substr($0, 5)
        printf "[h3]%s[/h3]\n", heading
        next
    }

    # `- **Lead-in** — prose` -> `[*][b]Lead-in[/b] — prose`
    /^- / {
        if (!in_list) { print "[list]"; in_list = 1 }
        item = substr($0, 3)
        # Only the leading **...** is a bold lead-in; convert it specifically
        # rather than globally, so any later ** in the prose is left alone.
        if (match(item, /^\*\*[^*]+\*\*/)) {
            bold = substr(item, 3, RLENGTH - 4)
            rest = substr(item, RLENGTH + 1)
            printf "[*][b]%s[/b]%s\n", bold, rest
        } else {
            printf "[*]%s\n", item
        }
        next
    }

    # Continuation line of a wrapped bullet: keep it attached to the item.
    /^[[:space:]]+[^[:space:]]/ {
        if (in_list) {
            sub(/^[[:space:]]+/, "")
            print
            next
        }
    }

    # Blank lines inside a list would render as a gap; drop them there and
    # keep them between blocks.
    /^[[:space:]]*$/ {
        if (!in_list) print ""
        next
    }

    { close_list(); print }

    END { close_list() }
'
