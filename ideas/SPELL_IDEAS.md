# Spell Ideas

Spells for the wizard's arsenal, drawn from mythology, folklore, game systems (D&D, Warcraft, Age of Mythology), and historical magical traditions. The wizard casts from atop the castle wall, assisting defenders against waves of attackers.

## Currently Implemented

| Spell | Rune Combo | Type | Damage Type | Description |
|---|---|---|---|---|
| **Magic Missile** | Q | Channeled projectile | Force | Rapid-fire homing missiles that accelerate over time |
| **Fireball** | W | AoE explosion | Fire | Explosive projectile with residual burning ground |
| **Teleport** | E | Utility | Force | Two-stage: place marker, then teleport units in radius to it |
| **Guardian Circle** | R | Defensive buff | Force | Grants temporary HP to units inside protective circle |
| **Disintegrate** | Q+W | Channeled beam | Force | Sustained green beam dealing continuous damage along a line |
| **Chain Lightning** | Q+E | Multi-target | Electric | Strikes nearest enemy, bounces to 4 additional targets |
| **Black Hole** | Q+R | Gravity field | Force | Pulls units inward, ramping damage, 20s cast time |
| **Wall of Stone** | W+E | Terrain obstacle | Force | Drag-to-place wall that blocks movement and pathfinding |
| **Raise the Dead** | W+R | Channeled resurrection | Necrotic | Resurrects corpses as Undead allies |
| **Finger of Death** | E+R | Instant beam | Necrotic | 1000 damage instant-kill beam, requires 50% mana |
| **Squall** | Spell Book | Concentration AoE | Frost | Persistent ice storm with slow, blocks other casting |

## Mechanical Gaps to Fill

Before diving into specific spells, here are the **gameplay mechanic types** currently missing from the roster. The existing spells are heavy on direct damage variants -- adding these categories would create more strategic depth.

| Gap | What It Means | Why It Matters |
|---|---|---|
| **Persistent Area Denial** | Damage-over-time zone enemies avoid | Forces pathing changes, controls space |
| **Non-Damage Crowd Control** | Slow, root, stun without killing | Keeps enemies alive for Raise the Dead |
| **Offensive Buffs** | Strengthen allied units' attacks | Force-multiplies existing defenders |
| **Enemy Debuffs** | Weaken enemy armor, damage, speed | Makes all your defenses more effective |
| **Trap/Delayed Spells** | Placed in advance, triggers on contact | Rewards prediction and planning |
| **Knockback/Push** | Shove enemies away from the king | Opposite of Black Hole's pull |
| **Channeled Heal** | Sustained healing over time | Keeps key defenders alive during fights |

---

## Backlog

### Battlefield Control (Terrain & Positioning)

Spells that reshape the battlefield itself -- walls, zones, terrain effects. Cast from the wall, the wizard has a god's-eye view perfect for this.

| Spell | Origin | Mythological Basis | Game Mechanic | Category |
|---|---|---|---|---|
| **Wall of Fire** | D&D (Evocation) | Surtr's fire from Muspelheim; the flames at the world's edge in Norse cosmology | Drag-to-place burning wall (like Wall of Stone but deals damage to enemies passing through). Doesn't block movement, but punishes crossing. | Area Denial |
| **Entangle** | Celtic/Druidic | Druids' power over plant life; the living forests of Celtic myth that could trap intruders | Vines erupt from the ground in a radius, rooting enemies in place for several seconds. Rooted enemies can still attack but cannot move. | Root/CC |
| **Spike Growth** | Celtic/Druidic | The magical thorns of the Fairy Thorn (Sceach); sacred hawthorn hedges that punished trespassers | Ground sprouts damaging spikes in an area. Enemies take damage proportional to distance moved through the zone. Lasts 15-20 seconds. | Area Denial + Slow |
| **Gilgamesh's Cedar Curse** | Mesopotamian | The sacred Cedar Forest guarded by Humbaba in the Epic of Gilgamesh | Spectral trees erupt from the ground, acting as temporary obstacles that also root nearby enemies in their branches. Blocks pathfinding like Wall of Stone but also damages. | Obstacle + Damage |
| **Ishtar's Seven Gates** | Mesopotamian | Ishtar's descent through seven gates to the underworld, stripped of power at each | Creates a series of 3 luminous gate arches enemies must pass through. Each gate strips a buff and reduces stats (armor, speed, damage) progressively. | Debuff Zone |
| **Grease** | D&D (Conjuration) / Arabic (oil trade) | Greek Fire and naphtha weapons of Byzantine/Arabic siege warfare | Covers ground in slippery oil. Enemies slide and move erratically with reduced speed. Can be ignited by Fireball for a devastating combo (burning ground). | Terrain + Combo |

