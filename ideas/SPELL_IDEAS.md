# Spell Ideas

Spells for the wizard's arsenal, drawn from classic fantasy, D&D traditions, and arcane lore. The wizard casts from atop the castle wall, assisting defenders against waves of attackers.

## Currently Implemented

| Spell | Type | Damage Type | Description |
|---|---|---|---|
| **Magic Missile** | Channeled projectile | Force | Rapid-fire homing missiles that accelerate over time |
| **Fireball** | AoE explosion | Fire | Explosive projectile with residual burning ground |
| **Teleport** | Utility | Force | Two-stage: place marker, then teleport units in radius to it |
| **Guardian Circle** | Defensive buff | Force | Grants temporary HP to units inside protective circle |
| **Disintegrate** | Channeled beam | Force | Sustained green beam dealing continuous damage along a line |
| **Chain Lightning** | Multi-target | Electric | Strikes nearest enemy, bounces to 4 additional targets |
| **Black Hole** | Gravity field | Force | Pulls units inward, ramping damage, 20s cast time |
| **Wall of Stone** | Terrain obstacle | Force | Drag-to-place wall that blocks movement and pathfinding |
| **Raise the Dead** | Channeled resurrection | Necrotic | Resurrects corpses as Undead allies |
| **Finger of Death** | Instant beam | Necrotic | 1000 damage instant-kill beam, requires 50% mana |
| **Squall** | Concentration AoE | Frost | Persistent ice storm with slow, blocks other casting |
| **Wall of Fire** | Persistent area denial | Fire | Drag-to-place burning wall that damages units passing through |
| **Entangle** | Crowd control | Nature | Roots all units in radius, preventing movement |
| **Haste** | Speed buff | Force | Grants movement speed boost to all units in radius |
| **Spike Growth** | Area denial + slow | Nature | Persistent zone that damages and slows all units |
| **Lightning Rod** | Targeted debuff | Electric | Marks a unit as a lightning rod, attracting chain lightning bounces |
| **Telekinesis** | Ingredient drop | Force | Drops spell ingredients from defeated enemies |
| **Meteor Fall** | Concentration AoE | Fire | Rains meteors that explode on impact and leave burning ground |
| **Mark of Death** | Single-target debuff | Necrotic | Marks an enemy to take increased damage from all sources |
| **Plague Wind** | Moving AoE | Necrotic | Toxic cloud that drifts across the battlefield dealing DoT |
| **Hypnotic Pattern** | AoE stun | Force | Mesmerizes all units in radius; damage breaks the effect |
| **Sleep** | AoE stun | Force | Puts units to sleep; first damage hit deals bonus damage |
| **Grease** | Terrain + combo | Nature | Slippery zone that slows; ignites on contact with fire |
| **Fog Cloud** | Evasion zone | Frost | Fog that grants evasion chance to all units inside |
| **Battle Hymn** | Attack buff | Force | Grants damage and attack speed boost to units in radius |
| **Healing Plume** | Area heal | Nature | Persistent zone that heals all units inside over time |
| **Berserker Rage** | Risky buff | Force | Grants massive damage boost but increases damage taken |
| **Phantasmal Force** | Summon decoys | Force | Spawns illusion infantry that enemies target |
| **Banishment** | Single-target removal | Force | Removes an enemy from the battlefield temporarily |
| **Polymorph** | Single-target control | Nature | Transforms an enemy into a helpless sheep |

## Mechanical Gaps to Fill

Before diving into specific spells, here are the **gameplay mechanic types** currently missing from the roster. The existing spells are heavy on direct damage variants -- adding these categories would create more strategic depth.

| Gap | What It Means | Why It Matters |
|---|---|---|
| ~~**Persistent Area Denial**~~ | ~~Damage-over-time zone enemies avoid~~ | ~~Filled by Wall of Fire, Spike Growth, Meteor Fall~~ |
| ~~**Non-Damage Crowd Control**~~ | ~~Slow, root, stun without killing~~ | ~~Filled by Entangle, Spike Growth, Hypnotic Pattern, Sleep, Grease~~ |
| ~~**Offensive Buffs**~~ | ~~Strengthen allied units' attacks~~ | ~~Filled by Battle Hymn, Berserker Rage~~ |
| ~~**Enemy Debuffs**~~ | ~~Weaken enemy armor, damage, speed~~ | ~~Filled by Mark of Death, Fog Cloud~~ |
| **Trap/Delayed Spells** | Placed in advance, triggers on contact | Rewards prediction and planning |
| **Knockback/Push** | Shove enemies away from the king | Opposite of Black Hole's pull |
| ~~**Channeled Heal**~~ | ~~Sustained healing over time~~ | ~~Filled by Healing Plume~~ |

