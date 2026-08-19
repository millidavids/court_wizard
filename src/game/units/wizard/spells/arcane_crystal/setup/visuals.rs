use bevy::prelude::*;

use super::super::auto::crystal_aoe_burst;
use super::super::components::*;
use super::super::constants::*;
use super::helpers::destroy_crystal;
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

/// Recolors a crystal's emissive to match what it is currently infused with, so
/// the player can read a crystal's state off the battlefield rather than
/// remembering what they last threw at it.
pub(crate) fn update_crystal_tint(
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut crystals: Query<(
        &ArcaneCrystal,
        &mut MeshMaterial3d<StandardMaterial>,
        &mut CrystalTint,
    )>,
) {
    for (crystal, mut material, mut tint) in &mut crystals {
        if tint.shown == crystal.infusion {
            continue;
        }
        let emissive = crystal
            .infusion
            .map_or(CRYSTAL_DEFAULT_EMISSIVE, |infusion| infusion.color());

        if tint.owns_material {
            let Some(mut existing) = materials.get_mut(&material.0) else {
                continue;
            };
            existing.emissive = emissive;
        } else {
            // First tint: clone off the shared handle before touching it.
            let Some(base) = materials.get(&material.0) else {
                continue;
            };
            let mut owned = base.clone();
            owned.emissive = emissive;
            material.0 = materials.add(owned);
            tint.owns_material = true;
        }
        tint.shown = crystal.infusion;
    }
}

/// Despawns expired crystals and their range indicators.
/// Triggers Prismatic Explosion if the talent is active.
#[allow(clippy::too_many_arguments)]
pub(crate) fn cleanup_expired_crystals(
    mut commands: Commands,
    crystals: Query<(Entity, &ArcaneCrystal, Has<PrismaticExplosion>)>,
    indicators: Query<(Entity, &CrystalRangeIndicator)>,
    targets: Query<
        (Entity, &Transform),
        (
            With<Health>,
            Without<Corpse>,
            Without<crate::game::pathfinding::StagingAttacker>,
        ),
    >,
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

            destroy_crystal(&mut commands, crystal_entity, crystal, &indicators);
        }
    }

    // Clean up orphaned indicators (e.g. crystal dispelled by enemy)
    for (indicator_entity, indicator) in &indicators {
        if crystals.get(indicator.crystal_entity).is_err() {
            commands.entity(indicator_entity).try_despawn();
        }
    }
}

/// Despawns infusion-spawned entities whose crystal is gone.
///
/// Backstop for destruction routes with no crystal-specific code to hook —
/// principally Dispel, which removes the crystal through the shared
/// `NetworkedSpellEffect` path. Without this, a dispelled Grease crystal would
/// leave its slick patches on the field for the rest of the level.
pub(crate) fn cleanup_orphaned_infusion_spawns(
    mut commands: Commands,
    owned: Query<(Entity, &CrystalOwned)>,
    crystals: Query<(), With<ArcaneCrystal>>,
) {
    for (entity, owner) in &owned {
        if crystals.get(owner.crystal).is_err() {
            commands.entity(entity).try_despawn();
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