### Direct Damage (Offensive Strikes)

New damage spells that offer fundamentally different targeting or interaction patterns from existing ones.

| Spell | Origin | Mythological Basis | Game Mechanic | Category |
|---|---|---|---|---|
| **Meteor Swarm** | D&D (Evocation) / Egyptian | Sekhmet's wrath raining fire; the "arrows of Ra" | Click 3-4 locations in sequence, then meteors crash down simultaneously at each point after a short delay. Massive fire damage in each radius. Ultimate damage spell. | Delayed Multi-AoE |
| **Indra's Vajra** | Hindu/Vedic | Indra's thunderbolt (Vajra) that shattered the demon Vritra and released the rivers | Throw a divine thunderbolt to a target point. On impact, it shatters into lightning shards that fly outward in all directions, hitting enemies in a large radius. Unlike Chain Lightning, damage radiates outward from impact. | AoE Burst |
| **Balor's Evil Eye** | Irish mythology | Balor of the Fomorians, whose single eye destroyed everything it gazed upon | Slow-charging beam that sweeps across the battlefield in an arc (not a fixed line like Disintegrate). Deals escalating damage the longer it touches a target. Long cooldown. | Sweeping Beam |
| **Sekhmet's Plague Wind** | Egyptian | Sekhmet the lion-goddess whose breath was pestilence and plague | Launches a poison cloud that drifts slowly across the battlefield with the wind. Enemies inside take damage over time. The cloud persists and moves, creating a rolling zone of death. | Moving AoE |
| **Marduk's Four Winds** | Mesopotamian | Marduk used the four cardinal winds to defeat the chaos dragon Tiamat | Winds converge from all four edges of the battlefield toward a target point. Enemies caught in the convergence take massive damage and are flung outward. | Convergence AoE |
| **Spear of Lugh** | Irish mythology | Lugh's spear (Gae Assail) that never missed its mark and always killed | Hurl a blazing spear at any visible enemy -- guaranteed hit, pierces armor, massive single-target damage. The anti-Behemoth spell. Very long cooldown. | Guaranteed Single-Target |

### Crowd Control (Slow, Stun, Fear, Confusion)

Non-lethal or low-damage spells that manipulate enemy behavior. Especially valuable when you want corpses intact for Raise the Dead.

| Spell | Origin | Mythological Basis | Game Mechanic | Category |
|---|---|---|---|---|
| **Fáfnir's Dread** | Norse mythology | The dragon Fáfnir radiated supernatural terror through the Helm of Dread (Ægishjálmr) | Creates an aura of fear in a radius. Weak enemies flee in panic; stronger enemies have reduced attack damage and move hesitantly. Breaks formations. | Fear |
| **Glám Dícenn** | Irish bardic magic | The devastating satirical curse that could raise blisters, cause shame, and kill warriors through words alone | The wizard chants a magical satire. Enemies in a cone take minor damage but suffer heavy debuffs: reduced morale, attack speed, and accuracy. Runes visibly fly from wizard toward targets. | Debuff (Cone) |
| **Hypnotic Pattern** | D&D (Illusion) / Arabic | The mesmerizing geometric patterns of Islamic art; djinn glamours from 1001 Nights | Swirling luminous pattern appears in an area. Enemies caught in it stop moving and attacking, charmed into inaction. Taking damage breaks the effect. | Stun/Charm |
| **Varunapasha** | Hindu/Vedic | Varuna's divine noose (pasha) that bound oath-breakers and demons inescapably | Magical lassoes snake out and drag 3-5 enemies together into a tight cluster, immobilizing them. Perfect setup for Fireball or Chain Lightning. | Root + Cluster |
| **Geas of Compulsion** | Irish/Celtic | Magical prohibitions (geasa) placed on heroes that compelled specific behaviors under threat of death | Forces enemies in an area to walk in a specific direction (e.g., away from the king, toward a wall, into a kill zone). Enemies resisting the compulsion take continuous damage. | Forced Movement |
| **Blodeuwedd's Curse** | Welsh mythology (Mabinogion) | Blodeuwedd was transformed into an owl as punishment by Gwydion | Transforms enemy units into harmless animals (owls, rabbits) for a duration. Transformed enemies cannot attack or use abilities. Limited targets -- works on many weak enemies or one elite. | Polymorph |
| **Sleep** | D&D (Enchantment) / Norse | Odin's power to put warriors to sleep; the Norse "sleep thorn" (svefnthorn) rune | AoE that puts weaker enemies to sleep. Sleeping units are completely disabled. Damage wakes them, but sleeping enemies take bonus damage from the first hit. | Sleep/Stun |

