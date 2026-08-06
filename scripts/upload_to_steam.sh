#!/bin/bash
# Upload existing platform zips to Steamworks (App ID 4550880, branch: staging).
#
# LOCAL FALLBACK ONLY. The normal path is GitHub Actions: dev-release.yml
# uploads to `staging` and release.yml promotes to the default branch (see
# steam/README.md). Builds uploaded by this script carry no `r<run_id>` tag in
# their description, so release.yml's promotion step cannot find them — promote
# a local upload by hand from the Steamworks dashboard.
#
# Usage:
#   ./scripts/upload_to_steam.sh <steam_username>
#
# The script auto-detects which platform zips exist at the repo root for the
# current Cargo.toml version and uploads only those depots. Build the zips
# first with ./scripts/package.sh <platform>.
#
# Compatible with macOS's stock bash 3.2 — no associative arrays or bash 4 features.
#
# Requires steamcmd on PATH and an interactive terminal so you can complete
# Steam Guard if prompted on first run.

set -e

cd "$(dirname "$0")/.."

if ! command -v steamcmd &> /dev/null; then
    echo "Error: steamcmd not found on PATH." >&2
    echo "  macOS:   brew install steamcmd" >&2
    echo "  Linux:   download from https://developer.valvesoftware.com/wiki/SteamCMD" >&2
    exit 1
fi

if [ -z "$1" ]; then
    echo "Usage: $0 <steam_username>" >&2
    exit 1
fi
STEAM_USER="$1"

VERSION=$(grep '^version = ' Cargo.toml | head -1 | sed 's/version = "\(.*\)"/\1/')
SHORT_SHA=$(git rev-parse --short HEAD)
echo "Court Wizard v$VERSION ($SHORT_SHA) — uploading as $STEAM_USER"

zip_suffix() {
    case "$1" in
        windows) echo "windows" ;;
        linux)   echo "linux" ;;
        macos)   echo "macos-universal" ;;
    esac
}

depot_id() {
    case "$1" in
        windows) echo "4550882" ;;
        linux)   echo "4550883" ;;
        macos)   echo "4550881" ;;
    esac
}

REPO_ROOT="$(pwd)"
STAGING="$REPO_ROOT/steam-content"
BUILDOUTPUT="$REPO_ROOT/steam-build-output"
rm -rf "$STAGING" "$BUILDOUTPUT"
mkdir -p "$STAGING" "$BUILDOUTPUT"

UPLOAD_PLATFORMS=""
DEPOT_LINES_FILE="$REPO_ROOT/steam/depot_lines.tmp"
: > "$DEPOT_LINES_FILE"
for p in windows linux macos; do
    ZIP="court_wizard-v${VERSION}-$(zip_suffix "$p").zip"
    if [ -f "$ZIP" ]; then
        echo ">>> Staging $p from $ZIP"
        mkdir -p "$STAGING/$p"
        unzip -q "$ZIP" -d "$STAGING/$p"
        UPLOAD_PLATFORMS="$UPLOAD_PLATFORMS $p"
        printf '\t\t"%s" "depot_%s.vdf"\n' "$(depot_id "$p")" "$p" >> "$DEPOT_LINES_FILE"
    else
        echo "    skipping $p (no zip found at $ZIP)"
    fi
done

if [ -z "$UPLOAD_PLATFORMS" ]; then
    rm -f "$DEPOT_LINES_FILE"
    echo "Error: no platform zips found for v$VERSION at the repo root." >&2
    echo "Build them first with ./scripts/package.sh <platform>." >&2
    exit 1
fi

# Generate the app build script with concrete values.
GENERATED="$REPO_ROOT/steam/app_build_4550880.generated.vdf"
awk -v version="$VERSION" \
    -v sha="$SHORT_SHA" \
    -v contentroot="$STAGING" \
    -v buildoutput="$BUILDOUTPUT" \
    -v depotfile="$DEPOT_LINES_FILE" '
    {
        gsub(/\$VERSION/, version)
        gsub(/\$SHA/, sha)
        gsub(/\$CONTENTROOT/, contentroot)
        gsub(/\$BUILDOUTPUT/, buildoutput)
        if ($0 ~ /\$DEPOT_LINES/) {
            while ((getline line < depotfile) > 0) print line
            close(depotfile)
            next
        }
        print
    }
' steam/app_build_4550880.vdf > "$GENERATED"
rm -f "$DEPOT_LINES_FILE"

echo ""
echo "Uploading to Steam (branch: staging):"
for p in $UPLOAD_PLATFORMS; do echo "  - $p ($(depot_id "$p"))"; done
echo ""
echo "If Steam Guard prompts, approve in the Steam Mobile app."
echo ""

steamcmd +login "$STEAM_USER" +run_app_build "$GENERATED" +quit

echo ""
echo "Done. This was a local upload, so release.yml cannot promote it — set it live"
echo "on 'default' by hand from the Steamworks dashboard when ready."
