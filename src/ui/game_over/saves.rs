//! Game over post-game saves and stats accumulation.

use bevy::prelude::*;

use crate::config::save_data::{SavedCrystal, SavedWall};
use crate::config::{ActiveSave, ConfigChanged, GameConfig};
use crate::game::constants::INITIAL_DEFENDER_COUNT;
use crate::game::game_mode::components::{
    GameMode, LevelRunStats, RogueliteRunState, is_roguelite_mode,
};
use crate::game::resources::{CurrentLevel, GameOutcome, KillStats, TimeTravelState};
use crate::game::units::archer::constants::INITIAL_ARCHER_DEFENDER_COUNT;
use crate::game::units::wizard::spells::arcane_crystal::components::ArcaneCrystal;
use crate::game::units::wizard::spells::wall_of_stone::components::WallOfStone;
use crate::ui::wizard_tower::WizardTowerTab;

/// Saves efficiency for current level to config when entering game over screen.
///
/// This system runs on OnEnter(InGameState::ScoreScreen) BEFORE setup_game_over_screen
/// to save efficiency, but DOES NOT update the level yet (that happens after UI displays).
pub(super) fn save_efficiency_to_config(
    current_level: Res<CurrentLevel>,
    mut config: ResMut<GameConfig>,
    kill_stats: Res<KillStats>,
    mut config_events: MessageWriter<ConfigChanged>,
    time_travel: Option<Res<TimeTravelState>>,
    game_mode: Option<Res<GameMode>>,
    game_outcome: Res<GameOutcome>,
) {
    if time_travel.is_some() {
        return;
    }
    // In Roguelite mode, efficiency is tracked in the run state, not in config
    if is_roguelite_mode(game_mode.as_deref()) {
        return;
    }
    // Defeat = 0% efficiency (king died)
    let efficiency = if game_outcome.is_defeat() {
        0.0
    } else {
        let total_defenders = (INITIAL_DEFENDER_COUNT + INITIAL_ARCHER_DEFENDER_COUNT) as f32;
        let defenders_lost = kill_stats.defenders_killed as f32;
        1.0 - (defenders_lost / total_defenders)
    };

    // Store efficiency ratio for current level (the level that was just played)
    config
        .efficiency_ratios
        .insert(current_level.0.to_string(), efficiency);

    // Trigger config save immediately
    config_events.write(ConfigChanged);
}

/// Updates level and saves to config after game over screen is displayed.
///
/// This system runs AFTER setup_game_over_screen so the UI shows the correct
/// level that was just played, not the next level.
pub(super) fn update_level_after_display(
    mut current_level: ResMut<CurrentLevel>,
    mut config: ResMut<GameConfig>,
    game_outcome: Res<GameOutcome>,
    mut config_events: MessageWriter<ConfigChanged>,
    time_travel: Option<Res<TimeTravelState>>,
    game_mode: Option<Res<GameMode>>,
) {
    if time_travel.is_some() {
        return;
    }
    let is_roguelite = is_roguelite_mode(game_mode.as_deref());
    // Update level based on win/loss
    if *game_outcome == GameOutcome::Victory {
        current_level.0 += 1;
        // Update highest level if surpassed (Endless only)
        if !is_roguelite && current_level.0 > config.highest_level_achieved {
            config.highest_level_achieved = current_level.0;
        }
    }
    // Defeat: keep current level - player retries the same level

    // Save current level to config (for next level in run or normal progression)
    config.current_level = current_level.0;

    // Trigger config save immediately
    config_events.write(ConfigChanged);
}

/// Saves permanent walls (Terraformer talent) on victory so they persist to the next level.
/// Non-permanent walls are not saved and will despawn with the rest of the gameplay screen.
pub(super) fn save_walls_on_victory(
    game_outcome: Res<GameOutcome>,
    mut config: ResMut<GameConfig>,
    walls: Query<&WallOfStone>,
    time_travel: Option<Res<TimeTravelState>>,
) {
    if time_travel.is_some() || *game_outcome != GameOutcome::Victory {
        return;
    }

    let saved: Vec<SavedWall> = walls
        .iter()
        .filter(|wall| wall.permanent)
        .map(|wall| SavedWall {
            center_x: wall.center.x,
            center_z: wall.center.z,
            half_length: wall.half_length,
            half_width: wall.half_width,
            forward_x: wall.forward.x,
            forward_z: wall.forward.z,
            height: wall.height,
            empowerment: wall.empowerment,
        })
        .collect();

    config.saved_walls = saved;
}

/// Saves all permanent crystals on victory so they persist to the next level.
pub(super) fn save_crystals_on_victory(
    game_outcome: Res<GameOutcome>,
    mut config: ResMut<GameConfig>,
    crystals: Query<&ArcaneCrystal>,
    time_travel: Option<Res<TimeTravelState>>,
) {
    if time_travel.is_some() || *game_outcome != GameOutcome::Victory {
        return;
    }

    let saved: Vec<SavedCrystal> = crystals
        .iter()
        .filter(|c| c.permanent)
        .map(|c| SavedCrystal {
            x: c.position.x,
            z: c.position.z,
            range: c.range,
            empowerment: c.empowerment,
        })
        .collect();

    config.saved_crystals = saved;
}

