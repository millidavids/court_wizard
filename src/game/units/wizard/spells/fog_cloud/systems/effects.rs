use super::super::components::{
    BlindingMistDebuff, BlindingMistZone, ChokingFogZone, FogCloudZone, RollingFogZone,
};
use super::super::constants;
use crate::game::multiplayer::components::{GhostEntity, GhostSpellEffect};
use crate::game::units::components::{Corpse, FogEvasionModifier, Health, Team};
use crate::game::units::wizard::components::Wizard;
use crate::game::units::wizard::spells::utils::xz_distance;
use crate::game::units::wizard::spells::vfx;
use crate::game::units::wizard::spells::visual_assets::SpellVisualAssets;
use bevy::prelude::*;

#[allow(clippy::type_complexity)]
pub fn apply_fog_cloud_evasion(
    mut commands: Commands,
    time: Res<Time>,
    mut zones: Query<&mut FogCloudZone, Without<GhostSpellEffect>>,
    mut targets: Query<
        (Entity, &Transform, Option<&mut FogEvasionModifier>),
        (Without<Corpse>, Without<GhostEntity>, Without<Wizard>),
    >,
) {
    let delta = time.delta_secs();
    for mut zone in &mut zones {
        zone.time_alive += delta;
        zone.time_since_last_tick += delta;
        if zone.time_since_last_tick >= zone.tick_interval {
            zone.time_since_last_tick = 0.0;
            for (entity, transform, existing_evasion) in &mut targets {
                let dist = xz_distance(zone.origin, transform.translation);
                if dist <= zone.radius {
                    if let Some(mut evasion) = existing_evasion {
                        evasion.refresh(zone.evasion_refresh_duration);
                    } else {
                        commands.entity(entity).insert(FogEvasionModifier::new(
                            zone.evasion_chance,
                            zone.evasion_refresh_duration,
                        ));
                    }
                }
            }
        }
    }
}

/// Tier 2: Blinding Mist — apply/refresh debuff on units inside fog zones with this talent.
#[allow(clippy::type_complexity)]
pub fn apply_blinding_mist(
    mut commands: Commands,
    zones: Query<(&FogCloudZone, &BlindingMistZone), Without<GhostSpellEffect>>,
    mut targets: Query<
        (Entity, &Transform, Option<&mut BlindingMistDebuff>),
        (Without<Corpse>, Without<GhostEntity>, Without<Wizard>),
    >,
) {
    for (zone, _) in &zones {
        for (entity, transform, existing_debuff) in &mut targets {
            if xz_distance(zone.origin, transform.translation) <= zone.radius {
                if let Some(mut debuff) = existing_debuff {
                    debuff.refresh();
                } else {
                    commands
                        .entity(entity)
                        .insert(BlindingMistDebuff::new(constants::BLINDING_MIST_RANGE_MULT));
                }
            }
        }
    }
}

/// Tick down BlindingMistDebuff timers and remove expired ones.
pub fn tick_blinding_mist_debuff(
    mut commands: Commands,
    time: Res<Time>,
    mut debuffs: Query<(Entity, &mut BlindingMistDebuff), Without<GhostEntity>>,
) {
    let delta = time.delta_secs();
    for (entity, mut debuff) in &mut debuffs {
        debuff.time_remaining -= delta;
        if debuff.time_remaining <= 0.0 {
            commands.entity(entity).remove::<BlindingMistDebuff>();
        }
    }
}

/// Tier 3: Choking Fog — deal minor DPS to non-ally units inside the fog.
#[allow(clippy::type_complexity)]
pub fn apply_choking_fog_damage(
    time: Res<Time>,
    mut zones: Query<(&FogCloudZone, &mut ChokingFogZone), Without<GhostSpellEffect>>,
    mut targets: Query<
        (&Transform, &Team, &mut Health),
        (Without<Corpse>, Without<GhostEntity>, Without<Wizard>),
    >,
) {
    // Multiplayer setup stage: units are immune to damage.
    if crate::game::units::components::is_setup_immune() {
        return;
    }
    let delta = time.delta_secs();
    for (zone, mut choking) in &mut zones {
        choking.tick_accumulator += delta;
        if choking.tick_accumulator >= choking.tick_interval {
            choking.tick_accumulator -= choking.tick_interval;
            let damage = choking.dps * choking.tick_interval;
            for (transform, _team, mut health) in &mut targets {
                let dist = xz_distance(zone.origin, transform.translation);
                if dist <= zone.radius {
                    health.current = (health.current - damage).max(0.0);
                }
            }
        }
    }
}

/// Tier 3: Rolling Fog — move the fog zone toward the nearest attacker approach direction.
#[allow(clippy::type_complexity)]
pub fn move_rolling_fog(
    time: Res<Time>,
    mut zones: Query<
        (&mut FogCloudZone, &mut Transform, &RollingFogZone),
        Without<GhostSpellEffect>,
    >,
    units: Query<(&Transform, &Team), (Without<Corpse>, Without<FogCloudZone>)>,
) {
    let delta = time.delta_secs();

    // Collect attacker positions
    let attacker_positions: Vec<Vec3> = units
        .iter()
        .filter(|(_, team)| **team == Team::Attackers)
        .map(|(t, _)| t.translation)
        .collect();

    for (mut zone, mut zone_transform, rolling) in &mut zones {
        // Find nearest attacker to move toward
        let mut nearest_dist = f32::MAX;
        let mut nearest_dir = Vec3::ZERO;

        for &attacker_pos in &attacker_positions {
            let diff = Vec3::new(
                attacker_pos.x - zone.origin.x,
                0.0,
                attacker_pos.z - zone.origin.z,
            );
            let dist = diff.length();
            if dist < nearest_dist && dist > 1.0 {
                nearest_dist = dist;
                nearest_dir = diff / dist;
            }
        }

        if nearest_dist < f32::MAX {
            let movement = nearest_dir * rolling.speed * delta;
            zone.origin.x += movement.x;
            zone.origin.z += movement.z;
            zone_transform.translation.x = zone.origin.x;
            zone_transform.translation.z = zone.origin.z;
        }
    }
}

/// Continuously spawns gray smoke particles from active fog cloud zones.
pub fn emit_fog_cloud_particles(
    time: Res<Time>,
    mut commands: Commands,
    mut zones: Query<&mut FogCloudZone>,
    assets: Res<SpellVisualAssets>,
) {
    let dt = time.delta_secs();
    let t = time.elapsed_secs();

    for mut zone in &mut zones {
        // Don't emit particles during fade-out
        let remaining = zone.duration - zone.time_alive;
        if remaining < constants::FADE_DURATION {
            continue;
        }

        zone.smoke_spawn_timer += dt;
        if zone.smoke_spawn_timer >= vfx::constants::FOG_SMOKE_SPAWN_INTERVAL {
            zone.smoke_spawn_timer -= vfx::constants::FOG_SMOKE_SPAWN_INTERVAL;

            vfx::systems::spawn_fog_smoke_puffs(
                &mut commands,
                &assets,
                zone.origin,
                zone.radius,
                vfx::constants::FOG_SMOKE_COUNT_PER_SPAWN,
                t,
            );
        }
    }
}

pub fn cleanup_fog_cloud_zone(
    mut commands: Commands,
    zones: Query<(Entity, &FogCloudZone), Without<GhostSpellEffect>>,
) {
    for (entity, zone) in &zones {
        if zone.time_alive >= zone.duration {
            commands.entity(entity).try_despawn();
        }
    }
}
