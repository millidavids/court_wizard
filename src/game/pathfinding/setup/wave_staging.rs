//! Wave staging: tagging, activation checks, speedup management, and targeting suppression.

use bevy::prelude::*;

use crate::game::constants::{
    CENTER_STAGING_INDEX, STAGING_ACTIVATION_RADIUS, STAGING_POINTS, WAVE_ACTIVATION_THRESHOLD,
    WAVE_STAGING_TIMEOUT,
};
use crate::game::units::components::{Corpse, Team};
use crate::game::units::wizard::archetypes::swordcerer::components::SwordcererAvatar;

use super::super::components::{FlowFieldInfluence, StagingAttacker, WaveGroup};

/// Tracks how long each wave has been in staging, for timeout-based force activation.
#[derive(Resource, Default)]
pub struct WaveStagingTimers {
    timers: std::collections::HashMap<u32, f32>,
}

/// Auto-tags newly spawned attackers with `StagingAttacker` and `WaveGroup`.
/// Detects entities with `FlowFieldInfluence::Attacker` that don't yet have a `WaveGroup`.
/// Assigns staging points from the `WaveStagingPlan` via round-robin.
/// Bosses always go to the center staging point (index 3).
/// Lazily computes the staging plan for a wave the first time attackers appear for it.
#[allow(clippy::too_many_arguments)]
pub fn tag_new_attackers(
    mut commands: Commands,
    wave_state: Option<Res<crate::game::resources::WaveState>>,
    game_seed: Option<Res<crate::game::seeded_rng::resources::GameSeed>>,
    current_level: Res<crate::game::resources::CurrentLevel>,
    mut staging_plan: ResMut<super::super::staging::WaveStagingPlan>,
    new_attackers: Query<
        (
            Entity,
            &Transform,
            &Team,
            &FlowFieldInfluence,
            Has<crate::game::units::boss::components::Boss>,
        ),
        (Without<WaveGroup>, Without<Corpse>),
    >,
) {
    let wave = wave_state.map(|w| w.current_wave).unwrap_or(0);

    if !staging_plan.has_wave(wave) {
        let seed = game_seed.as_ref().map(|s| s.0).unwrap_or(0);
        super::super::staging::compute_wave_staging(&mut staging_plan, seed, current_level.0, wave);
    }

    // Tunnel 0 (upper, z ≈ -375) → Left, tunnel 1 (lower, z ≈ -1575) → Right.
    let tunnel_z_midpoint = (crate::game::constants::ATTACKER_SPAWN_POINTS[0].1
        + crate::game::constants::ATTACKER_SPAWN_POINTS[1].1)
        / 2.0;

    for (entity, transform, team, influence, is_boss) in &new_attackers {
        if *team != Team::Attackers {
            continue;
        }
        match influence {
            FlowFieldInfluence::Attacker | FlowFieldInfluence::Assassin => {
                let staging_idx = if is_boss {
                    CENTER_STAGING_INDEX as u8
                } else {
                    let tunnel = if transform.translation.z > tunnel_z_midpoint {
                        super::super::staging::SpawnTunnel::Left
                    } else {
                        super::super::staging::SpawnTunnel::Right
                    };
                    staging_plan.next_staging_point(wave, tunnel)
                };
                commands
                    .entity(entity)
                    .insert((StagingAttacker(staging_idx), WaveGroup(wave)));
            }
            _ => {}
        }
    }
}

