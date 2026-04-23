use bevy::prelude::*;

use crate::state::{InGameState, MetaGameState};

use super::resources::ActiveTutorial;
use super::systems::*;

/// Plugin that manages the tutorial overlay system.
pub(crate) struct TutorialPlugin;

impl Plugin for TutorialPlugin {
    fn build(&self, app: &mut App) {
        let progress = load_tutorial_progress();
        app.insert_resource(progress);

        // Trigger tutorials on entering specific states
        app.add_systems(
            OnEnter(MetaGameState::WizardTower),
            (
                // Controller primer runs first: when a gamepad is active
                // and the menu nav tutorial hasn't played yet, it queues
                // ahead of the standard wizard-tower walkthrough so
                // controller users learn how to navigate before anything
                // else.
                trigger_controller_menus_tutorial,
                trigger_wizard_tower_tutorial,
                trigger_time_travel_tutorial,
                trigger_study_tutorial,
            )
                .chain(),
        )
        .add_systems(
            OnEnter(InGameState::Running),
            (
                trigger_controller_in_game_tutorial,
                trigger_in_game_tutorial,
            )
                .chain(),
        )
        .add_systems(OnEnter(InGameState::SpellBook), trigger_spell_book_tutorial)
        .add_systems(
            OnEnter(InGameState::CauldronMenu),
            trigger_cauldron_tutorial,
        );

        // Entity tagging systems (split to stay under Bevy's system param limit)
        app.add_systems(
            Update,
            (
                tag_wizard_tower_entities,
                tag_study_entities,
                tag_in_game_entities,
                tag_spell_book_entities,
                tag_cauldron_entities,
            )
                .run_if(resource_exists::<ActiveTutorial>),
        );

        // Core tutorial overlay and highlight systems
        app.add_systems(
            Update,
            (
                spawn_tutorial_overlay,
                apply_highlight,
                animate_glow,
                position_tutorial_panel,
                update_tutorial_content,
                handle_next_button.in_set(crate::ui::plugin::ButtonActionSet),
                handle_skip_button.in_set(crate::ui::plugin::ButtonActionSet),
            )
                .run_if(resource_exists::<ActiveTutorial>),
        );

        // Cleanup tutorials when leaving states
        app.add_systems(OnExit(MetaGameState::WizardTower), cleanup_tutorial)
            .add_systems(OnExit(InGameState::SpellBook), cleanup_tutorial)
            .add_systems(OnExit(InGameState::CauldronMenu), cleanup_tutorial)
            .add_systems(OnExit(InGameState::Tutorial), cleanup_tutorial);
    }
}
