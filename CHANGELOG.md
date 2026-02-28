# Changelog

All notable changes to this project will be documented in this file.

## [v0.3.26] - 2026-02-28

### Added
- **CRT channel change effect** when pressing Escape to go back to the main menu from Settings, Progress, Changelog, and Instructions

### Fixed
- Splash screen images (Rust logo, Bevy logo, studio art) no longer pop in late — they now preload during the initial black screen so they appear instantly
- Volume no longer spams log messages when adjusting the slider

## [v0.3.21] - 2026-02-28

### Added
- **Skip Splash Screen setting** — you can now toggle off the startup splash screen in Settings under the Game section
- **Escape key navigation** — pressing Escape on the Instructions, Progress, and Changelog screens now returns to the previous menu, just like the Settings page

### Changed
- Settings, Instructions, Progress, and Changelog pages now all share the same consistent dark background with a subtle border
- Master volume now properly affects music volume (previously master and music volumes were applied independently)

## [v0.3.3] - 2026-02-27

### Added
- **Black hole visual overhaul** — black holes now look like a cinematic singularity with a flat black circle facing the wizard, a tilted accretion disk below it, and two pulsing whitish-red torus rings around each
- **Screen desaturation effect** — certain powerful spells briefly flash the screen to greyscale through the CRT filter
- **Finger of Death ground scorch** — the beam now leaves a glowing burn mark on the ground where it hits
- **Fireball ground scorch** — fireballs now leave a burning ground scar at the impact point
- **Disintegrate ground scorch** — the beam scorches the earth beneath it
- **Grease fire explosion VFX** — igniting a grease slick now triggers a fiery explosion with sparks, smoke, and heat shimmer
- **Meteor ground fire VFX** — meteor impacts now show flames, smoke wisps, and heat shimmer on the burning ground they leave behind
- **Wall of Fire VFX** — fire walls now have visible flames, rising smoke, and heat shimmer along their length
- **Spell icons for all spells** — every spell now has a unique icon that loads instantly in the spell book and action bar

### Changed
- Spell book buttons now show the icon on the left with the spell name to the right, instead of centered
- Black hole spawns higher above the ground so it's easier to see
- Wall of Fire pushes enemies away more strongly
- Defenders now return to their positions between waves instead of chasing the last enemy across the map

### Removed
- Removed Hypnotic Pattern and Phantasmal Force spells from the game

## [v0.2.381] - 2026-02-27

### Added
- **Magic missile sparkle trail** — magic missiles now leave a glowing comet-like trail of white sparkle particles that slow down and fade behind them
- **Magic missile glow** — magic missiles now have a pulsing purple glow halo that follows them through the air
- **Disintegrate beam impact particles** — visible orange sparks now spray outward from where the disintegrate beam hits the battlefield

### Changed
- Spell effects now use a flat, low-poly art style that better matches the game's 2D pixel art aesthetic — fireballs, explosions, black holes, ice shards, lightning rods, arcane crystals, spell shields, brew bubbles, ingredient drops, and beam spells all use intersecting flat planes instead of smooth 3D shapes
- Fireball explosions now appear slightly above the ground so they're easier to see
- The disintegrate beam's origin flare is now a circular glow instead of a square
- Improved flow field performance — recalculations are now queued and limited to one per frame, preventing lag spikes during intense battles
- Meteors no longer cause pathfinding recalculations on their own — units just walk through the small fire pools quickly
- Defenders no longer jitter in place when all enemies are defeated

## [v0.2.350] - 2026-02-27

### Added
- **CRT channel-change effect** — a TV-style flicker plays when transitioning between screens (splash screens, menus, starting/ending battles). Includes horizontal tearing, chromatic aberration burst, a rolling bright bar, and a brief flash
- **Splash screen sequence** — the game now shows three splash screens (Rust, Bevy, The Cult of David) with the channel-change effect between each one

### Changed
- Simplified the start screen — just a "Click to Start" button on a black background, styled to match in-game buttons
- Simplified the loading screen — just shows "Loading..." text
- Default spells (Magic Missile, Telekinesis) now correctly show as unlocked on the Progress screen

