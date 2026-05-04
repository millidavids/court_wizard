# Changelog

All notable changes to this project will be documented in this file.

## [v0.7.304] - 2026-05-04

### Changed
- **Ray now has a beating heart at its core** — Ray's central body is no longer a plain green disc. It now shows a hand-drawn beating heart sprite that pulses continuously throughout the fight, no matter which direction Ray is moving. The five orbiting eyes still glow in their signature colors (orange, purple, pink, cyan, stone-gray), but the heart itself shows the artwork as-drawn with no color tint over it

## [v0.7.302] - 2026-05-03

### Changed
- **All four starter spells available from day one** — new players now begin with Magic Missile, Entangle, Guardian Circle, and Telekinesis already unlocked. You no longer need to research the two control and support roots separately — every spell tree has its starting spell ready to go from the very first battle
- **New wizards spawn with their action bar pre-filled** — when you create a new wizard, hotkey slots 1 through 4 are already set to Magic Missile, Entangle, Guardian Circle, and Telekinesis. You can jump straight into your first battle without opening the spell book first
- **The Shepherd can now cast Telekinesis** — Telekinesis was previously off-limits for the Shepherd. Since it pushes things around rather than dealing direct damage, the Shepherd now gets to use it alongside her other non-violent tools

## [v0.7.299] - 2026-04-30

### Added
- **Hand-drawn Dark Mage sprite** — the Dark Mage boss is no longer a flat colored shape. He now has a proper animated sprite with a hooded silhouette and flowing robes
- **Dark Mage floats above the battlefield** — like the Lich, he now hovers slightly off the ground with a gentle bob
- **Dark Mage casting animation** — during the red-circle telegraph wind-up before each spell fires, the Dark Mage swaps to a casting pose so you can see him channeling
- **Dark Meteor uses the proper fireball explosion** — when his meteor lands, the impact now plays the same fiery explosion you see from your Fireball spell — a glowing sphere, sparks, smoke plumes, heat shimmer, and procedural fire particles — instead of a flat orange circle on the ground
- **Dark Mage teleport sound and visuals** — when the Dark Mage blinks away after you close in on him, you now hear a teleport whoosh and see swirling purple energy rings at both his old and new positions, matching the look of your own Teleport spell

### Changed
- **Dark Mage now appears at level 15** — he used to show up at level 20. Hags now appear at level 20 instead of level 15. The boss order is now Ogre → Lich → Dark Mage → Hags → Ray every five levels
- **Dark Mage stays on screen when he teleports** — previously he could occasionally blink to a spot behind the camera or off the playable area. He now only picks teleport destinations that are visible on your screen

## [v0.7.294] - 2026-04-30

### Changed
- **Behind-the-scenes cleanup** — this update is a large internal reorganization of the game's code. Hundreds of files were broken up into smaller, more focused pieces and a lot of unused leftover code was removed. There are no gameplay changes — this groundwork makes future updates faster to ship and bugs easier to track down

## [v0.7.88] - 2026-04-29

### Added
- **Smaller, smarter notification toasts** — the popup that appears when you discover an ingredient, research a spell, brew a new combo, or unlock a new wizard is now a compact card in the top-right corner instead of a large banner. Toasts are shorter, less frequent, and more descriptive
- **Talent tier unlock toasts** — when one of your spells crosses a talent threshold (Apprentice, Adept, or Master), a gold notification tells you the tier just opened up so you know to visit the Wizard Tower and pick a new talent
- **First-time wizard toasts** — unlocking a brand new wizard archetype now shows its name and tagline as a purple toast. (Already-unlocked wizards re-triggered by an achievement no longer pop up redundantly)
- **Teleporter death animation** — the teleporter enemy now plays a 6-frame death animation when it falls, matching the other troops
- **Undead rise from corpses** — when Raise Dead, the Lich, or Lichdom (Finger of Death) raises a corpse, the body now visibly stands up by playing the death animation in reverse instead of popping upright instantly
- **Save backup when Clearing Progress** — Settings → Clear Progress now keeps one rollback backup of your previous save next to the active save file. The Player README in the game folder explains where to find it and how to restore from it if you change your mind
- **Endless tab "Start" button on fresh wizards** — for a wizard who hasn't beaten Level 1 yet, the right-hand button now says "Start" instead of "Continue Playing"

### Changed
- **Achievement popups removed from in-game** — earning an achievement no longer shows a popup over the battlefield. Steam's built-in achievement overlay handles the notification cleanly, so there's no longer two notifications fighting for the same moment
- **Clear Progress fully resets in the same session** — clearing progress now also wipes the in-memory achievement state immediately, so you no longer need to restart the game to see a true fresh start

### Fixed
- **Score screen no longer freezes after a battle** — finishing a level was sometimes leaving the score screen stuck in a broken state where buttons couldn't be clicked, kills kept counting up forever, and the game appeared frozen. The win/lose check is now confined to active gameplay, so the score screen settles cleanly the first time and stays interactive
- **Beating one level no longer skips ahead** — winning a level was occasionally advancing you two levels at once with the same stats recorded for both. Now each victory advances exactly one level
- **Lifetime kill totals stay accurate** — the per-battle kill counter is now reset at the start of every battle no matter how you got there. Previously, certain entry paths (like starting a roguelite run from the tower) carried over the previous battle's kills, which then doubled or tripled the lifetime totals shown on the score screen

## [v0.7.74] - 2026-04-29

### Changed
- **Streamlined in-game Changelog tab** — the Manual's Changelog tab now shows only the 5 most recent updates instead of the entire history. A new "View full changelog on the web" button at the bottom opens the website where you can browse every release going back to the start

## [v0.7.72] - 2026-04-29

### Added
- **Hand-drawn Lich sprite** — the Lich boss is no longer a flat colored ellipse. He now has a proper pixel-art portrait in the Compendium and a full animated sprite in battle, complete with hooded skeleton face, flowing robes, and skeletal hands
- **Lich floats above the battlefield** — the Lich now hovers slightly off the ground with a gentle bob, befitting an undead sorcerer
- **Lich casting animation** — when channeling Raise Dead or charging up Finger of Death, the Lich plays a casting animation with a glowing magenta orb in his hand so you can see the spell coming
- **Telegraph before Raise Dead** — the Lich now visibly winds up for 3 seconds before each undead wave appears, giving you a window to interrupt or reposition

