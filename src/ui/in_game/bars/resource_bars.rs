use bevy::prelude::*;

use super::super::components::*;
use super::super::constants::*;
use crate::config::GameConfig;
use crate::game::cauldron::components::{Cauldron, CauldronState};
use crate::game::components::ConcentrationSpell;
use crate::game::resources::{CurrentLevel, KillStats};
use crate::game::units::components::{Corpse, Health, Team};
use crate::game::units::king::components::King;
use crate::game::units::wizard::archetypes::gunslinger::GunState;
use crate::game::units::wizard::components::{CastingState, LocalWizard, Mana, PrimedSpell};
use crate::game::units::wizard::spells::utils::local_player_team;
use crate::networking::session::MultiplayerSession;

/// Updates the mana bar and reserved-mana bar widths to reflect the wizard's
/// current mana and any mana reserved by active concentration spells.
pub(crate) fn update_mana_bar(
    wizard_query: Query<&Mana, With<LocalWizard>>,
    concentration_spells: Query<&ConcentrationSpell>,
    mut mana_bar_query: Query<&mut Node, With<ManaBarFill>>,
    mut reserved_bar_query: Query<&mut Node, (With<ManaBarReservedFill>, Without<ManaBarFill>)>,
) {
    if let Ok(mana) = wizard_query.single() {
        let reserved: f32 = concentration_spells.iter().map(|c| c.mana_cost).sum();
        let reserved_pct = (reserved / mana.max).min(1.0) * 100.0;
        let mana_pct = (mana.percentage() * 100.0).min(100.0 - reserved_pct);

        if let Ok(mut node) = mana_bar_query.single_mut() {
            node.width = Val::Percent(mana_pct);
        }
        if let Ok(mut node) = reserved_bar_query.single_mut() {
            node.width = Val::Percent(reserved_pct);
        }
    }
}

/// Updates the ammo display for the gunslinger archetype.
pub(crate) fn update_ammo_display(
    gun_state: Option<Res<GunState>>,
    mut ammo_pieces: Query<(&AmmoPiece, &mut BackgroundColor)>,
    mut counter_text: Query<&mut Text, With<AmmoCounterText>>,
) {
    let Some(gs) = gun_state else {
        return;
    };

    let gun = gs.selected_gun;
    let ammo = gs.current_ammo();
    let per_piece = gun.ammo_per_ui_piece();
    let max_pieces = ammo.max / per_piece;

    // Update counter text
    if let Ok(mut text) = counter_text.single_mut() {
        **text = format!("{} / {}", ammo.current, ammo.max);
    }

    // Update ammo piece colors
    let lit_color = Color::srgba(1.0, 0.8, 0.2, 0.9);
    let dim_color = Color::srgba(0.3, 0.3, 0.3, 0.4);
    let reload_color = Color::srgba(0.5, 0.7, 1.0, 0.7);

    for (piece, mut bg) in &mut ammo_pieces {
        if piece.index >= max_pieces {
            bg.0 = Color::NONE;
            continue;
        }

        let ammo_at_piece = (piece.index + 1) * per_piece;

        if ammo.reloading {
            // During reload, progressively light up pieces
            let reloaded_ammo = (ammo.reload_progress() * ammo.max as f32) as u32;
            bg.0 = if ammo_at_piece <= reloaded_ammo {
                reload_color
            } else {
                dim_color
            };
        } else {
            bg.0 = if ammo_at_piece <= ammo.current {
                lit_color
            } else {
                dim_color
            };
        }
    }
}

/// Updates the cast bar width based on current wizard casting progress, brewing progress,
/// or reload progress for the gunslinger.
pub(crate) fn update_cast_bar(
    // `Option<&PrimedSpell>` so the bar still works for a Randomancer that hasn't
    // spun its wheel yet (no primed spell) — the query would otherwise be empty.
    wizard_query: Query<(&CastingState, Option<&PrimedSpell>), With<LocalWizard>>,
    cauldron_query: Query<&CauldronState, With<Cauldron>>,
    gun_state: Option<Res<GunState>>,
    mut cast_bar_query: Query<(&mut Node, &mut BackgroundColor), With<CastBarFill>>,
    mut overlay_query: Query<&mut Visibility, With<BrewingOverlay>>,
) {
    let is_brewing = cauldron_query
        .single()
        .is_ok_and(|state| state.is_brewing());

    // Check if gunslinger is reloading
    let reload_progress = gun_state.as_ref().and_then(|gs| {
        let ammo = gs.current_ammo();
        if ammo.reloading {
            Some(ammo.reload_progress())
        } else {
            None
        }
    });

    if let Ok((mut node, mut bg_color)) = cast_bar_query.single_mut() {
        if let Some(progress) = reload_progress {
            node.width = Val::Percent(progress * 100.0);
            bg_color.0 = RELOAD_BAR_COLOR;
        } else if is_brewing {
            if let Ok(state) = cauldron_query.single() {
                let progress_percent = state.progress() * 100.0;
                node.width = Val::Percent(progress_percent);
            }
            bg_color.0 = CAST_BAR_BREWING_FILL_COLOR;
        } else {
            if let Ok((casting_state, primed_spell)) = wizard_query.single() {
                let progress_percent = primed_spell
                    .map(|p| casting_state.progress(p.cast_time) * 100.0)
                    .unwrap_or(0.0);
                node.width = Val::Percent(progress_percent);
            }
            bg_color.0 = CAST_BAR_FILL_COLOR;
        }
    }

    // Toggle brewing/reload overlay visibility and text
    if let Ok(mut visibility) = overlay_query.single_mut() {
        *visibility = if reload_progress.is_some() || is_brewing {
            Visibility::Visible
        } else {
            Visibility::Hidden
        };
    }
}