## [v0.2.330] - 2026-02-26

### Added
- **Splash screen** — the game now opens with a "The Cult of David" splash screen that fades in and out before the main menu

## [v0.2.317] - 2026-02-26

### Added
- **Spell icons** — Fireball and Magic Missile now show their icons in the action bar, spell book, and study screen instead of just text
- Updated the roulette wheel with a new, sharper image
- Hotkey numbers on the action bar are now larger and easier to read

## [v0.2.316] - 2026-02-26

### Added
- **Wave system** — enemies now arrive in multiple waves instead of all at once. Each level has 2 or more waves (increasing with tier), with 60 seconds between each wave. A wave counter in the top-right corner shows your progress
- **"Wave incoming!" alert** — a brief red flash appears on screen when a new wave of enemies is about to arrive, so you have time to prepare
- Defenders and the King now return to their starting positions between waves instead of standing around on the battlefield
- Notifications (achievements, ingredients, spell research) now appear in the top-right corner instead of the top-center to avoid overlapping the wave alert

## [v0.2.311] - 2026-02-26

### Added
- **Permanent walls** — Wall of Stone spells now persist between levels. Win a level and your walls carry over to the next fight, letting you build up fortifications over time. Lose and you keep the walls from your last victory
- Dispelling a permanent wall removes it for good

### Fixed
- Units no longer try to walk through walls when targeting enemies on the other side — they'll find a way around instead
- The King now properly avoids walls instead of trying to walk through them
- Placing walls close together no longer traps units in gaps between them
- Units no longer slide along wall edges and get stuck on corners
- Pathfinding now correctly updates for all units when walls are placed or removed
- Rapidly placing multiple walls no longer causes some walls to be invisible to pathfinding

## [v0.2.298] - 2026-02-26

### Added
- **Healer** — a new enemy support unit that heals nearby wounded attackers
- **Castle wall artwork** — the castle platform now displays a stone wall texture instead of a flat gray box

### Changed
- Repositioned the castle, wizard, and spell origin for a better view of the battlefield
- Cauldron is now larger and positioned next to the wizard
- Undead units now have a consistent purple color across all unit types
- Suppressed harmless cursor positioning warnings in the browser console

## [v0.2.253] - 2026-02-26

### Added
- **Brute** — a new heavy attacker unit that replaces the Behemoth, appearing in later tiers
- **Ogre** — the boss has been reworked into the Ogre with enrage phases, knockback attacks, and a dedicated health bar labeled "Ogre"

### Changed
- **Tier-based level progression** — levels are now grouped into tiers of 5. Unit counts, elite/commander chances, and dispeller spawns all scale within each tier instead of endlessly ramping up. Every 5th level is a boss-only fight
- Dispellers are now more visually distinct with a stronger blue tint
- Elite and commander enemies now appear based on tier progression instead of flat level thresholds

### Fixed
- Spell targeting and UI buttons near the edges of the screen now correctly line up with your cursor — previously the CRT screen curvature caused them to drift away from where you were actually clicking

## [v0.2.249] - 2026-02-25

### Added
- **King's health bar** — a vertical green health bar now appears on the right side of the screen so you can always see how your King is doing

## [v0.2.240] - 2026-02-25

### Fixed
- The scanlines and pixel grid now curve with the screen — previously they appeared as a flat overlay on top of the rounded CRT effect

## [v0.2.238] - 2026-02-25

### Added
- **CRT screen effect** — the game now has a retro TV look with barrel distortion, scanlines, RGB subpixel grid, vignette, chromatic aberration, screen flicker, rounded screen corners, and a subtle phosphor glow on bright areas
- **New pixel font** — switched to Press Start 2P for a retro arcade feel across all menus and in-game text

### Changed
- Adjusted text sizes throughout the game to fit the new pixel font — wizard select, spell book, cauldron, action bar, and in-game buttons all properly sized now
- Cleaned up leftover build files to reduce download size

## [v0.2.219] - 2026-02-25

### Added
- **Dispel spell** — a new utility spell for the wizard that fires a fast bolt of nullifying energy at the cursor. On impact with the ground, an expanding white wave removes any spell effects it touches
- Dispeller units now fire the same dispel projectile instead of silently channeling — you can see the bolt fly toward spell effects and watch the expanding wave clear them