/// Saves all living terrain on victory.
#[allow(clippy::too_many_arguments)]
pub(super) fn save_terrain_on_victory(
    game_outcome: Res<GameOutcome>,
    mut config: ResMut<GameConfig>,
    trees: Query<
        &crate::game::terrain::tree::components::Tree,
        Without<crate::game::terrain::tree::components::BurningTree>,
    >,
    ponds: Query<&crate::game::terrain::pond::components::Pond>,
    bushes: Query<
        &crate::game::terrain::bush::components::Bush,
        Without<crate::game::terrain::bush::components::BurningBush>,
    >,
    boulders: Query<&crate::game::terrain::boulder::components::Boulder>,
    time_travel: Option<Res<TimeTravelState>>,
    current_level: Res<CurrentLevel>,
    active_save: Res<ActiveSave>,
    game_mode: Option<Res<GameMode>>,
) {
    if time_travel.is_some() || *game_outcome != GameOutcome::Victory {
        return;
    }

    config.saved_trees = trees
        .iter()
        .map(|t| crate::config::save_data::SavedTree {
            x: t.center.x,
            z: t.center.z,
            scale: t.radius
                / crate::game::terrain::tree::constants::tree_radius_for_variant(t.sprite_index),
            sprite_index: t.sprite_index,
        })
        .collect();

    config.saved_ponds = ponds
        .iter()
        .map(|p| crate::config::save_data::SavedPond {
            x: p.center.x,
            z: p.center.z,
            radius: p.radius,
        })
        .collect();

    config.saved_bushes = bushes
        .iter()
        .map(|b| crate::config::save_data::SavedBush {
            x: b.center.x,
            z: b.center.z,
            scale: b.radius / crate::game::terrain::bush::constants::BUSH_RADIUS,
            sprite_index: b.sprite_index,
        })
        .collect();

    // Save all living boulders (both terrain-placed and thrown)
    config.saved_boulders = boulders
        .iter()
        .filter(|b| !b.sinking)
        .map(|b| crate::config::save_data::SavedBoulder {
            x: b.center.x,
            z: b.center.z,
            scale: b.radius / crate::game::terrain::boulder::constants::ROCK_RADIUS,
            sprite_index: b.sprite_index,
        })
        .collect();

    // In Endless mode, also save a per-level terrain snapshot for time travel.
    if crate::game::game_mode::components::is_endless_mode(game_mode.as_deref()) {
        crate::config::save_data::save_level_terrain(&active_save, current_level.0, &config);
    }
}

#[allow(clippy::too_many_arguments)]
pub(crate) fn accumulate_mode_level_stats(
    game_mode: Option<Res<GameMode>>,
    mut roguelite_run: Option<ResMut<RogueliteRunState>>,
    kill_stats: Res<KillStats>,
    current_level: Res<CurrentLevel>,
    game_outcome: Res<GameOutcome>,
    active_save: Res<ActiveSave>,
    time_travel: Option<Res<TimeTravelState>>,
) {
    // Defeat = 0% efficiency (king died)
    let efficiency = if game_outcome.is_defeat() {
        0.0
    } else {
        let total_defenders = (INITIAL_DEFENDER_COUNT + INITIAL_ARCHER_DEFENDER_COUNT) as f32;
        1.0 - (kill_stats.defenders_killed as f32 / total_defenders)
    };

    let level_stats = LevelRunStats {
        level: current_level.0,
        efficiency,
        attackers_killed: kill_stats.attackers_killed,
        undead_killed: kill_stats.undead_killed,
        defenders_lost: kill_stats.defenders_killed,
        elapsed_time: kill_stats.elapsed_time,
    };

    match game_mode.as_deref() {
        Some(&GameMode::Roguelite) => {
            if let Some(ref mut run) = roguelite_run {
                run.level_stats.push(level_stats);
            }
        }
        Some(&GameMode::Endless) => {
            // Save best stats for this level (only on victory, not during time travel)
            if *game_outcome == GameOutcome::Victory && time_travel.is_none() {
                crate::config::save_data::update_endless_best_stats(&active_save, &level_stats);
            }
        }
        None => {}
    }
}

/// Persists the current roguelite run to disk so it can be resumed later.
pub(super) fn save_dormant_roguelite_run(
    active_save: &ActiveSave,
    roguelite_run: &Option<Res<RogueliteRunState>>,
    config: &GameConfig,
    roguelite_modifiers: &Option<Res<crate::game::game_mode::components::RogueliteModifiers>>,
    active_toggles: &Option<Res<crate::game::game_mode::components::ActiveToggles>>,
    game_seed: &Option<Res<crate::game::seeded_rng::resources::GameSeed>>,
) {
    if let Some(run) = roguelite_run {
        crate::config::save_data::save_current_roguelite_run(
            active_save,
            run,
            config,
            roguelite_modifiers.as_deref(),
            active_toggles.as_deref(),
            game_seed.as_ref().map(|s| s.0),
        );
    }
}

/// Sets the appropriate wizard tower tab based on the current game mode.
pub(super) fn insert_wizard_tower_tab(commands: &mut Commands, game_mode: Option<&GameMode>) {
    let tab = if is_roguelite_mode(game_mode) {
        WizardTowerTab::Roguelite
    } else {
        WizardTowerTab::Endless
    };
    commands.insert_resource(tab);
}
