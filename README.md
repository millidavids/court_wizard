# Court Wizard

A 3D wizard tower defense game built with Rust and Bevy.

## About

Play as a powerful wizard defending your castle from waves of attackers.

## Game Modes

- **Roguelite** - Fixed 25-level run with progression between battles
- **Endless** - Infinite scaling difficulty
- **Multiplayer** - Coming soon

## Gameplay

**Objective:**
- Victory: Eliminate all attackers and undead minions
- Defeat: Let all your defenders be killed

**Controls:**
- Mouse to aim and cast spells
- Select spells from the spell book UI
- Manage mana resources strategically

**Spells:**
- Magic Missile - Rapid-fire homing projectiles
- Fireball - AOE explosion spell
- Disintegrate - Powerful beam attack
- Chain Lightning - Chains between enemies
- Guardian Circle - Defensive protection
- Finger of Death - Single-target devastation
- Raise The Dead - Turn fallen enemies into undead allies
- Teleport - Reposition the wizard

**Save Data & Crash Logs:**
- Windows: `%APPDATA%\court_wizard\`
- Linux: `~/.local/share/court_wizard/`
- macOS: `~/Library/Application Support/court_wizard/`

If the game crashes, a `crash.log` file is written to the save data folder with details about what went wrong. Please include this file when reporting bugs.

[View Changelog](docs/CHANGELOG.md)

## Development

Built with:
- Rust
- Bevy game engine

### Build

```bash
# Debug build for current platform
./build_native.sh

# Cross-compile for Windows (from WSL2)
./build_native.sh windows

# Release build
./build_native.sh --release
./build_native.sh windows --release
```

## Credits

See [CREDITS.md](docs/CREDITS.md) for full attribution.
