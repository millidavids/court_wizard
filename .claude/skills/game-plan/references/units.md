# New Unit Reference

## Directory Structure

Each unit type lives under `src/game/units/unit_name/`:

```
unit_name/
├── mod.rs           # Module definition and re-exports
├── plugin.rs        # Plugin with system registration
├── components.rs    # Unit marker component + data components
├── constants.rs     # Balance values (health, speed, damage, etc.)
├── resources.rs     # Asset handles (meshes, textures, materials)
└── systems.rs       # Targeting, movement, combat, animation
```

## Step-by-Step

### 1. Create mod.rs

```rust
pub(in crate::game) mod components;
pub(crate) mod constants;
mod plugin;
pub(in crate::game) mod resources;
pub(in crate::game) mod systems;

pub use components::UnitName;
pub use plugin::UnitNamePlugin;
pub use resources::UnitNameAssets;
```

### 2. Create constants.rs

```rust
/// Unit health at base level.
pub const BASE_HEALTH: f32 = 100.0;

/// Movement speed (units per second).
pub const MOVE_SPEED: f32 = 200.0;

/// Base attack damage.
pub const ATTACK_DAMAGE: f32 = 10.0;

/// Attack range (distance to target before attacking).
pub const ATTACK_RANGE: f32 = 50.0;

/// Melee detection radius (for melee slowdown).
pub const MELEE_RADIUS: f32 = 80.0;

/// Sprite animation parameters.
pub const SPRITE_FRAMES: usize = 8;
pub const SPRITE_GRID_SIZE: usize = 4; // 4x2 grid for 8 frames
pub const ANIMATION_DURATION: f32 = 1.0;
pub const FRAME_DURATION: f32 = ANIMATION_DURATION / SPRITE_FRAMES as f32;

/// Collision radius for pathfinding and separation.
pub const COLLISION_RADIUS: f32 = 16.0;
```

### 3. Create components.rs

```rust
use bevy::prelude::*;

/// Marker component identifying this entity as a UnitName.
#[derive(Component)]
pub struct UnitName;
```

Additional components as needed (state machines, timers, etc.).

### 4. Create resources.rs

```rust
use bevy::prelude::*;

/// Preloaded assets for UnitName entities.
#[derive(Resource)]
pub struct UnitNameAssets {
    pub sprite_mesh: Handle<Mesh>,
    pub sprite_texture: Handle<Image>,
    pub defender_corpse_materials: [Handle<StandardMaterial>; CORPSE_MATERIAL_VARIANTS],
    pub attacker_corpse_materials: [Handle<StandardMaterial>; CORPSE_MATERIAL_VARIANTS],
}

/// Preload assets at startup (before any unit spawns).
pub fn preload_assets(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let sprite_texture = asset_server.load("images/unit_name_sprite.png");
    let sprite_mesh = meshes.add(Rectangle::new(64.0, 64.0));
    // Build corpse materials...
    commands.insert_resource(UnitNameAssets {
        sprite_mesh,
        sprite_texture,
        // ...
    });
}
```

### 5. Create systems.rs

Use shared systems from `src/game/units/systems.rs` wherever possible:

```rust
use bevy::prelude::*;
use super::components::UnitName;
use super::constants::*;
use crate::game::units::systems::update_melee_unit_targeting;
use crate::game::units::components::*;

/// Update targeting for this unit type.
/// Prefer using shared `update_melee_unit_targeting()` or similar.
pub fn update_targeting(
    unit_query: Query<(Entity, &Transform, &Team), (With<UnitName>, Without<Corpse>)>,
    enemies: Query<(Entity, &Transform, &Team, &Health), Without<Corpse>>,
    mut velocities: Query<&mut TargetingVelocity>,
) {
    // Use shared targeting logic when possible
    update_melee_unit_targeting(&unit_query, &enemies, &mut velocities);
}

/// Unit-specific movement (runs in MovementCalculationSet).
pub fn unit_movement(
    time: Res<Time>,
    mut query: Query<(&mut Transform, &Velocity, &Acceleration), With<UnitName>>,
) {
    // Apply velocity and acceleration
}

/// Combat logic for this unit.
pub fn unit_combat(
    attack_cycle: Res<GlobalAttackCycle>,
    mut query: Query<(&Transform, &mut AttackCycleOffset, &Damage), (With<UnitName>, Without<Corpse>)>,
    mut targets: Query<&mut Health, Without<Corpse>>,
) {
    // Attack when cycle offset matches
}
```

### 6. Create plugin.rs

```rust
use bevy::prelude::*;
use super::components::UnitName;
use super::{resources, systems};
use crate::game::plugin::{VelocitySystemSet, MovementSystemSet};
use crate::game::run_conditions::is_gameplay_running;
use crate::game::units::MovementCalculationSet;

pub struct UnitNamePlugin;

impl Plugin for UnitNamePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, resources::preload_assets)
            .add_systems(
                Update,
                (
                    systems::update_targeting
                        .in_set(VelocitySystemSet),
                    systems::unit_movement
                        .in_set(MovementCalculationSet),
                    systems::unit_combat,
                    systems::update_animation,
                )
                    .run_if(any_exist::<UnitName>())
                    .run_if(is_gameplay_running),
            );
    }
}
```

### 7. Registration Points

**Register plugin** in `src/game/units/plugin.rs`:
- Add `UnitNamePlugin` to the `.add_plugins((...))` tuple

**Add spawn task** to `src/game/loading/spawn_queue.rs`:
- Add variant to `SpawnTask` enum (e.g., `DefenderUnitName`, `AttackerUnitName`)
- Implement spawn logic in the task processor

**Add spawn helper** as a `pub(in crate::game)` function in systems.rs:
- `pub(in crate::game) fn spawn_single_unit_name(commands: &mut Commands, assets: &UnitNameAssets, ...)`
- Called by the spawn queue processor

**Add to level config** if unit count scales with difficulty/level.

**Add sprite asset** to `docs/assets/images/` (sprite sheet for animation).

### 8. Shared Systems Available

Before writing new logic, check these shared systems:

- `update_melee_unit_targeting()` - Generic nearest-enemy targeting
- `apply_separation` - Flocking separation force
- `apply_wall_avoidance` - Avoids wall of stone obstacles
- `suppress_targeting_through_walls` - Zeroes targeting when wall blocks LOS
- `calculate_effectiveness` - Ally/enemy ratio effectiveness modifier
- `apply_rough_terrain_slowdown` - Terrain speed reduction
- `enforce_wall_collision` - Physical wall collision
- `combat` - Global attack cycle combat resolution
- `convert_dead_to_corpses` - Death handling and corpse conversion

### 9. Unit Integration Checklist

- [ ] Plugin registered in `src/game/units/plugin.rs`
- [ ] SpawnTask variant added to spawn_queue.rs
- [ ] Spawn helper function created
- [ ] Level config updated for unit counts
- [ ] Assets preloaded in Startup system
- [ ] Sprite sheet added to docs/assets/images/
- [ ] Systems use VelocitySystemSet / MovementCalculationSet correctly
- [ ] All systems gated with `any_exist::<UnitName>()` and `is_gameplay_running`
- [ ] Shared targeting/movement systems used where applicable
- [ ] Corpse materials defined for all teams the unit can appear on
