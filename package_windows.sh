#!/bin/bash
set -e

# Package a Windows release zip locally.
# Usage:
#   ./package_windows.sh              # Build release and package
#   ./package_windows.sh --skip-build # Package from existing release build

SKIP_BUILD=false
for arg in "$@"; do
    case "$arg" in
        --skip-build) SKIP_BUILD=true ;;
    esac
done

VERSION=$(grep '^version = ' Cargo.toml | head -1 | sed 's/version = "\(.*\)"/\1/')
BIN_DIR="./target/x86_64-pc-windows-gnu/release"
ZIP_NAME="court_wizard-v${VERSION}-windows.zip"

if [ "$SKIP_BUILD" = false ]; then
    echo "Building Windows release..."
    ./build_native.sh windows --release
fi

if [ ! -f "$BIN_DIR/court_wizard.exe" ]; then
    echo "Error: $BIN_DIR/court_wizard.exe not found. Run without --skip-build first."
    exit 1
fi

# Create a temporary staging directory
STAGING=$(mktemp -d)
trap 'rm -rf "$STAGING"' EXIT

mkdir -p "$STAGING/court_wizard"
cp "$BIN_DIR/court_wizard.exe" "$STAGING/court_wizard/"
cp -r "$BIN_DIR/assets" "$STAGING/court_wizard/"
cp PLAYER_README.txt "$STAGING/court_wizard/README.txt"

# Create zip (use 7z if available, fall back to python zipfile)
if command -v 7z &> /dev/null; then
    (cd "$STAGING" && 7z a -tzip -r "$OLDPWD/$ZIP_NAME" court_wizard) > /dev/null
elif command -v python3 &> /dev/null; then
    python3 -c "
import zipfile, os, sys
with zipfile.ZipFile(sys.argv[1], 'w', zipfile.ZIP_DEFLATED) as zf:
    for root, dirs, files in os.walk(sys.argv[2]):
        for f in files:
            full = os.path.join(root, f)
            zf.write(full, os.path.relpath(full, sys.argv[3]))
" "$ZIP_NAME" "$STAGING/court_wizard" "$STAGING"
else
    echo "Error: No zip tool found. Install zip, 7z, or python3."
    exit 1
fi

SIZE=$(du -h "$ZIP_NAME" | cut -f1)
echo ""
echo "Packaged: $ZIP_NAME ($SIZE)"
