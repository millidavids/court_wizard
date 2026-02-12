# Changelog

All notable changes to this project will be documented in this file.

## [v0.0.579] - 2025-02-12

### Changed
- Switched to default Bevy font throughout the game (removed custom Davidfont)
- Reduced all UI font sizes by 30-35% to compensate for the default font appearing larger
- Finger of Death beam is now 3 times wider (easier to aim and hit enemies)
- When you lose a level, you now retry the same level instead of dropping down a level

## [v0.0.572] - 2025-02-11

### Changed
- **Save system completely redesigned** — one wizard per archetype instead of multiple named saves
  - Pick your archetype (RuneCaster, Randomancer, or Arcanorouter) directly from the Play menu
  - Each archetype can only have one save — starting a new wizard of the same type replaces the old one
  - No more managing multiple save slots or wizard names
  - Much simpler and cleaner — just pick an archetype and play
- **"Play" replaces "Begin, Wizard" and "Continue"** on the main menu
  - Single button takes you straight to the wizard select screen
  - See all three archetypes at once with their current progress
  - Archetypes you've played before show your current level
  - Pick any archetype to continue where you left off or start fresh
- Your existing saves are automatically upgraded to the new system when you launch the game

## [v0.0.564] - 2025-02-11

### Added
- **New wizard archetype: Arcanorouter** — a third way to play with dynamic resource allocation
  - Four vertical sliders let you balance power between Range, Mana Cost, Spell Power, and Cast Speed
  - All sliders share a fixed pool (adjusting one affects the others to keep things balanced)
  - Each slider can range from very weak (25%) to very strong (200%)
  - Keyboard controls: Q/A for Range, W/S for Mana, E/D for Power, R/F for Speed (each key changes by 10%)
  - Small color-coded bars at the bottom of the screen show your current allocations
  - Perfect for players who like fine-tuning their wizard's abilities on the fly
- Choose your wizard archetype (RuneCaster, Randomancer, or Arcanorouter) when starting a new game

### Changed
- Wizard stats are now the single source of truth for all spell effects
  - Range, mana cost, spell power, and cast speed are tracked on the wizard
  - All buffs and archetype effects modify these core stats
  - Spells now read directly from wizard stats for consistent behavior

## [v0.0.536] - 2025-02-10

