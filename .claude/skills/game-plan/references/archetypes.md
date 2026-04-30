# New Wizard Archetype Reference

Wizard archetypes are unique playstyles that modify how the wizard interacts with the game. Each archetype lives under `src/game/units/wizard/archetypes/`.

## Directory Structure

Use **feature-sliced layout**. The shape depends on archetype complexity:

**Simple archetype** (e.g., shepherd, an empty plugin):
```
archetype_name/
├── mod.rs        # mod declarations + pub use re-exports
└── plugin.rs     # Plugin registration only
```

**Standard archetype** (one mechanic, e.g., runes, roulette, arcanorouter):
```
archetype_name/
├── mod.rs
├── plugin.rs       # Plugin registration only
├── state.rs        # Resource + run condition + tuning constants
└── mechanic.rs     # The mechanic's components + systems
```

**Complex archetype** (multiple distinct features, e.g., gunslinger with 5 guns):
```
archetype_name/
├── mod.rs
├── plugin.rs
├── state.rs        # Resource, run conditions, shared types
├── feature_one.rs  # e.g., gun selection / slot UI
├── feature_two.rs  # e.g., bullet projectile + collision
├── feature_three.rs # e.g., reload mechanic
└── constants.rs    # only if many shared constants
```

**Hard rules:**
- `plugin.rs` does registration only.
- `mod.rs` does `mod` + `pub use` only.
- Files >300 lines split further.
- Components, systems, and constants for a single feature live together.

## Existing Archetypes for Reference

| Name | Directory | Mechanic |
|------|-----------|----------|
| RuneCaster | `archetypes/runes/` | Q/W/E/R key sequences empower spells |
| Randomancer | `archetypes/roulette/` | Roulette wheel selects random spells |
| Arcanorouter | `archetypes/arcanorouter/` | Slider-based stat allocation |
| Warglock (Gunslinger) | `archetypes/gunslinger/` | Replaces spells with 5 guns + ammo |
| Swordcerer (Battlemage) | `archetypes/battlemage/` | Enters battlefield as melee fighter |

## Step-by-Step

### 1. Add WizardType Variant

In `src/config/resources.rs`:
- Add variant to `WizardType` enum
- Implement `display_name()`, `description()`, `long_description()`, `locked_description()`
- Add to `WizardType::all()` array

### 2. Create Archetype Module

**mod.rs:**
```rust
pub(crate) mod state;
pub(crate) mod mechanic;
mod plugin;

pub use plugin::ArchetypeNamePlugin;
```

**state.rs** (resource + run condition + tuning constants):
```rust
use bevy::prelude::*;
use crate::config::resources::{GameConfig, WizardType};

/// Runtime state for the archetype.
#[derive(Resource, Debug, Clone, Default)]
pub struct ArchetypeNameState {
    // Archetype-specific state fields
}

/// Returns true when this archetype is selected.
pub(in crate::game::units::wizard::archetypes::archetype_name) fn is_archetype_name(
    config: Res<GameConfig>,
) -> bool {
    config.wizard_type == WizardType::ArchetypeName
}

// Tuning constants
pub(in crate::game::units::wizard::archetypes::archetype_name) const SOMETHING: f32 = 1.0;
```

**mechanic.rs** (the archetype's components + systems):
```rust
use bevy::prelude::*;
use super::state::ArchetypeNameState;

pub(in crate::game::units::wizard::archetypes::archetype_name) fn update_archetype_logic(/* ... */) {
    // ...
}
```

**plugin.rs** (registration only):
```rust
use bevy::prelude::*;
use super::state::{ArchetypeNameState, is_archetype_name};
use super::mechanic;
use crate::game::run_conditions::{is_gameplay_active, is_gameplay_running};

pub struct ArchetypeNamePlugin;

impl Plugin for ArchetypeNamePlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<ArchetypeNameState>()
            .add_systems(
                Update,
                mechanic::update_archetype_logic
                    .run_if(is_gameplay_active)
                    .run_if(is_gameplay_running)
                    .run_if(is_archetype_name),
            );
    }
}
```

### 3. Register Archetype

**Register plugin** in `src/game/units/wizard/archetypes/mod.rs`:
```rust
pub mod archetype_name;
// In ArchetypesPlugin::build():
app.add_plugins(archetype_name::ArchetypeNamePlugin);
```

### 4. Integration Considerations

**Spell system interaction:**
- If archetype modifies spell behavior, update `src/game/units/wizard/spells/` casting systems
- Use wizard component fields or resources that spells can check
- Example: Excremage overrides damage types, Roulette overrides spell selection

**UI requirements:**
- Custom UI elements go in `src/ui/` (action bar modifications, overlays, etc.)
- HUD indicators for archetype state (e.g., rune display, roulette wheel)
- Wizard select screen entry in `src/ui/main_menu/wizard_select/`

**Unlock conditions:**
- Add achievement that unlocks this archetype
- Gate in wizard select screen based on achievement resource
- Add locked_description flavor text

**Audio/visual:**
- Custom SFX overrides in `src/game/units/wizard/spells/audio.rs` (see Excremage pattern)
- Visual modifications in `src/game/units/wizard/spells/visual_assets.rs` (see Excremage recoloring)

### 5. Archetype Checklist

- [ ] WizardType variant added with all description methods
- [ ] Module created under archetypes/
- [ ] Plugin registered in archetypes/mod.rs
- [ ] Run condition created for archetype-active check
- [ ] State resource defined and initialized
- [ ] Core systems gated with archetype run condition
- [ ] Spell interaction defined (if modifying casting)
- [ ] UI elements created (HUD overlay, action bar changes)
- [ ] Unlock achievement defined
- [ ] Wizard select entry added
- [ ] Audio overrides added (if applicable)
- [ ] Visual overrides added (if applicable)