### Changed
- Dispeller units no longer stand still to channel — they fire a projectile and keep moving

## [v0.2.206] - 2026-02-25

### Added
- **Dispeller units** — a new spell-disrupting utility unit that seeks out persistent spell effects and channels to remove them
  - When a spell effect is on the battlefield (walls, fire zones, spike growth, etc.), dispellers will pathfind toward it and channel for 3 seconds to dispel it
  - When no spell effects exist, they fall back to shooting weak magic bolts at enemies
  - **Attacker dispellers** — starting at level 6, the enemy army brings their own dispellers to counter your wizard's spells, scaling up over time

## [v0.2.205] - 2026-02-24

### Added
- **Boss battles** — every 5th level, instead of the usual army, you'll face a single massive boss. It's huge, tough, and gets angrier as it takes damage
  - **Enrage** — the boss speeds up and hits harder at 75%, 50%, and 25% health, turning visibly redder as it rages
  - **Melee knockback** — the boss's attacks send defenders tumbling across the battlefield, with a smooth sliding effect as they skid through the dirt
  - **Boss health bar** — a health bar with percentage appears at the top of the screen during boss fights
  - **Archers focus fire** — defender archers will now prioritize shooting the boss even when other enemies are in melee range
  - The boss can't be pushed around by other units

### Fixed
- **Disintegrate** now properly hits large units — previously you had to aim right at the center of big targets like the boss, now aiming anywhere on the sprite works
- Fixed duplicate spells appearing in the spell research tree
- Action bar moved back to the lower left corner of the screen

## [v0.2.191] - 2026-02-24

### Fixed
- **Teleport** source circle (blue portal) no longer gets stuck on the battlefield after casting
- **Teleport** timer completion now teleports units from the correct location instead of wherever your mouse was pointing
- **Teleport** now works properly when cast by the guest player in multiplayer

## [v0.2.189] - 2026-02-24

### Fixed
- **Chain Lightning** can now directly target the Arcane Crystal — previously you had to hit a nearby unit and hope it bounced to the crystal
- **Arcane Crystal** now properly auto-casts Disintegrate — the beam was invisible and not dealing damage after absorbing the spell

## [v0.2.186] - 2026-02-24

### Changed
- **Magic Missile reworked** — now fires a burst of 3 powerful homing missiles instantly on click instead of charging up and channeling. Each missile hits much harder than before. Short cooldown between casts

## [v0.2.182] - 2026-02-23

### Added
- **Multiplayer** — play against another wizard in real-time! One player hosts and the other joins using invite codes. Each wizard controls their own army and spells
  - **Lobby system** — host or join a game, pick your wizard, and ready up before the match starts. Both players can see each other's ready status
  - **Peer-to-peer connection** — games connect directly between browsers using WebRTC, no server needed. Supports both copy-paste invite codes and LAN connections
  - **Full spell sync** — Spell zones, explosions, walls, projectiles, and beams all appear on both screens
  - **Status effects sync** — burning, frozen, and electrified visual effects show up on both clients
  - **King's Spell Shield** — in multiplayer, each King is protected by a translucent barrier that blocks all spell damage until fewer than 10% of that King's defenders remain. This prevents instant wins from targeting the King directly with powerful spells

### Changed
- Loading screen updated with a custom font

## [v0.1.215] - 2026-02-19

### Changed
- **Wall of Fire** now acts as a zoning tool rather than a kill zone — walking through it does very little direct damage, but units still catch fire afterward. Duration reduced from 36 to 20 seconds
- **Grease Fire** no longer deals burst damage when ignited — instead it applies a mild burning effect to units passing through. The slippery slow effect now continues even while the grease is on fire
- Different hazard spells now influence pathfinding differently — weaker hazards like Wall of Fire are easier for units to walk through, while dangerous ones like Spike Growth are strongly avoided
- Fireball no longer leaves lingering ground effects
- Elite and commander enemies are now much rarer
- Team colors updated — defenders are light gray, attackers are dark gray, with subtle tints for unit types (archers lighter, elites reddish, commanders orange, King blue)
- King's aura is now a flat circle on the ground instead of a sphere
- King and commander auras are slightly more visible

