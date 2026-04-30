use bevy::prelude::*;

use crate::config::GameConfig;
use crate::state::{AppState, InGameState};

use super::components::FortifiedHordeShield;
use super::systems::{
    animate_fortified_horde_glow, apply_defender_toggles, apply_endless_scaling,
    apply_fortified_horde, apply_roguelite_effectiveness, cleanup_fortified_horde_glow,
    cleanup_game_mode, init_roguelite_run, init_toggle_resources, tick_wizard_cycle,
    update_attrition_survivors, update_wizard_cycle_flash,
};

/// Plugin that manages game mode lifecycle.
pub struct GameModePlugin;

impl Plugin for GameModePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(AppState::MainMenu), cleanup_game_mode)
            .add_systems(
                OnEnter(AppState::MetaGame),
                init_roguelite_run.after(crate::game::shared_systems::init_level_from_config),
            )
            .add_systems(
                OnEnter(AppState::InGame),
                (
                    apply_endless_scaling,
                    apply_roguelite_effectiveness,
                    apply_defender_toggles,
                    apply_fortified_horde,
                    init_toggle_resources,
                ),
            )
            .add_systems(
                Update,
                (
                    animate_fortified_horde_glow
                        .run_if(crate::game::run_conditions::any_exist::<FortifiedHordeShield>()),
                    cleanup_fortified_horde_glow
                        .run_if(crate::game::run_conditions::any_exist::<FortifiedHordeShield>()),
                    tick_wizard_cycle
                        .run_if(resource_exists::<super::components::WizardCycleTimer>),
                    update_wizard_cycle_flash.run_if(crate::game::run_conditions::any_exist::<
                        super::components::WizardCycleFlash,
                    >()),
                    // Refresh Excremage colors when wizard type changes mid-game
                    crate::game::units::wizard::spells::visual_assets::refresh_spell_visuals_for_wizard
                        .run_if(|config: Res<GameConfig>| config.is_changed()),
                )
                    .run_if(crate::game::run_conditions::is_gameplay_running),
            )
            .add_systems(
                OnEnter(InGameState::ScoreScreen),
                update_attrition_survivors,
            );
    }
}
