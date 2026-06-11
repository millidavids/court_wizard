use bevy::prelude::*;

use super::super::auto::crystal_aoe_burst;
use super::super::components::*;
use super::super::constants::*;
use crate::game::units::components::{Corpse, Health, Team, TemporaryHitPoints};
use crate::game::units::king::components::SpellShield;
use crate::game::units::wizard::spells::disintegrate::components::{
    BeamEclipse, BeamGlow, BeamOriginFlare, DisintegrateBeam,
};
use crate::game::units::wizard::spells::utils::local_player_team;
use crate::game::units::wizard::spells::visual_assets::SpellVisualAssets;
use crate::networking::session::MultiplayerSession;

// ===== Crystal Visuals & Lifetime =====

/// Updates crystal rotation, pulse animation, and lifetime.
pub(crate) fn update_crystal_visuals(
    time: Res<Time>,
    mut crystals: Query<(&mut ArcaneCrystal, &mut Transform)>,
) {
    let delta = time.delta_secs();

    for (mut crystal, mut transform) in &mut crystals {
        if !crystal.permanent {
            crystal.time_alive += delta;
        }

        let sphere_radius = CRYSTAL_HEIGHT * crystal.empowerment / 3.0;

        // Rotation
        transform.rotate_y(ROTATION_SPEED * delta);

        // Pulse animation
        if crystal.pulse_timer > 0.0 {
            crystal.pulse_timer -= delta;
            let pulse_progress = crystal.pulse_timer / PULSE_DURATION;
            let scale_factor = 1.0 + (PULSE_SCALE - 1.0) * pulse_progress;
            transform.scale = Vec3::new(
                0.7 * sphere_radius * scale_factor,
                1.5 * sphere_radius * scale_factor,
                0.7 * sphere_radius * scale_factor,
            );
        } else {
            transform.scale = Vec3::new(
                0.7 * sphere_radius,
                1.5 * sphere_radius,
                0.7 * sphere_radius,
            );
        }
    }
}

/// Despawns expired crystals and their range indicators.
/// Triggers Prismatic Explosion if the talent is active.
#[allow(clippy::too_many_arguments)]
pub(crate) fn cleanup_expired_crystals(
    mut commands: Commands,
    crystals: Query<(Entity, &ArcaneCrystal, Has<PrismaticExplosion>)>,
    indicators: Query<(Entity, &CrystalRangeIndicator)>,
    targets: Query<(Entity, &Transform), (With<Health>, Without<Corpse>)>,
    mut health_query: Query<(
        &mut Health,
        Option<&mut TemporaryHitPoints>,
        Has<SpellShield>,
        &Team,
    )>,
    visual_assets: Res<SpellVisualAssets>,
    session: Option<Res<MultiplayerSession>>,
) {
    let caster_team = local_player_team(session.as_deref());
    for (crystal_entity, crystal, has_prismatic) in &crystals {
        if crystal.permanent {
            continue;
        }
        if crystal.time_alive >= crystal.duration {
            // Prismatic Explosion: deal massive damage on expiry
            if has_prismatic {
                crystal_aoe_burst(
                    &mut commands,
                    &visual_assets,
                    crystal.position,
                    crystal.range,
                    PRISMATIC_EXPLOSION_DAMAGE * crystal.empowerment,
                    PRISMATIC_EXPLOSION_RADIUS,
                    3.0,
                    0.5,
                    &targets,
                    &mut health_query,
                    caster_team,
                );
            }

            // Despawn active persistent beams
            for (beam_entities, _) in &crystal.active_beams {
                for beam_entity in beam_entities {
                    commands.entity(*beam_entity).try_despawn();
                }
            }

            // Despawn auto-disintegrate beam if present
            if let Some((beam_entities, _)) = &crystal.auto_disintegrate_beam {
                for beam_entity in beam_entities {
                    commands.entity(*beam_entity).try_despawn();
                }
            }

            commands.entity(crystal_entity).try_despawn();

            // Despawn associated range indicator
            for (indicator_entity, indicator) in &indicators {
                if indicator.crystal_entity == crystal_entity {
                    commands.entity(indicator_entity).try_despawn();
                }
            }
        }
    }

    // Clean up orphaned indicators (e.g. crystal dispelled by enemy)
    for (indicator_entity, indicator) in &indicators {
        if crystals.get(indicator.crystal_entity).is_err() {
            commands.entity(indicator_entity).try_despawn();
        }
    }
}

