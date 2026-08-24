#!/usr/bin/env bash
# Render a docs/CHANGELOG.md section as Steam BBCode.
#
# Steamworks has no supported Web API for creating events, so the event is
# posted by hand and this renders the text to paste. Two callers use it:
#   * release.yml writes the full output to the run's job summary and uploads it
#     as an artifact;
#   * steam-promote.yml feeds --headline/--summary/--body to the run summary
#     once the build is live, one block per field of the event form.
#
# Usage: scripts/changelog_to_bbcode.sh [options] [path/to/CHANGELOG.md]
#
#   --version <v>   Render the `## [v<v>]` block instead of the topmost one.
#                   Accepts `1.0.38` or `v1.0.38`.
#   --headline      Print only the announcement title: `v1.0.38 - 2026-08-15`.
#   --description   Print only the prose under `### Description`, as one line.
#                   Prints nothing (exit 0) when the section is absent.
#   --summary       Same prose, trimmed to Steam's 180-character Summary field,
#                   preferring to end on a sentence boundary.
#   --body          Print the BBCode body: no `[h2]` title line (it is the
#                   headline), and the `### Description` prose promoted to an
#                   unheaded lead paragraph. Steam's subtitle field caps at 120
#                   characters and the prose runs past that, so the body is
#                   where it has to live; "Description" is not a heading a
#                   player should ever see.
#
# With no mode flag the output is the whole section including the `[h2]` title
# and the Description block. That form is load-bearing — release.yml calls this
# with no arguments — so do not change it.
#
# Input (a `## [...]` block):
#     ## [v1.0.38] - 2026-08-15
#
#     ### Fixed
#     - **Lead-in** — plain-language prose.
#
# Output:
#     [h2]v1.0.38 — 2026-08-15[/h2]
#     [h3]Fixed[/h3]
#     [list]
#     [*][b]Lead-in[/b] — plain-language prose.
#     [/list]

set -euo pipefail

MODE=full
VERSION=""
CHANGELOG=""

usage() {
    sed -n '2,26p' "$0" | sed 's/^# \{0,1\}//'
}

while [ $# -gt 0 ]; do
    case "$1" in
        --headline)    MODE=headline ;;
        --description) MODE=description ;;
        --summary)     MODE=summary ;;
        --body)        MODE=body ;;
        --version)
            shift
            [ $# -gt 0 ] || { echo "error: --version needs a value" >&2; exit 1; }
            VERSION="$1"
            ;;
        --version=*)   VERSION="${1#--version=}" ;;
        -h|--help)     usage; exit 0 ;;
        -*)            echo "error: unknown option: $1" >&2; exit 1 ;;
        *)             CHANGELOG="$1" ;;
    esac
    shift
done

CHANGELOG="${CHANGELOG:-docs/CHANGELOG.md}"

if [ ! -f "$CHANGELOG" ]; then
    echo "error: changelog not found: $CHANGELOG" >&2
    exit 1
fi

