# Unit Ideas

Enemy units drawn from mythological lores across cultures — Frostheim, Olympar, Asuryan, Faewood, Ancient Sumora, and beyond. Each faction brings unique unit types, behaviors, and magical vulnerabilities that challenge the Court Wizard in different ways.

## Currently Implemented

| Unit | Team | Type | Health | Speed | Damage | Special Mechanic |
|---|---|---|---|---|---|---|
| **Infantry** | Defenders | Melee | 100 | 4.0 | 10 | Front-line defenders |
| **Archer** | Defenders | Ranged | 60 | 3.5 | 8 | Ranged support |
| **King** | Defenders | Elite | 200 | 3.0 | 15 | Loss condition if dies, grants aura buff |
| **King's Guard** | Defenders | Elite Melee | 150 | 4.0 | 20 | Elite bodyguards |
| **Attacker Infantry** | Attackers | Melee | 80 | 3.5 | 8 | Basic enemy melee |
| **Attacker Archer** | Attackers | Ranged | 50 | 3.0 | 6 | Basic enemy ranged |
| **Behemoth** | Attackers | Elite | 500 | 2.5 | 40 | Massive tank unit, spawns every 5 levels |
| **Undead** | Undead | Converted | Varies | 4.0 | Varies | Raised from corpses by wizard |

---

## Faction: Frostheim (Northern Warriors)

The armies of the frozen north — warriors who embrace death in battle, monstrous wolf-spawn, and giants born of primordial ice.

### Einherjar (Elite Warriors)

**Lore:** Slain heroes feasting in the Hall of Eternal Battle, called back to fight one last time. They *want* to die gloriously.

**Appearance:** Spectral northern warriors with glowing blue runes on their armor.

**Stats:**
- Health: 120
- Speed: 4.5
- Damage: 25
- Type: Melee Elite

**Mechanic - Glorious Death:**
- When reduced to low health (below 30%), Einherjar enter "Last Stand" — movement speed doubles, damage increases by 50%.
- They shout battle cries and glow brighter.
- When they finally die, they explode in a small AoE dealing damage to nearby defenders (final glory).

**Counter Strategy:**
- Burst them down quickly with high-damage spells before Last Stand triggers.
- Use crowd control (Entangle, Sleep) to prevent them from reaching your lines during Last Stand.
- Black Hole can clump them together before the death explosions.

### Draugr (Undead Husks)

**Lore:** Northern undead that won't stay buried. They're already dead, so killing them is more of a suggestion than a solution.

**Appearance:** Shambling corpses with glowing eyes, missing chunks, covered in grave dirt.

**Stats:**
- Health: 150 (very tanky)
- Speed: 2.0 (slow)
- Damage: 12
- Type: Melee Tank

