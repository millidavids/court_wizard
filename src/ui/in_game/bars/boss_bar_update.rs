use bevy::prelude::*;

use super::super::components::*;
use super::super::constants::*;
use crate::game::units::boss::components::Boss;
use crate::game::units::boss::hags::components::{Hag, HagIdentity, PermanentlyDead};
use crate::game::units::boss::lich::Lich;
use crate::game::units::boss::lich::components::{LichPhase, SoulPower};
use crate::game::units::components::{Corpse, Health};

pub(crate) fn update_ray_eye_health_bar(
    ray_eyes: Query<
        (&crate::game::units::boss::ray::RayEye, &Health),
        Without<crate::game::units::boss::ray::RayEyeDying>,
    >,
    mut fill_query: Query<(&mut Node, &mut BackgroundColor, &RayEyeHealthBarFill)>,
    mut text_query: Query<(&mut Text, &RayEyeHealthBarText)>,
    mut last_pct: Local<[i16; 5]>,
) {
    let mut health_by_eye: [Option<(f32, f32)>; 5] = [None; 5];
    for (eye, health) in ray_eyes.iter() {
        health_by_eye[eye.eye_type.index()] = Some((health.current, health.max));
    }

    let pct_int: [i16; 5] = std::array::from_fn(|i| match health_by_eye[i] {
        Some((current, max)) => ((current / max).clamp(0.0, 1.0) * 100.0) as i16,
        None => -1,
    });

    for (mut node, mut bg, marker) in fill_query.iter_mut() {
        let idx = marker.eye_type.index();
        if pct_int[idx] == last_pct[idx] {
            continue;
        }
        if pct_int[idx] < 0 {
            node.width = Val::Percent(0.0);
        } else {
            node.width = Val::Percent(pct_int[idx] as f32);
            if bg.0.alpha() < 0.5 {
                bg.0 = bg.0.with_alpha(1.0);
            }
        }
    }

    for (mut text, marker) in text_query.iter_mut() {
        let idx = marker.eye_type.index();
        if pct_int[idx] == last_pct[idx] {
            continue;
        }
        text.0 = format!("{}%", pct_int[idx].max(0));
    }

    *last_pct = pct_int;
}

/// Updates the boss health bar fill and text. Despawns the bar when the boss dies.
#[allow(clippy::too_many_arguments)]
pub(crate) fn update_boss_health_bar(
    mut commands: Commands,
    boss_query: Query<&Health, (With<Boss>, Without<Corpse>)>,
    hag_query: Query<
        (&HagIdentity, &Health),
        (With<Hag>, Without<Corpse>, Without<PermanentlyDead>),
    >,
    lich_query: Query<(&Health, &SoulPower, &LichPhase), (With<Lich>, Without<Corpse>)>,
    bar_query: Query<Entity, With<BossHealthBarRoot>>,
    mut fill_query: Query<&mut Node, (With<BossHealthBarFill>, Without<HagHealthBarFill>)>,
    mut text_query: Query<
        &mut Text,
        (
            With<BossHealthBarText>,
            Without<HagHealthBarText>,
            Without<LichBarLabel>,
        ),
    >,
    mut hag_fill_query: Query<
        (&mut Node, &mut BackgroundColor, &HagHealthBarFill),
        Without<BossHealthBarFill>,
    >,
    mut hag_text_query: Query<
        (&mut Text, &HagHealthBarText),
        (Without<BossHealthBarText>, Without<LichBarLabel>),
    >,
    mut label_query: Query<
        &mut Text,
        (
            With<LichBarLabel>,
            Without<BossHealthBarText>,
            Without<HagHealthBarText>,
        ),
    >,
    mut fill_bg_query: Query<
        &mut BackgroundColor,
        (With<BossHealthBarFill>, Without<HagHealthBarFill>),
    >,
) {
    // Build a fixed-size lookup of living hag health (no heap allocation)
    let mut living = [None::<f32>; 3]; // indexed: Justina=0, Martina=1, Josephina=2
    let mut any_hag = false;
    for (identity, health) in &hag_query {
        any_hag = true;
        let idx = match identity {
            HagIdentity::Justina => 0,
            HagIdentity::Martina => 1,
            HagIdentity::Josephina => 2,
        };
        living[idx] = Some((health.current / health.max * 100.0).clamp(0.0, 100.0));
    }

    if any_hag {
        // Single pass over UI elements — update or dim based on living lookup
        for (mut node, mut bg, fill) in &mut hag_fill_query {
            let idx = match fill.identity {
                HagIdentity::Justina => 0,
                HagIdentity::Martina => 1,
                HagIdentity::Josephina => 2,
            };
            if let Some(hp_percent) = living[idx] {
                node.width = Val::Percent(hp_percent);
            } else {
                node.width = Val::Percent(0.0);
                bg.0 = HAG_BAR_DEAD_COLOR;
            }
        }
        for (mut text, text_marker) in &mut hag_text_query {
            let idx = match text_marker.identity {
                HagIdentity::Justina => 0,
                HagIdentity::Martina => 1,
                HagIdentity::Josephina => 2,
            };
            if let Some(hp_percent) = living[idx] {
                **text = format!("{:.0}%", hp_percent);
            } else {
                **text = "Dead".to_string();
            }
        }

        // Remove bar when all hags are dead
        if living.iter().all(|h| h.is_none()) {
            for entity in &bar_query {
                commands.entity(entity).try_despawn();
            }
        }
    } else if let Ok((health, soul_power, phase)) = lich_query.single() {
        // Lich: show soul power in Phase 1, HP in Phase 2
        match phase {
            LichPhase::Approaching | LichPhase::Summoning => {
                // Soul power bar (filling from 0 to 100%)
                let percent = soul_power.percent();
                if let Ok(mut node) = fill_query.single_mut() {
                    node.width = Val::Percent(percent);
                }
                if let Ok(mut text) = text_query.single_mut() {
                    **text = format!("{:.0}%", percent);
                }
                if let Ok(mut label) = label_query.single_mut() {
                    **label = "Soul Power".to_string();
                }
            }
            LichPhase::Combat => {
                // Switch to HP display
                let hp_percent = (health.current / health.max * 100.0).clamp(0.0, 100.0);
                if let Ok(mut node) = fill_query.single_mut() {
                    node.width = Val::Percent(hp_percent);
                }
                if let Ok(mut text) = text_query.single_mut() {
                    **text = format!("{:.0}%", hp_percent);
                }
                // Update label and bar color to HP mode
                if let Ok(mut label) = label_query.single_mut() {
                    **label = "Health".to_string();
                }
                if let Ok(mut bg) = fill_bg_query.single_mut() {
                    bg.0 = BOSS_HEALTH_BAR_FILL_COLOR;
                }
            }
        }
    } else if let Some(health) = boss_query.iter().next() {
        // Original ogre update
        let hp_percent = (health.current / health.max * 100.0).clamp(0.0, 100.0);
        if let Ok(mut node) = fill_query.single_mut() {
            node.width = Val::Percent(hp_percent);
        }
        if let Ok(mut text) = text_query.single_mut() {
            **text = format!("{:.0}%", hp_percent);
        }
    } else {
        // Boss is dead or doesn't exist — remove the bar
        for entity in &bar_query {
            commands.entity(entity).try_despawn();
        }
    }
}