## [v0.1.199] - 2026-02-19

### Fixed
- Units no longer move slower on faster computers

## [v0.1.198] - 2026-02-19

### Added
- **Arcane Crystal** — a new utility spell that absorbs incoming spells and re-emits smaller versions at nearby targets
  - Place a floating crystal on the battlefield that lasts 25 seconds
  - Hit it with any spell and it fires back mini versions at units in range
  - Fireballs become a volley of 5 mini fireballs
  - Disintegrate channels 5 lesser beams while you hold it on the crystal
  - Finger of Death fires a burst of 5 purple beams
  - Magic Missile splits into 5 homing mini missiles that target enemies
  - Meteor Fall launches mini meteors at nearby units
  - Chain Lightning arcs to multiple targets
  - The crystal remembers the last spell that hit it and automatically re-casts it on a timer — stronger spells fire less often, weaker ones fire rapidly
  - Disintegrate auto-cast channels a constant beam that tracks its target
  - Magic Missile is a priority target — aim near the crystal and missiles will fly to it
- Arcane Crystal is available in the Wizard Tower under Misc spells

### Improved
- Units move more organically across the battlefield instead of following rigid grid paths

### Fixed
- Magic missiles no longer orbit around their targets endlessly

## [v0.1.173] - 2026-02-18

### Added
- **13 new spells** — the wizard's arsenal has grown from 17 to 30 spells!
  - **Meteor Fall** — rain down fiery meteors that leave burning ground behind
  - **Mark of Death** — curse an enemy to take bonus damage from all sources
  - **Plague Wind** — summon a toxic cloud that drifts across the battlefield
  - **Hypnotic Pattern** — mesmerize all units in an area, freezing them until they take damage
  - **Sleep** — put units to sleep; the first hit deals bonus damage and wakes them
  - **Grease** — coat the ground in slippery grease that slows everyone down; hit it with fire to ignite it!
  - **Fog Cloud** — create a foggy area where units have a chance to dodge attacks
  - **Battle Hymn** — boost nearby units' damage and attack speed
  - **Healing Plume** — create a healing zone that regenerates health for anyone standing in it
  - **Berserker Rage** — units deal more damage but also take more damage
  - **Phantasmal Force** — summon illusory decoys that distract enemies
  - **Banishment** — temporarily remove an enemy from the battlefield
  - **Polymorph** — turn an enemy into a helpless sheep
- **Grease + Fire combo** — igniting grease now shows fire visually spreading from the ignition point across the pool
- Grease can be ignited by Fireball, Wall of Fire, Disintegrate, Meteor Fall ground fires, and even chain-ignites from other burning grease pools
- Spell Book now displays spells in a 4-column grid layout

### Fixed
- Sleep and other crowd control effects now properly stop archers from attacking (both melee and ranged)
- Spell modifiers (Mark of Death, Sleep, Battle Hymn, etc.) now properly expire after their duration
- End Concentration button no longer overlaps the Spell Book when it's open

## [v0.1.153] - 2026-02-18

### Added
- **Elemental damage effects** — spells now leave lasting effects on units they hit based on their element:
  - **Fire spells** burn units over time — the more fire hits, the stronger the burn
  - **Frost spells** slow units down — repeated hits stack the slow effect
  - **Electric spells** build up a charge that randomly arcs lightning to nearby units (friend or foe!)
- All elemental effects wear off after a few seconds if not refreshed
- **Visual feedback** — units affected by elemental effects now glow with the element's color:
  - Fire: pulsing orange-red
  - Frost: steady blue tint
  - Electric: flickering yellow-white
  - Multiple effects blend together when stacked

### Changed
- All units move slower across the board
- Electric arcs from charged units can now hit anyone nearby, not just enemies — watch your positioning!

## [v0.1.128] - 2026-02-17

