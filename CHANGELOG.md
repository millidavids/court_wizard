# Changelog

All notable changes to this project will be documented in this file.

## [v0.0.229] - 2025-02-04

### Changed
- Defenders now spawn in a radial formation between the castle and the battlefield center
- Defenders wait idle until enemies get close (800 units), then all activate at once
- Archers spawn in the back row, infantry in the front row of the defender formation
- King spawns at the center of the defender formation
- Improved code organization by renaming spawn grid constants for clarity

## [v0.0.197] - 2025-02-03

### Added
- Loading screen that displays while the game loads
  - Shows spinning wizard hat logo with "Court Wizard" title
  - Smooth fade-out when game is ready
  - Error screen if game fails to load
- Damage type system for spells (infrastructure for future features)
  - Magic Missile, Disintegrate, and Black Hole deal Force damage
  - Fireball deals Fire damage
  - Chain Lightning deals Electric damage
  - Ice and Necrotic damage types reserved for future spells
  - Currently tracked but not affecting gameplay (ready for resistances/vulnerabilities)

### Fixed
- Removed border/outline from game canvas for cleaner appearance
- Fixed loading screen text wrapping issue

## [v0.0.191] - 2025-02-03

### Added
- Performance optimization system to reduce CPU usage
  - Game systems now skip execution when nothing needs to be processed
  - Significantly improves performance, especially when spells aren't active or units are eliminated

### Changed
- **Optimized all spell systems** to only run when spell effects are active:
  - Magic Missile: Systems run only when missiles exist
  - Fireball: Separate checks for projectiles, explosions, and residual effects
  - Black Hole: Systems run only when black holes exist
  - Wall of Stone: Systems run only when walls exist
  - Chain Lightning: Systems run only when bolts or arcs exist
  - Disintegrate, Finger of Death: Beam systems run only when beams exist
  - Guardian Circle, Teleport: Visual update systems run only when indicators exist
- **Optimized all unit systems** to only run when units exist:
  - Archer systems run only when archers exist; arrow systems run only when arrows exist
  - Infantry systems run only when infantry exist
  - King systems run only when king exists
- **Code organization improvements**:
  - Moved `any_exist` run condition to game-level for reuse across entire codebase
  - Created `src/game/input/run_conditions.rs` for input-related conditions
  - Moved input conditions (`mouse_left_not_consumed`, `mouse_right_not_held`, `spell_input_not_blocked`) to input module
  - Spell run conditions now re-export commonly used conditions for convenience
  - All run condition imports properly organized with re-exports at top of files
- Removed unnecessary `Clone` trait bound from `any_exist` function for better performance

### Performance
- Spell systems no longer execute empty queries every frame when spells aren't active
- Unit systems no longer execute when those unit types don't exist (e.g., during game over, or when unit types are eliminated)
- Run condition checks are extremely lightweight compared to full system execution

## [v0.0.183] - 2025-02-03

### Added
- **Black Hole spell**: New ultimate spell that creates a gravitational sphere pulling units in a spiral
  - Cast time: 20 seconds (high commitment, strategic positioning required)
  - Duration: 20 seconds after cast completes
  - Rune combination: Q+R for 25% empowerment bonus
  - Uses inverse square law physics for realistic gravity that intensifies with proximity
  - Pulls both living units and corpses toward the center
  - Corpses despawn when they touch the black hole sphere
  - Deals ramping damage to units in contact with the sphere (increases over 3 seconds)
  - Gravity strength ramps up over 5 seconds, growing stronger over time
  - Maximum pull range: 500 units
  - Visual: Dark purple sphere with emissive glow and vibration effect
- Persistent spell effect system: Game will not end while Black Hole or Wall of Stone effects are active
  - Prevents premature victory if enemies die but King gets pulled into Black Hole afterward
  - Makes Black Hole a high-risk spell requiring careful placement and timing

### Changed
- Unit movement system refactored to separate movement calculations from transform application
  - External forces (like Black Hole gravity) can now override unit self-imposed speed limits
  - Units maintain their normal max speed for self-movement but can exceed it when pulled by external forces
  - Velocity damping now properly allows external forces to build momentum
- Corpses now retain Velocity and Acceleration components to be affected by external forces
  - Corpse velocity reset to zero on death to prevent death momentum
  - Allows Black Hole and future area effects to interact with corpses

### Fixed
- Removed debug logging from movement and spell effect systems

## [v0.0.156] - 2025-02-02

### Added
- Instructions screen accessible from both main menu and pause menu explaining game mechanics
- Comprehensive gameplay guide covering controls, spell book, action bar, rune system, and tips
- Rune system: cast empowered spells using Q/W/E/R key combinations + Spacebar
- 9 spell combinations available via runes (4 single-rune, 5 two-rune combos)
- Spells cast via runes are 25% more powerful (increased damage, speed, radius, reduced cast time)
- Rune display in bottom-middle shows current sequence and validity
- Empowered spells have their effectiveness increased across all aspects for a single cast

### Changed
- Action bar reduced from 10 slots to 5 slots (keys 1-5)
- Empowerment system refactored from boolean to f32 multiplier for future extensibility
- Empowerment now properly resets after a single cast, including channeled spells
- Instructions button added to main menu landing screen
- Instructions button added to pause menu

## [v0.0.121] - 2025-02-02

### Added
- Action bar with 10 customizable spell slots at the bottom-left of the screen
- Keyboard shortcuts: press keys 1-9 and 0 to instantly cast assigned spells without opening spell book
- Spell assignment: hover over any spell in the spell book and press a number key to assign it to that slot
- Action bar configuration persists between game sessions via local storage
- Dynamic text sizing in action bar slots to fit longer spell names

### Changed
- Wall of Stone collision now intelligently guides units around walls based on their target direction
- Units flow around walls in the shortest path toward their destination instead of being pushed backward
- Improved wall collision with stronger repulsive force to prevent units from getting stuck
- Wall collision preserves unit speed while redirecting movement around obstacles
- Version button now only appears on the main menu (hidden during gameplay)

### Fixed
- Wall of Stone collision now works correctly for the King and all unit types
- Units no longer exceed their max speed when redirected by wall collisions

## [v0.0.88] - 2025-01-31

### Fixed
- Buttons now work correctly on mobile touch devices (spell book, menus, settings, sliders)

## [v0.0.86] - 2025-01-31

### Added
- New spell: Wall of Stone — click and drag to raise an impassable stone wall on the battlefield
- Wall blocks all unit movement, projectiles, arrows, beams, and chain lightning bounces
- Wall lasts 20 seconds then sinks into the ground before despawning
- Units steer around walls instead of walking into them
- Discord changelog notifications on push via GitHub Actions

### Changed
- Fireball now leaves a burning ground effect after the explosion, dealing damage over 5 seconds
- Burning ground effect flickers like fire and fades out over its last second
- Fireball mana cost reduced from 60 to 30
- Fireball initial explosion damage reduced from 50 to 25
- Fireball total damage (explosion + residual fire) now totals 100

## [v0.0.74] - 2025-01-31

### Added
- Fireball now leaves a burning ground effect after the explosion, dealing damage over 5 seconds to units standing in the fire
- Burning ground effect flickers like fire and fades out over its last second

### Changed
- Fireball mana cost reduced from 60 to 30
- Fireball initial explosion damage reduced from 50 to 25
- Fireball total damage (explosion + residual fire) now totals 100

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
