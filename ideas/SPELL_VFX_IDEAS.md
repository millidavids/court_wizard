# Spell Visual Effects Ideas

Ideas for unique, eye-catching visual effects to add to each spell. Organized by spell category. Effects are described in terms of what the player would see, with implementation notes where relevant.

**All effects use billboard sprites and flat 2D planes/quads** to keep the pixelated aesthetic consistent. No 3D meshes, no shader-based distortion, no volumetric rendering. Everything is flat sprites that either face the camera (billboards) or lie on the ground plane (flat quads).

---

## General VFX Principles

- **Billboards & quads only**: Every particle, indicator, and effect is either a camera-facing billboard sprite or a flat quad on the ground plane. This keeps everything consistent with the pixelated art style
- **Pixel-art sprites**: All VFX sprites should be hand-drawn or procedurally generated at low resolution, then scaled up with nearest-neighbor filtering to stay crispy
- **Readability first**: Effects must be clear from the wizard's elevated camera angle
- **Color language**: Fire = orange/red, Lightning = electric blue, Nature = green, Necrotic = purple/black, Frost = cyan/white, Force = gold/white
- **Layered effects**: Combine a base ground-plane quad + billboard particle layer + screen-space tint for maximum impact
- **Performance budget**: WASM target means particle counts need to stay reasonable — favor fewer, larger billboard sprites over hundreds of tiny ones

---

## Offense Spells

### Magic Missile
**Current VFX**: Blue glow halo + white sparkle trail
**Ideas**:
- **Arcane Rune Trail**: Each missile leaves behind small billboard sprites of pixelated arcane symbols that linger for 0.5s and then fade out — like the missile is "writing" in the air as it flies
- **Impact Flash Ring**: On hit, spawn a flat ground-plane quad with an expanding ring texture (pixelated concentric circles) that scales up and fades out quickly. Quick and satisfying
- **Color Shift on Lock**: Missiles swap to a brighter sprite frame or shift hue via color tint when they lock onto a target, giving visual feedback that homing has engaged
- **Convergence Lines**: When multiple missiles target the same enemy, spawn thin stretched billboard quads between each missile and the target — like pixelated targeting lines closing in

### Disintegrate
**Current VFX**: Pulsing beam with outer glow, origin flare, impact particles, color cycling
**Ideas**:
- **Pixel Dither Shimmer**: The area around the beam gets a few billboard sprites with a dithered semi-transparent pixel pattern that sway side to side — a lo-fi heat haze using animated sprite frames
- **Ground Scorch Trail**: Spawn small flat ground-plane quads with a dark scorch sprite beneath the beam as it sweeps. Each quad fades out over 2s. Shows where you've been aiming
- **Target Dissolution**: Enemies hit by the beam have their sprite alpha flicker rapidly (on/off frames) as if being erased pixel by pixel
- **Beam Crackle**: Small billboard sprites of pixelated lightning-bolt shapes spawn at random points along the beam and disappear after 1-2 frames, giving it a volatile feel

### Fireball
**Current VFX**: Orange glow, smoke wisps, spark burst on impact
**Ideas**:
- **Comet Tail**: In flight, spawn a stretched billboard quad behind the fireball using a tapered flame sprite that scales longer at higher speeds — a pixel-art fire streak
- **Shockwave Ring**: On explosion, spawn a flat ground-plane quad with an expanding ring sprite (pixelated radial lines) that scales outward and fades — a visible pressure wave
- **Crater Glow**: After explosion, leave a flat ground-plane quad with a pixel-art ember/glow sprite that tints from orange to dark red over 2-3 seconds before fading
- **Screenshake Lite**: Very subtle camera shake on detonation (1-2 pixel displacement for 0.2s) to sell the impact

### Chain Lightning
**Current VFX**: Lightning arcs with curved paths
**Ideas**:
- **Branching Forks**: Each arc spawns 1-2 short additional billboard quad strips that fork off partway along the main arc and fizzle out after 1-2 frames — more chaotic, natural lightning
- **Ground Spark Scatter**: At each bounce point, spawn a few small billboard sprites of pixelated spark/star shapes that scatter outward along the ground plane
- **Flash Illuminate**: Each strike spawns a brief flat ground-plane quad with a bright electric blue glow sprite centered on the hit point, illuminating nearby units for a frame or two
- **Diminishing Brightness**: Later bounce arcs use progressively lower alpha and thinner sprite strips, visually communicating the damage falloff

