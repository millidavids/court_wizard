# Changelog

All notable changes to this project will be documented in this file.

## [v0.0.69] - 2025-01-31

### Changed
- King's Guard units now lock to fixed orbital positions around the King instead of using cohesion forces
- Moved changelog to project root (no longer duplicated in docs/ and web/)

## [v0.0.65] - 2025-01-31

### Added
- King's Guard: 10 gold-colored infantry units that orbit the King and move with him
- Guards are individually targetable and killable

### Changed
- King now moves at full infantry speed
- King no longer has a movement speed cap

## [v0.0.58] - 2025-01-31

### Changed
- Attacker spawn system redesigned: enemies now spawn in a radial 6x6 grid arc along the wizard's spell range ring
- Spawn grid fills from center outward and close to far, with archers always behind the last infantry row
- Level scaling simplified: fixed number of infantry and archers added per level, spilling into new grid cells when exceeding 10 units per cell
- Attackers now start moving toward the castle immediately on spawn
- King unit no longer clusters with archers (zero cohesion)

## [v0.0.44] - 2025-01-31

### Added
- Tamper-resistant progress storage with signed verification

## [v0.0.42] - 2025-01-31

### Changed
- Renamed game from "The Game" to "Court Wizard" throughout
- Redesigned spellbook UI: spells now display in a horizontally scrollable list with buttons, instructions, and descriptions
- Spell buttons dynamically scale font size to fit spell names
- Spellbook scroll area now has a visible border frame
- Each spell now shows control instructions (e.g. "Click and hold to cast") and a gameplay description
- Build script now works on both macOS and WSL2/Linux

## [v0.0.23] - 2025-01-30

### Added
- Changelog screen accessible from main menu
- Scrollable changelog viewer with mouse wheel support
- Version/GitHub link button in main menu and pause menu (bottom-left corner)
- Clicking version button opens GitHub repository in new tab

### Changed
- Moved version display from in-game to menu screens only to prevent gameplay interference
- All buttons now have consistent styling with rounded borders and hover effects
- Changelog is now maintained in docs/ folder and automatically copied during builds

## [v0.0.6] - 2025-01-30

### Added
- GitHub link icon in top-right corner of webpage

### Changed
- Simplified build process: single index.html maintained in web/ folder, automatically copied to docs/ on release builds

## [v0.0.3] - 2025-01-30

### Added
- Version number display in bottom-left corner of screen
- Automatic version bumping with each build
- Teleport spell redesign:
  - First cast: Click and hold to place destination crosshair (follows mouse)
  - Second cast: Click and hold to grow teleport circle, release early to teleport fewer units
  - Right-click to cancel spell at any point
  - Holding right-click prevents casting
- King unit with special abilities:
  - Larger size and increased health/damage
  - Dynamic cohesion aura that rallies nearby defenders when enemies approach
  - Game ends in defeat if King dies
  - Special "The King died!" message on defeat screen

### Changed
- Teleport spell now has two-phase casting with visual feedback
- Defender infantry now spawn in single tight formation in front of King
- Reduced flocking cohesion to prevent excessive grouping during march
- Movement speed modifiers now properly affect unit acceleration and max speed

### Fixed
- Mouse input handling prevents spells from restarting when button is held after completion
- Right-click cancel properly resets spell state without immediate restart
- Teleport spell state management improved to prevent edge cases