// ===== Black Hole Interaction =====

/// Pulls crystals toward black holes and despawns them if they enter the sphere.
pub(crate) fn crystal_black_hole_interaction(
    mut commands: Commands,
    time: Res<Time>,
    black_holes: Query<&crate::game::units::wizard::spells::black_hole::components::BlackHole>,
    mut crystals: Query<(Entity, &mut ArcaneCrystal, &mut Transform)>,
    indicators: Query<(Entity, &CrystalRangeIndicator)>,
) {
    use crate::game::units::wizard::spells::black_hole::constants::GRAVITY_RANGE;

    let delta = time.delta_secs();

    for (crystal_entity, mut crystal, mut transform) in &mut crystals {
        for black_hole in &black_holes {
            let to_bh = black_hole.position - crystal.position;
            let distance = to_bh.length();

            // Check if crystal is inside the black hole sphere
            if black_hole.contains_point(crystal.position) {
                commands.entity(crystal_entity).try_despawn();
                for (indicator_entity, indicator) in &indicators {
                    if indicator.crystal_entity == crystal_entity {
                        commands.entity(indicator_entity).try_despawn();
                    }
                }
                break;
            }

            // Apply gravitational pull
            if distance > 0.01 && distance <= GRAVITY_RANGE {
                let gravity_strength = black_hole.gravitational_strength();
                let distance_factor = 1.0 / (distance * distance);
                let pull_strength = (gravity_strength * distance_factor).min(2500.0);
                let direction = to_bh.normalize();

                let displacement = direction * pull_strength * delta * 0.01; // Damped movement
                crystal.position += displacement;
                transform.translation = crystal.position;
            }
        }
    }
}

// ===== Range-Limited Despawn =====

/// Despawns crystal-spawned entities that exceed their max range from origin.
pub(crate) fn despawn_out_of_range_crystal_spawns(
    mut commands: Commands,
    spawns: Query<(Entity, &Transform, &CrystalSpawn)>,
) {
    use crate::game::units::wizard::spells::utils::xz_distance;
    for (entity, transform, crystal_spawn) in &spawns {
        if xz_distance(crystal_spawn.origin, transform.translation) > crystal_spawn.max_range {
            commands.entity(entity).try_despawn();
        }
    }
}

/// Despawns CrystalSpawn entities (non-beam) whose lifetime has expired.
/// This handles visual effects like resonance cascade burst rings and prismatic explosions.
pub(crate) fn cleanup_expired_crystal_visuals(
    time: Res<Time>,
    mut commands: Commands,
    mut spawns: Query<(Entity, &mut CrystalSpawn), Without<DisintegrateBeam>>,
) {
    let delta = time.delta_secs();
    for (entity, mut spawn) in &mut spawns {
        if let Some(ref mut lifetime) = spawn.lifetime {
            *lifetime -= delta;
            if *lifetime <= 0.0 {
                commands.entity(entity).try_despawn();
            }
        }
    }
}

/// Despawns crystal beams (and their visuals) that have exceeded their lifetime.
pub(crate) fn cleanup_expired_crystal_beams(
    mut commands: Commands,
    beams: Query<(Entity, &DisintegrateBeam, &CrystalSpawn)>,
    glow_query: Query<(Entity, &BeamGlow)>,
    flare_query: Query<(Entity, &BeamOriginFlare)>,
    eclipse_query: Query<(Entity, &BeamEclipse)>,
) {
    let mut despawned = Vec::new();
    for (entity, beam, crystal_spawn) in &beams {
        if let Some(lifetime) = crystal_spawn.lifetime
            && beam.time_alive > lifetime
        {
            commands.entity(entity).try_despawn();
            despawned.push(entity);
        }
    }

    if despawned.is_empty() {
        return;
    }

    for (entity, glow) in &glow_query {
        if despawned.contains(&glow.beam_entity) {
            commands.entity(entity).try_despawn();
        }
    }
    for (entity, flare) in &flare_query {
        if despawned.contains(&flare.beam_entity) {
            commands.entity(entity).try_despawn();
        }
    }
    for (entity, eclipse) in &eclipse_query {
        if despawned.contains(&eclipse.beam_entity) {
            commands.entity(entity).try_despawn();
        }
    }
}