### Finger of Death
**Current VFX**: Purple beam that fades
**Ideas**:
- **Soul Rip**: On kill, spawn a semi-transparent billboard sprite of a ghostly skull/silhouette at the corpse that rises upward and fades out — the soul being torn away
- **Charging Vortex**: During cast time, spawn small purple billboard sprites that orbit the wizard's hand position in a tightening spiral, condensing toward the beam origin. Shows the spell building power
- **Necrotic Veins**: Spawn several small flat ground-plane quads with dark vein/crack sprites radiating outward from the impact point, fading out over 1-2s
- **Screen Desaturation Pulse**: A very brief (0.1s) desaturation of the entire screen when the beam fires, selling it as a world-shaking moment

### Lightning Rod
**Current VFX**: Lightning bolts descending, arcs jumping between units
**Ideas**:
- **Crackling Aura**: Spawn persistent small billboard sprites of pixelated spark shapes that jump between the rod tip and base at random intervals — looks constantly electrified
- **Storm Clouds**: A billboard sprite of a small dark pixelated cloud hovers above the rod, with the sprite flickering to a brighter "flash" frame before each strike
- **Conductor Chain**: When arcs jump to enemies, spawn a few tiny billboard spark sprites that orbit the hit unit for 1-2s — residual charge effect
- **Ground Burns**: Each lightning strike spawns a flat ground-plane quad with a star-shaped scorch sprite that fades out

### Meteor Fall
**Current VFX**: Meteors falling with fire trails, ground fire zones
**Ideas**:
- **Shadow Preview**: Spawn a flat ground-plane quad with a dark circular shadow sprite that scales up where each meteor will land, giving 0.5s warning. Creates tension
- **Debris Scatter**: On impact, spawn small billboard sprites of pixelated rock chunks that fly outward in arcs (parabolic motion) and fade on landing
- **Sky Streak**: Each meteor has a stretched billboard quad trailing behind it with a bright orange streak sprite — visible even from far above
- **Ground Crater**: Each impact spawns a flat ground-plane quad with a darkened crater sprite that persists for the spell duration

### Mark of Death
**Current VFX**: (Minimal — indicator on marked unit)
**Ideas**:
- **Skull Brand**: A billboard sprite of a pixelated spectral skull floats above the marked unit, slowly bobbing up and down. Unmistakable "this one is doomed" signal
- **Death Tether**: A stretched billboard quad connecting the wizard to the marked target with a dark tendril sprite, pulsing alpha when damage is amplified
- **Doom Countdown**: Flat ground-plane quads with concentric ring sprites that slowly scale inward around the marked unit over the duration, creating visual urgency
- **Damage Flash Amplified**: When the marked target takes damage, the hit flash billboard sprite is larger and brighter than normal, reinforcing the amplification

### Plague Wind
**Current VFX**: (Moving cloud)
**Ideas**:
- **Miasma Tendrils**: The cloud is made of several overlapping billboard sprites of wispy pixel-art tendril shapes that sway and reach toward nearby enemies
- **Flies/Motes**: Tiny dark billboard dot sprites orbit within and around the cloud on randomized paths, giving a pestilent swarming quality
- **Wilting Ground**: Flat ground-plane quads with a darkened/discolored terrain sprite spawn beneath the cloud as it moves, fading out slowly to leave a trail
- **Sickly Aura on Units**: Affected units get small green billboard sprites of poison bubble shapes rising upward from their position

---

## Control Spells

### Black Hole
**Current VFX**: Vibrating sphere, units spiral inward
**Ideas**:
- **Accretion Disk**: A flat ground-plane quad with a spinning ring sprite (pixelated energy swirl) centered on the black hole, rotating via UV or entity rotation. Gets brighter as it grows
- **Stretch Toward Center**: Units near the edge get their sprite slightly scaled (stretched horizontally toward the center) to approximate gravitational pull visually
- **Lensing Ring**: A billboard sprite of a bright pixelated ring at the event horizon edge, pulsing in alpha
- **Debris Orbit**: Small billboard sprites of pixel-art rock/dust chunks orbit the singularity on circular paths, speeding up as they get closer before disappearing at center
- **Sound Design Note**: Low, ominous hum that increases in pitch as it grows