---

## Backlog

### Battlefield Control (Terrain & Positioning)

Spells that reshape the battlefield itself -- walls, zones, terrain effects. Cast from the wall, the wizard has a god's-eye view perfect for this.

| Spell | Inspiration | Lore Basis | Game Mechanic | Category |
|---|---|---|---|---|
| **Cedar Curse** | Conjuration | A primeval forest guardian's wrath — spectral trees erupt to entrap and crush | Spectral trees erupt from the ground, acting as temporary obstacles that also root nearby enemies in their branches. Blocks pathfinding like Wall of Stone but also damages. | Obstacle + Damage |
| **Seven Gates** | Abjuration | Wards that strip power layer by layer as intruders pass through, like descending into a warded dungeon | Creates a series of 3 luminous gate arches enemies must pass through. Each gate strips a buff and reduces stats (armor, speed, damage) progressively. | Debuff Zone |

### Direct Damage (Offensive Strikes)

New damage spells that offer fundamentally different targeting or interaction patterns from existing ones.

| Spell | Inspiration | Lore Basis | Game Mechanic | Category |
|---|---|---|---|---|
| **Divine Thunderbolt** | Evocation | A bolt of pure elemental fury, forged from storm and wrath, that shatters on impact | Throw a divine thunderbolt to a target point. On impact, it shatters into lightning shards that fly outward in all directions, hitting enemies in a large radius. Unlike Chain Lightning, damage radiates outward from impact. | AoE Burst |
| **Evil Eye** | Transmutation | A baleful gaze that withers all in its path — an ancient and feared power among archmages | Slow-charging beam that sweeps across the battlefield in an arc (not a fixed line like Disintegrate). Deals escalating damage the longer it touches a target. Long cooldown. | Sweeping Beam |
| **Four Winds** | Evocation | Summoning the cardinal winds themselves to converge with devastating force | Winds converge from all four edges of the battlefield toward a target point. Enemies caught in the convergence take massive damage and are flung outward. | Convergence AoE |
| **Unerring Spear** | Conjuration | A spectral javelin guided by divination magic — it cannot miss its mark | Hurl a blazing spear at any visible enemy -- guaranteed hit, pierces armor, massive single-target damage. The anti-Behemoth spell. Very long cooldown. | Guaranteed Single-Target |

### Crowd Control (Slow, Stun, Fear, Confusion)

Non-lethal or low-damage spells that manipulate enemy behavior. Especially valuable when you want corpses intact for Raise the Dead.

| Spell | Inspiration | Lore Basis | Game Mechanic | Category |
|---|---|---|---|---|
| **Dreadful Presence** | Enchantment | Projecting overwhelming supernatural terror — the aura of an ancient wyrm or lich | Creates an aura of fear in a radius. Weak enemies flee in panic; stronger enemies have reduced attack damage and move hesitantly. Breaks formations. | Fear |
| **Vicious Mockery** | D&D Enchantment | A devastating magical insult that wounds the spirit and saps the will to fight | The wizard chants a cutting magical verse. Enemies in a cone take minor damage but suffer heavy debuffs: reduced morale, attack speed, and accuracy. Runes visibly fly from wizard toward targets. | Debuff (Cone) |
| **Arcane Lasso** | Conjuration | Ethereal chains that drag foes together — a battle mage's crowd control specialty | Magical lassoes snake out and drag 3-5 enemies together into a tight cluster, immobilizing them. Perfect setup for Fireball or Chain Lightning. | Root + Cluster |
| **Geas** | D&D Enchantment | A magical compulsion laid upon the mind, forcing obedience under threat of agony | Forces enemies in an area to walk in a specific direction (e.g., away from the king, toward a wall, into a kill zone). Enemies resisting the compulsion take continuous damage. | Forced Movement |

### Buffs & Healing (Strengthen Allies)

Force-multiplier spells that make existing defenders more effective. A well-timed buff when enemies breach the front line can turn a loss into victory.

| Spell | Inspiration | Lore Basis | Game Mechanic | Category |
|---|---|---|---|---|
| **Phantom Feast** | Illusion/Conjuration | Conjuring a spectral banquet that restores vigor — the food of the fey courts | Creates a spectral feast area. Allied units near it regenerate health and gain temporary max HP increase. Persists for a duration. Stationary but powerful sustain. | Area Heal |
| **Mending Touch** | Abjuration | Channeling raw restorative magic into a single warrior, knitting flesh and bone | Instantly heals a single heavily wounded defender to full health and grants them temporary damage resistance. The "clutch save" heal. Single target, moderate cooldown. | Emergency Heal |

