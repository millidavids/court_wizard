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
        app.init_resource::<super::resources::PendingTutorials>();

        // Trigger tutorials on entering specific states. Both menu primers
        // are also re-evaluated each frame in Update via the modality
        // enforcer + Update triggers below, so switching input devices in
        // the tower swaps overlays cleanly.
        app.add_systems(
            OnEnter(MetaGameState::WizardTower),
            trigger_kbm_menus_tutorial,
        )
        // Per-tab tutorials fire the first time each Wizard Tower tab is
        // opened. The watcher runs in Update so it catches both the initial
        // tab inserted on entry and subsequent tab switches.
        .add_systems(
            Update,
            trigger_wizard_tower_tab_tutorial
                .run_if(in_state(MetaGameState::WizardTower))
                .run_if(
                    resource_exists::<crate::ui::wizard_tower::WizardTowerTab>
                        .and(resource_changed::<crate::ui::wizard_tower::WizardTowerTab>),
                ),
        )
        .add_systems(
            Update,
            trigger_wizard_select_tutorial
                .run_if(in_state(MetaGameState::WizardTower))
                .run_if(
                    resource_exists::<crate::ui::wizard_tower::RightPanelView>
                        .and(resource_changed::<crate::ui::wizard_tower::RightPanelView>),
                ),
        )
        .add_systems(
            Update,
            trigger_study_spell_selected_tutorial
                .run_if(in_state(MetaGameState::WizardTower))
                .run_if(
                    resource_exists::<crate::ui::wizard_tower::SelectedStudySpell>
                        .and(resource_changed::<crate::ui::wizard_tower::SelectedStudySpell>),
                ),
        )
        .add_systems(OnEnter(InGameState::Running), trigger_in_game_tutorial)
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

        // Modality enforcer: dismisses any active tutorial whose required
        // input modality doesn't match the current input device. Runs every
        // frame so switching between mouse and controller mid-overlay swaps
        // immediately. The KBM/controller menu primers also re-fire from
        // here when the modality flips back.
        //
        // Order: must run AFTER `update_tutorial_content` so the latter's
        // text-rebuild commands (which target the overlay's text entity) are
        // queued before the enforcer's overlay-despawn commands. Otherwise
        // the rebuild panics inserting onto an already-despawned entity.
        app.add_systems(
            Update,
            (
                enforce_tutorial_modality,
                drain_pending_tutorials,
                trigger_kbm_menus_tutorial.run_if(in_state(MetaGameState::WizardTower)),
            )
                .chain()
                .after(update_tutorial_content),
        );

        // Cleanup tutorials when leaving states
        app.add_systems(OnExit(MetaGameState::WizardTower), cleanup_tutorial)
            .add_systems(OnExit(InGameState::SpellBook), cleanup_tutorial)
            .add_systems(OnExit(InGameState::CauldronMenu), cleanup_tutorial)
            .add_systems(OnExit(InGameState::Tutorial), cleanup_tutorial);
    }
}
