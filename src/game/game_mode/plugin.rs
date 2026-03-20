use bevy::prelude::*;

use crate::config::GameConfig;
use crate::game::constants::endless_effectiveness_bonus;
use crate::game::resources::CurrentLevel;
use crate::game::units::components::{Effectiveness, Team};
use crate::state::AppState;

use super::components::{is_endless_mode, is_roguelite_mode, GameMode, RogueliteRunState};

/// Plugin that manages game mode lifecycle.
pub struct GameModePlugin;

impl Plugin for GameModePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(AppState::MainMenu), cleanup_game_mode)
            .add_systems(
                OnEnter(AppState::MetaGame),
                init_roguelite_run.after(crate::game::shared_systems::init_level_from_config),
            )
            .add_systems(OnEnter(AppState::InGame), apply_endless_scaling);
    }
}

fn cleanup_game_mode(mut commands: Commands) {
    commands.remove_resource::<GameMode>();
    commands.remove_resource::<RogueliteRunState>();
}

/// Initializes a new roguelite run when entering MetaGame in Roguelite mode.
/// Only runs if no existing run is in progress (i.e., first entry, not returning from battle).
fn init_roguelite_run(
    mut config: ResMut<GameConfig>,
    mut current_level: ResMut<CurrentLevel>,
    mut commands: Commands,
    game_mode: Option<Res<GameMode>>,
    existing_run: Option<Res<RogueliteRunState>>,
) {
    if !is_roguelite_mode(game_mode.as_deref()) {
        return;
    }
    if existing_run.is_some() {
        return; // Run already in progress (returning from a battle victory)
    }

    // Fresh roguelite run: reset level and transient state
    config.current_level = 1;
    current_level.0 = 1;
    config.saved_walls.clear();
    config.saved_crystals.clear();
    config.efficiency_ratios.clear();

    commands.insert_resource(RogueliteRunState {
        started_at: crate::config::save_data::current_timestamp(),
        level_stats: vec![],
    });
}

/// Applies an effectiveness bonus to all attacker units in Endless mode past the final
/// introduction tier. This makes each level progressively harder.
fn apply_endless_scaling(
    game_mode: Option<Res<GameMode>>,
    current_level: Res<CurrentLevel>,
    mut attackers: Query<(&mut Effectiveness, &Team)>,
) {
    if !is_endless_mode(game_mode.as_deref()) {
        return;
    }
    let bonus = endless_effectiveness_bonus(current_level.0);
    if bonus <= 0.0 {
        return;
    }
    for (mut eff, team) in &mut attackers {
        if *team != Team::Attackers {
            continue;
        }
        // Boost attacker base effectiveness — the recalculate system will
        // propagate this to `current` via the normal formula.
        eff.base += bonus;
    }
}