### Enemy Debuffs (Weaken Foes)

Reducing enemy capabilities makes ALL your defenses more effective -- archers, infantry, and spells all benefit.

| Spell | Inspiration | Lore Basis | Game Mechanic | Category |
|---|---|---|---|---|
| **Warding Glyphs** | Abjuration | Inscribing protective sigils on the ground that sap intruders of their strength | Place up to 7 small ward markers on the battlefield. Enemies crossing a ward are weakened: reduced damage and armor for several seconds. Wards persist until triggered a set number of times. | Trap + Debuff |
| **Threefold Curse** | Necromancy | A triple-layered hex that erodes body, mind, and spirit in sequence | Curses a single powerful enemy with three stacking debuffs applied over time: first reduces armor, second reduces healing, third causes a burst of damage if the target is still alive. Anti-boss spell. | Stacking Single-Target Debuff |
| **Retribution Hex** | Abjuration | A reflective ward that turns an attacker's own force back upon them | Reflects a portion of enemy damage back at the attacker. Cast on a group of defenders -- enemies hitting them take return damage. Makes attacking your front line painful. | Damage Reflection |
| **Arbiter's Bolt** | Divination | A bolt of judgment that seeks the most dangerous foe — guided by prescient magic | Lightning strikes the enemy with the highest kill count. Damage scales with how much harm that enemy has caused. Automatically targets the biggest threat. | Auto-Target Nuke |

### Summoning & Conjuration

Temporary allies or constructs that fight for you. One spell creating a persistent threat is excellent mana efficiency.

| Spell | Inspiration | Lore Basis | Game Mechanic | Category |
|---|---|---|---|---|
| **Summon Spirits** | Necromancy | Calling forth shades from the veil — not true resurrection, but echoes of the fallen | Summons spectral warriors from nearby corpses. Unlike Raise the Dead, these are ghostly -- they phase through walls, can't be targeted by physical attacks, but have a short duration and low HP. | Ghost Summon |
| **Beast-Calling** | Conjuration | The primal magic of beast-speakers and rangers — commanding nature's hunters | Summons a pack of wolves, boars, or ravens that harass enemies. Animals are fast and expendable. Different types have different behaviors (boars charge, wolves surround, ravens dive-bomb and distract). | Animal Summon |
| **Conjure Elemental** | D&D Conjuration | Binding an elemental spirit from the Inner Planes into temporary service | Summons a powerful elemental (fire, earth, water, air) that fights as a temporary elite unit. Each element has different strengths (fire = AoE damage, earth = tanky, water = healing aura, air = speed). | Elite Summon |
| **Giant Raptor** | Conjuration | Summoning a colossal bird of prey to seize and drop enemies from deadly heights | Summons a spectral raptor that swoops down, grabs a single large enemy, lifts it high, and drops it for massive fall damage. Single-target removal with flair. Works on Behemoths. | Single-Target Removal |

### Utility & Strategic Spells

Unique tools that don't fit other categories but enable creative strategies.

| Spell | Inspiration | Lore Basis | Game Mechanic | Category |
|---|---|---|---|---|
| **Greater Invisibility** | D&D Illusion | Bending light around allies so completely that even their footsteps vanish | Makes a group of defenders invisible until they attack. Enemies ignore invisible units and path past them, walking into ambushes. | Invisibility |
| **Time Stop** | D&D Transmutation | Freezing a sliver of time itself — rewinding the thread of fate to undo death | Rewinds time in a small area, resurrecting 2-3 recently fallen defenders where they died. Extremely expensive mana cost, very long cooldown. The "undo" button. | Time Rewind |
| **Shadow Gate** | Necromancy/Conjuration | Opening a rift to the Shadowfell that drags the living halfway into death | Opens a portal that pulls nearby enemies partially into the shadow plane. Rooted in place, they take damage over time as shadow hands claw at them. Visual: dark portal on the ground with grasping arms. | Root + DoT |
| **Purifying Flame** | Abjuration/Evocation | Sacred fire that burns away enchantments and wards while leaving flesh unscorched | Creates a line of purifying fire. Enemies crossing it take damage AND are stripped of all buffs and magical effects. Allied units can pass freely. Anti-magic barrier. | Purge + Barrier |
| **Resilient Sphere** | D&D Evocation | An indestructible bubble of force — protection or prison, depending on your intent | Encases a single unit (ally or enemy) in an indestructible bubble. They can't act but can't be harmed. Use on your king to save him, or on a dangerous enemy to remove them temporarily. | Stasis |

