# New UI Screen Reference

## Directory Structure

Each UI screen lives under `src/ui/screen_name/` (or nested under a parent like `src/ui/main_menu/screen_name/`). Use **feature-sliced layout**: one file per concern, not one big `systems.rs`.

**Simple screen** (game_over, instructions, single-page menu):
```
screen_name/
├── mod.rs        # mod declarations + pub use re-exports
├── plugin.rs     # Plugin registration only
├── setup.rs      # marker, button-action enum, spawn UI tree, constants
└── interaction.rs # button click + escape handlers
```

**Multi-tab screen** (settings, compendium, wizard_tower):
```
screen_name/
├── mod.rs
├── plugin.rs
├── setup.rs       # root container, tab switcher, marker
├── tab_one.rs     # tab one's panels + interaction
├── tab_two.rs     # tab two's panels + interaction
├── interaction.rs # cross-tab handlers (escape, back, etc.)
└── constants.rs   # only if many shared colors/dims; otherwise inline
```

**Complex HUD overlay** (in_game HUD with multiple bars):
```
in_game/
├── mod.rs
├── plugin.rs
├── spawn.rs       # HUD root setup
├── input.rs       # keyboard/gamepad shortcuts
├── bar_systems.rs # mana, ammo, level clock, etc.
├── boss_bar.rs    # boss health bar
├── wave_display.rs
└── buff_tracker.rs
```

**Hard rules:**
- `plugin.rs` does registration only.
- `mod.rs` does `mod` + `pub use` only.
- Files >300 lines split further.
- Components, systems, and constants for a single concern live together.

## Step-by-Step

### 1. Create setup.rs (marker, button-action enum, constants, spawn UI)

For simple screens, all of these belong in one file:

```rust
use bevy::prelude::*;
use crate::ui::components::ButtonStyle;
use crate::ui::systems::{spawn_button, spawn_page_container};

/// Marker component for cleanup.
#[derive(Component)]
pub(super) struct OnScreenName;

/// Button actions for this screen.
#[derive(Component, Debug, Clone, Copy)]
pub(super) enum ScreenButtonAction {
    ActionOne,
    Back,
}

// Constants — inline if few; move to constants.rs if shared across files
pub(super) const BACKGROUND_COLOR: Color = Color::hsla(0.0, 0.0, 0.08, 1.0);
pub(super) const TEXT_COLOR: Color = Color::hsla(0.0, 0.0, 0.9, 1.0);
pub(super) const BUTTON_STYLE: ButtonStyle = ButtonStyle {
    width: 250.0, height: 65.0,
    border_width: 3.0, font_size: 20.0,
    background: Color::hsla(0.0, 0.0, 0.15, 1.0),
    border: Color::hsla(0.0, 0.0, 0.3, 1.0),
    text_color: TEXT_COLOR,
};

/// Spawn the screen's UI hierarchy.
pub(in crate::ui::screen_name) fn setup(mut commands: Commands) {
    let container = spawn_page_container(&mut commands, OnScreenName, false, Overflow::clip_y());

    commands.entity(container).with_children(|parent| {
        parent.spawn((
            Text::new("Screen Title"),
            TextFont::from_font_size(28.0),
            TextColor(TEXT_COLOR),
            OnScreenName,
        ));

        spawn_button(parent, "Action One", ScreenButtonAction::ActionOne, &BUTTON_STYLE);
        spawn_button(parent, "Back", ScreenButtonAction::Back, &BUTTON_STYLE);
    });
}
```

### 2. Create interaction.rs (button + escape handlers)

```rust
use bevy::prelude::*;
use super::setup::ScreenButtonAction;
use crate::state::MenuState;

pub(in crate::ui::screen_name) fn button_action(
    mut button_clicked: MessageReader<crate::game::input::messages::MouseClicked>,
    button_query: Query<&ScreenButtonAction>,
    mut next_state: ResMut<NextState<MenuState>>,
) {
    for event in button_clicked.read() {
        if let Ok(action) = button_query.get(event.button) {
            match action {
                ScreenButtonAction::ActionOne => { /* ... */ }
                ScreenButtonAction::Back => next_state.set(MenuState::Landing),
            }
        }
    }
}
```

