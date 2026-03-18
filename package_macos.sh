#!/bin/bash
set -e

# Package a macOS release zip locally.
# Usage:
#   ./package_macos.sh                    # Build release and package (Apple Silicon)
#   ./package_macos.sh intel              # Build release and package (Intel)
#   ./package_macos.sh --skip-build       # Package from existing release build (Apple Silicon)
#   ./package_macos.sh intel --skip-build # Package from existing release build (Intel)

SKIP_BUILD=false
ARCH="macos"
ARCH_LABEL="apple-silicon"
TARGET="aarch64-apple-darwin"

for arg in "$@"; do
    case "$arg" in
        --skip-build) SKIP_BUILD=true ;;
        intel)
            ARCH="macos-intel"
            ARCH_LABEL="intel"
            TARGET="x86_64-apple-darwin"
            ;;
    esac
done

VERSION=$(grep '^version = ' Cargo.toml | head -1 | sed 's/version = "\(.*\)"/\1/')
BIN_DIR="./target/$TARGET/release"
ZIP_NAME="court_wizard-v${VERSION}-macos-${ARCH_LABEL}.zip"

if [ "$SKIP_BUILD" = false ]; then
    echo "Building macOS ($ARCH_LABEL) release..."
    ./build_native.sh "$ARCH" --release
fi

if [ ! -f "$BIN_DIR/court_wizard" ]; then
    echo "Error: $BIN_DIR/court_wizard not found. Run without --skip-build first."
    exit 1
fi

# Create a temporary staging directory
STAGING=$(mktemp -d)
trap 'rm -rf "$STAGING"' EXIT

mkdir -p "$STAGING/court_wizard"
cp "$BIN_DIR/court_wizard" "$STAGING/court_wizard/"
cp -r "$BIN_DIR/assets" "$STAGING/court_wizard/"

# Create zip
if command -v zip &> /dev/null; then
    (cd "$STAGING" && zip -r -q "$OLDPWD/$ZIP_NAME" court_wizard)
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
    echo "Error: No zip tool found. Install zip or python3."
    exit 1
fi

SIZE=$(du -h "$ZIP_NAME" | cut -f1)
echo ""
echo "Packaged: $ZIP_NAME ($SIZE)"