---

## Spell Combo Ideas

Spells that interact with each other create emergent strategy and "eureka" moments.

| Combo | Spells Involved | Interaction |
|---|---|---|
| **Inferno** | Grease + Fireball/Wall of Fire/Meteor Fall | Oil ignites for massive burning ground damage |
| **Death Factory** | Sleep/Entangle + Raise the Dead | Disable enemies without killing, then execute and immediately raise them |
| **Kill Box** | Wall of Stone + Spike Growth | Funnel enemies through a corridor of damaging terrain |
| **Hammer & Anvil** | Arcane Lasso + Fireball/Chain Lightning | Cluster enemies together, then hit them all with AoE |
| **Ghost Army** | Phantasmal Force + Greater Invisibility | Hide real defenders while phantoms draw enemy attention |
| **Divine Storm** | Squall + Chain Lightning | Wet enemies from ice storm take bonus electric damage |
| **Purge & Burn** | Purifying Flame + Wall of Fire | Strip enemy protections, then force them through fire |
| **Berserker Blitz** | Berserker Rage + Haste | Supercharged berserkers with extreme speed |
| **Marked for Execution** | Mark of Death + Finger of Death | Mark amplifies the massive single-target nuke |
| **Foggy Ambush** | Fog Cloud + Battle Hymn | Defenders gain evasion AND damage boost |
| **Sheep Slaughter** | Polymorph + any damage spell | Transform, then kill the helpless sheep permanently |
| **Hypnotic Snipe** | Hypnotic Pattern + Finger of Death | Stun a group, then pick off one at a time without breaking the stun |

---

## Design Principles

Lessons drawn from studying spell systems across D&D, Warcraft III, Total War: Warhammer, Kingdom Rush, Age of Mythology, Magicka, and Noita.

### What Makes Spells Fun in Defense Games

1. **Different interaction types matter more than different damage types.** "Fire damage vs ice damage" is cosmetic variety. "Damage vs crowd control vs terrain vs buffs" is strategic variety. Aim for spells that change *how the battle plays out*, not just how much damage you deal.

2. **Persistent effects create tactical depth.** A fireball deals damage and is gone. A wall, a burning zone, or a rooting field *persists* and forces ongoing decisions from both the player and the AI.

3. **Combos feel brilliant.** When players discover that Grease + Fireball = inferno, or Entangle + Raise the Dead = undead factory, they feel clever. Design spells with interaction hooks.

4. **Non-lethal CC synergizes with Raise the Dead.** This is unique to Court Wizard -- keeping enemies alive to raise them creates tension between "kill them now" and "save them for conversion." Lean into this.

5. **Risk/reward tradeoffs create drama.** Berserker Rage makes your soldiers powerful but fragile. Black Hole's 20-second cast is a huge commitment. Concentration spells lock out other casting. These decisions are where strategy lives.

6. **Visual clarity from the wall.** The wizard's elevated perspective means AoE boundaries, zone effects, and unit states need to be readable from above. Favor spells with clear ground-plane visuals (circles, zones, trails) over subtle unit-attached effects.

### Spell Variety Checklist

A well-rounded spell book should cover all these interaction types:

- [x] **Control enemy position** (teleport, push, pull, wall) -- Have: Teleport, Black Hole, Wall of Stone, Banishment
- [x] **Control enemy behavior** (slow, stun, fear, charm) -- Have: Squall (slow), Entangle (root), Spike Growth (slow), Hypnotic Pattern (stun), Sleep (stun), Grease (slow), Polymorph (disable)
- [x] **Control enemy capabilities** (silence, weaken, debuff) -- Have: Mark of Death (damage amp), Fog Cloud (evasion vs enemies), Berserker Rage (vulnerability)
- [x] **Enhance allied units** (buff attack, speed, defense) -- Have: Guardian Circle (temp HP), Haste (speed), Battle Hymn (damage + attack speed), Berserker Rage (damage), Healing Plume (heal)
- [x] **Create persistent zones** (damage fields, buff auras, terrain) -- Have: Wall of Stone, Wall of Fire, Spike Growth, Grease, Fog Cloud, Healing Plume, Meteor Fall (ground fire)
- [x] **Summon allies** (temporary fighters, distractions) -- Have: Raise the Dead, Phantasmal Force (decoys)
- [ ] **Manipulate terrain** (obstacles, passages, traps) -- Have: Wall of Stone. Could expand with traps.
- [x] **Direct damage** (burst, sustained, multi-target) -- Well covered with many damage spells including Meteor Fall, Plague Wind