### Buffs & Healing (Strengthen Allies)

Force-multiplier spells that make existing defenders more effective. A well-timed buff when enemies breach the front line can turn a loss into victory.

| Spell | Origin | Mythological Basis | Game Mechanic | Category |
|---|---|---|---|---|
| **Hamramr** | Norse berserker magic | Shape-strong warriors (hamrammr) who could take on animal fury and supernatural strength in battle | Transforms nearby allied infantry into berserkers. Massive damage and speed increase, but they lose defensive ability and take more damage. They ignore lethal damage for a few seconds (fight at 1 HP). | Offensive Buff (Risky) |
| **Óðrerir's Inspiration** | Norse mythology | The Mead of Poetry stolen by Odin that grants divine inspiration, eloquence, and battle-wisdom | All allied units in radius gain increased attack speed and damage for a duration. Inspired units glow with runic light. | Attack Buff |
| **Simurgh's Feathers** | Persian mythology | The Simurgh, benevolent mythical bird whose feathers could heal any wound | Drop glowing feathers on wounded defenders. Heals over time and grants temporary bonus effectiveness (attack speed). The wizard's primary healing spell. | Heal + Buff |
| **Phantom Feast** | Irish mythology | Supernatural feasts in Irish tales where eating grants power; Bricriu's Feast | Creates a spectral feast area. Allied units near it regenerate health and gain temporary max HP increase. Persists for a duration. Stationary but powerful sustain. | Area Heal |
| **Silver Arm** | Irish mythology | Dian Cecht crafted a fully functional silver arm for King Nuada, restoring him to kingship | Instantly heals a single heavily wounded defender to full health and grants them temporary damage resistance. The "clutch save" heal. Single target, moderate cooldown. | Emergency Heal |
| **Garuda's Swiftness** | Hindu/Vedic | Garuda, the divine eagle mount of Vishnu, embodiment of speed and enemy of serpents | Grants massive movement speed boost to all defenders in radius. They move like divine eagles, able to rapidly reposition or chase down fleeing enemies. | Speed Buff |

### Enemy Debuffs (Weaken Foes)

Reducing enemy capabilities makes ALL your defenses more effective -- archers, infantry, and spells all benefit.

| Spell | Origin | Mythological Basis | Game Mechanic | Category |
|---|---|---|---|---|
| **The Seven Knots of Isis** | Egyptian | Isis's seven magical protective knots from Book of the Dead, Spell 156 | Place up to 7 small ward markers on the battlefield. Enemies crossing a ward are weakened: reduced damage and armor for several seconds. Wards persist until triggered a set number of times. | Trap + Debuff |
| **Threefold Curse** | Celtic druidic magic | The power of three in Celtic magic; druids cursed in threes for devastating effect | Curses a single powerful enemy with three stacking debuffs applied over time: first reduces armor, second reduces healing, third causes a burst of damage if the target is still alive. Anti-boss spell. | Stacking Single-Target Debuff |
| **Al-Ayn** | Arabic folklore | The Evil Eye (al-ayn), one of the most feared curses in Middle Eastern tradition | Reflects a portion of enemy damage back at the attacker. Cast on a group of defenders -- enemies hitting them take return damage. Makes attacking your front line painful. | Damage Reflection |
| **Valbǫð** | Norse/Valkyrie magic | Valkyries choosing who lives and dies in battle; marking warriors for death | Marks enemy units for death. Marked enemies take heavily increased damage from all sources and cannot flee. Valkyrie silhouettes appear above them. | Damage Amplification |
| **Rashnu's Judgment** | Zoroastrian | Rashnu, the angel who judges souls with perfect truth at the Chinvat Bridge | Lightning strikes the enemy with the highest kill count. Damage scales with how much harm that enemy has caused. Automatically targets the biggest threat. | Auto-Target Nuke |