### 3. Create plugin.rs (registration only)

```rust
use bevy::prelude::*;
use super::setup::{self, OnScreenName};
use super::interaction;
use crate::ui::systems::cleanup_screen;
use crate::ui::plugin::ButtonActionSet;
use crate::state::MenuState;

pub struct ScreenNamePlugin;

impl Plugin for ScreenNamePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(MenuState::ScreenName), setup::setup)
            .add_systems(OnExit(MenuState::ScreenName), cleanup_screen::<OnScreenName>)
            .add_systems(
                Update,
                interaction::button_action
                    .in_set(ButtonActionSet)
                    .run_if(in_state(MenuState::ScreenName)),
            );
    }
}
```

### 4. Create mod.rs (declarations + re-exports only)

```rust
mod interaction;
mod plugin;
mod setup;

pub use plugin::ScreenNamePlugin;
```

### 6. Registration Points

**Add state variant** (if new screen):
- `src/state/states.rs` - Add variant to `MenuState`, `InGameState`, `PauseMenuState`, etc.

**Register plugin** in parent aggregator:
- Main menu screen: `src/ui/main_menu/plugin.rs`
- Pause menu screen: `src/ui/pause_menu/plugin.rs`
- In-game overlay: `src/ui/plugin.rs`

**Add escape handler** (if sub-screen):
- Use shared `escape_to_landing` or `escape_to_running` from `src/ui/systems.rs`
- Or write custom escape handler if behavior differs

**SP + MP dual support** (if in-game overlay):
```rust
// Register OnEnter/OnExit for both states:
.add_systems(OnEnter(InGameState::ScreenName), systems::setup)
.add_systems(OnEnter(MultiplayerGameState::ScreenName), systems::setup)
.add_systems(OnExit(InGameState::ScreenName), cleanup_screen::<OnScreenName>)
.add_systems(OnExit(MultiplayerGameState::ScreenName), cleanup_screen::<OnScreenName>)
// Update with OR condition:
.add_systems(
    Update,
    systems::button_action
        .run_if(in_state(InGameState::ScreenName).or(in_state(MultiplayerGameState::ScreenName))),
)
```

### 7. Shared UI Utilities

Available in `src/ui/systems.rs`:

- `spawn_page_container(commands, marker, is_pause, overflow)` - Root container with proper z-index
- `spawn_button(parent, text, action, style)` - Styled button with interaction
- `cleanup_screen::<T>(commands, query)` - Despawn all entities with marker T
- `handle_scroll::<T>(...)` - Scroll handling for scrollable containers
- `escape_to_landing(...)` - Escape key returns to landing page
- `escape_to_running(...)` - Escape key returns to gameplay (SP + MP aware)

### 8. State Hierarchy

```
AppState::MainMenu
  └─ MenuState: Landing, Settings, Changelog, Instructions, WizardSelect, Compendium, Multiplayer

AppState::InGame
  └─ InGameState: Running, Paused, SpellBook, CauldronMenu, ScoreScreen, Tutorial
      └─ PauseMenuState: Main, Settings, Instructions, Compendium

AppState::MetaGame
  └─ MetaGameState: WizardTower, Study, Compendium

AppState::MultiplayerGame
  └─ MultiplayerGameState: (mirrors InGameState)
```

### 9. UI Screen Checklist

- [ ] State variant added (if new state needed)
- [ ] Screen marker component defined
- [ ] Button action enum defined
- [ ] Constants define ButtonStyle with colors/dimensions
- [ ] Setup system spawns UI with marker on all entities
- [ ] Cleanup uses `cleanup_screen::<Marker>`
- [ ] Button actions use `ButtonActionSet` system set
- [ ] Update systems gated with `in_state()`
- [ ] Escape handler registered (for sub-screens)
- [ ] Plugin registered in parent aggregator
- [ ] SP + MP support added (if in-game overlay)
