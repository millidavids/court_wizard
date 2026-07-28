# macOS Packaging Assets

- `entitlements.plist` — hardened-runtime entitlements required for the Steam
  overlay; applied to the app's main executable by `macos-release.yml` when
  signing the bundle. Dylibs don't carry entitlements, so the nested
  `libsteam_api.dylib` is signed without them.
- `AppIcon.icns` — pre-built app icon, copied into the bundle by
  `scripts/package_macos_app.sh`. Pre-built (rather than generated in CI with
  `sips`) because the source logo is 48x48 pixel art: `sips`' smooth
  interpolation makes it blurry at Dock sizes.

## Regenerating AppIcon.icns

From a new `assets/images/logos/logo.png` (any square pixel-art source), on a
machine with ImageMagick + python3:

```bash
convert assets/images/logos/logo.png -scale 1056x1056 master.png   # nearest-neighbor upscale
for sz in 16 32 64 128 256 512 1024; do
    convert master.png -resize ${sz}x${sz} icon_${sz}.png           # smooth downscale
done
# then pack icon_*.png into AppIcon.icns (PNG-format entries:
# icp4/icp5/icp6/ic07-ic14) — see git history of this file for the packer
# one-liner, or use `iconutil -c icns` from an .iconset on a Mac.
```

The nearest-upscale-then-smooth-downscale order keeps pixel edges crisp at
large sizes instead of smearing 48px straight up to 1024.
