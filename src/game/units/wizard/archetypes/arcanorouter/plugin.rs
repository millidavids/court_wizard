use bevy::prelude::*;

use crate::{
    game::run_conditions::is_arcanorouter,
    state::{AppState, InGameState},
};

use super::{messages::SliderAdjustMessage, resources::ArcanoRouterState, systems::*};

/// Plugin for the Arcanorouter wizard archetype.
///
/// This archetype allows players to allocate resources between four stats:
/// - Range: Affects spell radius and distance
/// - Mana: Affects spell costs
/// - Power: Affects spell damage and effects
/// - Speed: Affects cast time
///
/// All sliders share a fixed pool of 400% (100% per slider by default),
/// and adjusting one automatically redistributes among the others.
pub(in crate::game) struct ArcanoRouterPlugin;

impl Plugin for ArcanoRouterPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<ArcanoRouterState>()
            .add_message::<SliderAdjustMessage>()
            .add_systems(
                Update,
                (
                    update_allocations,
                    sync_state_to_bonuses,
                    apply_bonuses_to_wizard_stats,
                    apply_wizard_stats_to_primed_spell,
                )
                    .chain()
                    .run_if(in_state(AppState::InGame))
                    .run_if(in_state(InGameState::Running))
                    .run_if(is_arcanorouter),
            )
            .add_systems(
                OnEnter(InGameState::GameOver),
                reset_allocations_on_game_over.run_if(is_arcanorouter),
            );
    }
}
