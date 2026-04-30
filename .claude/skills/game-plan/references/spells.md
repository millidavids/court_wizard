# New Spell Reference

## Major Theme

Magic is indiscriminate and dangerous. Unless specifically said, it should affect all entities equally without consideration for team.

## Directory Structure

Every spell lives under `src/game/units/wizard/spells/spell_name/`. Use **feature-sliced layout**: one file per concern, not one big `systems.rs`. Examples by spell archetype:

**Projectile spell** (fireball, magic_missile, dispel):
```
spell_name/
├── mod.rs        # mod declarations + pub use re-exports
├── plugin.rs     # Plugin registration only
├── casting.rs    # cast initiation + indicator + projectile spawn
├── projectile.rs # projectile component + movement + collision
├── explosion.rs  # AoE-on-impact (if applicable)
├── talents.rs    # talent params + talent-driven systems
└── constants.rs  # tuning shared across feature files (or inline if few)
```

**Zone/aura spell** (entangle, healing_plume, fog_cloud):
```
spell_name/
├── mod.rs
├── plugin.rs
├── casting.rs    # cast initiation + zone spawn
├── zone.rs       # zone component + lifecycle + AoE damage/effect
├── talents.rs
└── constants.rs
```

**Beam/channel spell** (disintegrate, finger_of_death):
```
spell_name/
├── mod.rs
├── plugin.rs
├── casting.rs        # cast initiation
├── beam_lifecycle.rs # beam spawn + tick + despawn
├── hit_detection.rs  # damage application along beam path
├── talents.rs
└── constants.rs
```

**Obstacle spell** (wall_of_stone, wall_of_fire):
```
spell_name/
├── mod.rs
├── plugin.rs
├── casting.rs
├── wall_lifecycle.rs # spawn + tick + cleanup
├── damage.rs         # only if wall causes damage (wall_of_fire)
├── talents.rs
└── constants.rs
```

**Hard rules:**
- `plugin.rs` does registration only.
- `mod.rs` does `mod` + `pub use` only.
- Files >300 lines split further unless cohesive.
- Components, systems, and constants for a single concern live together in that concern's file. Don't create `components.rs` / `systems.rs` just to follow a template.

## Step-by-Step

### 1. Create mod.rs

`mod.rs` declares submodules and re-exports the plugin. No logic.

```rust
pub(crate) mod casting;
pub(crate) mod projectile;        // or zone, or beam_lifecycle, etc.
pub(crate) mod talents;
mod plugin;
// optional: pub(crate) mod constants; if many feature files share constants

pub(super) use plugin::SpellNamePlugin;
```

### 2. Choose where constants live

- **Few constants** (~5–15 lines): inline at the top of the feature file that uses them, or alongside the `PRIMED_SPELL_NAME` constant in `casting.rs`.
- **Many constants** shared across feature files: create `constants.rs`.

Every spell needs a `PRIMED_*` constant — keep this with `casting.rs` (or `constants.rs` if shared):

```rust
use super::super::components::{PrimedSpell, Spell};

pub const PRIMED_SPELL_NAME: PrimedSpell = PrimedSpell {
    spell: Spell::SpellName,
    cast_time: CAST_TIME,
    empowerment: 1.0,
    empowerment_consumed: false,
    mana_multiplier: 1.0,
    range_multiplier: 1.0,
};

pub const CAST_TIME: f32 = 3.0;
pub const MANA_COST: f32 = 30.0;
pub const TOTAL_DAMAGE: f32 = 50.0;
pub const INDICATOR_COLOR: Color = Color::srgba(0.8, 0.2, 0.2, 0.3);
```

### 3. Write feature files (each owns its components + systems + constants)

**Prefer small, focused components over monolithic structs.** Each distinct behavior or status effect should be its own component so systems can query and filter on it independently.

**casting.rs example:**
```rust
use bevy::prelude::*;
use crate::game::units::wizard::spells::utils::{
    build_wizard_input, handle_spell_release, try_start_cast_with_indicator,
    update_indicator_position, commit_spell_cast,
};

pub const PRIMED_SPELL_NAME: PrimedSpell = /* ... */;

pub(in crate::game::units::wizard::spells::spell_name) fn handle_casting(
    mut commands: Commands,
    /* ... params ... */
) {
    // Use shared utils to start cast + indicator
    // On cast complete: spawn projectile/zone, commit_spell_cast(...)
}
```

**projectile.rs example** (for projectile spells):
```rust
use bevy::prelude::*;

#[derive(Component)]
pub struct SpellNameProjectile {
    pub velocity: Vec3,
    pub damage: f32,
    pub radius: f32,
}

pub(in crate::game::units::wizard::spells::spell_name) fn move_projectiles(/* ... */) { /* ... */ }
pub(in crate::game::units::wizard::spells::spell_name) fn check_collisions(/* ... */) { /* ... */ }
pub(in crate::game::units::wizard::spells::spell_name) fn despawn_distant(/* ... */) { /* ... */ }
```

