use bevy::prelude::*;
use bevy_steamworks::{Client, UploadScoreMethod};

use crate::config::{GameConfig, WizardType};
use crate::game::game_mode::components::{
    ActiveToggles, GameMode, ROGUELITE_MAX_LEVEL, RogueliteModifiers, RogueliteRunState,
    RunAggregateStats, ToggleModifier,
};
use crate::game::resources::{CurrentLevel, GameOutcome};

use super::constants::{LeaderboardId, display_type, sort_method, steam_board_name};
use super::resources::LeaderboardHandles;

pub(super) fn prewarm_leaderboard_handles(client: Res<Client>, handles: Res<LeaderboardHandles>) {
    for id in LeaderboardId::all() {
        if handles.map.contains_key(&id) {
            continue;
        }
        let tx = handles.sender();
        let name = steam_board_name(id);
        client.user_stats().find_or_create_leaderboard(
            name,
            sort_method(id),
            display_type(id),
            move |result| match result {
                Ok(Some(leaderboard)) => {
                    info!("Leaderboard '{name}' ready");
                    let _ = tx.send((id, leaderboard));
                }
                Ok(None) => {
                    warn!("Leaderboard '{name}' not found and could not be created");
                }
                Err(e) => {
                    warn!("Leaderboard '{name}' find error: {e:?}");
                }
            },
        );
    }
}

pub(super) fn drain_leaderboard_handle_results(mut handles: ResMut<LeaderboardHandles>) {
    handles.drain();
}

pub(super) fn handles_incomplete(handles: Res<LeaderboardHandles>) -> bool {
    handles.map.len() < LeaderboardId::all().len()
}

/// Submits run results to the appropriate Steam leaderboards on ScoreScreen entry.
///
/// Endless defeats go to `endless_level`. Roguelite run ends go to either the
/// "default" or "modified" board depending on whether any modifier deviates from
/// its default value. Full roguelite clears (level 25 victory) additionally submit
/// total run time to the matching clear-time board.
#[allow(clippy::too_many_arguments)]
pub(super) fn submit_run_scores_to_steam(
    client: Res<Client>,
    handles: Res<LeaderboardHandles>,
    game_outcome: Res<GameOutcome>,
    game_mode: Option<Res<GameMode>>,
    current_level: Res<CurrentLevel>,
    config: Res<GameConfig>,
    modifiers: Option<Res<RogueliteModifiers>>,
    toggles: Option<Res<ActiveToggles>>,
    roguelite_run: Option<Res<RogueliteRunState>>,
) {
    let submit = |id: LeaderboardId, score: i32, details: &[i32]| {
        let Some(handle) = handles.map.get(&id) else {
            debug!(
                "Leaderboard handle not ready for {:?}; skipping submission",
                id
            );
            return;
        };
        let name = steam_board_name(id);
        client.user_stats().upload_leaderboard_score(
            handle,
            UploadScoreMethod::KeepBest,
            score,
            details,
            move |result| match result {
                Ok(Some(uploaded)) => {
                    info!(
                        "Leaderboard '{name}': submitted {score} (rank {} -> {}, changed={})",
                        uploaded.global_rank_previous,
                        uploaded.global_rank_new,
                        uploaded.was_changed,
                    );
                }
                Ok(None) => {
                    warn!("Leaderboard '{name}': submission rejected by Steam");
                }
                Err(e) => {
                    warn!("Leaderboard '{name}': submission error: {e:?}");
                }
            },
        );
    };

    match game_mode.as_deref() {
        Some(GameMode::Endless) => {
            if !game_outcome.is_defeat() {
                return;
            }
            submit(LeaderboardId::EndlessLevel, current_level.0 as i32, &[]);
        }
        Some(GameMode::Roguelite) => {
            let Some(run) = roguelite_run else {
                return;
            };
            let levels_played = run.level_stats.len() as u32;
            let full_clear =
                *game_outcome == GameOutcome::Victory && levels_played >= ROGUELITE_MAX_LEVEL;
            let run_ended = game_outcome.is_defeat() || full_clear;
            if !run_ended {
                return;
            }

            let stats = RunAggregateStats::from_level_stats(&run.level_stats);
            let cleared_levels = if game_outcome.is_defeat() {
                (levels_played as i32 - 1).max(0)
            } else {
                levels_played as i32
            };

            let pure = is_pure_run(modifiers.as_deref(), toggles.as_deref());
            let base_score = cleared_levels.saturating_mul(100_000)
                + stats.total_kills as i32
                + (stats.avg_efficiency * 10_000.0) as i32
                - (stats.total_time.min(9_999.0) as i32);
            let final_score = if pure {
                base_score
            } else {
                let mult = modifier_multiplier(modifiers.as_deref(), toggles.as_deref());
                (base_score as f32 * mult) as i32
            };

            let details = build_roguelite_details(
                modifiers.as_deref(),
                toggles.as_deref(),
                config.wizard_type,
                cleared_levels,
                &stats,
            );

            let score_board = if pure {
                LeaderboardId::RogueliteScoreDefault
            } else {
                LeaderboardId::RogueliteScoreModified
            };
            submit(score_board, final_score, &details);

            if full_clear {
                let time_board = if pure {
                    LeaderboardId::RogueliteClearTimeDefault
                } else {
                    LeaderboardId::RogueliteClearTimeModified
                };
                submit(time_board, stats.total_time as i32, &details);
            }
        }
        None => {}
    }
}