### Wall of Stone
**Current VFX**: Semi-transparent preview, solid wall
**Ideas**:
- **Rising Animation**: Animate the wall's Y position upward over 0.3s (translate from below ground to final position). Spawn flat ground-plane quads with dust puff sprites at the base
- **Crack Decay**: As duration runs out, swap the wall's sprite/texture to progressively more cracked frames (3-4 frames of increasing cracks). More cracks = closer to breaking
- **Rubble Collapse**: When the wall expires, spawn several small billboard sprites of pixelated rock chunks that scatter outward with gravity and fade, rather than the wall just disappearing
- **Impact Dust**: If units are near the wall when it rises, spawn billboard sprites of small dust puff shapes that expand outward

### Wall of Fire
**Current VFX**: Burning wall with damage
**Ideas**:
- **Dynamic Flames**: The wall is made of multiple overlapping billboard flame sprites at varying heights along its length, each animating through pixel-art flame frames at offset timings. Occasional sprites scale taller for flare-ups
- **Pixel Shimmer**: Spawn a few semi-transparent billboard sprites with a dithered pixel pattern above the wall that bob gently — lo-fi heat haze
- **Ember Rain**: Small billboard sprites of single-pixel or 2-pixel ember dots drift upward and outward from the wall, slowly fading
- **Smoke Column**: Billboard sprites of small pixelated smoke puffs rise above the wall and drift, visible from the wizard's elevated position

### Entangle
**Current VFX**: Circle indicator
**Ideas**:
- **Vine Eruption**: Spawn billboard sprites of pixelated vine/root shapes that scale upward from the ground (Y animation from 0 to full size). Stagger the spawns outward from center for a growing effect
- **Struggling Animation**: Entangled units get a subtle oscillating X-offset on their sprite (1-2 pixel wobble), as if trying to break free
- **Leaf Scatter**: Small billboard sprites of pixelated leaf shapes scatter outward during the initial eruption, tumbling and fading
- **Wither on Expire**: When entangle ends, the vine billboard sprites tint from green to brown over 0.5s, then scale down to nothing — withering away

### Spike Growth
**Current VFX**: Spiky ground indicator
**Ideas**:
- **Sprouting Animation**: Spike billboard sprites scale up from zero over 0.5s, staggered across the zone for a spreading-outward feel
- **Blood Splatter**: When units take damage, spawn small flat ground-plane quads with red splatter sprites at the unit's feet
- **Spike Glint**: Individual spike billboard sprites briefly flash white for a single frame at random intervals — pixel-art light catches on sharp tips
- **Thorny Vine Pulse**: The ground-plane zone quad subtly pulses in scale (0.98-1.02x) giving the zone a living, breathing quality

### Squall
**Current VFX**: Ice projectiles falling, explosion effect
**Ideas**:
- **Howling Wind Lines**: Spawn elongated semi-transparent billboard sprites with diagonal streak textures across the storm area — pixelated wind lines
- **Frost Buildup**: A flat ground-plane quad with a frost/snow sprite that fades in gradually beneath the storm area, then slowly fades out after the storm ends
- **Ice Shard Trails**: Each falling ice chunk has a short stretched billboard quad trailing behind it with a blue streak sprite
- **Frozen Units Glint**: Units affected by the slow get a brief billboard sprite overlay of a pixelated ice crystal that flashes and fades

### Hypnotic Pattern
**Current VFX**: (Circle indicator)
**Ideas**:
- **Swirling Colors**: A flat ground-plane quad with an animated sprite sheet of rotating pixelated kaleidoscope frames (purple, gold, cyan shifting patterns)
- **Spiral Eyes**: Small billboard sprites of pixelated spiral/swirl symbols float above affected units' heads, bobbing gently
- **Floating Motes**: Billboard sprites of small pixelated glowing orb shapes drift lazily within the area on sine-wave paths — enchanted fireflies
- **Shatter on Damage**: When an affected unit breaks free, spawn several small billboard sprites of pixelated glass shard shapes that scatter outward and fade

### Sleep
**Current VFX**: (Circle indicator)
**Ideas**:
- **Zzz Bubbles**: Billboard sprites of pixelated "Z" letters rise from sleeping units, scaling up and fading as they drift upward — classic cartoon sleep
- **Lullaby Dust**: Small billboard sprites of golden sparkle dots drift downward within the spell area, settling toward the ground
- **Peaceful Glow**: A flat ground-plane quad with a warm golden circle sprite that pulses in alpha slowly, like breathing
- **Rude Awakening Flash**: When bonus damage triggers on wake-up, spawn a large bright billboard flash sprite and show an extra-large damage number to emphasize the hit