### Changed
- **Lich is much tankier against melee** — his melee damage reduction increased from 70% to 85%, so swords and bows tickle him while spells remain effective
- **Lich must close on the King to use Finger of Death** — he now has to walk within 800 units of the King before he can fire the death beam, instead of sniping from across the map
- **King is the Lich's last priority** — the Lich won't aim Finger of Death at the King until every other defender (including the King's Guard) has fallen
- **King is immune to Finger of Death until late** — even if the beam clips through the King while targeting another defender, he ignores the damage entirely until 80% of your defenders have died. After that he takes the existing 30% damage
- **Lich hitbox now matches his sprite** — the collision/targeting cylinder around the Lich was tiny relative to his rendered size, so swords and arrows often missed visually-clear hits. The hitbox now wraps the visible figure so attacks register where you'd expect

### Fixed
- **Killing the Lich is a Victory, even in a tight finish** — if the King or last defender fell in the same exchange that killed the Lich, the level was being recorded as a Defeat. Slaying the boss now resolves the level as a Victory regardless of who else dies in the final blow
- **Magic Missile no longer double-casts on hold** — holding the mouse / right-trigger to auto-cycle Magic Missile was firing two volleys in quick succession before settling into the cooldown rhythm. The cooldown gate is now airtight: only one volley per cooldown window, regardless of frame timing
- **Lich waits for the last wave to fully die** — the Lich was occasionally spawning the same instant the final wave dispatched, before its attackers had even materialized. He now correctly waits until every attacker from the last wave is dead before he appears
- **Lich faces the camera while approaching defenders** — the Lich's sprite was showing his back-of-hood whenever he walked toward the King. He now correctly shows his face when moving toward the camera and only flips to the back-facing sprite when retreating into the screen

## [v0.7.60] - 2026-04-28

### Added
- **Controller bindings diagram in Settings** — the Controls tab in Settings now shows a quick reference card at the bottom listing every gamepad binding (Aim, Cast Spell, Cancel Cast, Pick Spell, Confirm, Back, Spell Book, Cauldron, Cycle Tabs, Pause) with the actual button icons for your controller. The diagram automatically switches between Xbox, PlayStation, Switch, and Steam button artwork based on which gamepad is plugged in, and falls back to Xbox if none is connected

### Changed
- **No more controller-button popup tutorials** — the two pop-up walkthroughs that explained gamepad navigation on your first Wizard Tower visit and your first battle have been removed. The new bindings diagram in Settings serves the same purpose without interrupting play

## [v0.7.50] - 2026-04-28

### Changed
- **Code-quality cleanup** — an internal pass removed dead code and unused imports and tidied formatting across the project. No effect on gameplay

## [v0.7.49] - 2026-04-28

### Added
- **Native Linux build** — Court Wizard now runs on Linux alongside Windows and macOS, so players on Linux desktops can launch the game directly without any workarounds
- **Updated credits screen** — the credits now list the original artwork (spell icons, static sprites, and the menu backdrop) and the fonts used in the game (Press Start 2P, Oxford Runes), with attribution to their authors

## [v0.7.48] - 2026-04-27

### Changed
- **Cleaner project layout** — supporting documentation files have been tidied into a dedicated folder. No effect on gameplay or the game itself

## [v0.7.47] - 2026-04-27

### Added
- **Unit portraits in the Compendium** — every unit now has its own portrait in the detail panel. Bosses are rendered larger to match their stature, defenders show off their colors, and the various attacker variants (Brute, Elite, Commander, etc.) are tinted to make them easy to tell apart at a glance
- **Silhouettes for locked entries in the Compendium** — units you haven't fought yet now show as a black silhouette with a soft white glow behind them, so you can see the shape and size without spoiling what they look like up close. The same treatment applies to locked spells, which previously showed an empty space in the detail panel

### Fixed
- **Compendium opens with content showing immediately** — previously, opening the Compendium left the right panel empty until you switched tabs and came back. The list now appears straight away on first open
- **A few enemy types now mark as encountered** — Shielder, Assassin, and Aerialist were being missed by the encounter tracker, so they stayed silhouetted in the Compendium even after you'd seen them in battle. They now light up after their first appearance

## [v0.7.38] - 2026-04-27

### Changed
- **Updated to the latest game engine** — Court Wizard has been moved onto the newest version of its underlying engine. This is an under-the-hood refresh that lays the groundwork for future features and brings in upstream performance and stability fixes
- **Quieter screen filters** — the CRT, gravitational lensing, heat distortion, teleport distortion, color-blind correction, and high-contrast filters all do a tiny bit less setup work each frame, freeing up a sliver of performance for the rest of the game
- **Refreshed support libraries** — random number generation, networking, and several other supporting libraries were updated to current releases at the same time so nothing falls behind

## [v0.7.37] - 2026-04-27

### Added
- **Hand-drawn Hag sprites** — Justina, Martina, and Josephina are now full sprite-sheet characters instead of flat colored shapes. They walk, turn, and face the right direction while moving toward the king
- **Floating eye icons above the Hags** — the Invulnerability and Ability eyes now appear as small pulsing orbs hovering over the Hag who currently holds them, matching the eye look on Ray
- **Josephina swing animation on melee hits** — every time Josephina lands a melee attack she plays a quick swing animation
- **Casting animation on Hag spells** — Justina's chain lightning, Justina's fireballs, and Martina's teleport pull all play a casting animation when they fire
- **Josephina leap pose** — Josephina now visibly winds up while flying through the air and slams down on landing, instead of flat-walking through the leap
- **Color-matched Hag overlays** — each Hag's sprite has a subtle hue (orange / purple / green) that matches her health-bar segment so you can immediately tell who's who at a glance

### Changed
- **Hags walk to staging like normal attackers** — all three Hags now path through the tunnels to the center staging point during the buildup phase. They no longer wander aimlessly or use any eye abilities until the wave activates
- **Hags spread out on spawn** — the three Hags now spawn at staggered depths behind the wall instead of two of them landing on top of each other in the same tunnel
- **Justina kites at range** — Justina now stops advancing once she's within 800 units of the king and fights from there with chain lightning and fireballs instead of charging into melee
- **Justina's fireball blasts are bigger** — the explosion radius is roughly tripled, so the fireballs feel like proper boss spells
- **Martina only casts teleport pull near the king** — she has to be within 800 units of the king to teleport defenders to her position; otherwise she walks closer first. Once she casts, any defender on the map can be yanked
- **Martina's mind control aura is wider** — slightly bigger pull radius
- **Hag movement speed doubled** — the Hags close distance noticeably faster
- **Hag damage resistance doubled** — Hags now soak twice as much damage from incoming hits
- **Hag separation tightened up** — Hags push each other apart more firmly when they're about to overlap so their sprites no longer clip through each other when bunched up
- **Brighter, more obvious mind-control overlay** — units under mind control now glow a stronger, more saturated magenta so it's easy to spot who's been turned
- **Smaller, higher-floating Hag eyes** — the Invulnerability/Ability eye icons are now smaller and float higher above each Hag's head
- **Josephina is green on her health bar** — the Josephina segment swapped from brown to green so it's no longer easy to confuse with Justina's orange

### Fixed
- **Hags no longer clip into the ground** — they now sit cleanly on top of the battlefield instead of being half-buried
- **Hag sprites no longer rapid-flicker between facing directions** — when a Hag moves at a 45° angle there's now a buffer zone before her facing animation can flip, plus smoothing on the velocity used to pick the animation. No more vibrating between "walking down" and "walking left"
- **Hags walk toward the camera with the correct sprite** — previously they showed the back-facing sprite while moving toward the camera and vice versa; the rows are now mapped correctly
- **Eye transfers don't start during staging** — the Invulnerability and Ability eyes now stay on the Hags they were assigned to until the wave activates, so they no longer shuffle around during the buildup
- **Blind Hag walks straight at the king** — the eyeless Hag no longer drifts randomly; she follows the same path as the others, just without abilities
- **Josephina no longer floats higher each time she leaps** — if her Ability eye was yanked mid-air, she previously got stuck at whatever height she was at and the next leap stacked on top of that. She now drops back to ground height every time the leap ends, no matter how it ends
- **Hags approach the king on activation** — when a wave activates two Hags would sometimes be flung outward by the separation force instead of moving toward the king. The separation push is now scaled correctly so flow-field guidance toward the king dominates

## [v0.7.8] - 2026-04-24

### Added
- **Per-tab tutorials in the Wizard Tower** — opening Roguelite, Endless, or the Study for the first time now triggers a focused walkthrough for that specific tab, instead of one generic tower intro. Each tutorial covers what's actually on screen: modifier sliders, wizard select cards, time travel, the spell web, and so on
- **Wizard select tutorial** — the first time you click "Switch Wizard" you get a short walkthrough of the wizard cards panel
- **Spell-selected tutorial in the Study** — clicking a locked spell node for the first time triggers a short tutorial of the +/− Insight buttons and the talent tree. It won't fire on spells you already have unlocked (like Magic Missile)
- **Mouse + keyboard menu primer** — first-time tower entry on mouse and keyboard now shows a brief navigation primer, mirroring the controller version
- **Pulsing golden highlights on the thing the tutorial is talking about** — when a tutorial mentions the action bar, the tab row, the time travel list, the spell web, the +/− buttons, talents, or other UI, that element pulses with a gold border so you know exactly what's being referenced
- **Inline controller glyphs in tutorial text** — sentences like "Press [A] to select" now render the actual controller button icon for whichever controller you're using (Xbox, PlayStation, Steam Deck, Switch). The keyboard/mouse versions of the same tutorials swap to plain wording
- **Tutorial queue** — when more than one tutorial wants to play at the same time (e.g. opening a tab on the very first tower visit while the controller primer is still running), they now play back-to-back instead of the second one being silently dropped
- **Warglock guns have icons in the action bar** — each of the five guns now uses its own dedicated icon (assault rifle, magnum, shotgun, rocket launcher, flame thrower) rather than text names that overflowed the buttons
- **Lighter Warglock action bar background** — the slot backgrounds are noticeably lighter when playing as the Warglock so the dark gun icons stay easy to read
- **Kenney credit added** — the controller-button glyph fonts (Xbox / PlayStation / Steam Deck / Switch) are by Kenney and are now credited in the in-game Credits panel

### Changed
- **Roguelite mode boots straight into Gameplay settings** — the settings menu now defaults to the Gameplay tab instead of the Video tab
- **Spell names removed from action bar buttons** — the icons identify each spell already, and long names like "Crescent Strike" or "Forged in Fire" were spilling out of the buttons
- **Rocket Launcher moved to slot 4** — the Warglock's gun order is now Machine Gun, Magnum, Shotgun, Rocket Launcher, Flamethrower
- **Rocket Launcher rebalanced** — direct hit damage cut to a quarter of what it used to be, and the explosion now uses the same blast as a Fireball cast so they look and feel consistent
- **Flamethrower flames look like real fire and smoke** — both the in-flight projectiles and the burning patches they leave behind on the ground now use the same billowing fire-and-smoke effect as Wall of Fire and Grease ignition
- **Tutorial highlight animation simplified** — the gold border now stays a steady size and just pulses brighter and dimmer, instead of also growing inward
- **Tutorial windows position themselves so they don't cover what they highlight** — talking about the left panel? Window jumps to the right side. Talking about Time Travel at the top? Window drops to the bottom. Talking about the tabs? Stays centered so it doesn't block them
- **Action bar starts in the right layout immediately** — starting a run while a controller is plugged in now spawns the radial action bar straight away on frame 1, instead of starting linear and visibly morphing to radial. Same the other way around for mouse and keyboard
- **Down/Up navigation in menus** — the controller can now press Down to jump to the next row even when the columns of two rows don't line up exactly (e.g. settings sliders with mismatched widths)
- **Basic D-pad / stick navigation tutorial removed** — moving the highlight with D-pad and selecting with the bottom face button is intuitive enough that the controller primer no longer spends two steps explaining it. It still covers Back, bumpers for tabs, and Start to pause
- **Settings tab labels** — "Video" → "Gameplay" is the new default, and tabs now read in a more useful order

### Fixed
- **Warglock guns can now be fired and selected with a controller** — both the radial spell picker and the right trigger now work for the Warglock just like every other wizard. The right stick highlights a gun, the right trigger primes it, and pulling the right trigger from a centered stick fires the currently equipped gun
- **Gold radial-hover highlight is visible on the Warglock** — the slot the right stick is pointing at now actually lights up gold; previously the per-frame gun-selection highlight was overwriting it
- **Action bar buttons no longer have a giant outline around a tiny icon** — when starting a run on controller and switching to mouse, the outer button outline used to grow to mouse size while the inner content stayed at controller size. They now stay in lockstep through the morph
- **Button tops no longer get clipped** — the hotkey number and icon stack now fits cleanly inside each button without the top of the icon being cut off
- **Action bar no longer flickers / constantly re-animates** — the layout system now skips per-frame mutations once the morph has settled, so the action bar isn't constantly re-rendering itself
- **Clicking a spell in the Spell Book actually primes it now** — the Spell Book click was writing a "prime spell" message that nothing was listening for during the menu state. It's now picked up so clicking a spell primes it and keeps the menu open as intended
- **Reset Tutorials and Clear Progress popups now work with a controller** — the Confirm and Cancel buttons are reachable, and pressing Back / Circle dismisses the popup like Cancel
- **Tutorial windows on the Study no longer leak input to the spell web behind them** — the spell-web cursor and pan/zoom are paused while a tutorial is on screen
- **Spell-web cursor disappears entirely on mouse and keyboard** — it's a controller-only affordance and was previously visible in the corner during mouse play
- **Tutorials that don't match your input device dismiss themselves** — switch from controller to mouse mid-tutorial and the controller-flavored overlay dismisses cleanly so the keyboard-flavored counterpart can fire instead
- **Tab bumpers / clicking tabs are blocked while a tutorial is open** — prevents accidentally swapping tabs mid-walkthrough and triggering the wrong tutorial
- **Wizard select grid scrolls with the mouse wheel** — previously only the controller could navigate the wizard cards
- **Tutorial controller-glyph text size and ordering** — controller glyphs in tutorial messages render at a much larger, legible size; the Next button is now on the left of the popup with default focus, Skip is on the right
- **Crash when rebuilding a tutorial overlay during a state transition** — the overlay rebuild no longer panics when its host entity is being despawned in the same frame

## [v0.6.366] - 2026-04-23

### Added
- **Full controller support for the whole game** — every menu, every spell, every button can now be driven from an Xbox, PlayStation, Steam Deck, or Nintendo Switch controller. The game detects which controller is plugged in and swaps button prompts across the UI to match (A / Cross / Square)
- **Animated action bar that reshapes for your controller** — when you pick up a controller the horizontal spell bar smoothly morphs into a compact radial ring positioned near your wizard, then morphs back into the familiar row if you touch the mouse or keyboard again
- **Right-stick spell picker with a press flash** — flick the right stick in any of five directions to highlight a spell slot, then pull the right trigger to prime it. The chosen slot flashes yellow so you know the spell was actually committed
- **Brand-new controller tutorials** — plugging in a gamepad for the first time now triggers a menu-navigation primer (focus movement, select, back, tab cycling, pause) and an in-game combat primer (aim, radial spell picker, casting, spell book, cauldron)
- **Gamepad button glyphs on on-screen prompts** — the "Press SPACE to spin" prompt on the Randomancer wheel and the Q/W/E/R rune labels now show the correct controller glyphs (D-pad arrows, face buttons) instead of keyboard labels while you're playing on a controller
- **Controller glyph style setting** — pick Auto / Xbox / PlayStation / Steam Deck / Nintendo in the controls menu if you want to override the default detection

### Changed
- **Spell book stays open when you pick a spell** — clicking a spell now only primes it and keeps the menu open, so you can immediately drop it into an action-bar slot without reopening the book. Use the Back button or B / Circle to close when you're done
- **Magic Missile is a bit beefier per shot** — cooldown bumped from a third of a second to a full second. Hold the cast button and missiles keep flying at that new cadence
- **Chain Lightning takes a bit longer to cast** — its cast time doubled so you can see more clearly what you're about to unleash
- **Every charge spell gets a short breather after firing** — a one-second cooldown between casts so holding the cast button charges the next spell instead of slamming them out back-to-back
- **Menus always run at normal speed** — during the big staging speed-up before a wave, opening the spell book, cauldron, pause menu, or settings now drops the game clock back to 1× so the menu cursor isn't zooming around at triple speed

### Fixed
- **Magic Missile double-casting on hold** — holding the cast button no longer fires two volleys in quick succession before settling into the cooldown rhythm
- **Teleport no longer auto-advances from placement to origin circle** — phase one finalizes when you release the cast button, and phase two only begins on a brand-new press. No more accidental teleports from holding the button down
- **Score screen + pause menu lock controller focus** — navigating the score screen or the pause menu with a controller no longer leaks clicks through to the gameplay HUD buttons sitting behind the overlay
- **"Switch Wizard" no longer crashes the game** — pressing Switch Wizard in the tower is safe on first entry
- **Action bar no longer flickers on the score screen** — swapping between controller and mouse while looking at the score screen used to keep flipping the action bar between its linear and radial layouts in the background. It now stays put

## [v0.6.284] - 2026-04-17

### Added
- **Steam Leaderboards for Endless runs** — die in Endless mode and your final level lands on a global leaderboard. Your best run is always kept, so a worse run later won't knock you down
- **Steam Leaderboards for Roguelite runs** — every roguelite run submits to one of two boards when it ends. Runs with all modifier sliders at 100% and no toggles active go to a "pure" leaderboard for fair head-to-head competition; any modified run goes to a separate "modified" leaderboard where your slider and toggle choices factor into the score
- **Steam Leaderboards for Roguelite clear times** — beat the level 25 boss and your total time from level 1 through the finale lands on one of two clear-time boards (pure or modified). Faster times rank higher
- **Run conditions attached to each leaderboard entry** — every roguelite submission carries along the exact modifier slider values and toggle combination used for that run

## [v0.6.283] - 2026-04-17

### Added
- **Ponds react to your magic** — throwing fire, frost, or lightning at a pond now has real consequences on the battlefield instead of splashing harmlessly
- **Fire evaporates water into a fog cloud** — shoot a Disintegrate beam or land a Fireball on a pond and a pale fog cloud grows above it. The more fire damage you pump in, the bigger the cloud, up to roughly three pond-widths across. Single beam drips it in slowly; a full Fireball adds a big chunk all at once. The cloud lingers for ten seconds after your last shot before it dissipates
- **Frost freezes ponds solid** — repeated Squall hits turn a pond lighter blue and make it behave as ice. Units crossing a frozen pond move noticeably slower, and your defenders and attackers alike will route around it more aggressively. Ponds thaw naturally a few seconds after the cold stops
- **Electric damage supercharges a pond** — any lightning-based hit near a pond charges the water, and a charged pond then arcs lightning of its own to nearby units in a radius much larger than a normal shocked unit would reach. Pond shock lasts around six seconds after the last hit
- **Chain Lightning can jump to ponds** — ponds are now valid bounce points for Chain Lightning, letting you rope them into your chains and then use their much-larger shock radius to zap distant targets
- **Lightning Rod arcs to nearby ponds** — when your Lightning Rod fires, visible lightning arcs also jump to any ponds in range, triggering their pond shock and turning them into secondary damage emitters
- **Boulders catch fire and explode** — sustained fire damage heats a boulder up, and once it absorbs enough heat it detonates with an explosion twice the size and damage of a regular Fireball, then vanishes from the battlefield instantly instead of sinking. Heat slowly bleeds off if you stop pouring fire on it
- **Frozen ponds still take electric damage bonuses** — wet units in unfrozen ponds continue to take extra damage from electric arcs; a pond that's currently frozen stops making units wet until it thaws

## [v0.6.274] - 2026-04-17

### Added
- **A new final boss awaits at level 25** — a floating, many-eyed horror has joined the roguelite boss rotation as your capstone challenge, with five distinct eye beams that each inflict a different ordeal on your defenders
- **Five-part eye health bar** — the new boss's health is split into five sections, one per eye, each with its own color to match the beam it fires
- **Stone-cold stares and mass panic** — two brand-new status effects let the new boss petrify units in place or make them flee in terror
- **Bigger, brighter eye glow** — the new boss's eyes now shine in more vivid, saturated colors (warm gray, orange, purple, pink, and cyan) so you can instantly tell which ability is coming
- **Implosion sound effect** — a satisfying audio cue plays each time you destroy one of the boss's eyes
- **Encounter and defeat achievements** for reaching and surviving the new final boss

### Changed
- **Mind-controlled units actually fight their own side now** — previously, units hit by mind control would still try to attack enemies normally because their old targeting was overriding the mind control behavior. Mind-controlled defenders now properly switch to the attacker's path and attack their former allies
- **Each eye has its own health pool** — damaging a single eye only hurts that eye. Damage to the boss's body is split evenly across all five eyes. The boss falls only when every eye is destroyed
- **Eyes implode with style** — when an eye's health runs out, it stops pulling in magical particles and shrinks rapidly before despawning instead of just disappearing

### Fixed
- **Smoother performance during the new boss fight** — the new boss's health bar no longer does unnecessary work when the boss isn't on the field, and only updates on screen when a health value actually changes

### Added
- **9 new achievements** — completionist goals for unlocking all wizards, researching every spell, and maxing out all permanent bonuses, plus encounter and defeat achievements for the Lich and Dark Mage bosses, and defeat achievements for the Hags and Ogre Warlord

## [v0.6.203] - 2026-04-15

### Fixed
- **Wizard progress now saves properly for brand-new players** — if you started a fresh game and played a wizard you'd never used before, that wizard's level and progression could fail to save between sessions. The wizard tower now creates a save record the first time you visit it.
- **Time travel stays in its own lane** — replaying a past Endless level used to carry your main progression's trampled ground into the replay, and then any new mud trails you left in the replay could leak back into your real run. Replays now start on completely fresh grass, and nothing that happens during a replay affects your actual Endless progress.

### Changed
- **Save files are dramatically smaller** — trampled ground used to take up the vast majority of your save file. It's been rewritten to a much more compact format, so saves are a fraction of their previous size, write faster, and use far less of your Steam Cloud quota.

## [v0.6.195] - 2026-04-15

### Added
- **Magic enemies now have real casting animations** — the Healer, Shielder, Dispeller, and Teleporter all wind up their abilities with a visible cast, so you can actually see them preparing their effect before it goes off
- **Channeled cast visuals** — while these magic units are casting, a growing sphere of glowing particles draws in around them (green for the Healer, yellow for the Shielder, white for the Dispeller, blue for the Teleporter) — a clear "interrupt me now" cue on the battlefield
- **Longer, more interruptible casts** — the Healer, Shielder, and Dispeller now take a full five seconds to pull off their ability, and the Teleporter's big grab-and-yank on the King takes a full ten seconds, giving you much more time to react and kill them first
- **Teleporter has a ranged attack** — the Teleporter now fires weak magic bolts at your defenders while she closes on the King, so she's a threat even before she starts her channel

### Changed
- **Magic enemies hold still while casting** — once a Healer, Shielder, Dispeller, or Teleporter begins a cast, she locks in place for the full channel instead of drifting around, so interrupt timing is predictable
- **Magic enemies wait for their wave to go live** — none of these casters will begin an ability while they're still staging in the spawn tunnels; they only start casting once their wave has been released

### Fixed
- **Teleporter no longer turns invisible after her cast** — when the Teleporter finished channeling, her sprite could end up stuck on a broken frame that looked like she had vanished. She now cleanly returns to her walking animation after the teleport fires


### Changed
- **Black Hole looks like an actual black hole** — the spell now renders as a solid dark sphere instead of a white ring, with a subtle space-warping distortion around its edge so the battlefield bends as you watch
- **Black Hole is more dangerous** — the pull is noticeably stronger, reaches further across the battlefield, and the damage it deals each tick has been bumped up, so it's a more decisive answer to heavy waves

## [v0.6.180] - 2026-04-14

### Added
- **Health & Safety info in the Manual** — a new tab in the Manual screen covers photosensitive seizure guidance, recommendations for breaks and posture, and notes on which spells produce the brightest effects, so you can play more comfortably
- **Privacy Policy in the Manual** — a new tab explains exactly what the game stores (everything stays on your own computer), how Steam features interact with your account, and what happens during multiplayer sessions

## [v0.6.179] - 2026-04-14

### Changed
- **New menu music** — the main menu now plays "Pixel River" by Trevor Lentz, a calmer intro track to set the mood before the chaos begins
- **New battle music** — in-game battles now play "Sunshine Skirmish" by Trevor Lentz, a brighter and more driving score to match the pace of combat
- **Updated credits** — the Credits screen now lists music attributions and a new section for sound effect sources

## [v0.6.176] - 2026-04-14

### Added
- **Published hardware requirements** — minimum and recommended system specs are now documented so you know up front whether the game will run well on your machine

## [v0.6.175] - 2026-04-13

### Added
- **New enemy: the Teleporter** — a fragile spellcaster who slips past your defenders and channels next to your King. If you don't kill her in time, she yanks the 20 closest melee attackers right on top of him and wraps them in a huge burst of temporary shielding, turning a quiet moment into a sudden pileup on your throne. She cools down between casts, so interrupting her with a quick kill buys real time.

### Changed
- **Faster battlefield pacing** — every unit now moves about 50% faster across the board, so battles flow and waves reach the line sooner
- **Archers walk slightly slower than infantry** — archers now hang just behind the front line instead of keeping pace with it
- **Cleaner attack lanes** — the two tunnel spawns each stick to their own side of the battlefield when staging, so the streams of attackers no longer cross over each other on the approach

## [v0.6.160] - 2026-04-13

### Added
- **Seeded runs** — all randomness during a battle now uses the run's seed, so sharing a seed with someone else gives the exact same experience
- **Seed text input** — you can now click the seed field in the roguelite menu and type a seed manually, not just randomize it

### Changed
- **Consistent unit movement speed** — units no longer move faster when packed together; a lone straggler now keeps pace with the group
- **Slightly faster units** — all units move a bit quicker across the board to keep the action flowing
- **Thinner arcane rune lines** — the rings, polygons, and geometric patterns in the Wizard Tower background are now more delicate and refined

## [v0.6.158] - 2026-04-13

### Added
- **Varied attack formations** — enemy waves now split into groups and approach from different directions, so no two waves play the same way

## [v0.6.151] - 2026-04-12

### Added
- **Arcane rune background** — the Wizard Tower menu now features a rotating golden sigil behind the panels, built from concentric rings, geometric patterns, and mathematical curves
- **Progressive rune complexity** — the arcane sigil grows more elaborate as you unlock spells, adding new shapes and details with each discovery
- **Spell names orbit the rune** — your unlocked spell names circle the sigil in a mystical runic font, organized by category
- **Color-coded spell categories** — Offense, Control, Support, and Utility spells each have their own distinct color, visible in the Wizard Tower rune, the Spell Book, and the Compendium
- **Metatron's Cube** — a sacred geometry pattern of interlocking circles and lines appears within the rune as you unlock more spells, eventually filled with spinning Stars of David
- **F3 debug toggle** — press F3 on the Wizard Tower screen to hide the UI panels and see the full rune background (debug builds only)

## [v0.6.106] - 2026-04-12

### Changed
- **Golden panel borders** — menu panels, frames, and containers now have warm golden outlines instead of purple, while all buttons keep their purple styling
- **Active button glow** — selected buttons (like the current settings option) now have a visible purple glow that clearly distinguishes them from unselected buttons
- **Button glow cleanup** — the purple glow properly fades when a button is deselected instead of lingering
- **Concentration and weather buttons** — these now use the shared button styling system for consistent hover, press, and glow effects

## [v0.6.97] - 2026-04-11

### Changed
- **Spell hitboxes reworked** — all area-of-effect spells (fireball, meteor, chain lightning, sleep, haste, and more) now properly detect units based on their ground position, making spells hit more reliably against all unit sizes
- **Disintegrate beam collision** — the beam now checks against the full height of each unit's body instead of just a single point, so it hits more consistently when aimed at a unit's torso or feet
- **Fireball detonation** — fireballs now account for a unit's actual body size when deciding whether to explode on contact, instead of treating every unit as a tiny dot
- **Ogre hits harder in melee** — the Ogre's melee attack reaches further, matching his massive size

### Added
- **Debug hitbox visualization** — press F2 during gameplay to toggle semi-transparent cylinders showing each unit's collision area (useful for modding and debugging)

## [v0.6.82] - 2026-04-11

### Added
- **Accessibility settings tab** — a new dedicated tab in Settings with all accessibility options in one place
- **Reduce Flashes** — disables screen flashes, vignette pulses, and CRT flicker for players with photosensitivity
- **Reduce Motion** — disables screen-warping visual effects like gravitational lensing and heat shimmer
- **CRT Effect toggle** — lets you turn off the retro CRT TV look entirely (scanlines, screen curvature, vignette)
- **Game Speed slider** — slow the game down to 50% or speed it up to 200% to match your comfort level
- **Aim Assist** — when enabled, spell targeting snaps to the nearest unit so you don't need pixel-perfect aim
- **High Contrast mode** — boosts color contrast and saturation to help units and spells stand out on the battlefield
- **Auto-pause on alt-tab** — optionally pauses gameplay when you switch away from the game window
- **Roguelite assist tracking** — runs played with game speed changes or aim assist are marked with an asterisk in the Compendium

### Changed
- **Settings layout** — all settings rows now show dot leaders between labels and controls for a cleaner look
- **Accessibility options moved** — colorblind mode, color correction, and game speed moved from Graphics/Game tabs to the new Accessibility tab

### Fixed
- **Resolution buttons** — the selected resolution now properly highlights like other settings buttons

## [v0.6.66] - 2026-04-11

### Added
- **Colorblind mode** — three color correction options (Protanopia, Deuteranopia, Tritanopia) are now available in Settings > Graphics, with an adjustable strength slider
- **Escape key on Wizard Tower** — pressing Escape on the Wizard Tower screen now returns you to the main menu

### Fixed
- **Spell web zoom** — zooming in on the star field spell web in the Study now correctly zooms toward your mouse cursor instead of off to the side
- **Randomancer roulette after pausing** — the roulette wheel no longer freezes or shows overlapping text after pausing and resuming the game

## [v0.6.62] - 2026-04-11

### Added
- **Manual screen** — the Instructions, Changelog, Credits, and License pages are now combined into a single "Manual" screen with tabs along the top, accessible from both the main menu and pause menu
- **Formatted text in the Manual** — headings, bold text, lists, and links in the Manual now display with proper styling instead of showing raw markdown characters
- **License tab** — the game's license text is now viewable in-game from the Manual

### Changed
- **Simplified main menu** — the separate Changelog, Instructions, and Credits buttons have been replaced with a single "Manual" button
- **No console window on Windows** — the release build no longer opens a terminal window alongside the game

## [v0.6.48] - 2026-04-10

### Changed
- **Ogre hitbox matches his body** — the Ogre's collision area now extends from the ground to the top of his sprite, so spells and attacks connect more reliably
- **Ogre has less health** — the Ogre's health pool has been halved, but he's tougher against physical attacks
- **Ogre shrugs off melee** — defender units deal much less damage to the Ogre (85% physical resistance, up from 70%), making wizard spells the key to taking him down

## [v0.6.46] - 2026-04-10

### Added
- **Ogre boss sprite animations** — the Ogre now has full sprite sheet animations for walking, attacking, throwing, and charging instead of a plain colored shape
- **Ogre charge wind-up effect** — the Ogre flashes red and vibrates while channeling his charge, showing a wind-up pose before rushing forward
- **Ogre charge attack sprites** — each phase of the charge (wind-up, rushing, impact) shows a different attack pose
- **Ogre boulder throw animation** — the Ogre now winds up a throwing animation before hurling a boulder, instead of throwing instantly
- **Ogre sound effects** — the Ogre now grunts when throwing boulders, roars when charging, and swings with impact when attacking in melee
- **Charge stops on obstacles** — the Ogre's charge now ends early if he runs into a boulder, tree, or wall instead of passing through them
- **Debug level buttons** — added increase/decrease level buttons to the Endless tab for testing (debug builds only)

### Changed
- **Ogre is much larger** — the Ogre sprite is significantly bigger on the battlefield, matching his boss status
- **Ogre moves faster** — the Ogre's base movement speed has been doubled
- **Ogre charge overshoots** — the Ogre now charges past his target instead of stopping short, making positioning more important
- **Ogre hitbox is smaller** — the Ogre's collision area is tighter relative to his visual size, since the sprite has empty space around it

## [v0.6.23] - 2026-04-10

### Added
- **Wizard Tower central hub** — the main menu now opens to a tabbed "Wizard Tower" screen that serves as your home base, with tabs for Roguelite, Endless, and Study modes all in one place
- **Roguelite run persistence** — you can now close the game or switch to Endless mode and resume your Roguelite run later from the Wizard Tower, as long as you don't abandon it
- **Per-level terrain snapshots** — in Endless mode, time traveling to a previous level restores the battlefield terrain exactly as it was when you first played that level
- **Expandable wizard cards** — when choosing a wizard, cards now smoothly expand to reveal the full description and status, arranged in columns so only the expanded card's column shifts
- **Spell book header** — the spell book now has a "Spells" title and Back button in the header, matching the style of other menus
- **Hotkey toggle** — clicking an assigned hotkey slot in the spell book now unassigns the spell from that slot, instead of doing nothing
- **3D talent buttons** — talent cards in the Study now have the 3D pushable button effect with purple glow when selected

### Changed
- **Consolidated menus** — the old Game Mode Select, Run Modifiers, and Wizard Select screens have been merged into the Wizard Tower tabs, eliminating several layers of menu navigation
- **Consistent menu layout** — all menus now share the same layout: title in the top-left header, Back button in the top-right, content filling below
- **Study renders in panels** — the spell research web now renders inside the right panel of the Wizard Tower instead of taking over the full screen, with spell details in the left panel
- **Selected ingredients glow purple** — cauldron ingredients now show the pressed purple button effect when selected, matching the rest of the UI
- **Hotkey buttons glow when assigned** — in the spell book, hotkey slots show a permanent pressed effect when the current spell is assigned to them
- **Smoother ingredient selection** — toggling cauldron ingredients no longer causes the entire menu to flash and rebuild; only the brew preview panel updates
- **Abandon Run renamed** — the "End Run" button in Roguelite mode is now called "Abandon Run" for clarity
- **Smaller landing buttons** — main menu button text is slightly smaller to prevent overflow

### Fixed
- **Tree growth bug** — certain tree variants were growing larger with each level due to a save/load scaling error
- **Time travel level display** — the in-game level display now correctly shows the time-traveled level instead of your actual progression level
- **Time travel terrain** — time traveling no longer permanently changes your current level or overwrites your terrain
- **Window position saved on close** — the game window position is now reliably saved when closing via the X button or Alt+F4, not just the Exit button
- **Spell graph edge clipping** — spell web edges in the Study no longer visually leak outside the panel boundaries
- **Button press on load** — active tab buttons and other pre-selected buttons now correctly show the pressed 3D effect immediately when the screen loads, instead of appearing half-pressed

## [v0.5.818] - 2026-04-09

### Added
- **Mystical aura spheres** — the King and attacker commanders now have translucent energy spheres around them with slowly swirling magical patterns, replacing the old flat circles
- **Aura particles** — small glowing particles radiate outward from the King and commanders toward the edge of their aura, making the buff zone feel alive
- **Healing Plume aura sphere** — the healing zone is now a green energy sphere that gently grows when cast and shrinks as it fades
- **Spell cast bubbles** — Guardian Circle, Battle Hymn, Haste, Berserker Rage, Sleep, and Raise the Dead all show a brief magical sphere at the cast location that expands and fades
- **Dispel energy sphere** — dispel impacts now use the swirling energy sphere instead of flat expanding circles
- **Arcane Crystal aura** — the crystal's range indicator is now a pink energy sphere instead of a thin ring
- **Teleport visual effect** — teleporting shows a contracting sphere at the origin (collapsing inward) and an expanding sphere at the destination (arriving outward)

### Changed
- **Each spell has a unique aura color** — healing is green, guardian circle is cyan, battle hymn is gold, haste is yellow-green, berserker rage is red, sleep is white, raise the dead is purple, teleport is blue, and arcane crystal is pink
- **Aura spheres are very subtle** — the swirling energy is barely visible so it doesn't obscure the battlefield, with particles doing most of the visual work
- **Commander aura particles work for all commanders** — attacker commanders now also emit particles, not just the King

## [v0.5.796] - 2026-04-09

### Added
- **Spherical explosion effects** — all explosions (fireball, meteor, squall, berserker rage, and more) now use 3D spheres with a glowing fire gradient, replacing the old flat intersecting planes for a much more natural look
- **Organic fireball explosions** — fireball explosions spawn smaller sub-explosions that erupt from the surface as the main blast expands, creating an amorphous, bubbling fireball shape
- **Explosion fade-out** — all explosions now gradually fade to transparent as they dissipate, rather than abruptly disappearing
- **Continuous explosion particles** — fire sparks and smoke billow from the surface of explosions throughout their lifetime, not just at the initial burst
- **Frost-themed ice explosions** — squall ice impacts now have a frosty blue/white explosion sphere with white snow-like smoke particles instead of dark fire smoke
- **Progressive frost buildup** — enemies hit by ice attacks gradually turn blue and slow down more with each hit, eventually freezing solid when frost reaches maximum
- **Fire vs. frost interaction** — being on fire melts frost buildup, and frost reduces the intensity of burning, creating a natural elemental tug-of-war
- **Multi-concentration spells** — you can now concentrate on multiple spells at the same time (e.g., Squall and Meteor Fall together), with each one reserving a portion of your mana bar
- **Mana reservation display** — the mana bar shows a dark purple section on the right indicating how much mana is reserved by active concentration spells, with "Concentrating" text
- **Individual concentration cancel** — each active concentration spell gets its own cancel button, so you can end one without affecting the others
- **Sphere-cylinder collision** — explosion hit detection now properly accounts for unit height and width, making area damage more accurate
- **Concentration section in instructions** — the How to Play screen now explains how concentration spells work with mana reservation

### Changed
- **Slower explosion animations** — explosions expand and fade over a longer duration, making them easier to appreciate
- **Squall mana cost reduced** — from 40 to 30 mana
- **Meteor Fall mana cost reduced** — from 60 to 50 mana
- **Quieter spell sound effects** — reduced the volume of spell impacts and casts on the battlefield
- **Ice explosions hit all units** — squall ice impacts now damage and frost both attackers and defenders, consistent with the game's friendly fire design
- **Meteor ground fire is invisible** — removed the flat orange circle under meteor fires, leaving just the fire particles and smoke
- **Updated spell descriptions** — squall and meteor fall descriptions now mention mana reservation instead of the old concentration mechanic

### Fixed
- **Ice explosions now deal damage** — fixed a bug where squall ice impacts weren't hitting any units due to incorrect collision detection

## [v0.5.756] - 2026-04-08

### Added
- **3D pushable buttons** — all buttons across the game now have a physical depth effect with a raised front face and a visible edge underneath, giving them a tactile, clickable look
- **Button press animation** — buttons smoothly animate up on hover and push down on click, with a satisfying snap on press and a gentle return to rest
- **Keyboard press animation** — pressing hotkeys for action bar spells now shows the same push-down animation as clicking
- **Purple glow effects** — hovering and pressing buttons triggers a vibrant purple glow around the edges, with pressed buttons glowing even brighter
- **Active button state** — selected tabs, toggled modifiers, and chosen wizard types stay visually pressed down with a bright purple highlight until deselected
- **Frosted glass overlays** — menu backgrounds now have a subtle frosted texture effect instead of being plain transparent
- **Procedural parchment shader** — content panels use a noise-based texture for a weathered, organic feel
- **Ornate panel borders** — content panels have layered border frames with outline rings and depth shadows

### Changed
- **Fantasy color palette** — the entire UI shifted from cool blue/purple tones to warm charcoal browns with purple accents, creating a medieval fantasy feel
- **Purple is the accent color** — all borders, highlights, glows, selection indicators, slider bars, and interactive elements use consistent purple tones
- **Warmer text colors** — text now has a subtle warm parchment tint instead of pure white
- **Richer panel shadows** — panels use multi-layered warm-tinted shadows for more natural depth
- **Spellbook button hidden for some wizards** — the spellbook button is now hidden during gameplay for wizard types that don't manage spells

### Fixed
- **Barrel distortion click accuracy** — fixed a bug where the CRT screen effect could cause clicks to register on the wrong button near screen edges
- **Menu overlays block interaction** — opening a menu like the spellbook or cauldron now properly prevents hovering and clicking on buttons behind it
- **Compendium tab switching no longer flashes** — selecting a different item in the compendium no longer causes the entire list to visibly rebuild
- **Button states update immediately** — switching tabs or deselecting options now instantly returns the previous button to its resting state

## [v0.5.676] - 2026-04-07

### Changed
- **Wall of Stone is now Wall of Dirt** — the spell has been renamed and the wall now looks like a raised chunk of the battlefield itself, with grass on top and earthy brown sides
- **Grass-topped walls** — the top of the wall shows the same grass texture as the surrounding ground, blending seamlessly with the battlefield
- **Dirt border on wall tops** — a natural-looking dirt edge frames the grass on top of each wall, making it easy to see where the wall ends
- **Earthy wall sides** — the sides of the wall use the same dark reddish-brown as the stone terrain, with subtle texture variation
- **Ground disturbance when walls rise** — the ground around a rising wall gets churned up with a dirt patch that fades outward, as if the earth was displaced
- **Trampled ground is darker** — the brown color of trampled paths is now a deeper reddish-brown that matches the rest of the terrain
- **Trampled ground recovers faster** — grass grows back more quickly between levels

### Fixed
- **Temporary walls no longer persist after winning** — walls placed without the Terraformer talent now properly disappear when you win a level, instead of carrying over to the next battle

## [v0.5.670] - 2026-04-07

### Changed
- **Meteor explosions look like fireball explosions** — meteor impacts now use the same fiery radial gradient effect as fireball, with a glowing yellow-white core fading to deep orange at the edges
- **Meteor ground fire uses real fire** — the lingering fire pools left by meteor impacts now show the full procedural fire effect with dancing flames, flickering embers, and rising dark smoke, matching the fire from walls of fire and burning vegetation

### Fixed
- **Removed screen flash from meteor impacts** — the brief full-screen flash that occurred each time a meteor hit the ground has been removed to reduce eye strain

## [v0.5.666] - 2026-04-06

### Added
- **Procedural fire shader** — fire effects now use an animated noise-based shader with a full color gradient from white-hot cores through orange and red to dark edges, replacing the old flat-colored particles
- **Flickering embers** — small bright particles flicker at the base of all fire effects, adding a glowing coal-bed look
- **Rising smoke plumes** — large dark smoke clouds billow upward from fires with a soft circular shape, visible above the flames
- **Heat shimmer on all fire** — the wavy heat distortion effect now appears around burning trees, bushes, lava pools, grease fires, and fireball explosions, not just walls of fire
- **Fire spreads from walls of fire** — casting wall of fire near trees and bushes now sets them ablaze
- **Fire spreads from meteor ground fire** — the lingering fire left by meteor impacts now ignites nearby vegetation
- **Burning heat zone** — standing near burning trees and bushes gradually builds up a fire-over-time effect instead of dealing instant damage, rewarding players who move their units away quickly

### Changed
- **Denser fire particles** — fires now emit many more smaller particles that blend together into thick, cloud-like flames instead of a few large floating shapes
- **Layered fire animation** — fire particles are split into fast-flickering base flames, medium rising fire, and slow lingering smoke at the top, each with different speeds and sizes
- **Additive sparks and embers** — fire sparks and embers now glow brighter where they overlap, creating natural-looking hot spots
- **Smoke particles are round** — dark smoke puffs now fade smoothly at the edges instead of showing hard square corners
- **Fire stays on the trunk** — burning vegetation particles now only appear in the lower 70% of the sprite, keeping the treetops clear

### Fixed
- **Rare crash when units die near fire** — fixed a crash that could occur when a unit was despawned in the same frame as a visual effect tried to update its material

### Changed
- **New studio logo** — the splash screen now displays the Blackhearth Games logo

## [v0.5.633] - 2026-04-06

### Added
- **Ground texture variation** — the battlefield grass now has subtle color variation with darker and lighter patches, giving the field a more natural look instead of flat uniform green
- **Earthy forest floor** — the area under the tree concentration has more pronounced ground noise with brown earth tones, gradually blending into the open field
- **Darkened back wall area** — the ground near the back wall is darker, adding depth to the forested region
- **Sandy blending around the pond** — the water pool area now has a sandy-colored ground that blends in a natural circle around the pond
- **Rocky ground near the tunnels** — the area around the tunnel entrances has a dark reddish-brown stone texture that follows the wall border and blends organically into the grass
- **More wildflowers** — the battlefield now has significantly more scattered flowers and small plants across the open field
- **Floating particles** — small pollen-like motes now drift slowly across the battlefield, adding atmosphere

### Fixed
- **Units no longer walk through trees** — trees now physically block units just like boulders and stone walls do

## [v0.5.604] - 2026-04-06

### Added
- **Wind sway on vegetation** — trees, bushes, and flowers now gently sway in the wind, giving the battlefield a more alive feel without requiring sprite animation
- **Trees catch fire** — fire spells now ignite trees just like bushes. Burning trees stay on fire for the rest of the battle and are gone the next level
- **Burning trees deal damage** — standing inside a burning tree hurts just like a burning bush

### Changed
- **Shadows no longer stack** — overlapping shadows from units and terrain now blend into one uniform darkened area instead of getting darker where they overlap
- **Fire and smoke cover the full sprite** — burning trees and bushes now emit fire and smoke particles across their entire height instead of just at the base

## [v0.5.597] - 2026-04-05

### Added
- **Insight Bonuses constellation** — a new constellation appears to the right of the spell web in the Study screen. Spend Insight on permanent upgrades to Spell Damage, Spell Range, Cast Speed, and Mana Cost Reduction
- **Five concentric rings** — each bonus node shows 5 rings that fill with a clockwise animation as you invest Insight, matching the spell research visual style
- **Slider-based allocation** — assign Insight to bonuses using the same slider system as spell research, then commit everything together
- **Study screen zooms out on load** — the Study now starts zoomed out so you can see both the spell web and the new bonuses constellation at once

### Changed
- **"Study Spells" renamed to "Study"** — reflects that the screen now includes more than just spell research
- **Sliders track the mouse more accurately** — all sliders (spell research, bonuses, and settings) now use the barrel-corrected cursor position, fixing a slight offset when clicking
- **Sliders reach their min and max** — dragging a slider past either end now correctly snaps to the minimum or maximum value instead of getting stuck
- **Locked spell text scales with zoom** — the "???" text on locked spell nodes now shrinks when zoomed out so it doesn't overflow the circle

### Fixed
- **Clearing progress now fully resets** — clearing your progress now also resets permanent bonuses and unlocked toggle modifiers

## [v0.5.576] - 2026-04-05

### Added
- **Toggle modifiers for roguelite runs** — spend Insight to permanently unlock special run modifiers that change how the game plays. Enable or disable them before each run from the new Run Modifiers screen
- **Blood Magic modifier** — no passive mana regeneration. Instead, gain mana from enemy kills
- **Boss Parade modifier** — a boss appears every 3rd wave, cycling through all boss types
- **Fortified Horde modifier** — first-wave enemies arrive with temporary shields and a pulsing golden glow
- **Glass Cannon modifier** — your defenders hit twice as hard but have half health
- **Spell Rotation modifier** — forces you to use different spells in sequence. You can't cast the same spell twice in a row
- **Wizard Cycle modifier** — your wizard archetype automatically rotates through all your unlocked types every 30 seconds, with a banner announcing each switch
- **Scorched Earth modifier** — all ground-effect spells last 3 times longer
- **Veteran Defenders modifier** — half as many defenders, but each one is significantly tougher
- **Attrition modifier** — defenders who die during a level don't come back for the next one. Protect your army across the entire run
- **Rising Tide modifier** — each wave within a level spawns more enemies than the last
- **Urgent modifier** — moved from settings into a toggle modifier. The game keeps running while browsing menus
- **Insight bonus for modifiers** — active toggle modifiers grant bonus Insight at the end of each battle
- **Run Modifiers screen redesigned** — now uses a two-panel layout with your run summary on the left and all configuration options (seed, sliders, and toggle modifiers) on the right
- **Pause menu redesigned** — now shows live battle stats on the left panel (level, time, kills, efficiency, wave progress) with active modifiers listed for roguelite and best stats for endless mode
- **Tree sprites** — trees now have unique hand-drawn appearances with 5 different styles, replacing the old placeholder rectangles
- **Forest along the battlefield edge** — a dense forest of trees, bushes, and boulders spawns along the back wall, thinning out toward the center of the battlefield
- **Entangle now affects pathfinding** — the entangle spell zone is now treated as a hazard on the flow field, causing enemies to path around it

### Changed
- **Bigger trees** — trees are now larger with bigger shadows, giving the battlefield a more dramatic look. Thick-trunk tree variants have a wider collision zone
- **Bigger flowers** — flora decorations are 1.5x larger
- **Spell book disabled for RuneCaster and Randomancer** — these wizards now exclusively use their own casting mechanics (rune sequences and roulette spins). The spell book and action bar spell selection are locked
- **Randomancer can re-spin** — pressing the spin button again before casting now starts a new spin immediately, and the spin animation is faster
- **Rune-activated spell name appears above the rune buttons** — with a text shadow for visibility on light backgrounds
- **Wizard select grid stays fixed** — the wizard type grid no longer reflows when resizing the window
- **Shared two-panel layout** — spell book, cauldron, compendium, run modifiers, and pause menu now all use a consistent two-panel design

### Fixed
- **Modifier selections no longer persist between runs** — toggle modifier selections now reset when starting a fresh run instead of carrying over from the previous one
- **Screen flicker removed from toggle clicks** — toggling modifiers on and off no longer triggers the CRT screen transition effect
- **Excremage colors apply during Wizard Cycle** — spell materials correctly switch to brown when cycling to the Excremage and back to normal when cycling away
- **Warglock action bar updates during Wizard Cycle** — the action bar now correctly shows gun names when cycling to the Warglock
- **Arcanorouter bonuses reset on wizard switch** — range/speed/power bonuses from the Arcanorouter are properly removed when cycling to a different wizard type

## [v0.5.518] - 2026-04-03

### Fixed
- **Brewing progress bar updates in urgent mode** — when you open the cauldron menu while a brew is in progress and urgent mode is on, the progress bar on the HUD now continues updating instead of freezing
- **Cauldron menu refreshes when a brew finishes** — if a brew completes while the cauldron menu is open in urgent mode, the menu now automatically switches back to the ingredient selection screen so you can start another brew right away
- **"Brewing in progress" is now centered** — the brewing status message in the cauldron menu is now displayed in the center of the window instead of being stuck in the top-left corner

## [v0.5.514] - 2026-04-03

### Added
- **Tabbed settings menu** — the settings screen is now organized into four tabs: Graphics, Audio, Game, and Controls. Click a tab to switch between pages instead of scrolling through one long list
- **Resolution selection** — a new row in the Graphics tab lets you pick from common windowed resolutions (720p, 900p, 1080p, 1440p, 4K). Only available in windowed mode
- **Key rebinding** — every gameplay keybind can now be reassigned in the Controls tab. Click a key, then press the key you want to assign. Bindings are saved between sessions
- **Unbind keys** — press Backspace during key capture to remove a binding entirely. Unbound keys show as "-" in the controls list and action bar
- **Key conflict warnings** — if you try to bind a key that's already in use, you'll see a red warning showing what it conflicts with. Press the key again to confirm the swap
- **Window icon** — the game now shows its logo in the Windows taskbar and title bar instead of a blank icon
- **Save on exit** — your progress is now saved when closing the game via Alt+F4 or the window close button, so you won't lose recent changes
- **New achievement** — a hidden achievement has been added for players who really like clicking

### Changed
- **Wizard renamed** — the Gunslinger wizard is now called the Warglock
- **Locked wizard controls are hidden** — the Controls tab only shows key bindings for wizards you've unlocked. Locked wizards show a hint instead
- **Brightness moved to Graphics tab** — the brightness slider is now in the Graphics tab alongside VSync, display mode, and resolution
- **Credits updated** — removed outdated references to web-only libraries and features

## [v0.5.513] - 2026-04-02

### Changed
- **Simplified game mode selection** — removed the Story mode placeholder from the mode select screen. The available modes are now Roguelite, Endless, and Multiplayer (coming soon), displayed in a cleaner vertical layout

## [v0.5.511] - 2026-04-01

### Added
- **Boulder sprites** — boulders now have unique hand-drawn appearances instead of plain circles. Each boulder randomly picks from 10 different looks, whether placed as terrain or thrown by Brutes and Ogres
- **Bush sprites** — bushes now have varied, detailed appearances with 10 different styles that are randomly assigned each time they spawn
- **Thrown boulders spin through the air** — boulders thrown by Brutes and Ogres now tumble as they fly, landing upright when they hit the ground
- **Burning bush glow** — bushes that catch fire now show an orange tint over their sprite, making it easier to tell which bushes are burning at a glance

### Changed
- **Terrain now properly blocks the flow field** — boulders, trees, bushes, and ponds now correctly influence unit pathfinding. Units will navigate around obstacles instead of walking straight through them
- **Terrain collision matches visuals** — the area that boulders, bushes, and trees block now matches their actual visual size on screen, so pathfinding feels more accurate
- **Boulders darken when damaged** — boulders that take damage gradually darken, giving better visual feedback on their remaining health

### Fixed
- **Terrain obstacles were invisible to pathfinding** — fixed a timing issue where terrain spawned during level loading wasn't being registered with the pathfinding system, causing units to ignore obstacles entirely

## [v0.5.498] - 2026-04-01

### Changed
- **Cleaner code under the hood** — major cleanup pass across the entire game to improve code quality, reduce duplication, and follow best practices. No gameplay changes, just a healthier codebase
- **Combat and movement split into focused files** — the game's internal systems are now better organized, making future updates easier to develop and test
- **Spell casting is more consistent** — all spells now share the same underlying input and indicator logic, reducing the chance of one spell behaving slightly differently from another

## [v0.5.425] - 2026-04-01

### Added
- **Battlefield terrain** — trees, bushes, ponds, and boulders now spawn across the battlefield at the start of each run. The amount and placement varies each time based on the terrain seed
- **Trees block everything** — trees are impassable obstacles that block movement, arrows, magic missiles, chain lightning, and disintegrate beams. Only fire spells can burn them down
- **Bushes catch fire** — bushes slow units passing through them, but hit them with a fire spell and they'll ignite into a burning hazard that damages nearby units for the rest of the battle. Burned bushes disappear between levels
- **Ponds make units wet** — stepping into a pond applies the Wet status, which boosts electric damage taken by 50% and allows electric charges to spread between wet units. Wet lasts 10 seconds after leaving the water
- **Boulders on the battlefield** — boulders now appear as natural terrain in addition to being thrown by Brutes and Ogres. All boulders persist between levels if you win
- **Terrain density slider** — a new difficulty slider on the roguelite run setup screen lets you control how much terrain spawns, from 20% to 300%
- **Seeded runs** — every run gets a random seed that controls terrain layout, unit positions, and other randomized elements. You can type in a specific seed to replay the same battlefield setup, or share seeds with friends
- **Copy and paste seeds** — use Ctrl+C to copy and Ctrl+V to paste seeds on the run setup screen. Seeds can also be copied from the compendium
- **Seed displayed everywhere** — your run's seed shows on the pause menu, game over screen, and in the compendium run history
- **Run settings in compendium** — the compendium now shows the difficulty settings and seed used for each roguelite run
- **Terrain has shadows** — trees, bushes, boulders, and flowers all cast ground shadows
- **Size variation** — trees, bushes, and boulders spawn with slight size differences (80%–120%) for a more natural look

### Changed
- **Wet status from storms** — the Meteorologist's storm weather now applies the same Wet status as ponds. Electric shock now spreads between wet units regardless of whether a storm is active
- **Wet units appear darker** — units with the Wet status get a subtle dark blue-gray tint
- **Defeat means 0% efficiency** — losing a level now always reports 0% efficiency instead of tracking how many defenders survived before the king fell
- **License changed to GPL v3.0** — the game is now distributed under the GNU General Public License v3.0

### Fixed
- **Terrain slider alignment** — all difficulty sliders now align properly regardless of label length

## [v0.5.402] - 2026-03-31

### Added
- **Trampled ground** — units marching across the battlefield gradually turn the grass into muddy brown trails. The more units walk over the same area, the darker and muddier it gets. Between battles, the grass slowly grows back
- **Mud trails persist across levels** — trampled ground carries over between battles within the same run, so well-traveled routes stay visible throughout your campaign
- **Boulders can be teleported** — use the Teleport spell to relocate boulders thrown by Brutes and Ogres, clearing blocked paths or repositioning them strategically

### Changed
- **Flying units don't trample ground** — aerialists hover above the battlefield and don't leave mud trails like ground units do

## [v0.5.337] - 2026-03-30

### Added
- **Staff flare when casting spells** — every spell now produces a burst of colored light and sparks from the wizard's staff when cast. The color matches the spell's school of magic (fire spells glow orange, lightning spells flash white-blue, dark spells pulse purple, etc.)
- **Screen flash on big impacts** — lightning strikes, meteor explosions, and chain lightning bolts now produce a subtle screen flash on impact
- **Screen darkening on Finger of Death** — casting Finger of Death briefly darkens the edges of the screen for a more ominous feel
- **Healing zone sparkles** — the Healing Plume zone now has golden motes rising from the ground while active
- **Entangle zone particles** — entangled areas now have green nature particles drifting upward
- **Polymorph smoke poof** — a puff of amber smoke appears when a unit is polymorphed into a sheep
- **Banishment void effect** — dark purple smoke erupts from the spot where a banished unit disappears

## [v0.5.305] - 2026-03-30

### Added
- **Brute throws boulders** — instead of a close-range area attack, the Brute now hurls massive rocks at your defenders from a distance. Boulders land permanently on the battlefield as obstacles that block movement and line of sight
- **Ogre throws boulders too** — the Ogre boss now throws boulders between charges, littering the battlefield with obstacles
- **Boulders block everything** — landed boulders block unit pathing, arrows, spells, and line of sight just like walls. If a boulder completely blocks a unit's path, that unit will attack the boulder to break through
- **Spells can destroy boulders** — area-of-effect spells like Fireball, Meteor, and Squall deal damage to boulders, but Magic Missile won't target them
- **Boulder impact sound** — a satisfying thud plays when a boulder crashes down onto the battlefield
- **Aerialist arrow attacks** — flying units now fire visible arrow projectiles at ground targets instead of dealing invisible damage

### Changed
- **Brutes appear earlier** — Brutes now start showing up at level 11 instead of level 21
- **Brutes move faster** — Brutes now move at the same speed as infantry
- **Brute has normal melee** — when enemies get close, the Brute fights with standard single-target melee attacks
- **Fewer waves per level** — each level now has one fewer wave than before, maxing out at 6 waves in endless mode
- **Better brute shadows** — the Brute's shadow is now correctly sized to match its body instead of being oversized

## [v0.5.294] - 2026-03-30

### Added
- **New boss: The Dark Mage** — a sinister spellcaster who appears on level 20. He walks onto the battlefield from the tunnels, then begins casting devastating area-of-effect spells at your defenders
- **Telegraphed attacks** — every Dark Mage spell is preceded by a glowing red indicator on the ground, giving you time to move your units out of the way. Watch for the warning zones!
- **Dark Meteor** — a massive fireball falls from the sky and explodes on impact, dealing heavy damage in a large area
- **Shadow Lightning** — bolts of lightning strike down from above along a corridor, zapping everything in their path
- **Plague Cloud** — a persistent poison cloud that lingers on the battlefield, dealing damage over time to any defenders caught inside. Your units will try to path around it
- **Reactive teleportation** — when your defenders close in for melee, the Dark Mage teleports to a new position on the battlefield to keep his distance
- **Boss rotation in endless mode** — bosses now cycle in sequence (Ogre, Lich, Hags, Dark Mage) as you progress through endless mode, instead of repeating the same boss

### Changed
- **Dark Mage takes reduced melee damage** — he's tough to take down up close, but his real danger is the spells he casts from range

## [v0.5.279] - 2026-03-29

### Fixed
- **Elite and special enemies now appear in all waves** — previously, only the first wave of each level had elite, commander, dispeller, healer, and shielder enemies. Now every wave gets its own set of upgraded enemies based on the level's difficulty

## [v0.5.277] - 2026-03-29

### Added
- **Roguelite run modifiers** — before starting a roguelite run, you can now adjust three difficulty sliders: wave speed, enemy strength, and enemy count. Each ranges from 20% to 300%, letting you customize how hard (or easy) your run will be
- **New achievements for modifier runs** — a collection of new achievements for completing roguelite runs at various modifier settings, including some for pushing every slider to the max
- **New studio splash image** — updated the studio splash screen with a new image

### Changed
- **Game clock tracks real time** — the in-game clock now counts real seconds instead of speeding up during the between-wave fast-forward. Your battle times will be more accurate across all game modes
- **Modifier stats saved to run history** — your chosen modifier settings are recorded with each roguelite run so you can see what difficulty you played at

### Fixed
- **Enemy count modifier applies to all waves** — the enemy count slider now correctly affects every wave in a battle, not just the first one

## [v0.5.266] - 2026-03-28

### Added
- **New boss: The Lich** — an undead sorcerer who appears on level 10 as a surprise after the final wave of attackers is defeated. He marches to the battlefield and begins raising the dead from fallen soldiers. Kill enough of his undead to fill his soul power bar and he'll enter a devastating combat phase, firing Finger of Death beams at your defenders
- **Lich raises the dead** — the Lich resurrects corpses from anywhere on the battlefield as undead infantry. If there aren't enough corpses, he'll summon fresh undead to fill out his army
- **Finger of Undeath** — defenders killed by the Lich's death beam are raised as undead, turning your own fallen soldiers against you
- **Multiple retreats per level** — the King can now call for retreat more than once per battle. Higher difficulty levels grant more retreats

### Changed
- **Faster archers** — archers now move 50% faster across the battlefield
- **Bigger boss shadows** — boss unit shadows are now proportional to their actual size instead of being the same size as regular units
- **Game speeds up between waves** — if all attackers from a wave are killed before the next wave arrives, the game speeds up so you're not waiting around
- **Raised undead behave as infantry** — undead raised from archer, assassin, or other special unit corpses now fight as basic melee infantry instead of keeping their original abilities

### Fixed
- **Resurrected units no longer trigger wave activation** — undead raised from corpses no longer incorrectly cause wave activation messages

## [v0.5.235] - 2026-03-27

### Added
- **New enemy: Aerialist** — a flying attacker that soars above the battlefield, ignoring walls and obstacles. Aerialists swoop in wide arcs and attack from above with dive-bomb strikes. They appear starting at higher difficulty levels
- **Archers can shoot flying enemies** — arrows now arc upward to reach airborne targets, with proper ballistic physics that account for height
- **Unit shadows** — all units now cast a circular shadow on the ground beneath them, making it easier to track positions. Larger units have larger shadows, and flying units cast a slightly bigger shadow from their altitude

### Changed
- **Ground troops ignore flyers** — infantry, the King, and the King's Guard no longer try to chase enemies flying overhead. Only archers can engage airborne threats
- **Flying units pass over walls** — airborne enemies fly right over Wall of Stone and other obstacles without being blocked or deflected

## [v0.5.207] - 2026-03-26

### Added
- **Ogre charge attack** — the Ogre boss now charges across the battlefield, targeting defenders at a distance. A pulsing red rectangle telegraph warns you where the charge will land, giving you a few seconds to move units out of the way. Units caught in the charge take heavy damage and get knocked sideways
- **Ogre melee resistance** — the Ogre now shrugs off most damage from infantry and archers, taking only a fraction of their attacks. Spells remain fully effective against it

### Changed
- **Faster Ogre** — the Ogre boss moves faster across the battlefield
- **Stronger enrage scaling** — the Ogre's enrage bonuses now ramp up more aggressively at each health threshold

## [v0.5.188] - 2026-03-26

### Changed
- **Unified UI look** — all menu screens and overlays (spell book, cauldron, pause menu, game over, wizard select, game mode select, multiplayer) now use the same styled container with a translucent bordered panel, matching the instructions and settings screens
- **Semi-transparent panels** — detail panels, wizard cards, spell buttons, and list areas throughout the UI are now semi-transparent instead of solid black, letting the background show through for a more polished look
- **Brighter labels and descriptions** — category headers, section labels, disabled text, and other secondary text across all screens have been brightened for better readability
- **Consistent colors everywhere** — all UI screens now pull from a single shared color palette instead of each screen defining its own slightly different colors
- **Faster units** — all units move 10% faster across the battlefield
- **Tighter speed control** — units can no longer exceed their intended movement speed when grouped together

## [v0.5.176] - 2026-03-26

### Changed
- **Refreshed button styling** — all buttons throughout the game now have a translucent purple tint that blends with the background, replacing the old flat dark look
- **Text shadows everywhere** — all button text and page titles now have a subtle drop shadow that scales with the text size, making everything easier to read
- **Brighter muted text** — buttons that were previously hard to read (like "Close" and "Back" buttons) now have brighter text

## [v0.5.144] - 2026-03-25

### Changed
- **Attackers march in formation** — enemy units now approach the battlefield in proper military formation. Infantry and brutes lead the charge up front, archers follow behind, and sneaky units bring up the rear

## [v0.5.142] - 2026-03-25

### Added
- **Battlefield flowers** — small flowers and plants now grow scattered across the battlefield, giving each map a more natural feel. They get permanently trampled when units walk over them

### Changed
- **No more loading screen** — battles now start almost instantly instead of showing a loading screen. All units and terrain spawn in the blink of an eye

## [v0.5.133] - 2026-03-25

### Changed
- **Faster saving and loading** — your progress is now kept in memory while you play, so saving and loading happens almost instantly instead of reading from disk every time
- **Safer save files** — saves are now written using a crash-proof method. If something goes wrong mid-save, your previous save is preserved as a backup and restored automatically
- **Save file integrity checks** — your save file now includes a signature that detects if it was corrupted or tampered with. If a problem is found, you'll see a warning but your save will still load

### Fixed
- **Cleaned up unused code** — removed leftover code that was no longer being used, keeping things tidy under the hood

## [v0.5.127] - 2026-03-25

### Added
- **Multiplayer networking foundation** — behind-the-scenes groundwork for peer-to-peer multiplayer has been added. Two players will be able to connect directly without any servers by sharing a short connection code. This feature is not yet available in-game while it's being tested

## [v0.5.121] - 2026-03-24

### Added
- **Windowed and fullscreen display modes** — you can now switch between windowed and fullscreen in the Settings menu under Graphics. Your window size and position are remembered when switching back to windowed mode
- **Exit button** — the main menu now has an Exit button to close the game
- **Confirmation popups** — the "Reset Tutorials" and "Clear Progress" buttons in Settings now show a confirmation popup before taking action, so you won't accidentally wipe your progress
- **Scrollable settings** — the Settings page content is now inside a scrollable area with the title and back button always visible, matching the layout of other menu screens
- **Compendium detail scrolling** — the left detail panel in the Compendium (e.g., endless wizard stats) is now scrollable when content overflows

### Changed
- **Main menu layout** — the main menu buttons are now on the left side of the screen with the game title displayed large on the right, with a drop shadow effect
- **See-through menu overlays** — Settings, Instructions, Changelog, Credits, and Compendium screens now have translucent backgrounds so you can see the animated main menu scene behind them. Scrollable areas have their own slightly darker panel with a subtle shadow for contrast
- **Consistent back buttons** — all menu screens now use the same back button size and style
- **Volume sliders track the mouse accurately** — sliders now follow your cursor position directly instead of using mouse movement speed, which could feel too fast or too slow depending on your display settings
- **Improved credits page** — sprite art attribution is now shown directly in the credits text, and the full artist credit file (SPRITE_CREDITS.csv) is included in the game directory
- **Difficulty removed from Settings** — the difficulty option has been removed from the Settings menu since it wasn't being used

### Fixed
- **Display mode switching no longer crashes** — switching between windowed and fullscreen no longer causes rendering errors or freezes

## [v0.5.79] - 2026-03-23

### Added
- **Animated main menu background** — the main menu now features a layered parallax scene with a slowly drifting sky, passing clouds, and a castle tower in the foreground, giving the menu a more immersive feel

## [v0.5.72] - 2026-03-23

### Added
- **Steam integration** — the game now connects to Steam for achievement tracking and cloud saves, so your progress syncs across devices. If Steam isn't running, everything still works normally
- **Rune combos for all spells** — every spell in the game now has a rune combination. Combos follow the spell web: spells closer to the center use fewer runes, and each combo starts with the branch's root rune (Q, W, E, or R). The deepest spells require 4-rune sequences

### Changed
- **Rune input** — you can now press the same rune key twice in a row, which is needed for some of the new spell combos

## [v0.5.68] - 2026-03-22

### Added
- **Battlefield ground tiles** — the battlefield now has a textured ground made up of varied grass tiles instead of a flat solid color, giving the terrain more visual depth and personality

### Fixed
- **Defenders no longer chase inactive waves** — defenders will now correctly return to their positions and wait when the only attackers left on the field are still marching in and haven't entered battle yet
- **Talent selection cleanup** — if a talent was somehow selected in a tier that hasn't been unlocked yet, it will now be automatically cleared when loading into a game

## [v0.5.60] - 2026-03-22

### Added
- **Shielder animations** — shielders now have unique walking, attacking, casting, and death animations with dedicated sprite sheets
- **Healer animations** — healers now have unique walking, attacking, casting, and death animations with dedicated sprite sheets
- **Crowd ambience** — a muffled crowd sound plays throughout battles, adding to the atmosphere of the battlefield

## [v0.5.55] - 2026-03-21

### Added
- **Unique unit sprites** — undead, dispellers, and the swordcerer now have their own distinct walking, attacking, and death animations instead of borrowing other units' appearances
- **Dispeller casting animation** — dispellers now visually channel when casting their dispel ability
- **Swordcerer combat animations** — the swordcerer plays a casting animation when firing missiles and an attack animation when swinging its sword
- **Battle ambience** — a subtle sword-clashing sound plays during melee combat, growing louder as more units engage in close-quarters fighting

### Changed
- **Undead appearance** — undead units now have a more natural look with less purple tinting
- **Swordcerer appearance** — the swordcerer avatar no longer has a blue hue, showing off its unique sprite sheet clearly

### Fixed
- **Arcane crystal range ring** — the range indicator ring around permanent arcane crystals now properly disappears when the crystal is dispelled

## [v0.5.43] - 2026-03-21

### Added
- **Crash reporting** — if the game crashes, a detailed crash log is now saved to your save data folder automatically, making it easier to report bugs
- **Player readme** — game downloads now include a README file with info on where save data and crash logs are stored

## [v0.5.40] - 2026-03-21

### Added
- **Death animations** — units now play a death animation when they fall in battle, ending on a final fallen pose instead of instantly swapping to a corpse sprite
- **Melee attack animations** — infantry, archers, and assassins now visually swing when attacking in melee combat
- **Ranged shooting animations** — archers play a shooting animation when firing arrows
- **Assassin unique look** — assassins now have their own walking sprite instead of sharing the archer's appearance

### Changed
- **Smoother walking animations** — all unit walking animations now use updated 9-frame sprite sheets for more fluid movement
- **Elite and commander sizing** — upgraded units that grow larger no longer clip into the ground; they properly float at the right height

## [v0.5.34] - 2026-03-21

### Fixed
- **Smoother unit animations** — units no longer flicker between walking directions when moving diagonally; their facing direction stays stable until they clearly change course

## [v0.5.31] - 2026-03-20

### Added
- **Terrain hazards** — the lava pool on the battlefield now deals heavy damage to any unit that walks through it, and the water pool slows units down
- **Hazard avoidance** — units intelligently path around the lava pool and avoid the water pool when possible, thanks to flow field integration
- **Tunnel spawning** — attacker waves now spawn behind the right wall and march through the tunnel archways onto the battlefield, creating a more dramatic entrance
- **Wave staging system** — each wave of attackers gathers at a rallying point before charging toward the castle, giving you a moment to prepare
- **Staging speedup** — the game speeds up while attackers are marching to their rally point so you're not waiting around
- **Right wall occlusion** — units are hidden behind the right wall until they emerge through the tunnels

### Changed
- **Wave timing** — the next wave won't begin spawning until the current wave has finished rallying and activated
- **Battle timer** — the in-game timer doesn't start until the first wave of attackers activates, so marching time doesn't count against you
- **Defender awareness** — defenders now ignore attackers that haven't activated yet, only springing into action once the assault begins
- **Special unit behavior** — healers, shielders, dispellers, and archers on the attacker side won't use their abilities until their wave activates

## [v0.4.519] - 2026-03-20

### Added
- **Game mode selection** — a new menu screen lets you choose between different game modes before picking your wizard
- **Roguelite mode** — a fixed 25-level run where losing ends your attempt; you get a score screen with full run stats, then return to the main menu to try again
- **Endless mode** — an infinitely scaling mode where your progress saves between sessions; pick up where you left off and face increasingly tougher enemies past the final tier
- **Endless difficulty scaling** — beyond the last tier of unit introductions, attackers become progressively stronger and more numerous each level
- **Per-wizard Endless progression** — each wizard type tracks its own Endless mode level progress independently
- **Roguelite compendium tab** — view your recent runs and saved runs in the compendium, click any run to see a full level-by-level breakdown
- **Endless compendium tab** — view your best stats per level for each wizard type in the compendium
- **Save and unsave runs** — permanently save your favorite roguelite runs so they're never lost to history trimming
- **Roguelite score screen** — when a roguelite run ends, see a summary of your run performance including kills, efficiency, and time per level
- **Multiplayer placeholder** — multiplayer game mode appears on the selection screen as "Coming Soon"

### Changed
- **CRT flash effect** — reduced the intensity of the screen flash during menu transitions for a subtler look
- **Timewalking disabled in Roguelite** — you can't replay previous levels during a roguelite run
- **Mode-aware saving** — roguelite runs don't overwrite your endless mode progress or vice versa

## [v0.4.510] - 2026-03-20

### Added
- **Assassin units** — a new fast-moving attacker unit type that flanks around infantry to hunt down archers, appearing from level 6 onward
- **Assassin scaling** — assassin numbers increase as levels get harder, similar to how archer counts scale up

### Changed
- **Assassin flanking behavior** — assassins route around infantry formations to reach archers from the sides, but charge straight in when no archers remain
- **Assassin combat** — assassins deal massive damage to archers, attack much faster than normal units, and pass through non-assassin units
- **Assassin survivability** — assassins take less damage from archers but more damage from infantry, and are ignored by infantry and the King
- **Smoother pathfinding** — all unit navigation fields now update continuously in the background instead of on a delay, resulting in more responsive movement

## [v0.4.484] - 2026-03-19

### Changed
- **Unit visuals overhaul** — special unit types (brutes, healers, dispellers, shielders, commanders) now keep their base soldier appearance instead of swapping to a different model, making them easier to recognize on the battlefield
- **Colored glow effects** — special units now pulse with a distinctive color glow to show their role: healers glow green, dispellers glow white, shielders glow purple, commanders glow orange, and brutes glow purple
- **Elite glow** — elite units now glow red with a pulsing effect instead of changing their model, and the glow stacks with their role color if they have one
- **Brute appearance** — brutes now look like oversized infantry soldiers instead of a different shape, making them feel more like part of the army
- **Elite upgrades** — any attacker unit type can now become elite (not just basic infantry and archers), so you may encounter elite healers, elite commanders, and other enhanced specialists
- **Elite attack speed** — elite units now attack faster in addition to their existing health, damage, and movement bonuses
- **Shielded unit protection** — units protected by a shielder's spell shield now take 20% less damage from all non-spell sources (melee attacks, arrows, ground effects)

### Fixed
- **Dispel vs shields** — the Dispel spell now properly removes spell shields from protected units

## [v0.4.454] - 2026-03-18

### Added
- **Arcane Crystal talents** — the Arcane Crystal spell now has a full talent tree with 9 talents across 3 tiers, unlocked by absorbing and re-emitting spells through the crystal
- **Tier 1: Crystal modifiers** — choose between 25% stronger sub-projectiles, 40% larger crystal range, or 30% longer crystal duration
- **Tier 2: Absorption effects** — choose between 50% more sub-projectiles per absorption (also speeds up auto-crystal turrets), a powerful energy burst after every 3 absorptions, or a 30% chance to duplicate absorbed spells entirely
- **Tier 3: Powerful upgrades** — choose between placing up to 3 crystals that chain spells between each other, a massive explosion when the crystal expires, or turning the crystal into a permanent magic missile turret that persists between levels
- **Permanent crystal turrets** — with the Auto-Crystal talent, placed crystals become permanent fixtures that fire rapid magic missiles at nearby enemies and survive between levels unless dispelled
- **Dynamic text sizing** — spell and talent descriptions in the Study Spells panel now automatically shrink to fit when they're too long

## [v0.4.440] - 2026-03-18

### Added
- **Dispel talents** — the Dispel spell now has a full talent tree with 9 talents across 3 tiers, unlocked by dispelling spell effects
- **Tier 1: Casting modifiers** — choose between also removing buffs from affected units, 40% faster cooldown, or near-zero mana cost
- **Tier 2: Counter effects** — choose between recovering mana when you dispel a spell, dispelled effects detonating for area damage, or a faster and larger dispel projectile
- **Tier 3: Powerful upgrades** — choose between an instant radial pulse that wipes all spells in a huge area, redirecting dispelled offensive spells back at the nearest enemy, or creating a persistent anti-magic dead zone where no spells work
- **Battlefield backdrop** — the area behind the battlefield now has textured walls and a painted stone floor forming a three-sided enclosure
- **Lava pool effects** — the lava pool on the stone floor now emits persistent fire, smoke, and sparks
- **Water pool ripples** — the water pool on the stone floor now shows subtle expanding ring ripples
- **Fireball explosion gradient** — fireball explosions now have a bright yellow-to-orange radial gradient instead of a flat color
- **Scorched Earth fire VFX** — Scorched Earth zones now show rising fire and smoke instead of a flat orange circle

### Changed
- **Battlefield color** — the ground plane is now a deeper green
- **Explosion effects** — explosions now have more particles including dark smoke and heat shimmer for a more dramatic look

## [v0.4.381] - 2026-03-18

### Added
- **Banishment talents** — the Banishment spell now has a full talent tree with 9 talents across 3 tiers, unlocked by banishing enemies
- **Tier 1: Casting modifiers** — choose between a longer 12-second banish duration, 50% faster casting, or 30% reduced mana cost
- **Tier 2: Return effects** — choose between banished enemies taking heavy damage when they reappear, enemies reappearing at a random faraway location, or banishing two targets at once for extra mana
- **Tier 3: Powerful upgrades** — choose between enemies returning at half health no matter what, a mass banishment that removes all nearby enemies at once (high cost, short duration), or a finishing move that permanently removes low-health enemies
- **Banishment visual effects** — banishing a unit now shows a green lensing sphere that rapidly collapses inward, accompanied by a burst of green sparks

## [v0.4.373] - 2026-03-17

### Added
- **Berserker Rage talents** — the Berserker Rage spell now has a full talent tree with 9 talents across 3 tiers, unlocked by buffing units with rage
- **Tier 1: Rage modifiers** — choose between a much stronger damage boost at the cost of taking more damage, a safer rage with less damage bonus but lower vulnerability, or a 50% larger rage radius to buff more units at once
- **Tier 2: Survival effects** — choose between enraged units healing themselves for a portion of each attack's damage, a last-stand effect where enraged units survive a killing blow and become temporarily unkillable, or a frenzy mode that increases attack speed when below half health
- **Tier 3: Powerful upgrades** — choose between massively boosted damage and vulnerability, rage that spreads to a nearby ally whenever an enraged unit kills an enemy, or a death explosion where enraged units detonate for area damage when they die
- **Berserker Rage visual effects** — enraged units now glow red while the buff is active, and the Final Stand death explosion creates a fireball burst with sparks and smoke

## [v0.4.365] - 2026-03-17

### Added
- **Fog Cloud talents** — the Fog Cloud spell now has a full talent tree with 9 talents across 3 tiers, unlocked by evading attacks in the fog
- **Tier 1: Fog modifiers** — choose between a higher evasion chance (55% instead of 40%), a 40% larger fog radius, or a longer-lasting evasion effect that lingers 2 seconds after leaving the fog
- **Tier 2: Battlefield control** — choose between halving the attack range of all units inside the fog, making units inside the fog invisible to ranged attacks from outside, or a disorienting effect where units in the fog have a 20% chance to accidentally attack an ally
- **Tier 3: Powerful upgrades** — choose between spawning ghostly phantom decoys inside the fog that distract enemies, dealing damage over time to all units standing in the fog, or making the fog slowly drift toward incoming attackers
- **Fog Cloud visual overhaul** — Fog Cloud now uses a dense, swirling gray smoke particle effect instead of a flat ground circle, making the fog much more visible and atmospheric on the battlefield

## [v0.4.353] - 2026-03-16

### Added
- **Healing Plume talents** — the Healing Plume spell now has a full talent tree with 9 talents across 3 tiers, unlocked by restoring health with the healing zone
- **Tier 1: Zone modifiers** — choose between 40% stronger healing per tick, a 40% larger zone radius, or 50% longer zone duration
- **Tier 2: Secondary effects** — choose between periodically cleansing debuffs from all units inside the zone, excess healing converting into temporary HP (up to 20 bonus HP), or doubling healing for units below 30% health
- **Tier 3: Powerful upgrades** — choose between units that die inside the zone being resurrected after a short delay, the zone becoming mobile and following your cursor (at slightly reduced healing), or converting the nearest defender into a temporary healer who fires heal bolts at hurt allies for the zone's duration
- **Field Medic visual feedback** — units converted into temporary healers turn green so you can easily spot them on the battlefield, and revert to their original appearance when the zone expires

### Changed
- **Healing Plume effects are indiscriminate** — the healing zone and all its talent effects now affect all units inside, regardless of team, just like the base healing spell

## [v0.4.339] - 2026-03-16

### Added
- **Raise The Dead talents** — the Raise The Dead spell now has a full talent tree with 9 talents across 3 tiers, unlocked by raising corpses
- **Tier 1: Resurrection modifiers** — choose between a 60% larger resurrection radius, channeling that starts at maximum speed instead of ramping up, or 30% reduced mana cost per corpse
- **Tier 2: Undead enhancements** — choose between raised undead having 50% more HP and dealing 25% more damage, a poisonous aura around each raised undead that damages and sickens nearby enemies, or a corpse magnet that pulls nearby bodies toward your cursor while channeling
- **Tier 3: Powerful upgrades** — choose between raised undead becoming powerful Revenants with 5x HP and heavy damage that passively resurrect nearby corpses on their own, undead that explode in a fiery blast when they die dealing heavy area damage, or a chain reaction where enemies slain by your undead are automatically raised as new undead at no mana cost
- **Plague Bearer visual effects** — undead with the poison aura emit swirling green plague smoke and turn nearby enemies green as the poison takes hold
- **Undead Detonation visual effects** — exploding undead create a fireball-style explosion with sparks, smoke, and heat shimmer, and leave no corpse behind

### Fixed
- **Spell sounds no longer persist after battle** — all spell sound effects now stop immediately when the battle ends, instead of continuing to play through the results screen

## [v0.4.323] - 2026-03-16

### Added
- **Teleport talents** — the Teleport spell now has a full talent tree with 9 talents across 3 tiers, unlocked by teleporting units
- **Tier 1: Teleport modifiers** — choose between a 50% larger source circle, 40% faster second cast time, or a destination marker that lingers for 5 seconds allowing a second teleport to the same spot
- **Tier 2: Secondary effects** — choose between stunning teleported units for 2 seconds on arrival (then giving them a brief attack speed boost), swapping all units between two circles simultaneously, or an instant-cast emergency recall that brings nearby allies back to the King
- **Tier 3: Powerful upgrades** — choose between opening a persistent two-way portal that units can walk through for 10 seconds, teleporting all units at the target straight up into the sky where they fall and take impact damage, or scattering all units in the area to random locations across the battlefield at a reduced mana cost
- **Teleport visual effects** — casting Teleport now creates a spatial ripple distortion at both the source and destination, and Dimensional Rift portals bend light around them with a subtle gravitational lensing effect

## [v0.4.296] - 2026-03-15

### Added
- **Haste talents** — the Haste spell now has a full talent tree with 9 talents across 3 tiers, unlocked by buffing units with Haste
- **Tier 1: Haste modifiers** — choose between 40% stronger speed boost, 50% longer buff duration, or 50% faster cast time
- **Tier 2: Secondary effects** — choose between hasted units also attacking 20% faster, a damage bonus that kicks in for 2 seconds after the speed buff expires, or hasted units dodging the first attack against them
- **Tier 3: Powerful upgrades** — choose between doubling all speed and attack bonuses at the cost of halving the duration, leaving behind a slow field that slows all non-hasted units in the area, or the buff jumping to the nearest un-hasted ally when it expires (up to 4 jumps, losing effectiveness each time)

## [v0.4.281] - 2026-03-15

### Added
- **Mind Control talents** — the Mind Control spell now has a full talent tree with 9 talents across 3 tiers, unlocked by mind-controlling enemies
- **Tier 1: Mind Control modifiers** — choose between 40% longer effect duration on all mind control effects, controlled units dealing 25% more damage, or a 40% faster cast time
- **Tier 2: Secondary effects** — choose between controlling up to 5 units at once instead of 3, a demoralizing aura around controlled units that makes nearby enemies take more damage, or a confused state after mind control ends where the unit attacks random targets
- **Tier 3: Powerful upgrades** — choose between permanently taking over a single enemy, unleashing mass hysteria in a large area where all affected enemies attack anything near them regardless of team, or planting a sleeper agent that appears normal but betrays its allies with a surprise attack after a delay
- **Mass Hysteria indicator** — casting Mass Hysteria now shows a purple ring showing the area of effect while aiming

### Changed
- **Mind Control and Polymorph respect wizard range** — both spells now properly limit targeting to the wizard's spell range, preventing casting on enemies too far away

## [v0.4.262] - 2026-03-15

### Added
- **Polymorph talents** — the Polymorph spell now has a full talent tree with 9 talents across 3 tiers, unlocked by polymorphing enemies
- **Tier 1: Polymorph modifiers** — choose between a longer 14-second transformation, fragile sheep with very low health, or a 40% faster cast time
- **Tier 2: Secondary effects** — choose between explosive sheep that detonate on death for area damage, contagious polymorph that jumps to the nearest unit when it expires (and keeps jumping forever), or turning enemies into panicked pigs that flee at high speed
- **Tier 3: Powerful upgrades** — choose between making the transformation permanent if the sheep survives its full duration, transforming all enemies in an area at once for a steep mana cost, or summoning a friendly battle sheep with high health that headbutts enemies

## [v0.4.246] - 2026-03-14

### Added
- **Grease talents** — the Grease spell now has a full talent tree with 9 talents across 3 tiers, unlocked by slowing and burning enemies
- **Tier 1: Grease modifiers** — choose between 30% stronger slowing, a 40% larger puddle, or 50% more fire damage when ignited
- **Tier 2: Secondary effects** — choose between a chance to trip enemies when they enter the grease, increased spell damage against greased enemies, or fire that resets the zone's full duration
- **Tier 3: Powerful upgrades** — choose between chain ignition that spreads fire between nearby grease puddles, an explosive geyser that launches enemies into the air on ignition, or a self-regenerating puddle that reforms after burning out
- **Grease zone VFX** — grease puddles now have rising fumes, bubbling oil, and edge splatters for a more visceral look
- **Grease spawn animation** — grease puddles now expand outward when cast instead of appearing instantly
- **Grease fire VFX** — ignited grease now produces thick orange and black smoke instead of just changing color
- **Fall damage** — units launched into the air (like by the Grease Geyser talent) now take damage when they land based on how far they fell

### Changed
- **Grease puddles no longer render on top of units** — the grease circle now always appears beneath units walking through it, instead of occasionally popping in front of them

## [v0.4.227] - 2026-03-13

### Changed
- **Smarter spell effects** — status effects like sleep talents, battle hymn buffs, entangle thorns, and grease fire now work more cleanly behind the scenes, improving consistency and fixing edge cases
- **Sleep damage-over-time now respects shields** — the Night Terrors talent now correctly reduces temporary hit points before dealing health damage, matching how all other damage-over-time effects work

### Fixed
- **Night Terrors can now finish off enemies** — previously, damage from the Night Terrors sleep talent couldn't actually kill enemies; now enemies properly die if their health reaches zero from the effect
- **Contagious sleep no longer wastes processing** — the Narcoleptic Wave effect now properly cleans itself up after spreading, instead of lingering uselessly

## [v0.4.217] - 2026-03-13

### Added
- **Sleep talents** — the Sleep spell now has a full talent tree with 9 talents across 3 tiers, unlocked by putting enemies to sleep
- **Tier 1: Sleep modifiers** — choose between 40% longer sleep duration, a 40% larger sleep area, or 50% more bonus damage when enemies wake up
- **Tier 2: Secondary effects** — choose between a contagious sleep that spreads to nearby enemies after 3 seconds, minor damage per second while sleeping, or halved cast time with reduced mana cost
- **Tier 3: Powerful upgrades** — choose between making enemies nearly impossible to wake unless they take massive damage, causing sleeping enemies to sleepwalk back toward their spawn point for 30 seconds, or instantly finishing off any low-health enemies caught in the spell

### Fixed
- **Talent progress no longer lost when switching spells** — previously, opening the spell book mid-battle could reset your accumulated talent progress for the current spell

## [v0.4.202] - 2026-03-13

### Added
- **Squall talents** — the Squall spell now has a full talent tree with 9 talents across 3 tiers, unlocked by hitting enemies with ice
- **Tier 1: Storm modifiers** — choose between 30% more damage per shard, a 30% larger storm area, or a barrage of faster but slightly weaker ice shards
- **Tier 2: Secondary effects** — choose between permafrost that freezes enemies solid after multiple hits, heavy hailstones that deal triple damage, or blinding sleet that causes enemies to miss attacks
- **Tier 3: Powerful upgrades** — choose between a channeled blizzard you aim with your cursor that stacks a devastating slow, a mobile storm that follows your mouse, or lingering frozen ground that slows enemies long after the storm ends
- **Swirling snow effects** — active storms now have swirling snow particles drifting through the air
- **Spell aiming indicators** — Fireball, Black Hole, and Raise the Dead now show an aiming circle when casting, matching all other area spells

### Changed
- **Spell aiming circles redesigned** — all spell aiming reticles are now thin ring outlines instead of filled circles, making them much less distracting while still clearly showing the area of effect
- **Consistent aiming ring thickness** — the ring width is now the same for all spells regardless of their size, so small and large spells both look clean

### Fixed
- **Squall no longer hits friendly units** — ice explosions, frozen ground, and storm effects now correctly only affect enemies

## [v0.4.149] - 2026-03-12

### Added
- **King's Retreat** — when the King pushes too far toward the edge of the wizard's range, he calls for a retreat and all defenders fall back to safety for 15 seconds, giving you time to regroup
- **Retreat announcement** — a golden flash message appears on screen when the King calls for retreat, just like wave announcements

### Changed
- **Wave and retreat announcements padded** — text banners no longer run into the edges of the screen
- **Wall of Fire talent progress rebalanced** — talent tier thresholds adjusted to better match the spell's usage patterns

## [v0.4.140] - 2026-03-11

### Added
- **Spike Growth talents** — Spike Growth now has a full talent tree with 9 talents across 3 tiers, unlocked by damaging enemies with spikes
- **Tier 1: Zone modifiers** — choose between a 40% larger zone, 60% more damage per tick, or faster and cheaper casting
- **Tier 2: Secondary effects** — choose between a much stronger slow that enemies actively avoid, lingering poison damage after leaving the zone, or quicksand that roots units in place after 2 seconds
- **Tier 3: Powerful upgrades** — choose between a growing zone that extends its duration when enemies die inside, splitting into 3 smaller zones in a triangle pattern, or launching spike projectiles at nearby enemies every few seconds
- **Spike Growth vine effects** — spike growth zones now show animated green vine and red spike rings rising from the ground instead of a flat green circle
- **Back wall backdrop** — added a new scenic backdrop along the back wall of the battlefield

### Fixed
- **Plague Wind no longer permanently slows enemies** — enemies hit by Plague Wind would sometimes have their speed permanently reduced even after recovering; they now properly regain full speed after the sickness wears off

## [v0.4.123] - 2026-03-11

### Added
- **Entangle talents** — the Entangle spell now has a full talent tree with 9 talents across 3 tiers, unlocked by rooting enemies
- **Tier 1: Root modifiers** — choose between roots that last 50% longer, a 40% larger area at higher mana cost, or cheaper and faster casting
- **Tier 2: Secondary effects** — choose between thorny vines that damage all rooted units over time, a lingering slow after the root wears off, or mana regeneration for each enemy rooted
- **Tier 3: Powerful upgrades** — choose between an ever-expanding vine zone that roots newcomers, a protective blessing that shields your defenders instead of rooting them, or a strangling burst that deals heavy damage when roots expire
- **Entangle vine effects** — the entangle zone now sprouts dark green vine rings that rise from the ground, with animated vines that continuously grow and shrink during the spell's duration
- **Entangle base duration increased** — roots now last 15 seconds instead of 5, making the spell much more impactful

### Changed
- **Wall of Fire now redirects enemies** — enemies will now actively path around Wall of Fire instead of walking straight through it
- **Smoother unit movement** — units now follow more natural, fluid paths instead of moving in rigid 8-directional steps
- **Faster pathfinding updates** — attacker and defender pathfinding now updates simultaneously instead of one at a time

## [v0.4.100] - 2026-03-11

### Added
- **Wall of Stone talents** — the Wall of Stone spell now has a full talent tree with 9 talents across 3 tiers, unlocked by placing walls
- **Tier 1: Resource and build options** — choose between cheaper walls, tougher walls with extra width, or faster-casting walls that cost less mana but place two shorter walls
- **Tier 2: Defensive effects** — choose between walls that reflect damage back to attackers, an icy aura that slows nearby enemies, or walls that slowly regenerate their health
- **Tier 3: Powerful upgrades** — choose between walls that explode when destroyed, walls that last forever instead of crumbling, or placing extra walls at reduced cost
- **Wall rise and sink animations** — walls now dramatically rise up from the ground when placed and sink back down when they expire or are destroyed, with a brown dust cloud effect
- **Dispelled walls sink gracefully** — walls removed by Dispel now sink into the ground with dust effects instead of instantly vanishing

### Fixed
- **Units no longer get stuck when walls are removed** — pathfinding now properly updates when a wall blocking a path is destroyed or expires

## [v0.4.90] - 2026-03-10

### Added
- **Wall of Fire talents** — the Wall of Fire spell now has a full talent tree with 9 talents across 3 tiers, unlocked by burning enemies
- **Tier 1: Fire modifiers** — choose between doubled fire damage, a much wider and longer-lasting wall, or a longer but shorter-lived wall with extra damage
- **Tier 2: Burn effects** — choose between reducing healing on burning enemies, leaving a slowing zone after the wall expires, or setting enemies on fire even after they leave the wall
- **Tier 3: Devastating upgrades** — choose between enemies exploding on death inside the wall, placing two parallel walls at once, or fire that grows stronger the longer the wall burns
- **Wall of Fire smoke effects** — the wall now billows with thick orange and black smoke that rises and swirls, giving it a much more dramatic look with natural color and size variation
- **Heat distortion shader** — a subtle screen-space heat shimmer effect warps the air above active walls of fire, similar to how real heat distortion looks
- **Healing reduction mechanic** — a new system that lets effects reduce how much healing a unit receives

### Changed
- **Wall of Fire visuals reworked** — the wall is now rendered as rising smoke particles instead of a solid colored block, looking much more like actual fire

## [v0.4.66] - 2026-03-10

### Added
- **Plague Wind talents** — the Plague Wind spell now has a full talent tree with 9 talents across 3 tiers, unlocked by poisoning enemies
- **Tier 1: Cloud modifiers** — choose between stronger poison damage, a bigger cloud with shorter duration, or a slower cloud that lingers much longer
- **Tier 2: Debuff effects** — choose between a lingering poison on enemies who leave the cloud, making poisoned enemies take more damage from all sources, or slowing enemies inside the cloud
- **Tier 3: Devastating upgrades** — choose between clouds that spread from enemy deaths, splitting the cloud into two diverging plumes, or poison that permanently reduces enemies' max health
- **Plague Wind particle cloud** — the poison cloud is now rendered as a swirling volumetric particle effect instead of a flat green circle, giving it real depth and atmosphere
- **Click-drag casting for Plague Wind** — click to place the cloud origin, then drag to aim the direction it will travel. A green arrow indicator shows the wind direction while casting
- **Window position saving** — the game now remembers where you placed the window on your screen and restores it on next launch

### Changed
- **Plague Wind travels faster** — the cloud now moves at double its previous speed
- **Pandemic child clouds are bigger and last longer** — clouds spawned from enemy deaths are now 75% the size of the parent (up from 50%) and last 8 seconds (up from 4)
- **Gravitational lensing separated** — the black hole lensing visual effect now runs independently from the CRT screen filter for cleaner rendering

## [v0.4.47] - 2026-03-10

### Added
- **Black Hole talents** — the Black Hole spell now has a full talent tree with 9 talents across 3 tiers, unlocked by pulling enemies in
- **Tier 1: Core modifications** — choose between stronger gravitational pull, a larger but weaker black hole, or faster casting
- **Tier 2: New effects** — choose between a deadly inner zone that deals double damage, a slowing field that persists after leaving, or healing your nearest injured defender with damage dealt
- **Tier 3: Powerful upgrades** — choose between a massive explosion when the black hole expires, splitting into two smaller black holes, or periodic bursts that yank enemies to the center
- **Gravitational lensing effect** — active black holes now visually bend the light around them, creating a screen-space distortion that looks like real gravitational lensing
- **Black hole screen darkening** — the screen gradually dims as a black hole grows to full size, adding to the ominous atmosphere

### Changed
- **Black hole visuals reworked** — the black hole center is now rendered entirely through the lensing shader, creating a cleaner, more natural-looking dark core instead of a flat sprite

## [v0.4.25] - 2026-03-10

### Added
- **Lightning Rod talents** — the Lightning Rod spell now has a full talent tree with 9 talents across 3 tiers, unlocked by hitting enemies with arcs
- **Tier 1: Rod enhancements** — choose between a longer-lasting rod, faster lightning strikes, or a wider arc that hits more targets
- **Tier 2: New effects** — choose between chain arcs that jump to additional enemies, a slowing field on hit enemies, or supercharged strikes every few hits
- **Tier 3: Powerful upgrades** — choose between placing two rods at once, lightning that ramps up in power over time, or bonus strikes triggered by kills (with diminishing returns)

## [v0.4.17] - 2026-03-09

### Added
- **Mark of Death talents** — the Mark of Death spell now has a full talent tree with 9 talents across 3 tiers, unlocked by marking enemies in battle
- **Tier 1: Mark enhancements** — choose between stronger damage amplification, a longer-lasting mark, or a mana refund when the marked target dies
- **Tier 2: Death effects** — choose between the mark jumping to a nearby enemy on death, a burst of damage when the target's health gets low, or making your defenders focus on the marked target
- **Tier 3: Advanced curses** — choose between marking all enemies in an area at once, a necrotic explosion when a marked enemy dies, or a mark that grows stronger over time and can never be removed
- **Mark visual indicator** — marked enemies now display a glowing purple circle above them so you can easily see who's marked
- **Action bar keyboard feedback** — pressing a number key (1–5) now visually highlights the corresponding action bar button, matching the look of clicking it with the mouse

## [v0.4.16] - 2026-03-09

### Added
- **Finger of Death talents** — the Finger of Death spell now has a full talent tree with 9 talents across 3 tiers, unlocked by killing enemies with the beam
- **Tier 1: Casting options** — choose between a wider beam, mana refund on kills, or faster casting
- **Tier 2: Bonus effects** — choose between raising killed enemies as undead, a cheaper but weaker beam with shorter cooldown, or healing your nearest injured defender when the beam hits
- **Tier 3: Devastating upgrades** — choose between a sweeping arc that cuts through groups, explosive bursts on kills, or a cascading chain that jumps between enemies

### Changed
- **Finger of Death visual overhaul** — the beam now uses a pointed triangle shape with pulsing purple glow, color cycling, shimmer effects, and a bright flare at the origin
- **Finger of Death base damage reduced** — lowered from 1000 to 500 to better balance with the new talent options

## [v0.4.15] - 2026-03-09

### Added
- **Guardian Circle talents** — the Guardian Circle spell now has a full talent tree with 9 talents across 3 tiers, unlocked by shielding units in battle
- **Tier 1: Shield enhancements** — choose between stronger shields, longer-lasting shields, or a wider circle area (with slightly weaker shields)
- **Tier 2: Combat bonuses** — choose between an explosive burst when a shield breaks, a damage boost for shielded units, or faster casting speed
- **Tier 3: Advanced effects** — choose between damage reduction while shielded, a deathblow explosion when a shielded unit falls, or shields that jump to a nearby ally when a shielded unit dies

## [v0.4.14] - 2026-03-09

### Added
- **New wizard type: Psychopath** — a twisted wizard who thrives on carnage. Your spells deal 30% extra damage to your own defenders, and you must eliminate at least 70% of them before the last attacker falls — otherwise you lose. Winning isn't enough; you need destruction
- **Carnage meter on the score screen** — when playing as the Psychopath, a carnage tracker shows how many defenders you've eliminated and whether you've met the threshold
- **New defeat message** — if you fail to cause enough destruction as the Psychopath, you'll see a unique defeat screen explaining why

## [v0.4.13] - 2026-03-09

### Added
- **New wizard type: Shepherd** — a support-only wizard that cannot cast any spell that deals damage. In exchange, all support spells are 30% more powerful — bigger heals, stronger shields, longer buffs, and tougher walls. Win using only your defenders' strength
- **New achievement** — a new hidden achievement that unlocks the Shepherd wizard type. Can you figure out how to earn it?

### Changed
- **Discord update notifications now arrive after downloads are ready** — release announcements in Discord now wait until all platform builds are finished, so download links work immediately

## [v0.4.12] - 2026-03-08

### Added
- **macOS and Linux downloads** — the game now has official release builds for macOS (Apple Silicon) and Linux in addition to Windows

### Changed
- **Improved build system** — the build script now supports building for macOS and Linux targets directly

## [v0.4.11] - 2026-03-08

### Added
- **Native desktop app** — the game now runs as a standalone desktop application on Windows, Linux, and macOS instead of in a web browser
- **Barrel distortion cursor fix** — buttons and UI elements now respond correctly to clicks and hovers when the CRT screen curvature effect is active

### Changed
- **Save files moved to your computer** — game saves, settings, and progress are now stored in your system's data folder instead of the browser
- **Multiplayer temporarily disabled** — online multiplayer is offline while it's being rebuilt for the native version

### Removed
- **Browser version retired** — the web-based version has been replaced by the native desktop app

## [v0.3.412] - 2026-03-08

### Added
- **Destructible walls** — Walls of Stone now have health and can be destroyed by units. When units are completely trapped by walls with no path forward, they'll attack the walls to break free
- **Wall damage visuals** — walls gradually change color as they take damage, shifting from their normal color to a darker cracked appearance
- **Wall-aware archers** — archers no longer shoot through walls, and won't stop to fire at enemies when a wall is blocking the approach path. They'll navigate around walls like other units do

### Changed
- **Smarter wall targeting** — trapped units now push toward the king instead of running to the nearest wall on the map, so they converge on the walls that are actually blocking them
- **Better unit pathfinding around walls** — units navigate more smoothly around walls instead of trying to push straight through them

## [v0.3.400] - 2026-03-07

### Added
- **Meteorologist wizard type** — a new unlockable wizard that controls the weather on the battlefield. Press Q, W, or E to activate Storm, Blizzard, or Drought, each applying different status effects to all units on the field
- **Storm weather** — drenches and charges all units. Shocked units spread electricity to nearby wet targets, electric arcs chain to extra targets, and random lightning strikes hit the battlefield
- **Blizzard weather** — chills all units. Frost spells slow even harder than normal, and can freeze units solid briefly
- **Drought weather** — dries out all units. Fire spells create burning ground patches around their impact, and all healing is reduced
- **Weather intensity** — weather effects grow stronger the longer they persist, shown by a fill bar on each weather button
- **Weather visuals** — rain with sky darkening during storms, snowfall with ground whitening during blizzards, heat haze with ground browning during drought, and lightning strike beams during storms
- **Weather sounds** — ambient rain and blizzard sounds play during their respective weather conditions, and each lightning strike has its own sound effect
- **Weather bar UI** — a new set of clickable buttons at the bottom of the screen for switching weather, with hotkeys and a status description
- **New achievement** — a hidden achievement that unlocks the Meteorologist wizard type

### Changed
- **Squall sound quieter** — Squall wind impacts are now much quieter

## [v0.3.384] - 2026-03-07

### Added
- **Swordcerer wizard type** — a new unlockable wizard that can jump down from the tower and fight on the battlefield in person. Click "Enter the Fray" to choose where to spawn, then use WASD to move, left-click to shoot magic missiles, and right-click to swing a sword arc that lunges you forward
- **Swordcerer health bar** — a health bar appears at the bottom of the screen while fighting on the field. If your health runs out, you teleport back to the tower and can't re-enter the fray for the rest of the battle
- **Swordcerer mana usage** — magic missiles fired while on the field still cost mana, so you'll need to manage your mana pool even in close combat
- **New achievement** — a hidden achievement that unlocks the Swordcerer wizard type

## [v0.3.370] - 2026-03-07

### Added
- **Warglock wizard type** — a new unlockable wizard that ditches spells entirely in favor of 5 guns: Machine Gun, Magnum, Rocket Launcher, Shotgun, and Flamethrower. Each gun has its own ammo pool and reload timer
- **Ammo system** — each gun tracks its own ammo separately. Guns auto-reload when empty, or press R to reload early. Switching guns doesn't affect other guns' ammo
- **Ammo display** — a new ammo counter and visual ammo bar replaces the mana bar when playing as the Warglock
- **Reload bar** — the cast bar shows reload progress with amber fill when reloading a gun
- **Gun sound effects** — unique firing sounds for each gun type
- **Bullet hit flash** — enemies briefly flash white when hit by bullets
- **Bullet tracers** — light gray speed-line visuals follow each shot
- **Flamethrower physics** — flame projectiles arc downward with gravity and leave burning ground where they land
- **Rocket explosions** — rockets explode on impact with area-of-effect damage
- **New achievement** — a hidden achievement that unlocks the Warglock wizard type

## [v0.3.342] - 2026-03-06

### Added
- **New wizard type** — a 6th unlockable wizard specializing in brewing, with faster brew times, longer-lasting buffs, and a powerful once-per-battle special ingredient that removes dilution from your brews
- **New achievement** — a hidden achievement tied to collecting every ingredient, which unlocks the new wizard type

### Changed
- **Sound effects quieter** — all sound effects are now 30% quieter to better balance with the music

## [v0.3.339] - 2026-03-06

### Added
- **Level clock** — a timer now appears in the upper right corner of the screen during battles, showing how long you've been fighting. Can be toggled on or off in Settings
- **Urgent Mode** — when enabled (on by default), the battle keeps going even while you're browsing the Spell Book or Cauldron menu. Units keep fighting, spells keep flying, and time keeps ticking. Toggle it off in Settings if you prefer the classic pause-while-browsing behavior

## [v0.3.337] - 2026-03-06

### Added
- **Poison damage type** — certain spells now deal Poison damage, which stacks an effectiveness penalty on hit units. Enough poison buildup causes the target to become Sickened (briefly stunned) and then Smelly
- **Smelly debuff** — smelly units get a brown tint and cause nearby allies to flee in disgust, breaking their formation
- **Excremage wizard** — a new unlockable wizard type that converts all spells into Poop damage with brown visuals, gross sound effects, and smelly results
- **Cauldron bubbling sound** — brewing now plays a satisfying bubbling sound when a brew finishes

### Changed
- **Plague Wind and Spike Growth** now deal Poison damage instead of their previous types
- **Ingredient descriptions split** — the cauldron menu now shows only what each ingredient does, while the Compendium shows the full lore and flavor text
- **Spell sound effects** — improved cast and impact sounds with distinct audio for different spell phases

## [v0.3.318] - 2026-03-06

### Added
- **Compendium** — a brand new encyclopedia replaces the old Progress screen, with six browsable tabs: Spells, Ingredients, Units, Wizards, Achievements, and Stats. Select any item to see its full details in the left panel
- **Unit tracking** — enemy and defender unit types are now tracked as you encounter them, unlocking their Compendium entries
- **Unit encounter achievements** — earn a new achievement the first time you encounter each enemy unit type on the battlefield
- **Ingredient descriptions** — all ingredients now have detailed descriptions of their appearance and where they're found in the wild
- **Flavor text** — unlocked Compendium items now show witty flavor text at the bottom of their detail panel
- **Level history** — the Stats tab shows a scrollable list of every level you've completed with color-coded efficiency ratings (gold for perfect, red-to-green gradient for everything else)
- **Expanded stats** — the Stats tab now shows Battle Stats, Kill Stats, Collection progress (including Talents Unlocked), and Economy info in a two-column layout
- **Compendium everywhere** — accessible from the Main Menu, Wizard Tower, and Pause Menu

### Changed
- **Clear Progress moved to Settings** — the Clear Progress button now lives in the Settings page with a simple confirmation toast instead of a popup dialog

## [v0.3.310] - 2026-03-05

### Added
- **Tutorial system** — new players are now guided through the game with step-by-step tutorials that appear when visiting each screen for the first time. Tutorials cover the Wizard Tower hub, Study screen, in-game battles, spell book, cauldron menu, and time travel
- **Highlighted UI elements** — during tutorials, the element being explained gets a pulsing golden border so you know exactly what to look at
- **Tutorial settings** — tutorials can be toggled on/off and reset from the Settings menu
- **Time Travel tutorial** — a dedicated tutorial walks you through replaying past levels once you unlock Time Travel

### Changed
- **In-game tutorial pauses gameplay** — the battle tutorial now freezes the game while active so you can read at your own pace

## [v0.3.290] - 2026-03-05

### Added
- **Cauldron menu redesign** — the cauldron menu now has a two-panel layout with a detail preview on the left and categorized ingredient grid on the right. Ingredients are organized into Offense, Control, Support, and Utility categories for easier browsing
- **Brew preview** — selecting ingredients in the cauldron now shows a live preview of the brew's effects, combo bonuses, brew time, and buff duration before you commit
- **Buff tracker** — active brew buffs now appear as small labeled boxes in the top-left corner of the HUD, showing what each buff does and how long it lasts. Hover over a box to see the full effect details
- **Combo discovery is saved** — discovered ingredient combos are now remembered across sessions

### Changed
- **Archers hold position smarter** — archers now advance toward enemies until they're within effective range, then stop and fire instead of awkwardly standing at maximum distance
- **King's Guard stay closer** — the King's Guard now orbit tighter around the King

## [v0.3.277] - 2026-03-05

### Added
- **Telekinesis talents** — Telekinesis now has a full talent tree with 9 upgrades across 3 tiers:
  - **Tier 1**: Auto-Target (no need to aim near a drop), Quick Grab (near-instant cast time), Mana Efficiency (half mana cost)
  - **Tier 2**: Magnetic Pull (ingredients passively drift toward the wizard), Harvest (picking up ingredients damages nearby enemies with a light blue flash), Keen Senses (50% more ingredient drops)
  - **Tier 3**: Telekinetic Storm (one cast grabs ALL ingredients on the field at 3x mana), Transmutation (each ingredient collected boosts your next brew by 10%), Psychic Shockwave (pickups create an expanding shockwave that knocks enemies back)

### Changed
- **Telekinesis mana cost increased** — base mana cost doubled to balance with new talents
- **Telekinesis talents unlock faster** — talent tier thresholds significantly reduced

## [v0.3.264] - 2026-03-05

### Added
- **Chain Lightning talents** — Chain Lightning now has a full talent tree with 9 upgrades across 3 tiers:
  - **Tier 1**: Conducting Bolts (much longer bounce range, slightly less damage), High Voltage (harder-hitting initial strike but faster damage drop-off), Static Charge (hit enemies are slowed)
  - **Tier 2**: Forked Lightning (bolts split into 3 targets instead of 2), Overcharge (no damage drop-off but only hits one target per bounce), Magnetic Pull (hit enemies are yanked toward the bolt's origin)
  - **Tier 3**: Thunderstorm (fires 3 simultaneous chain lightning strikes at quadruple mana cost), Chain Reaction (killing an enemy causes an explosion and spawns a sub-chain from the corpse), Living Lightning (bounces up to 100 times at double mana cost)

### Changed
- **All units have more health** — every unit on the battlefield now has double the health, making battles last longer
- **Units pack tighter together** — units no longer space themselves out as much, allowing them to overlap slightly for denser formations

## [v0.3.256] - 2026-03-04

### Changed
- **All units are much bigger** — every unit on the battlefield is now significantly larger, making them easier to see and the battles feel more epic
- **Arrows look like arrows** — arrow projectiles are now dark brown rectangles instead of small circles
- **Telekinesis works anywhere** — you can now pick up ingredient drops from any distance on the battlefield

### Fixed
- **Walk animations fixed** — units no longer play the wrong animation when walking forward or backward

## [v0.3.234] - 2026-03-04

### Added
- **Meteor Fall talents** — the Meteor Fall spell now has a full talent tree with 9 upgrades across 3 tiers:
  - **Tier 1**: Dense Barrage (faster meteors), Scorching Impact (stronger explosions and ground fire), Wide Devastation (larger storm area and aiming circle)
  - **Tier 2**: Molten Core (longer and stronger ground fires), Tracking Meteors (meteors steer toward enemies), Aftershock (meteors knock enemies back)
  - **Tier 3**: Extinction Event (after channeling, one massive meteor strikes the entire area), Volcanic Eruption (meteors hitting near existing fires trigger escalating eruption bursts), Meteor Shower (triple the meteors at reduced size and cost)
- **Extinction Event ground fire** — the massive extinction meteor leaves a huge burning zone covering the entire storm area, and enemies reroute around it

### Changed
- **Meteor Fall impacts now louder** — meteor explosions play at full volume instead of half

## [v0.3.228] - 2026-03-04

### Added
- **Time Travel** — replay any previously beaten level from the Wizard Tower hub. A new panel on the right side shows all your cleared levels, with boss levels highlighted by name. Selecting a level and hitting "Start Time Travel" lets you replay it without affecting your progression. After the battle, you return to the tower at your real level

## [v0.3.220] - 2026-03-04

### Added
- **Disintegrate talents** — the Disintegrate spell now has a full talent tree with 9 unique upgrades across 3 tiers, including Focused Lens, Forked Beam, Sweeping Destruction, Annihilation Beam, and more
- **Arcane Crystal inherits disintegrate talents** — when the crystal absorbs or auto-casts disintegrate beams, it now uses your equipped talents (scaled down for balance). Forked beam creates 3-beam spreads from the crystal too
- **Talent progress UI** — the action bar now shows spell talent progress during battle

### Fixed
- **Arcane Crystal beams now despawn properly** — crystal beams no longer linger forever after targets die or the crystal expires
- **Annihilation beam smoke and particles** — visual effects no longer appear at the wizard's position when casting the sky beam variant
- **Arcane Crystal beam collision** — crystal beams now correctly hit ground-level enemies despite originating from the elevated crystal
- **Arcane Crystal no longer targets corpses** — the crystal only targets living enemies now
- **Eliminated entity despawn warnings** — replaced all entity despawns with safe versions that don't warn about already-removed entities

### Changed
- **Talents moved into spell modules** — talent definitions now live alongside each spell instead of in a separate global module

## [v0.3.182] - 2026-03-03

### Added
- **Credits screen** — view the game's credits from the main menu
- **Sprite-based corpses** — when units die, their corpses now show a random frame from their walking animation instead of plain colored circles. Each corpse looks slightly different for visual variety

### Fixed
- **Spell web works properly on all screen sizes** — zooming and panning the spell web no longer drifts off-center on ultrawide monitors or non-standard resolutions
- **Bosses can no longer be mind controlled** — the Mind Control spell now correctly skips boss enemies

### Changed
- **Attacker archers are lighter colored** — attacker archers now have a noticeably lighter tint than attacker infantry, making them easier to tell apart

## [v0.3.174] - 2026-03-03

### Added
- **Animated unit sprites** — infantry, archers, the king, and the king's guard now have walking sprite animations instead of plain colored circles. Units animate when moving and face the direction they're walking
- **Team-colored units** — defenders, attackers, undead, and the king's guard each have distinct color tinting so you can tell teams apart at a glance

### Changed
- **Defenders start closer to the wizard** — the defender army now spawns and rallies nearer to the castle wall

## [v0.3.165] - 2026-03-02

### Added
- **Spell talent system** — each spell now has a talent tree where using the spell in battle earns progress toward unlocking upgrades. Each spell has 3 tiers with 3 choices each — pick one per tier to customize how the spell works. Change your picks anytime from the Study screen
- **Talent progress persists forever** — progress carries over between battles and never resets
- **Talents for Magic Missile, Fireball, and Battle Hymn** — fully playable talent choices for these three spells, with placeholder talents shown for the rest

### Fixed
- **Defenders no longer crawl back to position** — after all enemies were defeated, defenders would sometimes inch back to their spots extremely slowly instead of walking at normal speed
- **Clicking the spell detail panel no longer interacts with the spell web behind it**

## [v0.3.138] - 2026-03-02

### Improved
- **Better game performance** — spells and effects that aren't active no longer run background checks every frame, reducing unnecessary work
- **Fixed a memory leak** — electric arc visual effects were quietly leaking graphics resources every frame; now properly cleaned up
- **Safer number handling** — distance comparisons across the game now handle edge cases gracefully instead of risking crashes

## [v0.3.131] - 2026-03-02

### Added
- **Menu music** — a new ambient track plays during the main menu and between battles
- **Smooth music transitions** — music crossfades smoothly when switching between menus and gameplay instead of cutting abruptly

## [v0.3.130] - 2026-03-02

### Added
- **More spell sound effects** — arcane crystal, banishment, battle hymn, berserker rage, chain lightning, healing plume, and meteor fall impacts now all have sound effects
- **Disintegrate channeling sound** — a continuous sound plays while the disintegrate beam is active
- **Black hole ambient sound** — black holes now emit a looping sound for their entire duration, louder when closer to the wizard

### Changed
- **Distant spells are quieter** — sound effects further from the wizard fade out more aggressively

### Fixed
- **Archers now retreat properly** — defender archers would sometimes get stuck in place instead of returning to their starting positions when there are no enemies on the field

## [v0.3.122] - 2026-03-02

### Added
- **Spell web** — the Study screen now shows your spells as a zoomable constellation-style web instead of a flat grid. Click nodes to see details, scroll to zoom, and drag to pan around
- **Starry sky background** — the spell web floats over a twinkling night sky with a subtle parallax effect as you pan and zoom
- **Spell research progress rings** — nodes with partial research show a radial progress ring around the spell icon

### Changed
- **Spell research tree reorganized** — spells are now grouped into four branches (Offense, Control, Support, Utility) radiating out from the center, each with a clear prerequisite chain
- **Panning is now limited** — you can no longer scroll endlessly past the edges of the spell web

## [v0.3.86] - 2026-03-01

### Added
- **New boss fight: The Hags** — a trio of witches with unique abilities, each with their own health bar section
- **Mind Control spell** — hold to cast on an enemy near your cursor, turning them against their allies for a limited time
- **Spell sound effects** — magic missile and fireball now play cast and impact sounds, with volume fading based on distance from the wizard

### Changed
- **Ogre boss is tougher** — doubled its health
- **Boss health bar** is wider and easier to read
- **Dispel now removes mind control** from affected units

## [v0.3.40] - 2026-02-28

### Changed
- **Dispellers now appear earlier** — enemy dispellers start showing up at level 11 instead of level 16
- **Healers now appear later** — enemy healers start showing up at level 16 instead of level 11

## [v0.3.38] - 2026-02-28

### Added
- **Flow field debug overlay** — press F3 during gameplay to see white triangles showing how units navigate the battlefield. Press again to switch between attacker and defender views, or press a third time to turn it off
- **Skip splash screen** now actually works — previously the setting was ignored

### Fixed
- **Spells no longer block huge square areas** — walls placed at an angle used to block a big square of space around them. Now only the actual area the wall covers is blocked, so units can navigate around diagonal walls much more tightly
- **Circular spell zones** (fire, poison, thorns, grease) no longer block square corners outside their actual radius
- Improved how the game calculates smooth paths near obstacles

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