### Added
- **Spell Research System** — you now learn new spells by studying them at the Wizard's Tower between battles instead of getting them from achievements
- **Arcane Insight** — a new currency earned after every battle (wins and losses!) based on how well you fight
- **Wizard's Tower overhaul** — the between-battle screen now shows a full spellbook with research progress for every spell
- Spells are organized into elemental chains — master one spell to unlock the next in its school
- Using a spell's element in battle gives double research speed for related spells
- Achievements now grant bonus Arcane Insight instead of unlocking spells directly
- The game over screen shows how much Insight you earned
- The progress screen now shows your research progress and Arcane Insight balance
- A "Spell Researched!" notification pops up when you complete a spell
- **Redesigned Spell Book** — the in-battle spell menu now has a two-column layout with spell details on the left and a categorized spell list on the right
- Spells are organized into categories: Offense, Control, Support, and Utility
- Click any spell in the list to see its full description, damage type, and casting instructions
- Assign spells to hotkey slots (1-5) directly from the spell detail panel
- **Return to Tower button** — when you lose a battle, you can now go back to the Wizard's Tower instead of only retrying or quitting

### Changed
- Disintegrate is now a Fire spell (was Force)
- Swapped the order of buttons on the Wizard's Tower — "Start Next Battle" is now on top

### Fixed
- Fixed the cast bar turning black and mana bar stopping after brewing a potion
- Fixed the Wizard's Tower showing "Level 1" instead of your actual level when first loading the game

## [v0.1.100] - 2026-02-17

### Added
- **New spell: Telekinesis** — use this spell to pick up glowing items dropped by fallen enemies on the battlefield
- **Ingredient drops** — enemies now occasionally drop ingredients when they die; use Telekinesis to grab them!
- Telekinesis is available from the start alongside Magic Missile

### Changed
- Brew ingredients are no longer unlocked randomly after winning battles
- You now discover new ingredients by collecting drops during battle — choose between casting offensive spells or grabbing ingredients!
- Drops persist on the battlefield until the end of the level
- A notification pops up when you collect a new ingredient
- Enemy commanders now have a visible aura circle beneath them
- Fewer elite enemies and commanders spawn at all levels

## [v0.1.86] - 2026-02-17

### Added
- **New spell: Lightning Rod** — place a metal tower on the battlefield that attracts lightning strikes every few seconds, sending arcs of electricity to all nearby units (watch out — it hits your troops too!)

### Changed
- Lightning Rod is now unlocked by the **Chain Reaction** achievement
- **Squall** is now unlocked by the **Archmage** achievement (25 wins) instead

## [v0.1.84] - 2026-02-16

### Improved
- **Smoother pathfinding** — units now navigate around obstacles more precisely
- **Elite enemies are darker** instead of brighter, making them look more menacing
- Units are better at avoiding hazards and obstacles on the battlefield
- Fewer elite enemies appear in the early levels

## [v0.1.78] - 2025-02-16

### Added
- **Elite enemies** — some attackers are now tougher, faster, and stronger (bright red infantry, hot pink archers)
- **Enemy commanders** — rare powerful units with golden armor that make nearby enemies stronger and faster
- Elites become more common as you reach higher levels
- Commanders appear starting at level 5 and become more frequent at higher levels

### Improved
- The King's Guard are now elite warriors with enhanced combat abilities
- Each level now feels more unique due to random enemy variety
- Higher levels are more challenging through stronger enemies, not just more enemies

## [v0.1.64] - 2025-02-14

### Added
- **4 new spells** to unlock through achievements:
  - **Wall of Fire** — drag to draw a line of fire that burns enemies who walk through it
  - **Entangle** — roots enemies in place so they can't move for a short time
  - **Haste** — speeds up your defenders so they can get into position faster
  - **Spike Growth** — creates a thorny zone that damages and slows all units inside it
- 4 new achievements to unlock the new spells — check the progress screen for hints!

### Improved
- **Wall of Fire is now a solid line** instead of a row of disconnected circles
- Wall of Fire is bigger — 50% longer and thicker than before
- Units are smarter about avoiding dangerous areas like Wall of Fire and Spike Growth
- Archers will now move out of fire and spike zones even if they were standing still shooting

## [v0.1.52] - 2025-02-13

### Added
- **7 new cauldron ingredients** to discover — experiment to find out what they do!