### Grease
**Current VFX**: (Ground zone)
**Ideas**:
- **Slick Sheen**: A flat ground-plane quad with a pixelated iridescent oil-slick sprite (a few frames of shifting rainbow highlights via animated sprite sheet)
- **Slip Trails**: Spawn small flat ground-plane quads with streak sprites behind units moving through grease — pixelated smear marks that fade
- **Ignition Spread**: When ignited, spawn flat ground-plane quads with fire sprites in a wave expanding outward from the ignition point, not all at once
- **Bubble Pop**: Small billboard sprites of pixel-art bubble shapes occasionally rise from the surface and pop (2-3 frame animation: appear, expand slightly, gone)

### Polymorph
**Current VFX**: (Transformation)
**Ideas**:
- **Poof Cloud**: Spawn several billboard sprites of pixelated pink/purple cloud puff shapes that expand outward from the transformation point and fade — a comedic cartoon "poof"
- **Wool Particles**: The sheep occasionally spawns a tiny billboard sprite of a white wool tuft that drifts slowly downward
- **Revert Warning**: Near end of duration, rapidly alternate the entity's sprite between sheep and original unit frames — a flickering transformation warning
- **Golden Sparkles**: Persistent small billboard sprites of pixelated star/sparkle shapes orbit the sheep slowly, distinguishing it from battlefield clutter

---

## Support Spells

### Guardian Circle
**Current VFX**: Pulsing circle indicator
**Ideas**:
- **Shield Flash**: When units inside take damage, spawn a brief billboard sprite of a pixelated shield/barrier shape at the hit point that flashes and fades — visual feedback that protection is working
- **Rune Ring**: Small billboard sprites of individual pixelated rune symbols orbit the circle's edge, slowly rotating around the circumference
- **Golden Particles**: Billboard sprites of small golden pixel dots drift upward through the zone, giving a holy/protected feeling
- **Shield Break Effect**: When temp HP is depleted on a unit, spawn billboard sprites of pixelated shard/fragment shapes that scatter outward from the unit — shield shattered

### Haste
**Current VFX**: (Circle indicator)
**Ideas**:
- **Speed Lines**: Affected units spawn elongated semi-transparent billboard sprites behind them as they move — stretched pixel-art dash lines that fade quickly, a classic "going fast" visual
- **Clock Motif**: The spell's ground-plane quad uses a sprite with a pixelated clock face design, with the clock hands animated to spin fast via sprite sheet frames
- **Bright Feet**: Spawn a brief small billboard sprite of a pixelated flash/sparkle at affected units' feet each time they take a step
- **Dust Trails**: Hasted units spawn small billboard sprites of dust puff shapes behind them more frequently than normal, making their speed visible at a glance

### Teleport
**Current VFX**: Growing source circle, pulsing destination crosshair
**Ideas**:
- **Portal Spiral**: Replace the source circle with a flat ground-plane quad using an animated sprite sheet of a swirling pixelated vortex. Matching vortex sprite at the destination
- **Flash Transition**: At the moment of teleport, spawn large bright billboard flash sprites at both source and destination, plus a stretched billboard quad between them with a crackling energy line sprite
- **Disorientation Wobble**: Teleported units get a brief oscillating X-scale wobble (squash and stretch, 0.8-1.2x) for 0.5s after arriving
- **Particle Stream**: During cast, spawn small billboard sprites of light pixel dots that travel along a path from source to destination, showing where units will go

### Raise the Dead
**Current VFX**: Resurrection indicator
**Ideas**:
- **Ground Crack Glow**: Spawn flat ground-plane quads with pixelated crack/fissure sprites around each rising corpse, tinted eerie green/purple and glowing in alpha
- **Spectral Rise**: Spawn a semi-transparent billboard sprite duplicate of the unit that rises upward first, then the actual unit entity rises and the ghost fades — filling in the afterimage
- **Dark Mist Seep**: Billboard sprites of dark pixelated wisp/smoke shapes drift upward from the ground around the resurrection zone
- **Eye Glow**: Newly raised undead get a small billboard sprite of glowing green/purple pixel dots at their eye position that fades over a few seconds