### Added
- **New wizard archetype: Randomancer** — a second way to play with different mechanics
  - Press SPACE to spin a colorful roulette wheel that randomly selects a spell
  - Selected spells are empowered with 1.75x power (higher than RuneCaster's 1.25x)
  - Adds unpredictability and forces you to adapt to whatever spell you get
  - Choose your archetype (RuneCaster or Randomancer) when starting a new game
- **Roulette wheel UI** with smooth spinning animation
  - Displays as a colorful spinning wheel at the bottom of the screen
  - Spins for 2 seconds with smooth deceleration before landing on a random spell
  - Shows the selected spell name above the wheel after spinning
  - Triangle indicator points to the selected wedge

### Changed
- **Save system now uses encrypted format** instead of signed checksums
  - Saves are now obfuscated with XOR encryption and base64 encoding
  - No more false "tampered save" warnings after game updates
  - Save format is simpler and more robust

### Fixed
- **Roulette wheel state resets properly** when the game ends but not when pausing

### Technical
- Reorganized code structure: archetype systems (runes and roulette) moved into `wizard/archetypes/` module

## [v0.0.513] - 2025-02-10

### Fixed
- **King death detection now works reliably** — the game correctly ends when the King dies
  - Previously, the King sometimes wouldn't trigger defeat when killed
  - Fixed by ensuring the King becomes a corpse when health reaches zero
  - Simplified defeat check to directly look for a dead King instead of tracking spawn status
- **Behemoth now attacks at regular intervals** instead of attacking every single frame
  - Fixed attack timing system to properly record attacks and respect the global attack cycle
- **All units can now attack properly regardless of height differences**
  - Attack range is now measured using horizontal distance only (ignoring vertical position)
  - This fixes issues where tall units like the Behemoth couldn't reach ground-level enemies
  - Applies to all combat: melee attacks, archer melee, and Behemoth AOE attacks

## [v0.0.504] - 2025-02-10

### Added
- **Wizard type selection screen** after clicking "Begin, Wizard" on the main menu
  - Choose your wizard type before starting a new game (currently RuneCaster)
  - Enter a unique name for your wizard to identify your save
- **Multiple save files** — up to 3 separate saves, each with their own wizard, level, and progress
- **"Continue" button** on the main menu to resume a previous save
  - If you have one save, it loads directly
  - If you have multiple saves, a save selection screen lets you pick which one to continue
  - Delete saves you no longer want from the save selection screen
- Old progress from before this update is automatically carried over to your first save slot

### Fixed
- Fixed a bug where clicking a menu button could cause the wizard to cast a spell when the battle starts

## [v0.0.497] - 2025-02-09

### Added
- **Wizard's Tower progression screen** appears after winning a battle
  - Victory now takes you to the tower before starting the next level
  - New screen shows your current level and lets you prepare for the next battle
  - "Start Next Battle" button to continue your journey
  - "Return to Menu" button if you want to take a break
  - Defeats skip the tower and let you retry immediately at a lower level
  - Placeholder for future features like spell unlocking, upgrades, and rewards

### Changed
- Victory screen now shows "Continue" button instead of level progression text
- Defeat screen shows "Try Again (Level X)" to indicate immediate retry

## [v0.0.495] - 2025-02-09

### Added
- Custom Davidfont now used throughout the entire game for better readability
- Multi-word spell names now display across multiple lines in action bar (easier to read at a glance)

### Improved
- Increased font sizes across the board for better readability:
  - Action bar spell names are now 16px (up from 10px)
  - In-game buttons are now 28px (up from 24px)
  - Hotkey indicators are now 11px (up from 10px)
- Reduced padding in action bar buttons to give more room for spell names
- Spell names now scale intelligently based on the longest line rather than total characters
- Better spell name formatting (e.g., "Wall of Stone" displays as "Wall of" / "Stone" instead of three separate lines)
- Organized audio and font files into dedicated folders within the assets directory

### Fixed
- Fixed font and audio file paths to use the correct assets directory structure
- Fixed custom font not loading on the HTML loading screen

## [v0.0.467] - 2025-02-07

### Added
- Yarrow ingredient that heals your defenders over time while the brew is active
- Colorful bubble explosion effect when a brew finishes — the bubble's color is based on the ingredients used

### Changed
- Cauldron now uses an ingredient mixing system instead of fixed brews
- Pick individual ingredients (Lavender for mana regen, Mugwort for spell power, Yarrow for healing) and combine them into a custom brew
- Mixing multiple ingredients dilutes each effect, so you can't just throw everything in at once
- The cauldron menu shows a live preview of what your brew will do before you start brewing

## [v0.0.458] - 2025-02-07

### Added
- Cauldron brewing system with two brews: Mana Surge (doubles mana regen) and Empowerment (increases spell power by 50%)
- Cauldron menu accessible during gameplay to select and start brews
- Cast bar shows brewing progress with a grayed-out overlay while a brew is active
- Ability to cancel an in-progress brew from the cauldron menu

### Improved
- Brews now use a flexible effect system so each brew only defines the effects it cares about
- Cauldron systems are smarter about when they run, skipping unnecessary work when not brewing

### Fixed
- Fixed brews not starting when selected from the cauldron menu

## [v0.0.438] - 2025-02-06

### Added
- Loading screen now appears when starting a new level or replaying after game over
- All units and game objects now spawn smoothly one at a time during loading

### Improved
- Archers now properly fire arrows at enemies when standing still
- Arrow projectiles are now properly sized and visible
- Units no longer turn into corpse colors when other units die
- Loading happens in a smart order: battlefield, castle, grid, king, units, wizard
- Each level starts with a completely clean battlefield

### Fixed
- Fixed issue where most units weren't spawning during loading
- Fixed wizard not spawning, which prevented spells from working
- Fixed archers spawning in wrong positions on the defender grid
- Fixed archers unable to attack due to constant tiny movements
- Fixed missing King's Guards and other units
- Fixed material sharing bug that caused all units to appear as corpses when one unit died
- Fixed corpse materials now use separate pre-loaded assets instead of modifying shared materials
- Fixed undead units now use correct bright green color when resurrected

## [v0.0.392] - 2025-02-05

### Added
- Background music now plays throughout the game and loops continuously
- Music respects volume settings from the settings menu (master volume and music volume)
- Volume changes apply in real-time while music is playing

### Changed
- Default music volume set to 30% (was 80%)
- Start screen now requires clicking "Click to Start" button before loading the game (enables audio to work in browsers)

### Fixed
- Optimized several systems to reduce unnecessary processing and improve performance
- Fixed audio interruptions during state transitions (menu to game, game to menu)

## [v0.0.346] - 2025-02-05

### Added
- **New Spell: Squall**
  - Summons a storm that continuously rains ice down on a targeted area
  - Ice projectiles fall from the sky and explode on impact, dealing frost damage
  - Hit enemies are slowed by 40% for a few seconds, making it harder for them to reach your defenses
  - Storm persists as long as you maintain concentration (currently lasts until manually cancelled or another spell is cast)
  - Perfect for controlling choke points and slowing down waves of enemies
- **New Spell Mechanic: Concentration**
  - Squall is the first spell that requires concentration to maintain
  - While concentrating on a spell, it continues to have an effect on the battlefield
  - A new UI appears above the action bar showing which spell you're concentrating on
  - Click "End Concentration" to stop the spell early, or cast another spell to automatically end it
  - Only one concentration spell can be active at a time

### Changed
- Ice damage type renamed to Frost damage for consistency
- Behemoths now spawn every 3 levels instead of every level

### Fixed
- Improved spell casting controls - you can now cast spells repeatedly without needing to click twice
- All spells (Fireball, Guardian Circle, Squall) now work consistently when clicking multiple times

## [v0.0.325] - 2025-02-04

### Added
- **New Enemy: Behemoth**
  - A massive, slow-moving tank unit that spawns every level (currently for testing)
  - Has 10 times the health of regular units (500 HP)
  - Attacks with devastating area-of-effect damage, hitting everything within 30 units of its target
  - Does 200 damage per attack - enough to wipe out groups of defenders
  - Watch out - behemoths can accidentally damage their own allies with their powerful attacks!
  - Spawns in the archer row alongside other attackers

### Changed
- Reduced Finger of Death mana cost by 50% to make it more viable against tough enemies

## [v0.0.253] - 2025-02-04

### Added
- **Revolutionary Flow Field Pathfinding System**
  - Units now use intelligent pathfinding to navigate around obstacles
  - Defenders wait at their spawn positions until enemies get close, then move toward the King's target
  - Attackers flow smoothly toward the King, automatically avoiding walls and obstacles
  - Units path around fire, walls, and other hazards instead of walking through them
  - Smooth, coordinated unit movement that looks more natural and tactical
- Units now avoid fireball explosions and burning ground using the new pathfinding
  - Explosions create a danger zone that units strongly avoid (100x movement cost)
  - Burning ground left by fireballs slows movement significantly (50x movement cost)
  - Units will find alternate routes around fire when possible

### Changed
- **Completely overhauled unit movement system**
  - Units now blend three types of movement: pathfinding, targeting, and flocking
  - When enemies are far away, units follow pathfinding routes
  - When enemies are close, units prioritize direct targeting
  - Units always maintain spacing with flocking behavior to avoid clustering
- Improved Wall of Stone obstacle detection
  - Added a buffer zone around walls to prevent units from clipping corners
  - Units maintain better distance from wall edges when pathfinding
- The King now intelligently selects targets for defenders
  - Focuses on the closest living enemy
  - Ignores dead units and won't target other defenders

### Fixed
- Defenders now properly reset when returning to the game from the main menu
- Units no longer get stuck walking into walls or obstacles (kinda)
- Removed leftover debug code and cleaned up compiler warnings

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
