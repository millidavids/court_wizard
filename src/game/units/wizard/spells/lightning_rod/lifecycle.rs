//! Lightning rod lifecycle: rod updates, strikes, and arc propagation.

use std::cmp::Ordering;

use super::casting::spawn_descending_strike;
use super::components::{LightningRod, LightningRodArc, LightningRodTalentParams, LightningStrike};
use super::constants::*;
use crate::game::terrain::messages::TerrainDamageMessage;
use crate::game::terrain::pond::components::Pond;
use crate::game::units::DamageType;
use crate::game::units::components::{
    Corpse, Health, SlowMovementModifier, TemporaryHitPoints, apply_spell_damage,
};
use crate::game::units::king::components::SpellShield;
use crate::game::units::wizard::components::Spell;
use crate::game::units::wizard::spells::lightning_bolt::{
    LightningBolt, LightningBoltConfig, spawn_lightning_bolt,
};
use crate::game::units::wizard::spells::utils::xz_distance;
use crate::game::units::wizard::spells::visual_assets::SpellVisualAssets;
use crate::game::units::wizard::talents::resources::BattleTalentProgress;
use bevy::prelude::*;

/// Compute talent parameters from active talent selections.
pub(super) fn update_lightning_rod(
    time: Res<Time>,
    mut commands: Commands,
    visual_assets: Res<SpellVisualAssets>,
    // Host-authoritative only — the guest's ghost rod must NOT also fire
    // strikes, or every rod tick would deal damage twice (once on each peer).
    mut rods: Query<
        (Entity, &mut LightningRod),
        Without<crate::game::multiplayer::components::GhostSpellEffect>,
    >,
) {
    let delta = time.delta_secs();

    for (entity, mut rod) in rods.iter_mut() {
        rod.time_alive += delta;
        rod.time_since_strike += delta;

        // Despawn if expired
        if rod.is_expired() {
            commands.entity(entity).try_despawn();
            continue;
        }

        // T1-1 Rapid Strikes: reduce interval
        let effective_interval = STRIKE_INTERVAL * rod.talent_params.strike_interval_mult;

        // Spawn lightning strike on interval
        if rod.time_since_strike >= effective_interval {
            rod.time_since_strike = 0.0;
            rod.strike_count += 1;

            // Calculate effective damage
            let mut arc_damage = ARC_DAMAGE * rod.empowerment * rod.talent_params.damage_mult;

            // T3-1 Tesla Coil: apply accumulated ramp, then increment for next strike
            arc_damage *= 1.0 + rod.damage_ramp;
            if rod.talent_params.tesla_coil {
                rod.damage_ramp += TESLA_COIL_RAMP_PER_STRIKE;
            }

            // T2-2 Overcharge: every Nth strike deals bonus damage
            if rod.talent_params.overcharge && rod.strike_count % OVERCHARGE_EVERY_N == 0 {
                arc_damage *= OVERCHARGE_DAMAGE_MULT;
            }

            // T1-2 Wider Arc: radius and targets
            let arc_radius = ARC_RADIUS * rod.empowerment * rod.talent_params.arc_radius_mult;
            let max_targets = ARC_MAX_TARGETS + rod.talent_params.extra_targets;

            let target_pos = Vec3::new(rod.position.x, TOWER_HEIGHT, rod.position.z);

            spawn_descending_strike(
                &mut commands,
                &visual_assets,
                LightningStrike {
                    target_pos,
                    speed: STRIKE_SPEED,
                    arc_damage,
                    arc_radius,
                    empowerment: rod.empowerment,
                    max_targets,
                    nexus_damage_mult: 1.0,
                    talent_params: rod.talent_params,
                },
            );
        }
    }
}