### Battle Hymn
**Current VFX**: (Circle indicator)
**Ideas**:
- **Musical Notes**: Billboard sprites of pixelated musical note shapes float upward from the center, drifting outward in waves
- **War Drums Pulse**: Flat ground-plane quads with ring sprites that scale outward from center rhythmically, pulsing like drumbeat shockwaves — one ring per beat
- **Red Aura**: Affected units get a semi-transparent billboard sprite overlay tinted red/orange that pulses gently while buffed
- **Banner Effect**: A billboard sprite of a pixelated war banner at the center, with a 2-3 frame sprite sheet animation of rippling in the wind

### Healing Plume
**Current VFX**: (Ground zone)
**Ideas**:
- **Rising Green Motes**: Billboard sprites of soft green pixel dots drift upward through the zone like gentle embers
- **Heartbeat Pulse**: The zone's ground-plane quad pulses in alpha with a slow rhythm synced to the heal tick rate — a visual heartbeat
- **Vine Growth**: Small billboard sprites of pixelated flower/vine shapes scale up around the zone's edge when it spawns, then tint brown and scale down when it expires
- **Health Sparkle**: Units being healed get small billboard sprites of green "+" symbols or sparkle shapes that float upward briefly on each heal tick

### Fog Cloud
**Current VFX**: (Ground zone)
**Ideas**:
- **Layered Fog Planes**: Stack 2-3 flat semi-transparent quads at different heights with pixelated cloud/fog sprites, each drifting at slightly different speeds — lo-fi depth effect
- **Silhouette Tint**: Units inside the fog get a darker color tint, making them appear as vague shapes from outside
- **Swirling Currents**: The fog plane quads slowly rotate at different rates and drift side to side, creating visible movement within the cloud
- **Evasion Flash**: When a unit evades an attack, spawn a brief billboard sprite of a pixelated "miss" slash or ghost image at the attack point

### Berserker Rage
**Current VFX**: (Circle indicator)
**Ideas**:
- **Red Veins**: Affected units get a semi-transparent billboard sprite overlay with a pixelated vein/crack pattern tinted red, pulsing in alpha
- **Rage Steam**: Small billboard sprites of red/dark pixel-art steam wisps rise from affected units periodically
- **Eye Glow Red**: Spawn small billboard sprites of red pixel dots at affected units' eye positions
- **Vulnerability Cracks**: A second semi-transparent billboard overlay sprite with a pixelated crack pattern on affected units — visually showing power at a cost
- **Rage Burst**: On activation, spawn a flat ground-plane quad with a red ring sprite that scales outward quickly from each affected unit and fades

### Phantasmal Force
**Current VFX**: (Decoy units)
**Ideas**:
- **Shimmer/Flicker**: Illusion entities periodically set their sprite alpha to 0 for a frame or two, creating a brief transparent flicker (visible only to the player)
- **Mirror Spawn**: On creation, spawn several billboard sprites of pixelated glass shard shapes that scatter outward from the cast point, each "becoming" an illusion as it lands
- **Spectral Glow**: Illusions have a semi-transparent billboard sprite of a faint blue/purple glow halo behind them to distinguish from real units for the player
- **Death Poof**: When illusions die, spawn billboard sprites of pixelated sparkle dust shapes that expand outward and fade — no corpse left behind

---

## Utility Spells

### Telekinesis
**Current VFX**: Indicator ring around target drop
**Ideas**:
- **Levitation Effect**: Animate the ingredient entity's Y position upward, then move it in an arc toward the wizard — simple transform animation
- **Force Lines**: Spawn thin stretched billboard quads between the wizard's position and the floating ingredient with a pixelated energy line sprite
- **Shimmer Trail**: The ingredient spawns small billboard sprites of sparkle pixel dots along its travel path
- **Catch Flash**: Brief billboard sprite of a golden flash/starburst when the wizard receives the ingredient

### Banishment
**Current VFX**: (Unit disappears)
**Ideas**:
- **Portal Vortex**: Spawn a flat ground-plane quad beneath the enemy with an animated sprite sheet of a pixelated swirling dark portal. Scale the enemy entity down to zero as they "sink in," then remove the portal quad
- **Dimensional Crack**: At the unit's last position, spawn a billboard sprite of a pixelated cracked-glass pattern that fades over 1s
- **Return Warning**: Before the banished unit returns, spawn the portal ground-plane quad again with the vortex sprite, plus billboard sprites of pixelated energy crackle shapes — 1s warning
- **Displacement Shimmer**: A small flat ground-plane quad with a faintly pulsing pixelated shimmer sprite marks the return spot for the duration