/// Checks if 90% of a wave's living staging attackers have reached their
/// assigned staging points. Each unit is checked against its own staging point.
/// Also force-activates after a timeout to prevent stalling.
/// Dead units (Corpse) are excluded so lava kills don't block activation.
pub fn check_wave_activation(
    mut commands: Commands,
    time: Res<Time<Real>>,
    mut kill_stats: ResMut<crate::game::resources::KillStats>,
    mut staging_timers: ResMut<WaveStagingTimers>,
    mut staging_plan: ResMut<super::super::staging::WaveStagingPlan>,
    staging_query: Query<(Entity, &WaveGroup, &StagingAttacker, &Transform), Without<Corpse>>,
    swordcerer_avatars: Query<&Transform, (With<SwordcererAvatar>, Without<Corpse>)>,
) {
    use std::collections::HashMap;

    // No active staging → nothing to do. Skips work for the bulk of the battle
    // after every wave has activated.
    if staging_query.is_empty() {
        return;
    }

    let activation_radius_sq = STAGING_ACTIVATION_RADIUS * STAGING_ACTIVATION_RADIUS;
    // Aggro distance for the swordcerer avatar to wake a staging wave. Tighter
    // than the staging arrival radius — represents "the avatar walked into the
    // wave's space" rather than "near the staging point."
    const SWORDCERER_AGGRO_RADIUS: f32 = 300.0;
    let swordcerer_aggro_radius_sq = SWORDCERER_AGGRO_RADIUS * SWORDCERER_AGGRO_RADIUS;
    // Use real time so the 5x speedup doesn't shorten the timeout
    let dt = time.delta_secs();

    // At most one swordcerer avatar exists at a time.
    let avatar_pos: Option<Vec2> = swordcerer_avatars
        .single()
        .ok()
        .map(|t| Vec2::new(t.translation.x, t.translation.z));

    let mut wave_counts: HashMap<u32, (u32, u32)> = HashMap::new();
    let mut waves_aggroed_by_avatar: std::collections::HashSet<u32> =
        std::collections::HashSet::new();
    for (_entity, wave_group, staging, transform) in &staging_query {
        let (total, arrived) = wave_counts.entry(wave_group.0).or_insert((0, 0));
        *total += 1;
        let point = STAGING_POINTS[staging.0 as usize];
        let staging_pos = Vec2::new(point.0, point.1);
        let unit_pos = Vec2::new(transform.translation.x, transform.translation.z);
        if unit_pos.distance_squared(staging_pos) <= activation_radius_sq {
            *arrived += 1;
        }
        if let Some(ap) = avatar_pos
            && unit_pos.distance_squared(ap) <= swordcerer_aggro_radius_sq
        {
            waves_aggroed_by_avatar.insert(wave_group.0);
        }
    }

    // Check each wave
    for (wave, (total, arrived)) in &wave_counts {
        if *total == 0 {
            continue;
        }

        // Tick staging timer for this wave
        let elapsed = staging_timers.timers.entry(*wave).or_insert(0.0);
        *elapsed += dt;

        let ratio = *arrived as f32 / *total as f32;
        let timed_out = *elapsed >= WAVE_STAGING_TIMEOUT;
        let aggroed_by_swordcerer = waves_aggroed_by_avatar.contains(wave);

        if ratio >= WAVE_ACTIVATION_THRESHOLD || timed_out || aggroed_by_swordcerer {
            if aggroed_by_swordcerer {
                info!(
                    "Wave {} activated by swordcerer avatar proximity ({}/{} units staged)",
                    wave, arrived, total
                );
            } else if timed_out {
                info!(
                    "Wave {} force-activated after {:.1}s timeout ({}/{} units within radius)",
                    wave, elapsed, arrived, total
                );
            } else {
                info!(
                    "Wave {} activated! ({}/{} units within staging radius)",
                    wave, arrived, total
                );
            }
            // Mark battle as started (enables the game timer)
            if !kill_stats.battle_started {
                kill_stats.battle_started = true;
                info!("Battle started — game timer running");
            }
            // Remove StagingAttacker from all units of this wave
            for (entity, wave_group, _, _) in &staging_query {
                if wave_group.0 == *wave {
                    commands.entity(entity).remove::<StagingAttacker>();
                }
            }
            staging_timers.timers.remove(wave);
            staging_plan.remove_wave(*wave);
        }
    }
}