/// Moves lightning strikes downward and triggers arcs when they hit the rod.
#[allow(clippy::too_many_arguments)]
pub(super) fn update_lightning_strikes(
    time: Res<Time>,
    mut commands: Commands,
    visual_assets: Res<SpellVisualAssets>,
    mut strikes: Query<(Entity, &mut LightningBolt, &LightningStrike)>,
    mut screen_flash: MessageWriter<crate::game::crt_effect::ScreenFlashMessage>,
    mut units: Query<
        (
            Entity,
            &Transform,
            &mut Health,
            Option<&mut TemporaryHitPoints>,
            Has<SpellShield>,
            Option<&mut SlowMovementModifier>,
        ),
        (Without<Corpse>, Without<LightningStrike>),
    >,
    ponds: Query<&Pond>,
    mut talent_progress: Option<ResMut<BattleTalentProgress>>,
    mut terrain_damage: MessageWriter<TerrainDamageMessage>,
) {
    let delta = time.delta_secs();

    for (entity, mut bolt, strike) in strikes.iter_mut() {
        // Drive the bolt's end downward; the jagged-bolt system re-jitters
        // around start/end every frame so the descent looks natural.
        bolt.end.y -= strike.speed * delta;

        // Check if bolt has reached the rod
        if bolt.end.y <= strike.target_pos.y {
            // Snap end to the rod top so the frozen afterimage shows the bolt
            // touching the rod, not overshooting through the ground.
            bolt.end.y = strike.target_pos.y;
            terrain_damage.write(TerrainDamageMessage {
                position: strike.target_pos,
                radius: strike.arc_radius,
                damage: strike.arc_damage,
                damage_type: DamageType::Electric,
            });
            screen_flash.write(crate::game::crt_effect::ScreenFlashMessage {
                color: [1.0, 1.0, 1.0],
                duration: 0.15,
                intensity: 0.04,
            });

            // Spawn arcs to nearby units
            let kills = spawn_arcs_to_nearby_units(
                &mut commands,
                &visual_assets,
                strike.target_pos,
                strike.arc_damage,
                strike.arc_radius,
                strike.empowerment,
                strike.max_targets,
                &strike.talent_params,
                &mut units,
                &mut talent_progress,
            );

            // Also arc to nearby ponds (shock already applied via TerrainDamageMessage above).
            for pond in ponds.iter() {
                if xz_distance(pond.center, strike.target_pos) <= strike.arc_radius + pond.radius {
                    spawn_arc(
                        &mut commands,
                        &visual_assets,
                        strike.target_pos,
                        pond.center,
                        strike.empowerment,
                    );
                }
            }

            // T3-2 Lightning Nexus: kills trigger a bonus strike with compounding falloff
            if strike.talent_params.lightning_nexus && kills > 0 {
                let next_mult = strike.nexus_damage_mult * LIGHTNING_NEXUS_FALLOFF;
                // Only spawn bonus if damage is still meaningful (> 5% of original)
                if next_mult >= 0.05 {
                    spawn_descending_strike(
                        &mut commands,
                        &visual_assets,
                        LightningStrike {
                            target_pos: strike.target_pos,
                            speed: STRIKE_SPEED,
                            arc_damage: strike.arc_damage * LIGHTNING_NEXUS_FALLOFF,
                            arc_radius: strike.arc_radius,
                            empowerment: strike.empowerment,
                            max_targets: strike.max_targets,
                            nexus_damage_mult: next_mult,
                            talent_params: strike.talent_params,
                        },
                    );
                }
            }

            // End the active phase. The shared `update_lightning_bolts` path
            // sees `lifetime <= 0` next iteration and transitions the bolt
            // into its afterimage fade.
            //
            // Remove the `LightningStrike` component so this system stops
            // matching the entity. Otherwise the clamped `end.y` keeps
            // satisfying the impact check every frame for the entire
            // afterimage duration, re-applying damage and re-spawning arcs.
            bolt.lifetime = 0.0;
            commands.entity(entity).remove::<LightningStrike>();
        }
    }
}