### Summoning & Conjuration

Temporary allies or constructs that fight for you. One spell creating a persistent threat is excellent mana efficiency.

| Spell | Origin | Mythological Basis | Game Mechanic | Category |
|---|---|---|---|---|
| **Útiseta** | Norse Seidr | "Sitting out" on burial mounds to commune with and summon spirits of the dead | Summons spectral warriors from nearby corpses. Unlike Raise the Dead, these are ghostly -- they phase through walls, can't be targeted by physical attacks, but have a short duration and low HP. | Ghost Summon |
| **Beast-Calling** | Celtic/Druidic | Druids' reputed mastery over animals; Cernunnos commanding beasts of the forest | Summons a pack of wolves, boars, or ravens that harass enemies. Animals are fast and expendable. Different types have different behaviors (boars charge, wolves surround, ravens dive-bomb and distract). | Animal Summon |
| **Conjure Elemental** | D&D (Conjuration) / Arabic | Djinn of the four elements in Arabic folklore; elemental spirits in many traditions | Summons a powerful elemental (fire, earth, water, air) that fights as a temporary elite unit. Each element has different strengths (fire = AoE damage, earth = tanky, water = healing aura, air = speed). | Elite Summon |
| **Roc's Fury** | Arabic (1001 Nights) | The Roc (Rukh), the colossal bird that could carry elephants in its talons | Summons a spectral Roc that swoops down, grabs a single large enemy, lifts it high, and drops it for massive fall damage. Single-target removal with flair. Works on Behemoths. | Single-Target Removal |
| **Gwydion's Phantoms** | Welsh mythology (Mabinogion) | Gwydion fab Dôn created illusory armies and phantom objects to deceive enemies | Creates phantom duplicate defenders that look real to enemies. Phantoms deal no damage but enemies target them. When hit, phantoms vanish in a puff of mist. Splits enemy attention. | Illusion/Decoy |

### Utility & Strategic Spells

Unique tools that don't fit other categories but enable creative strategies.

| Spell | Origin | Mythological Basis | Game Mechanic | Category |
|---|---|---|---|---|
| **Féth Fíada** | Irish Druidic magic | Manannán mac Lir's enchanted mist that concealed the Otherworld from mortal eyes | Blankets an area in supernatural fog. Allied units inside gain evasion (enemies miss more often). Enemies inside have reduced accuracy and lose target tracking. | Mist/Evasion |
| **Fíth-Fath** | Celtic/Scottish magic | Druidic glamour spell that made the caster invisible or unrecognizable | Makes a group of defenders invisible until they attack. Enemies ignore invisible units and path past them, walking into ambushes. | Invisibility |
| **Seal of Sulaiman** | Arabic/Islamic legend | King Solomon's ring that gave him dominion over djinn, animals, and the wind | Banishes a single powerful enemy from the battlefield temporarily. They vanish for 10-15 seconds, then reappear confused. Removes a key threat during a critical push. | Banishment |
| **Tablet of Destinies** | Mesopotamian | The divine tablets that granted control over fate; possession meant cosmic authority | Rewinds time in a small area, resurrecting 2-3 recently fallen defenders where they died. Extremely expensive mana cost, very long cooldown. The "undo" button. | Time Rewind |
| **Duat Shadow Gate** | Egyptian | The Duat underworld; gates between the mortal realm and the land of the dead | Opens a portal that pulls nearby enemies partially into the underworld. Rooted in place, they take damage over time as shadow hands claw at them. Visual: dark portal on the ground with grasping arms. | Root + DoT |
| **Atar's Sacred Flame** | Zoroastrian | Atar, the divine fire of Ahura Mazda that purifies evil and reveals truth | Creates a line of purifying fire. Enemies crossing it take damage AND are stripped of all buffs and magical effects. Allied units can pass freely. Anti-magic barrier. | Purge + Barrier |
| **Resilient Sphere** | D&D (Evocation) | Protective magical bubbles found across many traditions; djinn bottle prisons | Encases a single unit (ally or enemy) in an indestructible bubble. They can't act but can't be harmed. Use on your king to save him, or on a dangerous enemy to remove them temporarily. | Stasis |

