# New Spell Reference

## Major Theme

Magic is indiscriminate and dangerous. Unless specifically said, it should affect all entities equally without consideration for team.

## Directory Structure

Every spell lives under `src/game/units/wizard/spells/spell_name/`:

```
spell_name/
├── mod.rs           # Module definition and re-exports
├── plugin.rs        # Plugin with system registration
├── components.rs    # Spell entity component(s) and visual markers
├── constants.rs     # All spell parameters and balance values
└── systems.rs       # Casting, movement, collision, effects, cleanup
```

## Step-by-Step

### 1. Create mod.rs

```rust
pub(crate) mod components;
pub(crate) mod constants;
mod plugin;
pub(crate) mod systems;

pub(super) use plugin::SpellNamePlugin;
```

### 2. Create constants.rs

Every spell needs a `PRIMED_*` constant defining its cast parameters:

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

// Projectile spells:
pub const PROJECTILE_SPEED: f32 = 3000.0;
pub const PROJECTILE_COLLISION_RADIUS: f32 = 15.0;

// Area spells:
pub const RADIUS: f32 = 100.0;
pub const DURATION: f32 = 5.0;

// Damage:
pub const TOTAL_DAMAGE: f32 = 50.0;

// Visual constants:
pub const INDICATOR_COLOR: Color = Color::srgba(0.8, 0.2, 0.2, 0.3);
```

### 3. Create components.rs

```rust
use bevy::prelude::*;

/// Main spell entity component.
#[derive(Component)]
pub struct SpellName {
    pub velocity: Vec3,
    pub damage: f32,
    pub radius: f32,
    // ... spell-specific fields
}

impl SpellName {
    pub fn new(velocity: Vec3, damage: f32, radius: f32) -> Self {
        Self { velocity, damage, radius }
    }
}

/// Visual effect marker (if spell has secondary visuals).
#[derive(Component)]
pub struct SpellNameExplosion {
    pub timer: f32,
    pub radius: f32,
}
```

### 4. Create systems.rs

Common system patterns:

```rust
use bevy::prelude::*;
use super::components::*;
use super::constants::*;
use crate::game::units::wizard::components::{Wizard, WizardCastState};

/// Handle spell casting (click to cast).
pub fn handle_casting(
    mut commands: Commands,
    mouse_button: Res<ButtonInput<MouseButton>>,
    wizard_query: Query<(&Transform, &WizardCastState), With<Wizard>>,
    // ... camera, cursor position, etc.
) {
    // Check cast state, spawn spell entity
}

/// Move projectile spells each frame.
pub fn move_projectiles(
    time: Res<Time>,
    mut query: Query<(&mut Transform, &SpellName)>,
) {
    for (mut transform, spell) in &mut query {
        transform.translation += spell.velocity * time.delta_secs();
    }
}

/// Check projectile collisions with enemies.
pub fn check_collisions(
    mut commands: Commands,
    spell_query: Query<(Entity, &Transform, &SpellName)>,
    enemy_query: Query<(Entity, &Transform, &Health), With<Enemy>>,
) {
    // Distance checks, apply damage, despawn on hit
}

/// Tick area effect duration and despawn when expired.
pub fn tick_lifetime(
    mut commands: Commands,
    time: Res<Time>,
    mut query: Query<(Entity, &mut SpellName)>,
) {
    for (entity, mut spell) in &mut query {
        spell.timer -= time.delta_secs();
        if spell.timer <= 0.0 {
            commands.entity(entity).try_despawn();
        }
    }
}
```

### 5. Create plugin.rs

Three common spell plugin patterns:

**Projectile spell** (fireball, magic missile):
```rust
use bevy::prelude::*;
use super::components::{SpellName, SpellNameExplosion};
use super::systems;
use super::super::run_conditions::*;
use crate::game::run_conditions::is_spell_effects_active;
use crate::game::units::wizard::components::Spell;

pub struct SpellNamePlugin;

impl Plugin for SpellNamePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (
                systems::handle_casting
                    .run_if(spell_is_primed(Spell::SpellName))
                    .run_if(spell_input_not_blocked)
                    .run_if(mouse_left_not_consumed)
                    .run_if(mouse_held_or_wizard_casting),
                (
                    systems::move_projectiles,
                    systems::check_collisions,
                    systems::despawn_distant,
                )
                    .chain()
                    .run_if(any_exist::<SpellName>()),
            )
                .run_if(is_spell_effects_active),
        );
    }
}
```

**Area/field spell** (haste, entangle):
```rust
pub struct SpellNamePlugin;

impl Plugin for SpellNamePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (
                systems::handle_casting
                    .run_if(spell_is_primed(Spell::SpellName))
                    .run_if(spell_input_not_blocked)
                    .run_if(mouse_left_not_consumed)
                    .run_if(mouse_held_or_wizard_casting),
                (
                    systems::apply_area_effect,
                    systems::tick_lifetime,
                    systems::update_visual,
                )
                    .chain()
                    .run_if(any_exist::<SpellName>()),
            )
                .run_if(is_spell_effects_active),
        );
    }
}
```

**Obstacle spell** (wall of stone, wall of fire):
```rust
pub struct SpellNamePlugin;

impl Plugin for SpellNamePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (
                systems::handle_cancel
                    .run_if(spell_is_primed(Spell::SpellName)),
                systems::handle_casting
                    .run_if(spell_is_primed(Spell::SpellName))
                    .run_if(spell_input_not_blocked)
                    .run_if(mouse_left_not_consumed)
                    .run_if(mouse_held_or_wizard_casting),
                (
                    systems::tick_lifetime,
                    systems::animate_visual,
                    systems::cleanup_expired,
                )
                    .chain()
                    .run_if(any_exist::<SpellName>()),
            )
                .run_if(is_spell_effects_active),
        );
    }
}
```

### 6. Register the Spell

**Add the Spell variant** to `src/game/units/wizard/components.rs`:
- Add variant to `Spell` enum
- Update all match arms (display name, icon path, description, mana cost, etc.)

**Register module** in `src/game/units/wizard/spells/mod.rs`:
```rust
pub(crate) mod spell_name;
// Add constants re-export:
pub(in crate::game::units::wizard) use spell_name::constants as spell_name_constants;
```

**Register plugin** in `src/game/units/wizard/spells/plugin.rs`:
- Add `SpellNamePlugin` to one of the `.add_plugins((...))` tuples (max 8 per tuple)

**Add spell icon** to `docs/assets/images/icons/spell_name.png` (64x64 recommended).

**Add audio** (optional) to `docs/assets/audio/sound_effects/spell_name_cast.ogg` and register in `src/game/units/wizard/spells/audio.rs`.

### 7. Spell Categories Checklist

- [ ] Spell variant added to `Spell` enum with all match arms
- [ ] `PRIMED_*` constant defined
- [ ] Plugin registered in spells plugin.rs
- [ ] Module registered in spells mod.rs
- [ ] Constants re-exported
- [ ] Icon asset added
- [ ] Audio asset added (if applicable) and registered in audio.rs
- [ ] Visual assets use shared `SpellVisualAssets` meshes/materials where possible
- [ ] All systems gated with `run_if(is_spell_effects_active)`
- [ ] Casting system uses standard run condition chain (primed + input checks)