if [ -n "$VERSION" ]; then
    # Literal prefix matching, not regex: a version contains dots, and escaping
    # "## [v1.0.38]" into an awk regex is a reliable way to silently match
    # nothing. Same reasoning, and same shape, as steam-promote.yml's own
    # changelog extraction and post_to_bluesky.py's version_block().
    #
    # Selecting by version matters for a workflow_dispatch that names an older
    # release: main's changelog has already moved on, and the top block there
    # would ship the wrong notes.
    section=$(awk -v hdr="## [v${VERSION#v}]" '
        index($0, hdr) == 1 { inblock = 1; print; next }
        inblock && index($0, "## [") == 1 { exit }
        inblock { print }
    ' "$CHANGELOG")
else
    # Everything from the first `## [` heading up to (not including) the next.
    section=$(awk '
        /^## \[/ { if (seen) exit; seen = 1 }
        seen     { print }
    ' "$CHANGELOG")
fi

if [ -z "$section" ]; then
    if [ -n "$VERSION" ]; then
        echo "error: no '## [v${VERSION#v}]' section found in $CHANGELOG" >&2
    else
        echo "error: no '## [' section found in $CHANGELOG" >&2
    fi
    exit 1
fi

if [ "$MODE" = "headline" ]; then
    # `## [v1.0.38] - 2026-08-15` -> `v1.0.38 - 2026-08-15`.
    # A plain hyphen, not the em dash the [h2] line uses: this has to match the
    # titles of the announcements already on the hub.
    heading=${section%%$'\n'*}
    heading=${heading#'## ['}
    if [[ "$heading" == *"] - "* ]]; then
        printf '%s - %s\n' "${heading%%] - *}" "${heading#*] - }"
    else
        printf '%s\n' "${heading%%]*}"
    fi
    exit 0
fi

if [ "$MODE" = "description" ] || [ "$MODE" = "summary" ]; then
    # Joined to one line: both fields it feeds are single-line.
    #
    # Deliberately no fallback when the section is absent — post_to_bluesky.py
    # falls back to the first bullet because a 300-grapheme post needs *some*
    # body, whereas an event with no summary is perfectly fine. Empty output
    # and exit 0 is the "there isn't one" signal.
    #
    # `--summary` additionally trims to Steam's 180-character cap. A blind cut
    # lands mid-clause, so pack whole sentences and only fall back to a
    # word-boundary cut when even the first sentence overruns.
    printf '%s\n' "$section" | awk -v limit="${SUMMARY_LIMIT:-180}" -v trim="$([ "$MODE" = summary ] && echo 1 || echo 0)" '
        /^### / {
            inside = (tolower($0) == "### description")
            next
        }
        inside && NF {
            gsub(/^[[:space:]]+|[[:space:]]+$/, "")
            line = line (line == "" ? "" : " ") $0
        }
        END {
            if (line == "") exit
            if (!trim || length(line) <= limit) { print line; exit }
            rest = line
            while (match(rest, /[^.!?]*[.!?]+[[:space:]]*/)) {
                sentence = substr(rest, RSTART, RLENGTH)
                candidate = out sentence
                probe = candidate
                sub(/[[:space:]]+$/, "", probe)
                if (length(probe) > limit) break
                out = candidate
                rest = substr(rest, RSTART + RLENGTH)
                if (rest == "") break
            }
            sub(/[[:space:]]+$/, "", out)
            if (out == "") {
                out = substr(line, 1, limit - 1)
                sub(/[[:space:]][^[:space:]]*$/, "", out)
                out = out "…"
            }
            print out
        }
    '
    exit 0
fi

# emit_title: print the `[h2]` line. desc_heading: print `[h3]Description[/h3]`
# above its prose. Body mode drops both — the title becomes the announcement
# headline, and the prose leads the body unheaded.
emit_title=1
desc_heading=1
if [ "$MODE" = "body" ]; then
    emit_title=0
    desc_heading=0
fi

rendered=$(printf '%s\n' "$section" | awk -v emit_title="$emit_title" -v desc_heading="$desc_heading" '
    # Close any open [list] before starting a new block.
    function close_list() {
        if (in_list) { print "[/list]"; in_list = 0 }
    }

    # `## [v1.0.38] - 2026-08-15` -> `[h2]v1.0.38 — 2026-08-15[/h2]`
    # Also tolerates a heading with no date.
    /^## \[/ {
        if (!emit_title) { next }
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

    # `### Fixed` -> `[h3]Fixed[/h3]`. In body mode the Description heading is
    # swallowed while its prose falls through to the plain-text rule below, so
    # the section becomes an unheaded lead paragraph rather than disappearing.
    /^### / {
        close_list()
        heading = substr($0, 5)
        if (!desc_heading && tolower(heading) == "description") { next }
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
')

if [ "$MODE" = "body" ]; then
    # Dropping the title and the Description block leaves blank lines at the
    # top; strip leading and trailing blanks so the event body starts on
    # content. Not applied to the default output, which must stay byte-for-byte
    # what release.yml already publishes.
    printf '%s\n' "$rendered" | sed '/./,$!d' | sed -e :a -e '/^[[:space:]]*$/{$d;N;ba' -e '}'
else
    printf '%s\n' "$rendered"
fi
