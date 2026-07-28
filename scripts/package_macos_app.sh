#!/bin/bash
set -e

# Build a universal (arm64 + x86_64) macOS binary and assemble it into a proper
# Court Wizard.app bundle (Contents/MacOS + Contents/Resources + Info.plist).
#
# Usage:
#   ./scripts/package_macos_app.sh                # Build both targets, assemble, zip
#   ./scripts/package_macos_app.sh --skip-build   # Assemble + zip from existing release builds
#   ./scripts/package_macos_app.sh --no-zip       # Build + assemble only (CI signs, then zips)
#
# Output:
#   dist/macos/court_wizard/Court Wizard.app        (plus README.txt + controller_config/)
#   court_wizard-v<version>-macos-universal.zip     (unless --no-zip)
#
# NOTE: this script does NOT sign anything. Signing/notarization/stapling is
# done by .github/workflows/macos-release.yml, which runs this with --no-zip,
# signs the bundle with rcodesign, and zips only after stapling. A zip produced
# locally by this script is UNSIGNED.
#
# Compatible with macOS's stock bash 3.2 — no associative arrays or bash 4 features.

cd "$(dirname "$0")/.."

if [ "$(uname -s)" != "Darwin" ]; then
    echo "Error: this script requires macOS (lipo, ditto)." >&2
    exit 1
fi

SKIP_BUILD=false
NO_ZIP=false
for arg in "$@"; do
    case "$arg" in
        --skip-build) SKIP_BUILD=true ;;
        --no-zip)     NO_ZIP=true ;;
        *)
            echo "Error: Unknown argument '$arg'." >&2
            echo "Valid flags: --skip-build, --no-zip" >&2
            exit 1
            ;;
    esac
done

VERSION=$(grep '^version = ' Cargo.toml | head -1 | sed 's/version = "\(.*\)"/\1/')
ARM_TARGET="aarch64-apple-darwin"
INTEL_TARGET="x86_64-apple-darwin"
BUNDLE_ID="com.blackhearthgames.courtwizard"
# Keep LSMinimumSystemVersion (Info.plist below) in sync with the deployment
# target both architectures are compiled against.
MIN_MACOS="11.0"
APP_ROOT="dist/macos/court_wizard"
APP="$APP_ROOT/Court Wizard.app"
ZIP_NAME="court_wizard-v${VERSION}-macos-universal.zip"

STAGING=$(mktemp -d)
trap 'rm -rf "$STAGING"' EXIT

if [ "$SKIP_BUILD" = false ]; then
    echo "Building v$VERSION release binaries (deployment target: macOS $MIN_MACOS)..."
    MACOSX_DEPLOYMENT_TARGET="$MIN_MACOS" cargo build --release --target "$ARM_TARGET"
    MACOSX_DEPLOYMENT_TARGET="$MIN_MACOS" cargo build --release --target "$INTEL_TARGET"
fi

ARM_BIN="target/$ARM_TARGET/release/court_wizard"
INTEL_BIN="target/$INTEL_TARGET/release/court_wizard"
for f in "$ARM_BIN" "$INTEL_BIN"; do
    if [ ! -f "$f" ]; then
        echo "Error: $f not found. Run without --skip-build first." >&2
        exit 1
    fi
done

# True when the Mach-O at $1 contains both x86_64 and arm64 slices.
is_universal() {
    case "$(lipo -archs "$1")" in
        *x86_64*arm64*|*arm64*x86_64*) return 0 ;;
        *) return 1 ;;
    esac
}

# Locate the Steam redistributable dylib produced by steamworks-sys. Valve
# ships it universal; if this copy is somehow single-arch, merge the two
# per-target copies. Either way the bundle must end up with both archs.
# Newest-mtime first: a cached CI target dir can hold more than one
# steamworks-sys out dir, and only the freshest matches the binary just built.
find_steam_dylib() {
    find "target/$1" -path "*/build/steamworks-sys-*/out/libsteam_api.dylib" \
        -exec ls -t {} + 2>/dev/null | head -1
}

STEAM_DYLIB=$(find_steam_dylib "$ARM_TARGET")
if [ -z "$STEAM_DYLIB" ]; then
    echo "Error: libsteam_api.dylib not found under target/$ARM_TARGET — Steam features would be missing." >&2
    exit 1