/// Finds nearby units and spawns lightning arcs from the rod to each target.
/// Returns the number of units killed by the arcs.
#[allow(clippy::too_many_arguments)]
fn spawn_arcs_to_nearby_units(
    commands: &mut Commands,
    assets: &SpellVisualAssets,
    rod_top: Vec3,
    damage: f32,
    radius: f32,
    empowerment: f32,
    max_targets: usize,
    talent_params: &LightningRodTalentParams,
    units: &mut Query<
        (
            Entity,
            &Transform,
            &mut Health,
            Option<&mut TemporaryHitPoints>,
            Has<SpellShield>,
            Option<&mut SlowMovementModifier>,
        ),
        (Without<Corpse>, Without<LightningStrike>),
    >,
    talent_progress: &mut Option<ResMut<BattleTalentProgress>>,
) -> u32 {
    // Collect targets sorted by distance (closest first)
    let mut targets: Vec<(Entity, Vec3, f32)> = units
        .iter()
        .map(|(entity, transform, _, _, _, _)| {
            let pos = transform.translation;
            let dist = Vec3::new(rod_top.x, 0.0, rod_top.z).distance(Vec3::new(pos.x, 0.0, pos.z));
            (entity, pos, dist)
        })
        .filter(|(_, _, dist)| *dist <= radius)
        .collect();

    targets.sort_by(|a, b| a.2.partial_cmp(&b.2).unwrap_or(Ordering::Equal));
    targets.truncate(max_targets);

    let mut kills = 0u32;
    let mut hit_entities: Vec<Entity> = Vec::new();
    let hit_count = targets.len() as u32;

    // Apply damage and spawn arc visuals
    for (target_entity, target_pos, _) in &targets {
        kills += apply_arc_hit(commands, units, *target_entity, damage, talent_params);
        hit_entities.push(*target_entity);
        spawn_arc(commands, assets, rod_top, *target_pos, empowerment);
    }

    // T2-0 Chain Reaction: chain from each hit target to additional nearby enemies
    if talent_params.chain_reaction {
        let chain_damage = damage * CHAIN_REACTION_DAMAGE_MULT;

        // Collect chain targets from each primary target
        let mut chain_targets: Vec<(Entity, Vec3, Vec3)> = Vec::new(); // (entity, pos, source_pos)

        for (primary_entity, primary_pos, _) in &targets {
            // Find closest unit to the primary target that wasn't already hit
            let mut candidates: Vec<(Entity, Vec3, f32)> = units
                .iter()
                .filter_map(|(entity, transform, _, _, _, _)| {
                    if hit_entities.contains(&entity) || entity == *primary_entity {
                        return None;
                    }
                    let pos = transform.translation;
                    let dist = Vec3::new(primary_pos.x, 0.0, primary_pos.z)
                        .distance(Vec3::new(pos.x, 0.0, pos.z));
                    if dist <= radius {
                        Some((entity, pos, dist))
                    } else {
                        None
                    }
                })
                .collect();

            candidates.sort_by(|a, b| a.2.partial_cmp(&b.2).unwrap_or(Ordering::Equal));
            candidates.truncate(CHAIN_REACTION_EXTRA_TARGETS);

            for (chain_entity, chain_pos, _) in candidates {
                if !chain_targets.iter().any(|(e, _, _)| *e == chain_entity) {
                    chain_targets.push((chain_entity, chain_pos, *primary_pos));
                    hit_entities.push(chain_entity);
                }
            }
        }

        // Apply chain damage
        for (chain_entity, chain_pos, source_pos) in &chain_targets {
            kills += apply_arc_hit(commands, units, *chain_entity, chain_damage, talent_params);
            spawn_arc(commands, assets, *source_pos, *chain_pos, empowerment);
        }
    }

    // Track talent progress: "Enemies struck by arcs"
    if let Some(progress) = talent_progress
        && hit_count > 0
    {
        progress.increment(Spell::LightningRod, hit_count);
    }

    kills
}

/// Applies arc damage to a single target, optionally slows it (Magnetic Field), and returns 1 if killed.
fn apply_arc_hit(
    commands: &mut Commands,
    units: &mut Query<
        (
            Entity,
            &Transform,
            &mut Health,
            Option<&mut TemporaryHitPoints>,
            Has<SpellShield>,
            Option<&mut SlowMovementModifier>,
        ),
        (Without<Corpse>, Without<LightningStrike>),
    >,
    entity: Entity,
    damage: f32,
    talent_params: &LightningRodTalentParams,
) -> u32 {
    let Ok((_, _, mut health, mut temp_hp, has_spell_shield, mut slow)) = units.get_mut(entity)
    else {
        return 0;
    };

    let was_alive = health.current > 0.0;

    apply_spell_damage(
        commands,
        entity,
        &mut health,
        temp_hp.as_deref_mut(),
        damage,
        DamageType::Electric,
        has_spell_shield,
    );

    let killed = u32::from(was_alive && health.current <= 0.0);

    // T2-1 Magnetic Field: slow hit enemies
    if talent_params.magnetic_field {
        if let Some(existing_slow) = &mut slow {
            existing_slow.apply(MAGNETIC_FIELD_SLOW, MAGNETIC_FIELD_SLOW_DURATION);
        } else {
            commands.entity(entity).insert(SlowMovementModifier::new(
                MAGNETIC_FIELD_SLOW,
                MAGNETIC_FIELD_SLOW_DURATION,
            ));
        }
    }

    killed
}

/// Spawns a jagged ground-arc lightning bolt between two points. A
/// `LightningRodArc` marker is attached to the parent so the multiplayer
/// snapshot collector can serialize it.
fn spawn_arc(
    commands: &mut Commands,
    assets: &SpellVisualAssets,
    start: Vec3,
    end: Vec3,
    empowerment: f32,
) {
    let arc_width = ARC_WIDTH * empowerment;
    let bolt = spawn_lightning_bolt(
        commands,
        assets.unit_rect.clone(),
        assets.lightning_rod_arc.clone(),
        start,
        end,
        LightningBoltConfig {
            width: arc_width,
            lifetime: ARC_LIFETIME,
            peak_height: 0.0,
            jitter_amplitude: 10.0,
            segments: 14,
            fork_count: 1,
            fork_segments: 3,
            fork_length: arc_width * 4.0 + 12.0,
            afterimage_duration: 0.25,
        },
    );
    commands.entity(bolt).insert(LightningRodArc { start, end });
}