## [v0.1.50] - 2025-02-13

### Added
- **Ingredient discovery system** — new ingredients are randomly unlocked after completing levels
- When you unlock a new ingredient, it's highlighted in green on the Wizard Tower screen
- You now start with only Lavender unlocked; other ingredients must be discovered through gameplay

### Changed
- **Most spells are now locked at the start** — you begin with only Magic Missile available
- Other spells unlock automatically as you earn specific achievements
- Locked spells and ingredients show mysterious hints instead of their names on the progress screen
- The cauldron brewing menu now only shows ingredients you've discovered
- Clearing your progress resets both spell and ingredient unlocks back to the starting set

## [v0.1.30] - 2025-02-13

### Improved
- Magic Missile now targets enemies near your mouse cursor instead of shooting randomly
- Aiming your cursor at specific enemies makes Magic Missile much more effective
- The spell still has some randomness, but heavily favors enemies you're pointing at

## [v0.1.24] - 2025-02-13

### Improved
- Game now loads almost 3 times faster in your browser
- Download size reduced by 66% - from 67 MB to just 23 MB
- Much better experience on mobile devices and slower internet connections
- Optimized game files without affecting any gameplay or visuals

## [v0.1.23] - 2025-02-13

### Changed
- The cauldron now pulses and glows with different colors while brewing potions
- Cauldron brewing animations are now smoother and more subtle

## [v0.1.6] - 2025-02-12

### Changed
- The cauldron now displays an animated bubbling sprite instead of a plain black circle
- The cauldron sprite faces the camera and stays upright regardless of viewing angle

## [v0.1.0] - 2025-02-12

### Added
- **"QWER" achievement** — press Q, W, E, or R during a battle to unlock the Rune Caster wizard type
- **"Random Magic Surge" achievement** — every time you cast a spell, there's a 1 in 100 chance this triggers and unlocks the Randomancer wizard type
- **Wizard types are now unlockable** — only the Boring Ole Mage is available at the start; Rune Caster, Randomancer, and Arcanorouter must all be earned through achievements
- Locked wizard types on the select screen now show only a mysterious hint instead of their name

### Changed
- Clearing progress now properly resets all achievements and locks wizard types again
- Progress screen lists now show unlocked items at the top

## [v0.0.611] - 2025-02-12

### Added
- **Boring Ole Mage** — a new starter wizard archetype with a 5% bonus to all stats, now the default wizard for new players
- **20 new achievements** across two categories:
  - **Victory & Progression** — win milestones (5, 10, 25, 50, 100, 200 wins), level milestones (10, 25, 50, 100), and retry milestones (5 and 15 retries on the same level)
  - **Defeat & Failure** — lose your first battle, king dies, total wipe, lose in under 30 seconds, kill 90% of attackers but still lose, no deaths for 2 minutes then lose, kill 10 defenders with spells, kill the king with a spell
- **Lifetime kill stats** — attackers killed, defenders lost, and undead killed are now tracked across all battles and shown on the game over screen and progress screen
- Total wins and games played are now tracked across sessions
- **"Slider Fiddler" achievement** — unlocked by adjusting any slider in the settings (volume or brightness)
- **Arcanorouter is now a hidden wizard** — must be unlocked by earning the "Slider Fiddler" achievement

### Changed
- **Wizard select screen redesigned** — archetypes now appear in a grid with room for up to 16 wizard types, with a detail panel showing full descriptions
- **Progress screen** — categories are now side-by-side columns that scroll independently
- Locked items on the progress screen now show flavor text hints instead of their names

## [v0.0.586] - 2025-02-12

### Added
- **Achievements** — earn achievements for completing milestones during gameplay
  - "First Victory" — win your first battle
  - "Friendly Fire" — accidentally kill one of your own defenders with a spell
  - A gold popup appears near the top of the screen when you unlock an achievement (fades after 5 seconds)
  - Achievements are saved and won't trigger again once earned
- **Progress screen** — new menu accessible from both the main menu and pause menu
  - View all achievements and whether you've earned them
  - See all unlockable spells, ingredients, and wizard types
  - "Clear Progress" button lets you reset all achievements and unlockables

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