---

## Spell Combo Ideas

Spells that interact with each other create emergent strategy and "eureka" moments.

| Combo | Spells Involved | Interaction |
|---|---|---|
| **Greek Fire** | Grease + Fireball | Oil ignites for massive burning ground damage |
| **Death Factory** | Sleep/Entangle + Raise the Dead | Disable enemies without killing, then execute and immediately raise them |
| **Kill Box** | Wall of Stone + Spike Growth | Funnel enemies through a corridor of damaging terrain |
| **Hammer & Anvil** | Varunapasha + Fireball/Chain Lightning | Cluster enemies together, then hit them all with AoE |
| **Ghost Army** | Gwydion's Phantoms + Fíth-Fah | Hide real defenders while phantoms draw enemy attention |
| **Divine Storm** | Squall + Chain Lightning | Wet enemies from ice storm take bonus electric damage |
| **Purge & Burn** | Atar's Sacred Flame + Wall of Fire | Strip enemy protections, then force them through fire |
| **Berserker Blitz** | Hamramr + Garuda's Swiftness | Supercharged berserkers with extreme speed |

---

## Design Principles

Lessons drawn from studying spell systems across D&D, Warcraft III, Total War: Warhammer, Kingdom Rush, Age of Mythology, Magicka, and Noita.

### What Makes Spells Fun in Defense Games

1. **Different interaction types matter more than different damage types.** "Fire damage vs ice damage" is cosmetic variety. "Damage vs crowd control vs terrain vs buffs" is strategic variety. Aim for spells that change *how the battle plays out*, not just how much damage you deal.

2. **Persistent effects create tactical depth.** A fireball deals damage and is gone. A wall, a burning zone, or a rooting field *persists* and forces ongoing decisions from both the player and the AI.

3. **Combos feel brilliant.** When players discover that Grease + Fireball = inferno, or Entangle + Raise the Dead = undead factory, they feel clever. Design spells with interaction hooks.

4. **Non-lethal CC synergizes with Raise the Dead.** This is unique to Court Wizard -- keeping enemies alive to raise them creates tension between "kill them now" and "save them for conversion." Lean into this.

5. **Risk/reward tradeoffs create drama.** Hamramr makes your soldiers powerful but fragile. Black Hole's 20-second cast is a huge commitment. Concentration spells lock out other casting. These decisions are where strategy lives.

6. **Visual clarity from the wall.** The wizard's elevated perspective means AoE boundaries, zone effects, and unit states need to be readable from above. Favor spells with clear ground-plane visuals (circles, zones, trails) over subtle unit-attached effects.

### Spell Variety Checklist

A well-rounded spell book should cover all these interaction types:

- [ ] **Control enemy position** (teleport, push, pull, wall) -- Have: Teleport, Black Hole, Wall of Stone
- [ ] **Control enemy behavior** (slow, stun, fear, charm) -- Have: Squall (slow). Need more.
- [ ] **Control enemy capabilities** (silence, weaken, debuff) -- Missing entirely.
- [ ] **Enhance allied units** (buff attack, speed, defense) -- Have: Guardian Circle (temp HP). Need offensive buffs.
- [ ] **Create persistent zones** (damage fields, buff auras, terrain) -- Have: Wall of Stone. Need damage zones.
- [ ] **Summon allies** (temporary fighters, distractions) -- Have: Raise the Dead. Could add more variety.
- [ ] **Manipulate terrain** (obstacles, passages, traps) -- Have: Wall of Stone. Could expand.
- [ ] **Direct damage** (burst, sustained, multi-target) -- Well covered with 5+ damage spells.
