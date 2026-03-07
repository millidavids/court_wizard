# New UI Screen Reference

## Directory Structure

Each UI screen lives under `src/ui/screen_name/` (or nested under a parent like `src/ui/main_menu/screen_name/`):

```
screen_name/
├── mod.rs           # Module definition and re-exports
├── plugin.rs        # Plugin with OnEnter/OnExit/Update registration
├── components.rs    # Screen marker + button action enum
├── constants.rs     # ButtonStyle, colors, dimensions
└── systems.rs       # Setup (spawn UI) and interaction handlers
```

## Step-by-Step

### 1. Create components.rs

```rust
use bevy::prelude::*;

/// Marker component for all entities belonging to this screen.
/// Used for cleanup when exiting the screen state.
#[derive(Component)]
pub(super) struct OnScreenName;

/// Button actions for this screen.
#[derive(Component, Debug, Clone, Copy)]
pub(super) enum ScreenButtonAction {
    ActionOne,
    ActionTwo,
    Back,
}
```

### 2. Create constants.rs

```rust
use bevy::prelude::*;
use crate::ui::components::ButtonStyle;

// Colors
pub const BACKGROUND_COLOR: Color = Color::hsla(0.0, 0.0, 0.08, 1.0);
pub const TEXT_COLOR: Color = Color::hsla(0.0, 0.0, 0.9, 1.0);
pub const BUTTON_BACKGROUND: Color = Color::hsla(0.0, 0.0, 0.15, 1.0);
pub const BUTTON_BORDER: Color = Color::hsla(0.0, 0.0, 0.3, 1.0);

// Dimensions
pub const BUTTON_WIDTH: f32 = 250.0;
pub const BUTTON_HEIGHT: f32 = 65.0;

// Composite style
pub const BUTTON_STYLE: ButtonStyle = ButtonStyle {
    width: BUTTON_WIDTH,
    height: BUTTON_HEIGHT,
    border_width: 3.0,
    font_size: 20.0,
    background: BUTTON_BACKGROUND,
    border: BUTTON_BORDER,
    text_color: TEXT_COLOR,
};
```

### 3. Create systems.rs

```rust
use bevy::prelude::*;
use super::components::*;
use super::constants::*;
use crate::ui::systems::{spawn_button, spawn_page_container};
use crate::state::MenuState; // or InGameState, etc.

/// Spawn the screen's UI hierarchy.
pub fn setup(mut commands: Commands) {
    let container = spawn_page_container(
        &mut commands,
        OnScreenName,
        false,                   // true if this is a pause menu sub-screen
        Overflow::clip_y(),      // or Overflow::visible()
    );

    commands.entity(container).with_children(|parent| {
        // Title
        parent.spawn((
            Text::new("Screen Title"),
            TextFont::from_font_size(28.0),
            TextColor(TEXT_COLOR),
            OnScreenName,
        ));

        // Buttons
        spawn_button(parent, "Action One", ScreenButtonAction::ActionOne, &BUTTON_STYLE);
        spawn_button(parent, "Back", ScreenButtonAction::Back, &BUTTON_STYLE);
    });
}

/// Handle button clicks.
pub fn button_action(
    mut button_clicked: MessageReader<crate::game::input::messages::MouseClicked>,
    button_query: Query<&ScreenButtonAction>,
    mut next_state: ResMut<NextState<MenuState>>,
) {
    for event in button_clicked.read() {
        if let Ok(action) = button_query.get(event.button) {
            match action {
                ScreenButtonAction::ActionOne => {
                    // Handle action
                }
                ScreenButtonAction::Back => {
                    next_state.set(MenuState::Landing);
                }
                _ => {}
            }
        }
    }
}
```

### 4. Create plugin.rs

```rust
use bevy::prelude::*;
use super::systems;
use super::components::OnScreenName;
use crate::ui::systems::cleanup_screen;
use crate::ui::plugin::ButtonActionSet;
use crate::state::MenuState; // or InGameState, etc.

pub struct ScreenNamePlugin;

impl Plugin for ScreenNamePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
                OnEnter(MenuState::ScreenName),
                systems::setup,
            )
            .add_systems(
                OnExit(MenuState::ScreenName),
                cleanup_screen::<OnScreenName>,
            )
            .add_systems(
                Update,
                systems::button_action
                    .in_set(ButtonActionSet)
                    .run_if(in_state(MenuState::ScreenName)),
            );
    }
}
```

### 5. Create mod.rs

```rust
mod components;
mod constants;
mod plugin;
mod systems;

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
