# New Unit Reference

## Directory Structure

Each unit type lives under `src/game/units/unit_name/`. Use **feature-sliced layout**: one file per concern. Most simple units only need 2–4 files; complex units (with multiple abilities) split further.

**Simple unit** (infantry, archer, brute):
```
unit_name/
├── mod.rs        # mod declarations + pub use re-exports
├── plugin.rs     # Plugin registration only
├── spawn.rs      # marker component + spawn helper + asset preload
└── combat.rs     # combat / movement / animation (or split further)
```

**Complex unit with multiple abilities** (healer, dispeller, teleporter):
```
unit_name/
├── mod.rs
├── plugin.rs
├── spawn.rs        # marker component + spawn helper + asset preload
├── targeting.rs    # ability-specific targeting
├── ability.rs      # the unique ability (heal, dispel, teleport-attack)
└── animation.rs    # only if many animation states
```

**Hard rules:**
- `plugin.rs` does registration only.
- `mod.rs` does `mod` + `pub use` only.
- Files >300 lines split further by concern.
- Components, systems, and constants for a single concern live together in that concern's file.

## Step-by-Step

### 1. Create mod.rs

```rust
pub(crate) mod spawn;
pub(crate) mod combat;
mod plugin;

pub use spawn::{UnitName, UnitNameAssets};
pub use plugin::UnitNamePlugin;
```

### 2. Tuning constants

Place tuning constants at the top of the feature file that uses them. If many constants are shared across feature files, create a `constants.rs`:

```rust
// At top of spawn.rs (or constants.rs):
pub(in crate::game::units::unit_name) const BASE_HEALTH: f32 = 100.0;
pub(in crate::game::units::unit_name) const MOVE_SPEED: f32 = 200.0;
pub(in crate::game::units::unit_name) const ATTACK_DAMAGE: f32 = 10.0;
pub(in crate::game::units::unit_name) const ATTACK_RANGE: f32 = 50.0;
pub(in crate::game::units::unit_name) const MELEE_RADIUS: f32 = 80.0;
pub(in crate::game::units::unit_name) const COLLISION_RADIUS: f32 = 16.0;

// Sprite animation
pub(in crate::game::units::unit_name) const SPRITE_FRAMES: usize = 8;
pub(in crate::game::units::unit_name) const SPRITE_GRID_SIZE: usize = 4;
pub(in crate::game::units::unit_name) const ANIMATION_DURATION: f32 = 1.0;
pub(in crate::game::units::unit_name) const FRAME_DURATION: f32 =
    ANIMATION_DURATION / SPRITE_FRAMES as f32;
```

### 3. spawn.rs — marker, asset resource, preload, spawn helper

```rust
use bevy::prelude::*;

#[derive(Component)]
pub struct UnitName;

#[derive(Resource)]
pub struct UnitNameAssets {
    pub sprite_mesh: Handle<Mesh>,
    pub sprite_texture: Handle<Image>,
    // corpse materials, etc.
}

pub(in crate::game::units::unit_name) fn preload_assets(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut meshes: ResMut<Assets<Mesh>>,
) {
    let sprite_texture = asset_server.load("images/unit_name_sprite.png");
    let sprite_mesh = meshes.add(Rectangle::new(64.0, 64.0));
    commands.insert_resource(UnitNameAssets { sprite_mesh, sprite_texture });
}

pub(in crate::game) fn spawn_single_unit_name(
    commands: &mut Commands,
    assets: &UnitNameAssets,
    /* ... position, team, etc. ... */
) {
    // Spawn the unit entity with all its components.
}
```

### 4. combat.rs — targeting, movement, combat

Use shared helpers from the cross-cutting `units/` feature files (combat, movement, targeting, etc.) wherever possible:

```rust
use bevy::prelude::*;
use super::spawn::UnitName;
use crate::game::units::combat::update_melee_unit_targeting; // post-Phase-9
use crate::game::units::core::{Health, Team};

pub(in crate::game::units::unit_name) fn update_targeting(/* ... */) {
    update_melee_unit_targeting(/* ... */);
}

pub(in crate::game::units::unit_name) fn unit_movement(/* ... */) { /* ... */ }
pub(in crate::game::units::unit_name) fn unit_combat(/* ... */) { /* ... */ }
```

### 5. plugin.rs — registration only

```rust
use bevy::prelude::*;
use super::spawn::{self, UnitName};
use super::combat;
use crate::game::plugin::{VelocitySystemSet, MovementSystemSet};
use crate::game::run_conditions::is_gameplay_running;
use crate::game::units::MovementCalculationSet;

pub struct UnitNamePlugin;

impl Plugin for UnitNamePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, spawn::preload_assets)
            .add_systems(
                Update,
                (
                    combat::update_targeting.in_set(VelocitySystemSet),
                    combat::unit_movement.in_set(MovementCalculationSet),
                    combat::unit_combat,
                )
                    .run_if(any_exist::<UnitName>())
                    .run_if(is_gameplay_running),
            );
    }
}
```

### 6. Registration Points

**Register plugin** in `src/game/units/plugin.rs`:
- Add `UnitNamePlugin` to the `.add_plugins((...))` tuple

**Add spawn task** to `src/game/loading/spawn_queue.rs`:
- Add variant to `SpawnTask` enum (e.g., `DefenderUnitName`, `AttackerUnitName`)
- Implement spawn logic in the task processor

**Add spawn helper** as a `pub(in crate::game)` function in `spawn.rs`:
- `pub(in crate::game) fn spawn_single_unit_name(commands: &mut Commands, assets: &UnitNameAssets, ...)`
- Called by the spawn queue processor

**Add to level config** if unit count scales with difficulty/level.

**Add sprite asset** to `assets/images/` (sprite sheet for animation).

### 7. Shared Helpers Available

Before writing new logic, check these shared helpers (post-Phase-9 paths):

- `crate::game::units::combat::update_melee_unit_targeting` — Generic nearest-enemy targeting
- `crate::game::units::movement::*` — Separation, wall avoidance, weighted movement
- `crate::game::units::dots::*` — DoT processing
- `crate::game::units::status_effects::update_timed_modifier` — Generic CC tick
- `crate::game::shared_systems::*` — Effectiveness, ambience, shadows, cleanup
- `crate::game::combat_systems::*` — Global attack cycle, invulnerability, corpse conversion

### 8. Unit Integration Checklist

- [ ] Plugin registered in `src/game/units/plugin.rs`
- [ ] SpawnTask variant added to `spawn_queue.rs`
- [ ] Spawn helper function in `spawn.rs`
- [ ] Level config updated for unit counts
- [ ] Assets preloaded in `Startup` system
- [ ] Sprite sheet added to `assets/images/`
- [ ] Systems use `VelocitySystemSet` / `MovementCalculationSet` correctly
- [ ] All systems gated with `any_exist::<UnitName>()` and `is_gameplay_running`
- [ ] Shared targeting/movement helpers used where applicable
- [ ] Corpse materials defined for all teams the unit can appear on
- [ ] Each feature file <300 lines; if larger, split further by concern
