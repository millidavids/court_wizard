#!/bin/bash
set -e

# Copy the Steam In-Game Actions manifests (assets/controller_config/*.vdf)
# into the given destination directory.
#
# Steam auto-discovers the IGA manifest at the depot/install ROOT:
# controller_config/game_actions_<appid>.vdf. The manifests also live under
# assets/ (for the runtime SetInputActionManifestFilePath fallback), but Steam
# only looks at the root — so every packaged layout mirrors them there. Both
# the main app and the Playtest read the same depot; each app picks the file
# named for its own id.
#
# Usage: ./scripts/copy_iga_manifests.sh <dest_dir>
# Must be run from the repo root (all callers cd there first).

DEST="$1"
if [ -z "$DEST" ]; then
    echo "Usage: $0 <dest_dir>" >&2
    exit 1
fi

if [ -d "assets/controller_config" ]; then
    mkdir -p "$DEST"
    cp assets/controller_config/*.vdf "$DEST/" 2>/dev/null || true
fi