/// A "pure" run has every roguelite slider at 100% and zero active toggles.
/// These are eligible for the fair-competition `*_default` leaderboards.
fn is_pure_run(mods: Option<&RogueliteModifiers>, toggles: Option<&ActiveToggles>) -> bool {
    mods.is_some_and(|m| m.is_default()) && toggles.is_none_or(|t| t.count() == 0)
}

fn modifier_multiplier(mods: Option<&RogueliteModifiers>, toggles: Option<&ActiveToggles>) -> f32 {
    let Some(m) = mods else {
        return 1.0;
    };
    let toggle_count = toggles.map_or(0, |t| t.count());
    let terrain_mult = m.terrain_density.max(0.5);
    m.game_speed
        * m.enemy_effectiveness
        * m.enemy_count
        * terrain_mult
        * (1.0 + 0.1 * toggle_count as f32)
}

fn build_roguelite_details(
    mods: Option<&RogueliteModifiers>,
    toggles: Option<&ActiveToggles>,
    wizard_type: WizardType,
    cleared_levels: i32,
    stats: &RunAggregateStats,
) -> Vec<i32> {
    let (gs, ee, ec, td) = mods.map_or((100, 100, 100, 100), |m| {
        (
            (m.game_speed * 100.0) as i32,
            (m.enemy_effectiveness * 100.0) as i32,
            (m.enemy_count * 100.0) as i32,
            (m.terrain_density * 100.0) as i32,
        )
    });
    let bitmask = toggles.map_or(0, toggle_bitmask);
    vec![
        gs,
        ee,
        ec,
        td,
        bitmask,
        wizard_ordinal(wizard_type),
        cleared_levels,
        stats.total_kills as i32,
        (stats.avg_efficiency * 10_000.0) as i32,
    ]
}

fn toggle_bitmask(toggles: &ActiveToggles) -> i32 {
    ToggleModifier::all()
        .iter()
        .enumerate()
        .fold(0, |acc, (i, &t)| {
            if toggles.is_active(t) {
                acc | (1 << i)
            } else {
                acc
            }
        })
}

/// Stable ordinal for a wizard type, used for leaderboard details encoding.
///
/// Backed by `WizardType::all()` ordering — append-only; never reorder that list
/// or historical leaderboard details will be misinterpreted.
fn wizard_ordinal(w: WizardType) -> i32 {
    match WizardType::all().iter().position(|&wt| wt == w) {
        Some(idx) => idx as i32,
        None => {
            warn!(
                "wizard_ordinal: {w:?} missing from WizardType::all() — leaderboard detail will be -1"
            );
            -1
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pure_run_requires_default_modifiers_and_no_toggles() {
        let default_mods = RogueliteModifiers::default();
        assert!(is_pure_run(Some(&default_mods), None));
        assert!(is_pure_run(
            Some(&default_mods),
            Some(&ActiveToggles::default())
        ));

        let bumped = RogueliteModifiers {
            game_speed: 1.5,
            ..Default::default()
        };
        assert!(!is_pure_run(Some(&bumped), None));

        let with_toggle = ActiveToggles::new(vec![ToggleModifier::GlassCannon]);
        assert!(!is_pure_run(Some(&default_mods), Some(&with_toggle)));
    }

    #[test]
    fn modifier_multiplier_is_one_for_defaults() {
        let default_mods = RogueliteModifiers::default();
        let mult = modifier_multiplier(Some(&default_mods), None);
        assert!((mult - 1.0).abs() < 0.001);
    }

    #[test]
    fn toggle_bitmask_encodes_active_slots() {
        let toggles = ActiveToggles::new(vec![
            ToggleModifier::ManaDrought,
            ToggleModifier::BossParade,
        ]);
        let mask = toggle_bitmask(&toggles);
        let mana_idx = ToggleModifier::all()
            .iter()
            .position(|&t| t == ToggleModifier::ManaDrought)
            .unwrap();
        let boss_idx = ToggleModifier::all()
            .iter()
            .position(|&t| t == ToggleModifier::BossParade)
            .unwrap();
        assert_eq!(mask, (1 << mana_idx) | (1 << boss_idx));
    }
}