fi
if ! is_universal "$STEAM_DYLIB"; then
    INTEL_DYLIB=$(find_steam_dylib "$INTEL_TARGET")
    if [ -z "$INTEL_DYLIB" ]; then
        echo "Error: libsteam_api.dylib is single-arch ($(lipo -archs "$STEAM_DYLIB")) and no Intel copy exists to merge." >&2
        exit 1
    fi
    echo "Merging single-arch libsteam_api.dylib copies into a universal dylib..."
    lipo -create "$STEAM_DYLIB" "$INTEL_DYLIB" -output "$STAGING/libsteam_api.dylib"
    STEAM_DYLIB="$STAGING/libsteam_api.dylib"
fi

echo "Assembling $APP..."
rm -rf "$APP_ROOT"
mkdir -p "$APP/Contents/MacOS" "$APP/Contents/Resources"

# Universal binary. The binary finds libsteam_api.dylib next to itself via the
# @loader_path rpath set in .cargo/config.toml for both Apple targets.
lipo -create "$ARM_BIN" "$INTEL_BIN" -output "$APP/Contents/MacOS/court_wizard"
chmod 755 "$APP/Contents/MacOS/court_wizard"
if is_universal "$APP/Contents/MacOS/court_wizard"; then
    echo "Universal binary: $(lipo -archs "$APP/Contents/MacOS/court_wizard")"
else
    echo "Error: lipo produced a non-universal binary ($(lipo -archs "$APP/Contents/MacOS/court_wizard"))." >&2
    exit 1
fi
cp "$STEAM_DYLIB" "$APP/Contents/MacOS/libsteam_api.dylib"

# Data goes in Contents/Resources — the game resolves it there when running
# inside a bundle (src/config/resource_paths.rs). Data files in Contents/MacOS
# would break codesign's resource sealing.
cp -R assets "$APP/Contents/Resources/assets"
./scripts/copy_iga_manifests.sh "$APP/Contents/Resources/controller_config"
if [ -f "docs/SPRITE_CREDITS.csv" ]; then
    cp docs/SPRITE_CREDITS.csv "$APP/Contents/Resources/"
fi

# Pre-built app icon (see packaging/macos/README.md — the 48px pixel-art logo
# goes blurry through sips' smooth interpolation, so the .icns is committed).
if [ ! -f "packaging/macos/AppIcon.icns" ]; then
    echo "Error: packaging/macos/AppIcon.icns missing — see packaging/macos/README.md." >&2
    exit 1
fi
cp packaging/macos/AppIcon.icns "$APP/Contents/Resources/AppIcon.icns"

cat > "$APP/Contents/Info.plist" <<PLIST
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
	<key>CFBundleDevelopmentRegion</key>
	<string>en</string>
	<key>CFBundleDisplayName</key>
	<string>Court Wizard</string>
	<key>CFBundleExecutable</key>
	<string>court_wizard</string>
	<key>CFBundleIconFile</key>
	<string>AppIcon</string>
	<key>CFBundleIdentifier</key>
	<string>$BUNDLE_ID</string>
	<key>CFBundleInfoDictionaryVersion</key>
	<string>6.0</string>
	<key>CFBundleName</key>
	<string>Court Wizard</string>
	<key>CFBundlePackageType</key>
	<string>APPL</string>
	<key>CFBundleShortVersionString</key>
	<string>$VERSION</string>
	<key>CFBundleVersion</key>
	<string>$VERSION</string>
	<key>LSApplicationCategoryType</key>
	<string>public.app-category.games</string>
	<key>LSMinimumSystemVersion</key>
	<string>$MIN_MACOS</string>
	<key>NSHighResolutionCapable</key>
	<true/>
</dict>
</plist>
PLIST

# Install-root extras, OUTSIDE the bundle (safe to place before signing —
# they are not part of the sealed .app):
# - README.txt for players
# - controller_config/: Steam's IGA auto-discovery never looks inside an .app,
#   only at the install root. The copy in Contents/Resources is what the game
#   itself reads at runtime.
cp docs/PLAYER_README.txt "$APP_ROOT/README.txt"
./scripts/copy_iga_manifests.sh "$APP_ROOT/controller_config"
echo "Included install-root controller_config/ (Steam IGA auto-discovery)."

if [ "$NO_ZIP" = true ]; then
    echo ""
    echo "Assembled (unsigned, no zip): $APP"
    exit 0
fi

# ditto preserves exec bits and metadata that plain zip tooling can drop.
rm -f "$ZIP_NAME"
ditto -c -k --keepParent "$APP_ROOT" "$ZIP_NAME"
SIZE=$(du -h "$ZIP_NAME" | cut -f1)
echo ""
echo "Packaged: $ZIP_NAME ($SIZE)"
echo "NOTE: this zip is UNSIGNED — signed builds come from macos-release.yml."
