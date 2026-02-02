//! Instructions text content.

pub(super) const INSTRUCTIONS_TEXT: &str = "HOW TO PLAY

OBJECTIVE
Defend the castle from incoming enemies by casting spells. Survive as long as possible to achieve a high score.

CONTROLS
- Mouse: Aim spells and interact with UI
- Left Click: Cast spells (hold to channel)
- Escape: Pause game / Open menus
- Keys 1-5: Quick-cast spells from action bar
- Keys Q/W/E/R: Input rune sequences
- Spacebar: Activate rune spell

SPELL BOOK
Click the \"Spells\" button in the top-left to open the spell book. Browse all available spells and click one to prime it for casting. The currently primed spell appears at the bottom-left of the screen.

ACTION BAR
The action bar at the bottom-left shows your 5 quick-cast slots (keys 1-5).

To assign a spell to a slot:
1. Open the spell book
2. Hover over a spell
3. Press a number key (1-5) to assign to that slot

Press keys 1-5 anytime to instantly prime that spell without opening the spell book.

RUNE SYSTEM
Cast empowered spells using rune combinations. The rune display in the bottom-middle shows your current sequence.

Single Runes:
  Q - Magic Missile
  W - Fireball
  E - Teleport
  R - Guardian Circle

Two-Rune Combos:
  QW - Disintegrate
  QE - Chain Lightning
  WE - Wall of Stone
  WR - Raise the Dead
  ER - Finger of Death

How to use:
1. Press rune keys (Q/W/E/R) to build a sequence
2. Press Spacebar to activate when the sequence is valid
3. Spells cast via runes are 25% more powerful!

MANA MANAGEMENT
- Each spell costs mana to cast
- Mana regenerates over time
- Watch your mana bar (top-left) to ensure you can cast
- Some powerful spells have special requirements (e.g., Finger of Death requires full mana)

SPELL TYPES
Instant Spells: Cast immediately when you click
Channeled Spells: Hold the mouse button to continuously cast (Magic Missile, Disintegrate, Raise the Dead)
Placed Spells: Click to place at cursor location (Guardian Circle, Teleport)
Drag Spells: Click and drag to define placement (Wall of Stone)

TIPS
- Use the rune system for 25% bonus damage on critical spells
- Experiment with different spell combinations
- Position walls strategically to funnel enemies
- Teleport can reposition both enemies and allies
- Guardian Circle's temporary HP can save units from death";