/// Updates the overlay text to show "Reloading..." or "Brewing..." as appropriate.
pub(crate) fn update_overlay_text(
    gun_state: Option<Res<GunState>>,
    cauldron_query: Query<&CauldronState, With<Cauldron>>,
    mut text_query: Query<&mut Text, With<BrewingOverlayText>>,
) {
    let is_reloading = gun_state
        .as_ref()
        .is_some_and(|gs| gs.current_ammo().reloading);
    let is_brewing = cauldron_query.single().is_ok_and(|s| s.is_brewing());

    if let Ok(mut text) = text_query.single_mut() {
        if is_reloading {
            **text = "Reloading...".to_string();
        } else if is_brewing {
            **text = "Brewing...".to_string();
        }
    }
}

/// Updates the level display text when the current level changes.
pub(crate) fn update_level_display(
    current_level: Res<CurrentLevel>,
    mut level_display_query: Query<&mut Text, With<LevelDisplay>>,
) {
    if current_level.is_changed()
        && let Ok(mut text) = level_display_query.single_mut()
    {
        **text = format!("Level: {}", current_level.0);
    }
}

/// Updates the past victory display text when the current level changes.
pub(crate) fn update_past_victory_display(
    current_level: Res<CurrentLevel>,
    config: Res<GameConfig>,
    mut past_victory_query: Query<&mut Text, With<PastVictoryDisplay>>,
) {
    if current_level.is_changed()
        && let Ok(mut text) = past_victory_query.single_mut()
    {
        if let Some(past_efficiency) = config.efficiency_ratios.get(&current_level.0.to_string()) {
            **text = format!("Best: {:.1}%", past_efficiency * 100.0);
        } else {
            **text = String::new();
        }
    }
}

/// Updates the level clock display text with elapsed time.
///
/// Only updates text when the displayed second changes to avoid per-frame allocations.
/// Only updates visibility when the config setting changes.
pub(crate) fn update_level_clock(
    kill_stats: Res<KillStats>,
    config: Res<GameConfig>,
    mut clock_query: Query<(&mut Text, &mut Visibility), With<LevelClockDisplay>>,
) {
    for (mut text, mut visibility) in &mut clock_query {
        if config.is_changed() {
            let target = if config.show_level_clock {
                Visibility::Inherited
            } else {
                Visibility::Hidden
            };
            *visibility = target;
        }
        if config.show_level_clock && kill_stats.is_changed() {
            let total_secs = kill_stats.elapsed_time as u32;
            let mins = total_secs / 60;
            let secs = total_secs % 60;
            let new_text = format!("{mins}:{secs:02}");
            if text.0 != new_text {
                text.0 = new_text;
            }
        }
    }
}

/// Updates the king health bar fill based on the local player's team's king health.
pub(crate) fn update_king_health_bar(
    king_query: Query<(&Health, &Team), (With<King>, Without<Corpse>)>,
    mut fill_query: Query<&mut Node, With<KingHealthBarFill>>,
    session: Option<Res<MultiplayerSession>>,
) {
    let Ok(mut fill_node) = fill_query.single_mut() else {
        return;
    };

    // The team this player commands: Defenders in single-player and as the
    // multiplayer host, Attackers as the guest. The multiplayer wizard has no
    // `Team` component (only the single-player wizard does), so the old
    // `Query<&Team, With<LocalWizard>>` never matched in MP and the bar stayed
    // empty. Derive the team from the peer role instead.
    let local_team = local_player_team(session.as_deref());

    // Find the king matching the local player's team
    for (health, team) in &king_query {
        if *team == local_team {
            let hp_percent = (health.current / health.max * 100.0).clamp(0.0, 100.0);
            fill_node.height = Val::Percent(hp_percent);
            return;
        }
    }

    // No matching king found (dead) — show empty bar
    fill_node.height = Val::Percent(0.0);
}
