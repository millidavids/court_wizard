# Core System Reference

For adding game-wide systems that don't fit neatly into spell/unit/UI categories. Examples: pathfinding, combat modifiers, weather effects, new game modes, config additions.

## Module Structure

Core systems live directly under `src/game/`. Use **feature-sliced layout**: one file per concern.

**Simple system** (one cohesive concept, e.g., seeded_rng):
```
system_name/
├── mod.rs        # mod declarations + pub use re-exports
├── plugin.rs     # Plugin registration only
└── core.rs       # The component(s) + resource + system + constants
```

**Multi-feature system** (e.g., pathfinding, achievements, multiplayer):
```
system_name/
├── mod.rs
├── plugin.rs
├── feature_one.rs # one concern: components + systems + constants
├── feature_two.rs # another concern
├── feature_three.rs
├── messages.rs    # only if messages span features
└── constants.rs   # only if many cross-feature constants
```

**Hard rules:**
- `plugin.rs` does registration only.
- `mod.rs` does `mod` + `pub use` only.
- Files >300 lines split further by concern.
- Components, systems, and constants for a single concern live together.

## Component Design

**Prefer small, focused components over monolithic structs.** Status effects, conditions, and behavioral modifiers should each be their own `#[derive(Component)]` so systems can query and filter on them independently with `With<T>`, `Without<T>`, and `any_with_component::<T>`. Numeric-only modifiers applied once at creation time can stay as fields. See the [talents reference](../references/talents.md) for detailed examples.

## Plugin Template

```rust
use bevy::prelude::*;
use super::{resources::*, messages::*, systems};
use crate::game::run_conditions::is_gameplay_running;

pub struct SystemNamePlugin;

impl Plugin for SystemNamePlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<SystemNameState>()
            .add_message::<SystemNameMessage>()
            .add_systems(
                OnEnter(AppState::InGame),
                systems::init_system,
            )
            .add_systems(
                Update,
                (
                    systems::update_system_a,
                    systems::update_system_b,
                )
                    .chain()
                    .run_if(is_gameplay_running),
            )
            .add_systems(
                OnExit(AppState::InGame),
                systems::cleanup_system,
            );
    }
}
```

## Registration

Register in `src/game/plugin.rs` (GamePlugin):
```rust
.add_plugins((
    // ... existing plugins
    SystemNamePlugin,
))
```

## System Ordering

The game's Update schedule runs in this order:

1. **VelocitySystemSet** - Targeting, flocking, separation (parallel, immutable queries)
2. *Between sets* - Wall suppression, effectiveness, rough terrain
3. **MovementSystemSet** - Apply velocities to transforms
4. **PostCombatSet** - Wall collision, combat, invulnerability, corpse conversion
5. *After PostCombatSet* - Win/lose conditions

Place new systems in the appropriate set or use `.after()`/`.before()` for precise ordering:

```rust
.add_systems(
    Update,
    systems::your_system
        .after(VelocitySystemSet)
        .before(MovementSystemSet)
        .run_if(is_gameplay_running),
)
```

## Adding Config/Save Data

If the system needs persistent settings:

1. Add field to `GameConfig` in `src/config/resources.rs`:
```rust
#[serde(default = "default_value_fn")]
pub new_setting: SettingType,
```

2. Add default function and update `Default` impl.

3. Update config serialization in `src/config/systems.rs` if needed.

4. Config changes auto-save via debounce timer (no extra work needed).

## Adding to Spawn Queue

If the system spawns entities per-level:

1. Add variant to `SpawnTask` in `src/game/loading/spawn_queue.rs`
2. Handle the variant in `process_spawn_queue()` match arm
3. Add tasks to queue in `init_loading_progress()` system

## Message Pattern

For inter-system communication:

```rust
// messages.rs
use bevy::prelude::*;

#[derive(Message, Debug, Clone)]
pub struct SystemNameMessage {
    pub data: SomeData,
}
```

```rust
// Writing messages
fn sender_system(mut writer: MessageWriter<SystemNameMessage>) {
    writer.write(SystemNameMessage { data: some_data });
}

// Reading messages
fn receiver_system(mut reader: MessageReader<SystemNameMessage>) {
    for msg in reader.read() {
        // Handle message
    }
}
```

## Run Conditions

Common run conditions available in `src/game/run_conditions.rs`:

- `is_gameplay_running` - Game is actively simulating (not paused)
- `is_gameplay_active` - Game is loaded (includes paused)
- `is_spell_effects_active` - Spell systems should run
- `is_local_wizard_active` - Local wizard is controllable
- `in_state(State::Variant)` - Bevy built-in state check
- `any_exist::<T>()` - At least one entity with component T exists
- `resource_exists::<T>` - Resource T has been inserted

Create custom run conditions in a `run_conditions.rs` file when needed:

```rust
pub fn should_system_run(state: Res<SystemNameState>) -> bool {
    state.is_active
}
```

## Core System Checklist

- [ ] Module created under src/game/
- [ ] Plugin registered in GamePlugin (src/game/plugin.rs)
- [ ] Resources initialized with `.init_resource()` or `.insert_resource()`
- [ ] Messages registered with `.add_message()`
- [ ] Systems ordered correctly relative to existing sets
- [ ] All Update systems have `run_if()` guards
- [ ] Config/save data updated (if persistent)
- [ ] Spawn queue updated (if per-level entities)
- [ ] Cleanup system registered on OnExit(AppState::InGame)
- [ ] Works in both SP and MP (if applicable)