/// Manages game speed: `STAGING_SPEEDUP` (5x) when only staging (unactivated)
/// attackers exist, baseline otherwise.
///
/// Drops the speedup to baseline whenever a full-screen in-game menu (spell
/// book / cauldron) is open — menu navigation and scrolling behave erratically
/// at 5x. When the player returns to `Running` with staging still active the
/// speedup resumes automatically on the next frame.
#[allow(clippy::too_many_arguments)]
pub fn manage_staging_speedup(
    mut time: ResMut<Time<Virtual>>,
    staging_query: Query<(), (With<StagingAttacker>, Without<Corpse>)>,
    activated_attackers: Query<&Team, (With<WaveGroup>, Without<StagingAttacker>, Without<Corpse>)>,
    wave_state: Option<Res<crate::game::resources::WaveState>>,
    config: Res<crate::config::GameConfig>,
    sp_state: Option<Res<State<crate::state::InGameState>>>,
    mp_state: Option<Res<State<crate::state::MultiplayerGameState>>>,
) {
    // Multiplayer has no staging phase — armies are spawned in full at match
    // start. WaveState still exists as a global resource (default
    // `waves_complete: false`) which would otherwise leave `waves_remaining`
    // true forever in MP, locking the game at 5x. Force baseline game speed
    // whenever the player is in an *active* MP state — `Disconnected` and
    // `ScoreScreen` are deliberately left alone so a future slow-motion
    // replay or end-of-match effect can override speed without this system
    // stamping it back to baseline every frame.
    if mp_state.as_ref().is_some_and(|s| {
        matches!(
            *s.get(),
            crate::state::MultiplayerGameState::Running
                | crate::state::MultiplayerGameState::Paused
                | crate::state::MultiplayerGameState::SpellBook
                | crate::state::MultiplayerGameState::CauldronMenu
        )
    }) {
        let base_speed = config.game_speed as f64;
        if (time.relative_speed_f64() - base_speed).abs() > 0.01 {
            time.set_relative_speed_f64(base_speed);
        }
        return;
    }

    let has_staging = !staging_query.is_empty();

    let has_activated = activated_attackers
        .iter()
        .any(|team| *team == Team::Attackers);

    let waves_remaining = wave_state.map(|w| !w.waves_complete).unwrap_or(false);

    let speed_eligible = !has_activated && (has_staging || waves_remaining);

    // Any in-game menu overlay — SpellBook, CauldronMenu, Paused, etc. —
    // suppresses the speedup. Only `Running` (SP) or `Running` (MP) keeps it on.
    let in_menu_overlay = sp_state
        .as_ref()
        .is_some_and(|s| !matches!(*s.get(), crate::state::InGameState::Running))
        || mp_state
            .as_ref()
            .is_some_and(|s| !matches!(*s.get(), crate::state::MultiplayerGameState::Running));

    let should_speedup = speed_eligible && !in_menu_overlay;

    let current_speed = time.relative_speed_f64();
    let base_speed = config.game_speed as f64;
    let target_speed = if should_speedup {
        crate::game::constants::STAGING_SPEEDUP * base_speed
    } else {
        base_speed
    };

    if (current_speed - target_speed).abs() > 0.01 {
        time.set_relative_speed_f64(target_speed);
    }
}

/// Zeroes out targeting velocity for staging attackers so they only follow
/// the staging flow field. Without this, targeting systems point them at
/// enemies and the movement weighting lets targeting override the flow field.
pub fn suppress_staging_targeting(
    mut query: Query<&mut crate::game::units::components::TargetingVelocity, With<StagingAttacker>>,
) {
    for mut targeting in &mut query {
        if targeting.velocity != Vec3::ZERO || targeting.distance_to_target != f32::MAX {
            targeting.velocity = Vec3::ZERO;
            targeting.distance_to_target = f32::MAX;
        }
    }
}
