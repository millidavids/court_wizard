# Changelog

All notable changes to this project will be documented in this file.

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
