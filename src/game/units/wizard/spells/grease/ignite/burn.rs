use super::super::super::super::components::Spell;
use super::super::casting::write_grease_obstacle;
use super::super::components::{GreaseIgnited, GreaseRegenerating, GreaseZone};
use super::super::constants;
use crate::game::pathfinding::{ObstacleChanged, ObstacleType};
use crate::game::units::DamageType;
use crate::game::units::components::{
    Airborne, Corpse, Health, RootedModifier, Team, TemporaryHitPoints,
    apply_spell_damage_with_team,
};
use crate::game::units::king::components::SpellShield;
use crate::game::units::wizard::spells::disintegrate::components::DisintegrateBeam;
use crate::game::units::wizard::spells::fireball::components::FireballExplosion;
use crate::game::units::wizard::spells::meteor_fall::components::MeteorGroundFire;
use crate::game::units::wizard::spells::utils::local_player_team;
use crate::game::units::wizard::spells::utils::xz_distance;
use crate::game::units::wizard::spells::wall_of_fire::components::WallOfFireEffect;
use crate::game::units::wizard::talents::resources::BattleTalentProgress;
use crate::networking::session::MultiplayerSession;
use bevy::prelude::*;

/// Scans fire sources and ignites non-ignited grease zones on contact, applying burst damage and talent effects.
#[allow(clippy::too_many_arguments)]
pub fn check_grease_ignition(
    mut commands: Commands,
    mut zones: Query<
        (Entity, &mut GreaseZone),
        (
            Without<GreaseIgnited>,
            Without<GreaseRegenerating>,
            Without<crate::game::multiplayer::components::GhostSpellEffect>,
        ),
    >,
    ignited_zone_query: Query<&GreaseZone, With<GreaseIgnited>>,
    fire_units: Query<
        &Transform,
        (
            With<crate::game::units::components::FireDoT>,
            Without<Corpse>,
        ),
    >,
    fireball_explosions: Query<&FireballExplosion>,
    wall_of_fires: Query<&WallOfFireEffect>,
    meteor_ground_fires: Query<&MeteorGroundFire>,
    disintegrate_beams: Query<&DisintegrateBeam>,
    #[allow(clippy::type_complexity)] mut targets: Query<
        (
            Entity,
            &Transform,
            &mut Health,
            Option<&mut TemporaryHitPoints>,
            Has<SpellShield>,
            &Team,
        ),
        (
            Without<Corpse>,
            Without<crate::game::multiplayer::components::GhostEntity>,
        ),
    >,
    mut obstacle_events: MessageWriter<ObstacleChanged>,
    mut talent_progress: Option<ResMut<BattleTalentProgress>>,
    session: Option<Res<MultiplayerSession>>,
) {
    let caster_team = local_player_team(session.as_deref());
    // Collect ignited zone positions for chain-ignition checks
    let ignited_zones: Vec<(Vec3, f32)> = ignited_zone_query
        .iter()
        .map(|z| (z.origin, z.radius))
        .collect();

    for (zone_entity, mut zone) in &mut zones {
        // Track ignition source point
        let mut ignition_pos: Option<Vec3> = None;

        // Check if any already-ignited grease zone overlaps this one
        // Chain Combustion talent extends the range
        let chain_range_mult = if zone.talent_params.chain_combustion {
            constants::CHAIN_COMBUSTION_RANGE_MULT
        } else {
            1.0
        };
        for &(ignited_origin, ignited_radius) in &ignited_zones {
            let dist = xz_distance(zone.origin, ignited_origin);
            let to_this = Vec2::new(
                zone.origin.x - ignited_origin.x,
                zone.origin.z - ignited_origin.z,
            );
            if dist <= (zone.radius + ignited_radius) * chain_range_mult {
                // Ignition point: nearest edge of ignited zone toward this zone
                let dir = if dist > 0.001 {
                    to_this / dist
                } else {
                    Vec2::X
                };
                let edge = Vec2::new(ignited_origin.x, ignited_origin.z) + dir * ignited_radius;
                ignition_pos = Some(Vec3::new(edge.x, 0.0, edge.y));
                break;
            }
        }

        // Check if any unit with FireDoT is inside the grease zone
        if ignition_pos.is_none() {
            for fire_transform in &fire_units {
                let dist = xz_distance(zone.origin, fire_transform.translation);
                if dist <= zone.radius {
                    ignition_pos = Some(Vec3::new(
                        fire_transform.translation.x,
                        0.0,
                        fire_transform.translation.z,
                    ));
                    break;
                }
            }
        }

        // Check if any fireball explosion overlaps the grease zone
        if ignition_pos.is_none() {
            for explosion in &fireball_explosions {
                let dist = xz_distance(zone.origin, explosion.origin);
                if dist <= zone.radius + explosion.max_radius {
                    ignition_pos = Some(Vec3::new(explosion.origin.x, 0.0, explosion.origin.z));
                    break;
                }
            }
        }

        // Check if any wall of fire overlaps the grease zone
        if ignition_pos.is_none() {
            for wall in &wall_of_fires {
                let dist = wall.distance_to_point(zone.origin);
                if dist <= zone.radius + wall.half_width {
                    // Closest point on wall line to zone center
                    let p = Vec2::new(zone.origin.x, zone.origin.z);
                    let a = Vec2::new(wall.start.x, wall.start.z);
                    let b = Vec2::new(wall.end.x, wall.end.z);
                    let ab = b - a;
                    let ap = p - a;
                    let ab_len_sq = ab.length_squared();
                    let t = if ab_len_sq > 0.0001 {
                        (ap.dot(ab) / ab_len_sq).clamp(0.0, 1.0)
                    } else {
                        0.0
                    };
                    let closest = a + ab * t;
                    ignition_pos = Some(Vec3::new(closest.x, 0.0, closest.y));
                    break;
                }
            }
        }

        // Check if any meteor ground fire overlaps the grease zone
        if ignition_pos.is_none() {
            for fire in &meteor_ground_fires {
                let dist = xz_distance(zone.origin, fire.origin);
                if dist <= zone.radius + fire.radius {
                    ignition_pos = Some(Vec3::new(fire.origin.x, 0.0, fire.origin.z));
                    break;
                }
            }
        }

        // Check if any disintegrate beam passes through the grease zone at ground level
        if ignition_pos.is_none() {
            for beam in &disintegrate_beams {
                let to_zone = zone.origin - beam.origin;
                let projection = to_zone.dot(beam.direction);
                let clamped_proj = projection.clamp(0.0, beam.current_length());
                let closest = beam.origin + beam.direction * clamped_proj;
                if closest.y > constants::IGNITION_HEIGHT_THRESHOLD {
                    continue;
                }
                let dist = xz_distance(zone.origin, closest);
                if dist <= zone.radius + beam.beam_width() {
                    ignition_pos = Some(Vec3::new(closest.x, 0.0, closest.z));
                    break;
                }
            }
        }

        if let Some(ign_point) = ignition_pos {
            commands
                .entity(zone_entity)
                .insert(GreaseIgnited::new(ign_point));

            // Talent: Lingering Flames — reset time_alive so fire burns for the full zone duration
            if zone.talent_params.lingering_flames {
                zone.time_alive = 0.0;
            }

            // Apply one-time burst fire damage only near the ignition point
            if zone.ignite_damage > 0.0 {
                let burst_radius = zone.radius * constants::IGNITION_BURST_RADIUS_FRACTION;
                for (entity, transform, mut health, mut temp_hp, has_spell_shield, team) in
                    &mut targets
                {
                    let dist = xz_distance(ign_point, transform.translation);
                    if dist <= burst_radius {
                        apply_spell_damage_with_team(
                            &mut commands,
                            entity,
                            &mut health,
                            temp_hp.as_deref_mut(),
                            zone.ignite_damage * zone.empowerment,
                            DamageType::Fire,
                            has_spell_shield,
                            caster_team,
                            *team,
                        );
                    }
                }
            }

            // Talent: Grease Geyser — launch enemies upward at ignition
            if zone.talent_params.grease_geyser {
                let mut units_launched: u32 = 0;
                for (entity, transform, _health, _temp_hp, _has_spell_shield, _team) in &mut targets
                {
                    let dist = xz_distance(zone.origin, transform.translation);
                    if dist <= zone.radius {
                        commands.entity(entity).insert((
                            Airborne::new(
                                constants::GEYSER_LAUNCH_VELOCITY,
                                constants::GEYSER_GRAVITY,
                                transform.translation.y,
                                DamageType::Fire,
                            ),
                            RootedModifier::new(constants::GEYSER_ROOT_DURATION),
                        ));
                        units_launched += 1;
                    }
                }
                if units_launched > 0
                    && let Some(ref mut progress) = talent_progress
                {
                    progress.increment(Spell::Grease, units_launched);
                }
            }

            // Upgrade pathfinding to hazard
            write_grease_obstacle(
                zone.origin,
                zone.radius,
                ObstacleType::Hazard(5.0),
                &mut obstacle_events,
            );
        }
    }
}