**Mechanic - Already Dead:**
- When "killed," Draugr collapse but aren't destroyed — they're stunned for 10 seconds.
- After 10 seconds, they reanimate at 50% health and resume their advance.
- Can only be permanently destroyed by: fire damage (Fireball, Wall of Fire), disintegration (Disintegrate, Finger of Death), or holy magic.
- Cannot be raised by Raise the Dead (they're already undead).

**Counter Strategy:**
- Use fire-based spells to permanently destroy them.
- Disintegrate or Finger of Death works instantly.
- Otherwise, they just keep getting back up — very dangerous if ignored.

### Fenrir's Whelps (Dire Wolves)

**Lore:** Children of the great wolf Fenrir. Fast, vicious, and they hunt in packs.

**Appearance:** Massive wolves the size of horses, glowing red eyes, frost on their fur.

**Stats:**
- Health: 70 (fragile)
- Speed: 6.0 (very fast)
- Damage: 15
- Type: Fast Melee

**Mechanic - Pack Hunters:**
- Whelps gain +10% damage for each other whelp within a small radius (stacks up to +50%).
- They naturally flock together, making them dangerous in groups.
- Prioritize attacking isolated defenders or the King.

**Counter Strategy:**
- AoE spells (Fireball, Chain Lightning) devastate their clustered formations.
- Wall of Stone splits the pack, reducing their damage bonus.
- Guardian Circle protects the King from their focused assault.

### Frost Giant

**Lore:** Ice giants from the Frozen Wastes. They hate warmth, they hate fire, and they especially hate tiny wizards throwing fireballs at them.

**Appearance:** Towering humanoid covered in ice and snow, wielding a massive club or icicle spear.

**Stats:**
- Health: 800
- Speed: 2.0 (slow)
- Damage: 60
- Type: Elite Tank

**Mechanic - Frost Aura:**
- Radiates a freezing aura in a large radius — all defenders near the giant are slowed by 40%.
- Immune to frost/ice damage (Squall does nothing).
- Takes double damage from fire spells (Fireball, Wall of Fire).
- Every 20 seconds, slams the ground causing an ice shockwave that damages and slows defenders in a wide cone.

**Counter Strategy:**
- Fire is your best friend — melt them with Fireball spam.
- Use Wall of Stone to block their ground slam shockwave.
- Teleport your defenders away from the frost aura.

### Valkyrie (Flying Support)

**Lore:** Shield-maidens of the All-Father who choose the worthy slain. They're not here to fight — they're here to make sure their warriors *can* fight.

**Appearance:** Winged armored women on flying horses, glowing with divine light.

**Stats:**
- Health: 100
- Speed: 5.0 (flying, ignores obstacles)
- Damage: 0 (support unit)
- Type: Flying Support

**Mechanic - Chooser of the Slain:**
- Valkyries fly over the battlefield healing nearby allies (20 HP/s aura).
- When an allied unit dies near a Valkyrie, she resurrects them once at 50% HP (each Valkyrie can do this once per battle).
- Valkyries don't attack — they only support.
- Flying units cannot be hit by melee attacks, only spells and archer fire.

**Counter Strategy:**
- Priority target — kill them before they resurrect your enemies.
- Magic Missile and Chain Lightning auto-target flying units.
- Finger of Death one-shots them despite their flying.

---

## Faction: Olympar (Classical Armies)

The armies of the mountain gods and the underworld — legendary heroes, bronze-clad hoplites, and monsters from the depths of the abyss.

### Hoplite Phalanx (Shield Wall)

**Lore:** Elite city-state warriors in perfect formation. "Come back with your shield or on it."

**Appearance:** Bronze-armored soldiers with large round shields and long spears, marching in tight formation.

**Stats:**
- Health: 90
- Speed: 3.0 (slow when shielded)
- Damage: 12
- Type: Melee Tank

**Mechanic - Phalanx Formation:**
- When 3+ Hoplites are adjacent, they form a phalanx — damage reduction 60%, immune to knockback.
- Phalanx moves slowly but is nearly invincible from the front.
- Weak from behind or flanks — breaking formation removes the bonus.

**Counter Strategy:**
- Teleport units behind them to break formation.
- Black Hole pulls them out of formation.
- AoE damage (Fireball) bypasses armor when they're clustered.
- Wall of Stone splits the phalanx, leaving individuals vulnerable.

### Harpy (Flying Skirmisher)

**Lore:** Winged women with talons for feet. They screech, they dive, they steal food from your plate at dinner.

**Appearance:** Human torso with bird wings and talons, shrieking and dive-bombing.

**Stats:**
- Health: 50 (fragile)
- Speed: 5.5 (flying)
- Damage: 10
- Type: Flying Melee

**Mechanic - Dive Bomb:**
- Harpies circle above the battlefield, then dive to attack.
- During dive, speed increases to 8.0 and damage doubles (20).
- After attacking, they fly back up (vulnerable during ascent).
- Shrieking aura reduces defender morale — defenders near Harpies attack 20% slower.

**Counter Strategy:**
- Catch them during their dive or ascent with Magic Missile.
- Chain Lightning chains between multiple Harpies.
- Squall slows their flight speed, making them easier to hit.

### Hydra (Multi-Head Boss)

**Lore:** The legendary swamp serpent. Cut off one head, two more grow back. Ancient heroes had fire; you have a wizard. Close enough.

**Appearance:** Massive serpent with multiple snake heads, growing from a central body.

**Stats:**
- Health: 1000 (shared pool)
- Speed: 2.5
- Damage: 30 per head
- Type: Elite Boss

**Mechanic - Regenerating Heads:**
- Starts with 3 active heads.
- When dealt 200 damage, one head is "severed" (visual effect) but 2 new heads grow back.
- Heads increase max health by 100 and add another attacking head (+30 DPS).
- Can grow up to 9 heads maximum.
- Fire damage cauterizes heads — damage from fire spells prevents head regrowth.
- Only way to kill: burn it with fire or deal massive burst damage (3000+) before it regenerates.

**Counter Strategy:**
- Fire spells ONLY (Fireball, Wall of Fire) to prevent regeneration.
- Alternatively, Finger of Death can one-shot it if you have the mana.
- Do NOT use sustained damage spells (Disintegrate, Magic Missile) — you'll just make it stronger.

### Minotaur (Berserker)

**Lore:** Bull-headed man trapped in a maze. Now the maze is a battlefield, and he's very, very angry about it.

**Appearance:** Massive bull-headed humanoid wielding a giant axe, covered in scars.

**Stats:**
- Health: 400
- Speed: 3.0 (charges up to 7.0)
- Damage: 50
- Type: Elite Melee

**Mechanic - Labyrinth Charge:**
- Every 15 seconds, the Minotaur targets the furthest defender (usually the King).
- Charges in a straight line at 7.0 speed, smashing through units in its path.
- Units hit during charge take 50 damage and are knocked aside.
- After charge, Minotaur is stunned for 3 seconds (vulnerable).

**Counter Strategy:**
- Wall of Stone blocks the charge path — Minotaur crashes into it and is stunned.
- Teleport the King away from charge path.
- Entangle roots the Minotaur, canceling the charge.
- Use the post-charge stun window to nuke it with damage spells.

### Medusa (Petrification)

**Lore:** Don't look her in the eyes. Seriously. Your defenders will learn this the hard way.

**Appearance:** Serpent-haired woman with glowing green eyes, wielding a bow.

**Stats:**
- Health: 200
- Speed: 3.5
- Damage: 15 (ranged)
- Type: Elite Ranged

**Mechanic - Petrifying Gaze:**
- Every 10 seconds, Medusa unleashes her gaze in a cone.
- Defenders in the cone are turned to stone (stunned) for 8 seconds.
- Petrified units take double damage from all sources (brittle stone).
- Medusa's arrows prioritize petrified targets.
- Looking away (facing away from Medusa) grants immunity to gaze.

**Counter Strategy:**
- Kill her quickly before gaze triggers.
- Teleport defenders out of gaze cone.
- Use spells that don't require line-of-sight (Black Hole, Fireball AoE).
- Raise the Dead turns petrified corpses into undead (they're immune to gaze).

### Cerberus (Three-Headed Hound)

**Lore:** Guardian of the realm of the dead. Three heads, three times the biting. Zero times the obedience training.

**Appearance:** Massive three-headed dog wreathed in shadowy flames.

**Stats:**
- Health: 600
- Speed: 4.0
- Damage: 20 per head (60 total)
- Type: Elite Melee

**Mechanic - Triple Threat:**
- Cerberus can attack 3 different targets simultaneously (one per head).
- Each head targets the nearest defender independently.
- When below 50% HP, Cerberus breathes shadow fire in a cone (30 damage AoE).
- Cannot be charmed, slept, or feared (guardian of Hades).

**Counter Strategy:**
- Separate your defenders to split Cerberus's damage.
- Use Guardian Circle to absorb the multi-target attacks.
- Fire damage is still effective despite the shadow flames.
- Entangle roots all three heads at once.

---

## Faction: Asuryan (Divine Demons)

The armies of divine rebels and demon princes — shapeshifters, serpent-warriors, and champions blessed by the gods with terrible boons.

### Rakshasa (Shapeshifter)

**Lore:** Demonic shapeshifters that can look like anyone. Trust no one. Not even your own infantry.

**Appearance:** Starts as a defender-lookalike, then reveals true form — tiger-headed humanoid with backward hands.

**Stats:**
- Health: 150
- Speed: 4.0
- Damage: 25
- Type: Melee Elite

**Mechanic - Deceptive Form:**
- Rakshasas spawn disguised as friendly defenders (visually identical).
- They walk alongside your troops until they reach the middle of the battlefield.
- Then they reveal their true form and attack nearby defenders from behind.
- Wizard can detect them: disguised Rakshasas shimmer slightly when moused over.

**Counter Strategy:**
- Watch for shimmer effect and pre-emptively nuke suspected units.
- AoE spells (Fireball, Chain Lightning) hit disguised Rakshasas without needing to identify them.
- Once revealed, they're just tough melee units — standard counters apply.

### Naga (Serpent Archer)

**Lore:** Divine serpent-people from the underworld. Excellent archers, terrible at walking (no legs).

**Appearance:** Human torso on a massive snake body, wielding ornate bow, cobra hood flared.

**Stats:**
- Health: 100
- Speed: 3.0 (slithering)
- Damage: 20 (ranged, poison)
- Type: Ranged Elite

**Mechanic - Venom Arrows:**
- Naga arrows inflict poison — 5 damage per second for 10 seconds (stacks).
- Defenders hit multiple times take escalating poison damage.
- Immune to entangle/root (they're already on the ground).
- Weak to cold — Squall doubles slow effect on Naga.

**Counter Strategy:**
- Prioritize killing them to prevent poison stacking.
- Guardian Circle grants temp HP to absorb poison damage.
- Squall's frost slow is extra effective.
- Wall of Stone blocks their line of sight.

### Asura Champion (Divine Warrior)

**Lore:** Demons granted boons by the gods through extreme penance. Immortal until their boon expires.

**Appearance:** Four-armed warrior wreathed in divine light, wielding multiple weapons.

**Stats:**
- Health: 300
- Speed: 4.5
- Damage: 30 per attack (attacks twice per cycle due to extra arms)
- Type: Elite Melee

**Mechanic - Divine Boon:**
- Asura Champions are immune to damage for the first 30 seconds of battle (glowing golden aura).
- After 30 seconds, the boon expires (aura fades) and they become mortal.
- While invulnerable, they can still be crowd controlled (rooted, slowed, pushed).
- Extra arms = double attack speed.

**Counter Strategy:**
- Stall them with crowd control until boon expires (Entangle, Wall of Stone).
- Black Hole wastes their invulnerability time by pulling them away from targets.
- Once mortal, focus fire before they deal massive damage.

### Garuda (Sky Lord)

**Lore:** Divine eagle, mount of the Sky God, enemy of serpents. Huge, majestic, and very territorial.

**Appearance:** Massive golden eagle with human-like face, divine radiance, wielding a chakram.

**Stats:**
- Health: 500
- Speed: 6.0 (flying)
- Damage: 40 (diving attack)
- Type: Flying Elite

**Mechanic - Serpent Hunter:**
- Garuda deals triple damage to serpent-type units (not relevant in this context, but lore-accurate).
- Swooping Strike: Garuda dives from above, hitting all units in a line for 40 damage, then ascends.
- Creates wind gusts that push defenders away from the King when flying overhead.
- Immune to ground-based hazards (Spike Growth, Wall of Fire).

**Counter Strategy:**
- Magic Missile and Chain Lightning for flying units.
- Time AoE spells (Fireball) to hit during swooping dive when Garuda is low.
- Finger of Death ignores flying status.

### Kumbhakarna (Sleeping Giant)

**Lore:** Demon giant cursed to sleep for six months at a time. When he wakes up, he's cranky and hungry.

**Appearance:** Absolutely massive humanoid, bigger than Frost Giants, initially asleep on the ground.

**Stats:**
- Health: 1500
- Speed: 1.5 (groggy)
- Damage: 100
- Type: Mega Boss

**Mechanic - Deep Slumber:**
- Kumbhakarna spawns asleep — completely invulnerable, doesn't move or attack.
- After 60 seconds, he wakes up (dramatic animation, roar).
- While awake, each attack hits all units in a small AoE (cleaving swings).
- Takes triple damage from all sources while sleeping (defenseless).
- Every 30 seconds awake, he yawns and falls back asleep for 20 seconds.

**Counter Strategy:**
- Nuke him while asleep with triple damage modifier.
- Finger of Death deals 3000 damage if he's sleeping — instant kill.
- If he wakes up, survive until he falls asleep again.
- Don't let your defenders clump — his cleave hits multiple units.

---

## Faction: Faewood (Fae Courts)

The Fae Courts and ancient warriors — druids commanding nature, faerie tricksters, and cursed heroes.

### Fir Bolg Warrior (Ancient Infantry)

**Lore:** Original inhabitants of the land, driven underground. They remember every slight and hold grudges for centuries.

**Appearance:** Stocky warriors in earth-toned armor, wielding stone weapons, covered in ritual tattoos.

**Stats:**
- Health: 110
- Speed: 3.5
- Damage: 14
- Type: Melee

**Mechanic - Blood Grudge:**
- Fir Bolg deal +50% damage to units that have killed another Fir Bolg this battle.
- They glow red with rage when in "grudge mode."
- The more Fir Bolg you kill, the deadlier the survivors become.

**Counter Strategy:**
- Don't let them snowball — kill them all quickly or don't engage.
- AoE spells wipe groups simultaneously, preventing grudge stacking.
- Crowd control prevents grudge attacks.

### Druid Wildshaper (Support Caster)

**Lore:** Masters of nature magic who blur the line between human and beast. Currently a bear. Might be an owl later. It's complicated.

**Appearance:** Human druid that periodically shifts into animal forms (bear, wolf, raven, stag).

**Stats:**
- Health: 200 (varies by form)
- Speed: 4.0 (varies by form)
- Damage: 15 (varies by form)
- Type: Elite Support/Melee

**Mechanic - Wild Shape:**
- Every 20 seconds, shifts to a random animal form with different stats:
  - **Bear:** +200 HP, slow, high damage (tank form)
  - **Wolf:** +2.0 speed, normal HP, normal damage (fast form)
  - **Raven:** Flying, -100 HP, ranged attacks (aerial form)
  - **Stag:** Healing aura for allies, +1.5 speed, low damage (support form)
- Cannot be targeted during transformation (brief invulnerability window).

**Counter Strategy:**
- Adapt to current form — nuke bear form with damage, catch raven with Magic Missile.
- Time spells to hit just after transformation when vulnerable.
- Finger of Death ignores form — always kills.

### Banshee (Screaming Herald)

**Lore:** Harbinger of death. When you hear her wail, someone is about to die. Probably you.

**Appearance:** Ghostly woman in tattered robes, floating, with glowing white eyes and long flowing hair.

**Stats:**
- Health: 80 (ethereal, hard to hit)
- Speed: 4.0 (floating, ignores obstacles)
- Damage: 0 (support)
- Type: Flying Support

**Mechanic - Wail of Death:**
- Every 15 seconds, Banshee screams (audible sound effect).
- Scream marks the nearest defender — marked unit takes +100% damage and has -50% accuracy.
- Mark lasts 10 seconds.
- Only one mark active at a time (screaming again changes the target).
- Ethereal: takes 50% reduced damage from physical attacks (melee, arrows), full damage from magic.

**Counter Strategy:**
- Magic damage bypasses ethereal defense.
- Kill her before she marks your King or elite defenders.
- Guardian Circle can protect marked units with temp HP.

### Dullahan (Headless Rider)

**Lore:** Headless horseman carrying his own head. Uses it to mark who dies next. Very efficient, very disturbing.

**Appearance:** Headless knight on a black horse, holding a glowing skull, draped in black cloak.

**Stats:**
- Health: 300
- Speed: 5.0 (mounted)
- Damage: 35
- Type: Elite Cavalry

**Mechanic - Death's Messenger:**
- Dullahan rides to the King and throws his skull at the nearest defender.
- Skull marks the target — after 10 seconds, target instantly dies (no save).
- Only magic can remove the mark: Guardian Circle or Teleport dispels it.
- Dullahan is immune to fear, charm, and sleep (already dead).

**Counter Strategy:**
- Kill Dullahan before he throws the skull.
- If mark is placed, immediately Guardian Circle or Teleport the marked unit.
- Entangle roots him before he can throw.

### Leshy (Forest Guardian)

**Lore:** Nature spirit that protects the forest. You're on his forest. He'd like you to leave. Violently.

**Appearance:** Humanoid made of wood, moss, and vines, with antlers and glowing green eyes.

**Stats:**
- Health: 250
- Speed: 3.0
- Damage: 20
- Type: Elite Melee

**Mechanic - Rooted Strength:**
- Leshy is rooted in place (doesn't move) but has extremely long reach (attacks in a large radius).
- Summons vine tendrils that attack all defenders in range.
- Takes 50% reduced damage while rooted.
- Can be uprooted by: Teleport, Black Hole pull, or dealing 300+ damage in one hit (breaks roots).
- Once uprooted, becomes mobile but loses damage reduction.

**Counter Strategy:**
- Don't enter his radius — ranged attackers and spells only.
- Uproot him to remove damage reduction, then focus fire.
- Wall of Stone blocks his vine tendrils.

---

## Faction: Ancient Sumora (Desert Kingdoms)

Ancient armies of the desert kingdoms — winged bulls, scorpion-men, and demons from the sands.

### Lamassu (Guardian Colossus)

**Lore:** Winged bull with a human head. Originally guarded palace gates. Now guards the path to your castle. Same job, different employer.

**Appearance:** Massive stone statue of a winged bull with a bearded human face, glowing with divine light.

**Stats:**
- Health: 700
- Speed: 2.0 (slow, flying)
- Damage: 30
- Type: Flying Tank

**Mechanic - Stone Guardian:**
- Lamassu is made of stone — 70% damage reduction from physical attacks.
- Flying — ignores obstacles and ground hazards.
- Protective Aura: All allies near Lamassu gain +20% damage reduction.
- Weak to magic damage (cracks in stone).

**Counter Strategy:**
- Magic damage bypasses stone armor.
- Disintegrate and Finger of Death work exceptionally well.
- Black Hole pulls it down, making it easier for archers to hit weak points.

### Girtablilu (Scorpion-Man)

**Lore:** Half man, half scorpion. All nightmare fuel. Guards the mountains where the sun rises. Currently guarding the approach to your castle.

**Appearance:** Human torso on a giant scorpion body, wielding a bow, tail raised with dripping stinger.

**Stats:**
- Health: 180
- Speed: 4.0
- Damage: 25 (ranged + melee)
- Type: Elite Hybrid

**Mechanic - Venomous Sting:**
- Girtablilu switches between ranged (bow) and melee (stinger) based on distance.
- Stinger attacks inflict deadly poison — 20 damage/second for 5 seconds.
- Only one poison application per unit (doesn't stack).
- Immune to Spike Growth (armored underbelly).

**Counter Strategy:**
- Kill before it reaches melee range.
- If poisoned, Guardian Circle provides temp HP to survive.
- Fire damage cauterizes the poison (reduces duration).

### Pazuzu (Demon Wind)

**Lore:** Demon king of the wind. Ironically also protects against other demons. He's complicated. And windy.

**Appearance:** Winged demon with dog face, scorpion tail, talons, and a very bad attitude.

**Stats:**
- Health: 350
- Speed: 5.5 (flying)
- Damage: 40
- Type: Elite Flying Melee

**Mechanic - Storm Bringer:**
- Pazuzu creates wind currents that push defenders away from the King (reverse Black Hole effect).
- Every 10 seconds, summons a dust devil that deals AoE damage and scatters units.
- Flying — ignores obstacles.
- Immune to wind-based spells (Squall does nothing).

**Counter Strategy:**
- Anchor defenders with Entangle to resist wind push.
- Magic Missile for reliable flying unit damage.
- Finger of Death ends his wind nonsense instantly.

### Utukku (Invisible Demon)

**Lore:** Demons that hide in shadows and possess the unwary. You can't see them. Your defenders can't see them. But they're there.

**Appearance:** Barely visible shadowy outline, only fully visible when attacking.

**Stats:**
- Health: 100
- Speed: 4.5
- Damage: 30 (ambush)
- Type: Melee Stealth

**Mechanic - Shadow Veil:**
- Utukku are invisible until they attack.
- First attack from stealth deals triple damage (90).
- After attacking, they become visible for 5 seconds, then re-stealth.
- Magic reveals them — area spells (Fireball, Squall) reveal Utukku in radius.

**Counter Strategy:**
- Blanket the battlefield with AoE to reveal them.
- Once visible, focus fire during the 5-second window.
- Guardian Circle protects against ambush damage.

### Chaos Dragon Spawn (Dragon Whelps)

**Lore:** Children of the primordial chaos dragon. Baby dragons. Still bigger than your house.

**Appearance:** Small dragons (car-sized) with scales matching their element (red, blue, green, black, white).

**Stats:**
- Health: 200
- Speed: 4.0 (flying)
- Damage: 25 + elemental
- Type: Flying Elite

**Mechanic - Elemental Affinity:**
- Each spawn has an element: Fire, Ice, Acid, Shadow, Lightning.
- Immune to their own element (Fire Spawn immune to fire, etc.).
- Breath weapon AoE every 15 seconds (elemental damage in cone).
- Spawn in groups of 3-5 with mixed elements.

**Counter Strategy:**
- Identify element and use opposite damage type.
- Chain Lightning chains between flying spawn.
- Wall of Stone blocks breath weapon cones.

---

## Faction: Voskyar (Winter Wastes)

Winter spirits, witches of the forest, and undead horrors from the frozen wastes.

### Leshen (Forest Horror)

**Lore:** Ancient forest spirit with a deer skull for a head and the temperament of a very territorial stag. The trees are his friends. You are not.

**Appearance:** Tall humanoid figure made of gnarled wood, wearing a deer skull, antlers spreading wide, roots trailing.

**Stats:**
- Health: 400
- Speed: 2.5
- Damage: 35
- Type: Elite Melee

**Mechanic - Forest Master:**
- Leshen summons ravens every 10 seconds that dive-bomb defenders (flying minions with 20 HP, 10 damage).
- Can teleport to any tree/wooden obstacle on the battlefield (instant repositioning).
- Rooting Presence: Defenders in melee range are slowed by 50% (roots grab at feet).

**Counter Strategy:**
- Magic Missile clears ravens automatically.
- Wall of Stone prevents teleport (no trees to jump to).
- Fire damage makes Leshen panic (wood burns).

### Rusalka (Drowned Maiden)

**Lore:** Spirit of a drowned woman. Lures victims with singing, then drowns them. Very committed to her theme.

**Appearance:** Pale woman with long wet hair, wearing a tattered wet dress, floating above the ground, water dripping.

**Stats:**
- Health: 120
- Speed: 3.5 (floating)
- Damage: 15
- Type: Melee Elite

**Mechanic - Drowning Touch:**
- Rusalka's attacks inflict "Drowning" debuff — unit takes 10 damage/second and has reduced movement speed (-30%).
- Drowning lasts 8 seconds or until dispelled.
- Water trail: Leaves pools of water where she walks — defenders in water are slowed.
- Vulnerable to frost (freezes the water, stunning her for 3 seconds).

**Counter Strategy:**
- Squall freezes her solid.
- Avoid water pools on the ground.
- Guardian Circle dispels Drowning debuff.

### Baba Yaga's Hut (Walking House)

**Lore:** A house that walks on giant chicken legs. Inside: a witch. Outside: chaos. Architecturally questionable, magically terrifying.

**Appearance:** Wooden hut on enormous scaly chicken legs, tilted at odd angles, windows glowing.

**Stats:**
- Health: 600
- Speed: 3.0
- Damage: 40 (kicks with chicken feet)
- Type: Elite Siege

**Mechanic - Mobile Witchcraft:**
- The hut spawns Baba Yaga herself (she emerges to cast spells, then retreats inside).
- Baba Yaga casts debuffs on defenders (hex: -30% damage, curse: +50% damage taken).
- Hut cannot be damaged while Baba Yaga is inside (she's casting a shield).
- When she emerges to curse defenders, hut becomes vulnerable.

**Counter Strategy:**
- Wait for Baba Yaga to emerge, then nuke the hut.
- Entangle roots the chicken legs (hilarious and effective).
- Teleport defenders away from curse radius.

### Vodyanoy (Water Demon)

**Lore:** Male water spirit that drowns swimmers and demands tribute. Looks like a frog had a bad day. Smells worse.

**Appearance:** Bloated green frog-man covered in algae, wielding a club, dripping water.

**Stats:**
- Health: 300
- Speed: 2.5 (slow on land, fast in water)
- Damage: 25
- Type: Elite Melee

**Mechanic - Water Dependency:**
- Vodyanoy spawns with a water bubble around him (visual aura).
- While bubble persists, he's at full strength.
- Bubble depletes over 40 seconds on land (no water sources).
- When bubble expires, he becomes "Dried Out" — -50% HP, -50% damage, -1.0 speed.
- Fire damage depletes bubble 3x faster.

**Counter Strategy:**
- Kite him with defenders until bubble expires.
- Fire spells dry him out instantly.
- Wall of Stone traps him on land longer.

### Zmey (Three-Headed Dragon)

**Lore:** Eastern dragon with three heads and a serious case of "too many opinions." They argue with each other mid-battle.

**Appearance:** Massive three-headed dragon, each head a different color (red, blue, green), serpentine body.

**Stats:**
- Health: 900
- Speed: 3.0 (ground) / 5.0 (flying when enraged)
- Damage: 50 per head
- Type: Mega Boss

**Mechanic - Arguing Heads:**
- Each head has a different breath weapon: Fire (red), Ice (blue), Poison (green).
- Zmey breathes randomly every 10 seconds (one head attacks at a time).
- When reduced to 50% HP, heads stop arguing and attack simultaneously (all 3 breath weapons in rapid succession).
- At 25% HP, Zmey takes flight (becomes flying unit) and rains breath weapons from above.

**Counter Strategy:**
- Spread defenders to avoid multi-breath attacks.
- Weak to opposing elements (fire head takes ice damage, etc.).
- Finger of Death before it takes flight.

---

## Faction: D&D Monster Manual

Classic D&D monsters that every adventurer fears — aberrations from the Far Realm, devils from the Nine Hells, and iconic dungeon denizens.

### Mind Flayer (Illithid)

**Lore:** Tentacle-faced psychic horrors from the Underdark. They eat brains. Your defenders have brains. This is a problem.

**Appearance:** Purple-skinned humanoid with octopus-like head, four tentacles, flowing robes, levitating.

**Stats:**
- Health: 200
- Speed: 3.5 (levitating, ignores obstacles)
- Damage: 20 + mind blast
- Type: Elite Psionic

**Mechanic - Mind Blast:**
- Every 15 seconds, Mind Flayer unleashes a psychic cone attack.
- Defenders hit by mind blast are stunned for 5 seconds (clutching heads in pain).
- Stunned units take +50% damage and cannot attack or move.
- Mind Flayer prioritizes targets with highest intelligence (King, elite units).
- Immune to charm and fear (alien mind).

**Counter Strategy:**
- Kill before mind blast triggers, or immediately after (during cooldown).
- Spread defenders to minimize cone impact.
- Teleport stunned units away from danger.
- Guardian Circle provides temp HP buffer during stun.

### Beholder

**Lore:** A floating ball of eyes with a giant mouth and serious anger management issues. Each eye fires a different ray of doom. It's having a very bad day, and so are you.

**Appearance:** Large spherical body with central eye, ten eyestalks, wide toothy maw, levitating.

**Stats:**
- Health: 600
- Speed: 2.5 (floating)
- Damage: 30 per ray
- Type: Elite Boss

**Mechanic - Eye Rays:**
- Beholder fires a random eye ray every 3 seconds at different targets:
  - **Disintegration Ray:** 100 instant damage (green beam)
  - **Fear Ray:** Target flees for 5 seconds (purple beam)
  - **Petrification Ray:** Target turns to stone, stunned 8 seconds (gray beam)
  - **Death Ray:** 50 damage + poison DoT (black beam)
  - **Telekinesis Ray:** Flings target backwards (blue beam)
- Central eye emits anti-magic cone — spells cast in cone are canceled (no mana refund).
- Beholder slowly rotates, sweeping anti-magic cone across battlefield.

**Counter Strategy:**
- Cast spells OUTSIDE the anti-magic cone.
- Track cone rotation and time your spells.
- High sustained damage (multiple medium spells better than one big spell that might be canceled).
- Entangle roots it, making cone position predictable.

### Gelatinous Cube

**Lore:** A 10-foot cube of transparent acidic jelly that slowly oozes down dungeon corridors, dissolving everything it touches. Subtle it is not.

**Appearance:** Large translucent cube with partially dissolved debris floating inside, semi-transparent, slowly pulsing.

**Stats:**
- Health: 400
- Speed: 1.5 (extremely slow)
- Damage: 40 (engulf)
- Type: Siege Tank

**Mechanic - Engulf:**
- Gelatinous Cube moves in a straight line toward the King.
- Any unit it touches is engulfed (absorbed inside the cube).
- Engulfed units take 15 acid damage per second and cannot act.
- After 10 seconds, engulfed units are dissolved (instant death).
- Cube can hold up to 5 units simultaneously.
- Immune to physical damage (blades pass through), weak to magic.

**Counter Strategy:**
- Magic damage only (immune to arrows and melee).
- Teleport engulfed units out of the cube before they dissolve.
- Wall of Stone redirects its path away from King.
- Disintegrate and Finger of Death work well (ironic justice).

### Lich

**Lore:** An undead archmage who achieved immortality through dark magic. He's had centuries to practice spellcasting. You've had maybe a few weeks. Good luck.

**Appearance:** Skeletal figure in tattered archmage robes, glowing eye sockets, floating grimoire, phylactery amulet glowing.

**Stats:**
- Health: 500
- Speed: 3.0 (floating)
- Damage: Varies (casts spells)
- Type: Elite Caster Boss

**Mechanic - Archmage's Arsenal:**
- Lich casts wizard spells at defenders:
  - **Power Word Kill:** Instantly kills target below 50% HP (15s cooldown)
  - **Cone of Cold:** Frost damage + slow in a cone (10s cooldown)
  - **Cloudkill:** Poison cloud that persists and drifts (20s cooldown)
  - **Counterspell:** Cancels your spell, refunds half mana (reactive)
- Has a phylactery (small glowing object) hidden on battlefield — Lich resurrects at phylactery location 10 seconds after death unless phylactery is destroyed.
- Phylactery has 50 HP and is targetable.

**Counter Strategy:**
- Find and destroy phylactery FIRST, then kill Lich.
- Phylactery glows faintly — use Fireball to AoE search for it.
- Finger of Death vs Lich in a wizard duel (ironic).
- Interrupt Lich's spells with damage spikes.

### Rust Monster

**Lore:** A creature that eats metal. Armor, swords, shields — it's all lunch. Your heavily armored defenders are suddenly very worried.

**Appearance:** Insectoid creature with antennae that glow when near metal, armored carapace, mandibles.

**Stats:**
- Health: 150
- Speed: 4.0
- Damage: 10 (bite) + rust
- Type: Elite Debuffer

**Mechanic - Rust Touch:**
- Rust Monster's antennae dissolve metal on touch.
- Defenders hit by antennae lose 30% armor permanently (stacks).
- After 3 hits, defender has 0 armor and takes full damage from all sources.
- Rust Monster prioritizes heavily armored units (King's Guard, King).
- Does not eat wooden shields or leather armor (only metal).

**Counter Strategy:**
- Kill quickly before it removes armor from elites.
- Crowd control (Entangle, roots) to prevent it reaching armored units.
- Magic damage ignores armor, so rusted units can still fight.
- Teleport armored units away.

### Owlbear

**Lore:** An owl crossed with a bear. Because nature said "why not?" and gave it the temperament of both: territorial and hungry.

**Appearance:** Massive bear body with owl head, feathered limbs, sharp talons and beak, angry screeching.

**Stats:**
- Health: 350
- Speed: 4.5
- Damage: 35
- Type: Elite Melee

**Mechanic - Feral Frenzy:**
- Owlbear enters frenzy when damaged below 50% HP.
- During frenzy: +50% damage, +1.5 speed, attacks twice per attack cycle.
- Frenzy lasts until Owlbear kills a target or 15 seconds elapse.
- Multi-attack: claws then beak, hitting same target twice.

**Counter Strategy:**
- Burst it down before frenzy triggers (above 50% HP).
- If frenzied, give it a low-value target to kill (zombie, corpse).
- Entangle stops frenzy movement.
- Guardian Circle tanks the double hits.

### Aboleth

**Lore:** Ancient psychic horror older than the gods. Lives in dark water, enslaves minds, very upset about being summoned to a battlefield.

**Appearance:** Massive eel-like fish with three eyes, pulsing with psychic energy, trailing slime, swimming through the air.

**Stats:**
- Health: 450
- Speed: 3.0 (swimming through air)
- Damage: 25 + enslave
- Type: Elite Psionic

**Mechanic - Enslave:**
- Aboleth fires psychic pulse every 12 seconds at target defender.
- Enslaved defender switches to Attacker team for 10 seconds (fights for enemy).
- Enslaved unit glows with purple psychic energy.
- After 10 seconds, enslavement breaks and defender returns to normal.
- Aboleth cannot enslave the King (too strong-willed).

**Counter Strategy:**
- Kill enslaved defenders before they damage allies (grim but effective).
- Teleport enslaved units away from battlefield.
- Guardian Circle blocks enslavement pulse (temp HP intercepts psychic attack).
- Focus fire Aboleth to prevent multiple enslavements.

### Tarrasque

**Lore:** The apocalypse beast. Kaiju-sized, nearly invincible, eats cities for breakfast. If you're fighting this, something has gone terribly, terribly wrong.

**Appearance:** Colossal dinosaur-like creature covered in spiky armor plates, massive horns, earth-shaking footsteps.

**Stats:**
- Health: 2500
- Speed: 2.0 (slow but unstoppable)
- Damage: 150 (bite)
- Type: Mega Boss

**Mechanic - Legendary Resistance:**
- Tarrasque is immune to all crowd control (cannot be rooted, slowed, stunned, teleported).
- Has 80% damage reduction from all sources.
- Regenerates 20 HP per second.
- Every 20 seconds, performs ground slam — AoE damage in huge radius, knocks units back.
- Can only be damaged by spells while standing (briefly vulnerable after ground slam).

**Counter Strategy:**
- Wait for ground slam vulnerability window, then burst damage.
- Focus on highest DPS spells during vulnerable phase.
- Finger of Death ignores damage reduction (true damage).
- Wall of Stone doesn't stop it, but slows it down slightly.
- This is a DPS race — kill before it reaches King.

### Mimic

**Lore:** A treasure chest that's actually a monster. Trust issues: the game. If only there were treasure chests on this battlefield to disguise as... wait.

**Appearance:** Disguised as debris/rocks/barrels on battlefield, then reveals teeth, tongue, and pseudopods.

**Stats:**
- Health: 180
- Speed: 0 (stationary) / 3.0 (revealed)
- Damage: 30 (ambush)
- Type: Ambush Melee

**Mechanic - False Appearance:**
- Mimics spawn disguised as battlefield debris (rocks, barrels, etc.).
- Remain stationary until a defender walks within melee range.
- Then they spring: 30 damage ambush attack + grapple (root) for 5 seconds.
- Grappled defender takes 15 damage/second (being digested).
- After ambush, Mimic becomes mobile and chases other defenders.

**Counter Strategy:**
- Blanket suspicious debris with AoE spells to reveal Mimics.
- Once revealed, they're standard melee units.
- Defenders grappled can be freed with Teleport.
- Fireball reveals AND damages hidden Mimics.

### Displacer Beast

**Lore:** A panther with tentacles that isn't quite where it appears to be. Your eyes say "there," physics says "actually over here." Cats are already hard enough to hit.

**Appearance:** Black panther with six legs and two tentacle appendages, shimmering with displacement aura.

**Stats:**
- Health: 200
- Speed: 5.5 (very fast)
- Damage: 25 + tentacle swipe
- Type: Fast Elite Melee

**Mechanic - Displacement:**
- Displacer Beast appears 3 feet away from its actual position (visual shimmer).
- Attacks against it have 50% miss chance (defenders swing at the illusion).
- Magic targeting (spells) also affected — projectiles curve toward illusion and miss 30% of the time.
- Taking damage flickers displacement briefly (1 second vulnerable).
- Tentacle swipe knocks defenders back.

**Counter Strategy:**
- AoE spells (Fireball, Chain Lightning) catch both real and illusory positions.
- Sustained damage (Disintegrate beam) eventually hits during flicker.
- Entangle roots the real beast even if you target the illusion.
- High volume of attacks overcomes miss chance.

### Pit Fiend (Devil)

**Lore:** Greater devil from the Infernal Planes. Made of hate, fire, and contractual obligations. The only thing more dangerous than his mace is his fine print.

**Appearance:** Massive red-skinned devil with bat wings, horns, wielding a flaming mace, wreathed in hellfire.

**Stats:**
- Health: 700
- Speed: 3.5 (ground) / 5.0 (flying when wings spread)
- Damage: 50 + fire
- Type: Elite Boss

**Mechanic - Infernal Tactics:**
- Alternates between ground combat (melee) and aerial bombardment (flying).
- **Ground Phase (30s):** Melee attacks with burning mace, leaves fire pools.
- **Aerial Phase (20s):** Flies up, rains fireballs down (AoE), unreachable by melee.
- Fear aura: Weak defenders within radius flee in terror.
- Immune to fire damage, weak to cold/holy magic.

**Counter Strategy:**
- Ground phase: Melee and spells both work.
- Aerial phase: Only spells/arrows work, use Magic Missile or Chain Lightning.
- Squall's frost damage is extra effective (devil hates cold).
- Entangle during ground phase locks him down.

### Balor (Demon)

**Lore:** Greater demon of flame and shadow, wielding a whip and sword, both on fire. When it dies, it explodes. Even death is an attack with this thing.

**Appearance:** Towering demon wreathed in flame, bat wings, horns, flaming sword and whip, standing in pillar of fire.

**Stats:**
- Health: 650
- Speed: 4.0 (flying)
- Damage: 60 (sword) + 30 (whip)
- Type: Elite Boss

**Mechanic - Death Throes:**
- Balor wields two weapons: sword (high damage, close) and whip (medium damage, reach).
- Whip can hit units at medium range and pulls them closer (drags defender toward Balor).
- Aura of flame: All units near Balor take 10 fire damage/second.
- **Death Explosion:** When killed, Balor explodes dealing 100 fire damage in a huge radius.
- Immune to fire, weak to cold and holy.

**Counter Strategy:**
- Do NOT cluster defenders near it (death explosion will kill them all).
- Kill with ranged spells from safety.
- Warn defenders to flee when Balor is low HP (explosion telegraphed).
- Teleport defenders out of explosion radius.
- Squall makes it weaker.

### Purple Worm

**Lore:** A 40-foot-long burrowing worm with a mouth like a tunnel and an attitude like a bad Tuesday. Mostly encountered when it burrows up beneath your feet. Surprise!

**Appearance:** Massive segmented purple worm, circular mouth with teeth rings, armored segments, erupting from the ground.

**Stats:**
- Health: 800
- Speed: 3.0 (burrowing, ignores obstacles)
- Damage: 80 (swallow)
- Type: Siege Boss

**Mechanic - Burrow and Swallow:**
- Purple Worm burrows underground (invisible) and surfaces beneath defenders.
- When surfacing, knocks all nearby units into the air (small AoE stun).
- Bite attack attempts to swallow a defender whole.
- Swallowed defender takes 25 acid damage/second, cannot act, doomed after 15 seconds.
- Worm can swallow up to 2 defenders at once.
- Killing the worm frees swallowed defenders (they crawl out, covered in slime).

**Counter Strategy:**
- Detect burrowing by ground tremor visual (ripples in dirt).
- Move defenders away from tremor before it surfaces.
- Teleport swallowed defenders out.
- Alternatively, kill worm quickly to free them.
- Magic damage penetrates armored segments.

---

## Special Unit Types

### Commanders (Buff Aura Units)

Elite units that enhance nearby allies. Killing them weakens the army.

| Unit | Faction | Aura Effect | Counter |
|---|---|---|---|
| **Einherjar Captain** | Frostheim | +30% damage to all allies in radius | Priority target, Finger of Death |
| **Hoplite Strategos** | Olympar | Grants +40% armor to allies in phalanx | Break formation with Teleport/Black Hole |
| **Asura General** | Asuryan | Allies attack 50% faster | AoE damage to kill general + troops together |
| **Leshy Ancient** | Faewood | Allies regenerate 5 HP/s | Fire damage to burn the healer |

### Lieutenants (Mini-Bosses)

Tougher than regular units, weaker than bosses, often have unique mechanics.

| Unit | Faction | Gimmick | Counter |
|---|---|---|---|
| **Berserker Champion** | Frostheim | Damage increases as HP drops | Burst damage to kill before rage peaks |
| **Gorgon Sister** | Olympar | Petrification gaze (weaker than Medusa) | Same as Medusa, just less deadly |
| **Rakshasa Lord** | Asuryan | Disguises as King's Guard | Watch for shimmer, AoE to reveal |
| **Bone Knight** | Voskyar | Revives once per battle | Fire damage on second death |

### Flying Units Summary

| Unit | HP | Speed | Role | Counter |
|---|---|---|---|---|
| **Valkyrie** | 100 | 5.0 | Healer/Resurrect | Magic Missile, Chain Lightning |
| **Harpy** | 50 | 5.5 | Dive bomber | Chain Lightning multi-target |
| **Garuda** | 500 | 6.0 | Elite striker | Finger of Death, timed Fireball |
| **Lamassu** | 700 | 2.0 | Flying tank | Magic damage only |
| **Pazuzu** | 350 | 5.5 | Wind pusher | Entangle + Magic Missile |
| **Dragon Spawn** | 200 | 4.0 | Elemental breath | Elemental counters |

### Subterranean Units

Units that burrow underground and emerge behind your lines.

| Unit | Faction | Mechanic | Counter |
|---|---|---|---|
| **Dvergr Sapper** | Frostheim | Tunnels under walls, emerges near King | Magic reveals tunneling (shimmer), Wall of Stone blocks emergence |
| **Antlion Horror** | Ancient Sumora | Creates sand pit that pulls defenders in | Teleport defenders out, Entangle stops pull |
| **Mole-Rat Swarm** | Voskyar | Burrows as a group, emerges to attack archers | AoE damage when they surface |

---

## Design Principles for Enemy Units

### What Makes Units Interesting

1. **Unique counters create decision-making.** Units that require specific spell responses (Hydra needs fire, Draugr needs holy/fire, Frost Giant weak to fire) force players to think about spell composition and timing.

2. **Vulnerability windows reward skill.** Units like Minotaur (stunned after charge) or Baba Yaga's Hut (vulnerable when witch emerges) reward observant players who time their spells correctly.

3. **Formation/positioning mechanics create spatial puzzles.** Phalanx, pack hunters, and aura buffs make positioning matter. Spells like Teleport, Black Hole, and Wall of Stone become tactical tools, not just damage dealers.

4. **Support units change priorities.** Healers, resurrectors, and buffers force players to shift focus from "biggest threat" to "most multiplicative threat." Killing a Valkyrie before she resurrects 5 enemies is worth more than killing a tank.

5. **Defensive mechanics that can be bypassed.** Invulnerability, damage reduction, and regeneration are only interesting if there's a clever way around them (Asura's timed boon, Vodyanoy's water bubble, Lamassu's magic weakness).

6. **Resource tension.** Units that drain mana indirectly (by forcing expensive spell responses) or directly (future idea: mana-draining attacks) create resource management challenges.

### Unit Variety Checklist

- [x] **Basic melee** (bread and butter)
- [x] **Basic ranged** (archer types)
- [x] **Tanks** (high HP, slow)
- [x] **Fast skirmishers** (low HP, high speed)
- [x] **Elite warriors** (balanced, dangerous)
- [x] **Flying units** (ignore obstacles)
- [x] **Support units** (healers, buffers)
- [x] **Bosses** (high HP, unique mechanics)
- [x] **Stealth units** (invisible/disguised)
- [x] **Regenerating units** (need specific counters)
- [x] **Formation-based** (stronger together)
- [x] **Summoners** (spawn minions)
- [x] **Siege units** (target King/structures)
- [ ] **Mana drainers** (future idea)
- [ ] **Spell reflectors** (future idea)

---

## Future Expansion Ideas

### Yokai Isles (Spirit Realm)
- **Oni** (demon ogres with spiked clubs)
- **Kappa** (water imps that drown victims)
- **Tengu** (crow-demons with wind magic)
- **Jorōgumo** (spider-woman shapeshifter)
- **Gashadokuro** (giant skeleton made of famine victims)

### Khemset Empire (Undead Legions)
- **Mummy Lord** (cursed pharaoh with plague aura)
- **Anubis Guard** (jackal-headed warriors)
- **Scarab Swarm** (flesh-eating beetles)
- **Sphinx** (riddle-based mechanic?)
- **Apep Serpent** (chaos snake that devours light)

### Sunstone Kingdoms (Jungle Warriors)
- **Jaguar Warrior** (elite melee, stealth in forest)
- **Eagle Warrior** (flying, dive attacks)
- **Star Demon** (descends from sky)
- **Cipactli** (crocodile-monster, amphibious)

### Dragon Dynasties (Eastern Kingdoms)
- **Jiangshi** (hopping vampire, drains life)
- **Dragon Turtle** (armored tank, water jets)
- **Nine-Tailed Fox** (illusions and charm)
- **Terracotta Soldier** (constructs, reforming)

---

## Notes for Implementation

### AI Behavior Patterns
- **Aggressive:** Charge directly at King (Minotaur, Cerberus)
- **Flanking:** Try to circle around defenders (Whelps, Harpies)
- **Formation-Holding:** Stay grouped (Phalanx, Pack Hunters)
- **Kiting:** Stay at range, retreat when approached (Naga, Girtablilu)
- **Ambush:** Hide/stealth then strike (Rakshasa, Utukku)
- **Support:** Stay behind front line, buff allies (Valkyrie, Druid)

### Visual Clarity
- **Auras:** Glowing circles on ground (Frost Giant frost, Leshy roots)
- **Status Effects:** Overhead icons (Marked, Cursed, Blessed, Drowning)
- **Telegraphing:** Wind-up animations for big attacks (Minotaur charge, Hydra head growth)
- **Weakness Indicators:** Color shifts when vulnerable (Baba Yaga emerging, Vodyanoy dried out)

### Progression Curve
- **Early Levels (1-10):** Basic units with simple mechanics (Infantry, Archers, Wolves)
- **Mid Levels (11-30):** Elite units with one special mechanic (Einherjar, Hoplites, Naga)
- **Late Levels (31-50):** Multi-mechanic units and mini-bosses (Hydra, Minotaur, Asura)
- **Boss Levels (every 10?):** Mega-bosses with phased fights (Kumbhakarna, Zmey, Tiamat)

### Balance Considerations
- **DPS Check Units:** High HP that require sustained damage (Frost Giant, Cerberus)
- **Burst Check Units:** Low HP but deadly if not killed fast (Banshee, Medusa)
- **Puzzle Units:** Specific counter required (Draugr needs fire, Asura needs time)
- **Resource Drain:** Force expensive spell usage (Valkyrie resurrects, Hydra regeneration)