### Arcane Crystal
**Current VFX**: Rotating crystal, pulse on absorption
**Ideas**:
- **Prismatic Refraction**: Spawn flat ground-plane quads around the crystal with pixelated rainbow caustic sprites that slowly rotate via entity rotation
- **Absorption Tendrils**: When absorbing a spell, spawn thin stretched billboard quads from the crystal toward the incoming spell with pixelated energy tendril sprites
- **Charge Level Glow**: The crystal's billboard sprite swaps to brighter frames or increases color tint intensity as it absorbs more spells
- **Mini Spell Aesthetics**: Reflected mini-spells use the same billboard sprite systems as their full-size versions, just with smaller scale — tiny fireballs, thin lightning arcs, etc.
- **Crystal Resonance**: The crystal billboard sprite's scale oscillates with increasing amplitude as it charges up — visible vibration

### Dispel
**Current VFX**: White expanding sphere on impact
**Ideas**:
- **Shatter Effect**: When the bolt hits a spell effect, spawn billboard sprites of pixelated glass shard shapes in the spell's colors that scatter outward and fade — the spell visually shattering
- **Purge Wave**: The expanding impact uses a flat ground-plane quad with a ring sprite that briefly tints to the inverse of the dispelled spell's color
- **Trail of Nullification**: The dispel bolt spawns small semi-transparent billboard sprites along its path that dim nearby magical effects by reducing their alpha temporarily
- **Rune Break**: Spawn billboard sprites of pixelated broken rune fragment shapes that scatter outward from the dispelled effect

---

## Screen-Space & Global Effects

These effects aren't tied to a specific spell but could enhance the overall magic feel. Note: screen-space effects are the one exception to the billboard/quad rule — they operate as full-screen overlays.

### Mana Visualization
- **Wizard Aura**: A billboard sprite of a pixelated glow halo behind the wizard that tints from bright blue (full mana) to dim/gray (low mana)
- **Casting Glow**: While casting, the wizard's glow halo billboard sprite tints to the spell's associated color

### Environmental Reactions
- **Grass Sway**: Nearby billboard grass sprites briefly tilt/offset away from explosion points (offset their X position for a frame, then ease back)
- **Dust Kickup**: Ground-impact spells spawn billboard sprites of pixelated dust puff shapes that expand and fade

### Camera Effects (Use Sparingly)
- **Screenshake**: Very subtle camera shake for large explosions (Fireball, Meteor, Black Hole formation) — 1-2 pixel displacement for 0.2s
- **Brief Zoom Pulse**: For ultimate-tier spells (Finger of Death, Black Hole), a barely perceptible 1% zoom-in and back
- **Color Flash**: A full-screen semi-transparent quad overlay that flashes a spell's color for a single frame (purple for necrotic, orange for fire)

### Combo Visuals
- **Grease + Fire**: Flat ground-plane quads with fire sprites spread across the grease in a wave from the ignition point, not all at once
- **Sleep + Damage**: Spawn a larger-than-normal billboard hit flash sprite with the bonus damage number displayed at increased scale
- **Mark of Death + Any**: Damage number billboard sprites on marked targets tint red instead of white

---

## Implementation Priority

### Quick Wins (Low effort, high impact)
1. Impact flash ring ground-plane quads on Magic Missile hits
2. Shockwave ring ground-plane quad on Fireball explosion
3. Zzz billboard sprites on Sleep
4. Poof cloud billboard sprites on Polymorph
5. Speed line billboard sprites on Haste-buffed units
6. Skull brand billboard sprite on Mark of Death
7. Y-position rising animation for Wall of Stone

### Medium Effort, High Payoff
1. Accretion disk ground-plane quad with animated sprite for Black Hole
2. Vine eruption billboard sprites for Entangle
3. Portal vortex ground-plane quads for Banishment/Teleport
4. Soul rip billboard sprite on Finger of Death kill
5. Storm cloud billboard sprite above Lightning Rod
6. Charging vortex orbiting billboard sprites for Finger of Death cast
7. Crack decay sprite-frame swaps for Wall of Stone

### Ambitious (High effort, spectacular results)
1. Dithered pixel-shimmer billboard sprites for heat haze (Disintegrate, Wall of Fire)
2. Layered fog plane quads at multiple heights for Fog Cloud
3. Scale-stretch toward center for units near Black Hole
4. Ground scorch trail quads for Disintegrate
5. Multi-billboard dynamic flame variation for Wall of Fire
6. Screenshake + color flash system for explosions