/// Updates fire spread timer for burning grease zones (controls smoke VFX and burn damage radius).
pub fn update_grease_fire_spread(
    time: Res<Time>,
    mut zones: Query<(&GreaseZone, &mut GreaseIgnited)>,
) {
    let delta = time.delta_secs();
    for (_zone, mut ignited) in &mut zones {
        ignited.fire_spread_time += delta;
    }
}

/// Applies burn damage from ignited grease zones.
/// During fire spread, only damages units within the current fire radius.
pub fn apply_grease_burn(
    mut commands: Commands,
    time: Res<Time>,
    mut zones: Query<
        (&mut GreaseZone, &GreaseIgnited),
        Without<crate::game::multiplayer::components::GhostSpellEffect>,
    >,
    #[allow(clippy::type_complexity)] mut targets: Query<
        (
            Entity,
            &Transform,
            &mut Health,
            Option<&mut TemporaryHitPoints>,
            Has<SpellShield>,
            &Team,
        ),
        (
            Without<Corpse>,
            Without<crate::game::multiplayer::components::GhostEntity>,
        ),
    >,
    mut talent_progress: Option<ResMut<BattleTalentProgress>>,
    session: Option<Res<MultiplayerSession>>,
) {
    let caster_team = local_player_team(session.as_deref());
    let delta = time.delta_secs();
    for (mut zone, ignited) in &mut zones {
        zone.time_alive += delta;
        zone.time_since_last_tick += delta;
        if zone.time_since_last_tick >= zone.ignite_burn_tick {
            zone.time_since_last_tick = 0.0;

            // During spread phase, scope damage to current fire radius from ignition point
            let fire_radius =
                ignited.current_fire_radius(zone.radius, constants::FIRE_SPREAD_DURATION);
            let spreading = ignited.fire_spread_time < constants::FIRE_SPREAD_DURATION;

            let burn_damage =
                zone.ignite_burn_damage * zone.empowerment * zone.talent_params.burn_damage_mult;
            let mut units_burned: u32 = 0;

            for (entity, transform, mut health, mut temp_hp, has_spell_shield, team) in &mut targets
            {
                let in_burn_area = if spreading {
                    // Check distance from ignition point during spread
                    xz_distance(ignited.ignition_point, transform.translation) <= fire_radius
                } else {
                    // Fire fully spread — use full zone radius from center
                    xz_distance(zone.origin, transform.translation) <= zone.radius
                };

                if in_burn_area {
                    apply_spell_damage_with_team(
                        &mut commands,
                        entity,
                        &mut health,
                        temp_hp.as_deref_mut(),
                        burn_damage,
                        DamageType::Fire,
                        has_spell_shield,
                        caster_team,
                        *team,
                    );
                    units_burned += 1;
                }
            }

            // Track talent progress for burns
            if units_burned > 0
                && let Some(ref mut progress) = talent_progress
            {
                progress.increment(Spell::Grease, units_burned);
            }
        }
    }
}