**talents.rs example:**
```rust
use bevy::prelude::*;
use crate::game::units::wizard::talents::resources::ActiveTalents;

#[derive(Default)]
pub(super) struct SpellNameTalentParams { /* ... */ }

pub(in crate::game::units::wizard::spells::spell_name) fn compute_talent_params(
    active_talents: Option<&ActiveTalents>,
) -> SpellNameTalentParams { /* ... */ }
```

**Status effects applied to units** (debuffs, buffs, conditions) should be separate components — not boolean flags inside a larger modifier. This enables `With<T>`/`Without<T>` query filters and `any_with_component::<T>` run conditions. See the [talents reference](talents.md) for detailed guidance.

### 4. Create plugin.rs

Three common spell plugin patterns:

`plugin.rs` does registration ONLY — import each system from its feature file, not from a single `systems` module.

**Projectile spell** (fireball, magic_missile):
```rust
use bevy::prelude::*;
use super::casting;
use super::projectile::{self, SpellNameProjectile};
use super::super::run_conditions::*;
use crate::game::run_conditions::is_spell_effects_active;
use crate::game::units::wizard::components::Spell;

pub struct SpellNamePlugin;

impl Plugin for SpellNamePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (
                casting::handle_casting
                    .run_if(spell_is_primed(Spell::SpellName))
                    .run_if(spell_input_not_blocked)
                    .run_if(mouse_left_not_consumed)
                    .run_if(mouse_held_or_wizard_casting),
                (
                    projectile::move_projectiles,
                    projectile::check_collisions,
                    projectile::despawn_distant,
                )
                    .chain()
                    .run_if(any_exist::<SpellNameProjectile>()),
            )
                .run_if(is_spell_effects_active),
        );
    }
}
```

**Zone/aura spell** (entangle, healing_plume):
```rust
use super::casting;
use super::zone::{self, SpellNameZone};

impl Plugin for SpellNamePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (
                casting::handle_casting
                    .run_if(spell_is_primed(Spell::SpellName))
                    .run_if(spell_input_not_blocked)
                    .run_if(mouse_left_not_consumed)
                    .run_if(mouse_held_or_wizard_casting),
                (
                    zone::apply_area_effect,
                    zone::tick_lifetime,
                    zone::update_visual,
                )
                    .chain()
                    .run_if(any_exist::<SpellNameZone>()),
            )
                .run_if(is_spell_effects_active),
        );
    }
}
```

**Obstacle spell** (wall_of_stone, wall_of_fire):
```rust
use super::casting;
use super::wall_lifecycle::{self, SpellNameWall};

impl Plugin for SpellNamePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (
                casting::handle_cancel
                    .run_if(spell_is_primed(Spell::SpellName)),
                casting::handle_casting
                    .run_if(spell_is_primed(Spell::SpellName))
                    .run_if(spell_input_not_blocked)
                    .run_if(mouse_left_not_consumed)
                    .run_if(mouse_held_or_wizard_casting),
                (
                    wall_lifecycle::tick_lifetime,
                    wall_lifecycle::animate_visual,
                    wall_lifecycle::cleanup_expired,
                )
                    .chain()
                    .run_if(any_exist::<SpellNameWall>()),
            )
                .run_if(is_spell_effects_active),
        );
    }
}
```

### 5. Register the Spell

**Add the Spell variant** to `src/game/units/wizard/spell_enum.rs` (or `wizard/components.rs` if pre-Phase-10):
- Add variant to `Spell` enum
- Update all match arms (display name, icon path, description, mana cost, etc.)

**Register module** in `src/game/units/wizard/spells/mod.rs`:
```rust
pub(crate) mod spell_name;
// Re-export the PRIMED_* constant from the casting feature file:
pub(in crate::game::units::wizard) use spell_name::casting as spell_name_constants;
```

**Register plugin** in `src/game/units/wizard/spells/plugin.rs`:
- Add `SpellNamePlugin` to one of the `.add_plugins((...))` tuples (max 8 per tuple)

**Add spell icon** to `assets/images/icons/spell_name.png` (64x64 recommended).

**Add audio** (optional) to `assets/audio/sound_effects/spell_name_cast.ogg` and register in `src/game/units/wizard/spells/audio.rs`.

### 6. Spell Categories Checklist

- [ ] Spell variant added to `Spell` enum with all match arms
- [ ] `PRIMED_*` constant defined (in `casting.rs` or `constants.rs`)
- [ ] Plugin registered in spells plugin.rs
- [ ] Module registered in spells mod.rs
- [ ] Constants re-exported
- [ ] Icon asset added
- [ ] Audio asset added (if applicable) and registered in audio.rs
- [ ] Visual assets use shared `SpellVisualAssets` meshes/materials where possible
- [ ] All systems gated with `run_if(is_spell_effects_active)`
- [ ] Casting system uses standard run condition chain (primed + input checks)
- [ ] Casting uses shared utils (`try_start_cast_with_indicator`, `commit_spell_cast`, etc.) — no inline duplication
- [ ] Each feature file <300 lines; if larger, split further by concern
