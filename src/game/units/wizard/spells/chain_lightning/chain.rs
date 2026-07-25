//! Chain lightning bounce processing and on-hit effects.

use super::casting::spawn_arc;
use std::cmp::Ordering;
use std::collections::HashSet;

use super::super::super::components::Spell;
use super::components::*;
use super::constants;
use crate::game::components::OnGameplayScreen;
use crate::game::terrain::messages::TerrainDamageMessage;
use crate::game::terrain::pond::components::Pond;
use crate::game::units::DamageType;
use crate::game::units::components::{
    Corpse, Health, Knockback, SlowMovementModifier, Team, TemporaryHitPoints,
    apply_spell_damage_with_team,
};
use crate::game::units::king::components::SpellShield;
use crate::game::units::wizard::components::Wizard;
use crate::game::units::wizard::spells::arcane_crystal::components::ArcaneCrystal;
use crate::game::units::wizard::spells::lightning_rod::LightningRod;
use crate::game::units::wizard::spells::utils::local_player_team;
use crate::game::units::wizard::spells::visual_assets::SpellVisualAssets;
use crate::game::units::wizard::spells::wall_of_stone::components::WallOfStone;
use crate::game::units::wizard::talents::resources::BattleTalentProgress;
use crate::networking::session::MultiplayerSession;
use bevy::prelude::*;

#[allow(clippy::too_many_arguments, clippy::type_complexity)]
pub fn process_chain_lightning_bounces(
    time: Res<Time>,
    mut commands: Commands,
    visual_assets: Res<SpellVisualAssets>,
    mut bolts: Query<
        (Entity, &mut ChainLightningBolt),
        Without<crate::game::multiplayer::components::GhostSpellEffect>,
    >,
    mut groups: Query<&mut ChainLightningGroup>,
    #[allow(clippy::type_complexity)] mut enemies: Query<
        (
            Entity,
            &Transform,
            &Team,
            &mut Health,
            Option<&mut TemporaryHitPoints>,
            Has<SpellShield>,
        ),
        (
            Without<Corpse>,
            Without<crate::game::multiplayer::components::GhostEntity>,
            Without<Wizard>,
            Without<crate::game::pathfinding::StagingAttacker>,
        ),
    >,
    mut rods: Query<(Entity, &Transform, &mut LightningRod)>,
    crystals: Query<(Entity, &Transform), With<ArcaneCrystal>>,
    ponds: Query<(Entity, &Pond)>,
    walls: Query<&WallOfStone>,
    rocks_query: Query<&crate::game::terrain::boulder::components::Boulder>,
    trees_query: Query<&crate::game::terrain::tree::components::Tree>,
    mut slow_query: Query<&mut SlowMovementModifier>,
    mut talent_progress: Option<ResMut<BattleTalentProgress>>,
    mut terrain_damage: MessageWriter<TerrainDamageMessage>,
    session: Option<Res<MultiplayerSession>>,
) {
    let caster_team = local_player_team(session.as_deref());
    // Collect bolt data to avoid borrow conflicts when spawning child bolts
    let mut bolts_to_process: Vec<(Entity, ChainLightningBoltSnapshot)> = Vec::new();

    for (bolt_entity, mut bolt) in &mut bolts {
        bolt.bounce_delay_timer -= time.delta_secs();

        if bolt.bounce_delay_timer <= 0.0 && bolt.bounces_remaining > 0 {
            bolts_to_process.push((
                bolt_entity,
                ChainLightningBoltSnapshot {
                    group_entity: bolt.group_entity,
                    current_damage: bolt.current_damage,
                    damage_type: bolt.damage_type,
                    bounces_remaining: bolt.bounces_remaining,
                    last_hit_position: bolt.last_hit_position,
                    empowerment: bolt.empowerment,
                    split_depth: bolt.split_depth,
                    split_count: bolt.split_count,
                    damage_falloff: bolt.damage_falloff,
                    static_charge: bolt.static_charge,
                    magnetic_pull: bolt.magnetic_pull,
                    chain_reaction: bolt.chain_reaction,
                    bounce_range_mult: bolt.bounce_range_mult,
                },
            ));
            // Mark as done so it gets cleaned up
            bolt.bounces_remaining = 0;
        }

        // Despawn bolt if no more bounces and timer expired
        if bolt.bounces_remaining == 0 && bolt.bounce_delay_timer <= 0.0 {
            commands.entity(bolt_entity).try_despawn();
        }
    }

    // Snapshot obstacle geometry once per frame — shared across all bolts processed this tick.
    let wall_snapshot: Vec<_> = walls.iter().collect();
    let rock_snapshot: Vec<_> = rocks_query.iter().filter(|r| !r.sinking).collect();
    let tree_snapshot: Vec<_> = trees_query.iter().collect();

    // Process collected bolts
    for (bolt_entity, snapshot) in bolts_to_process {
        // Look up shared hit list
        let Ok(mut group) = groups.get_mut(snapshot.group_entity) else {
            // Group was despawned (shouldn't happen), clean up bolt
            commands.entity(bolt_entity).try_despawn();
            continue;
        };

        let bounce_range =
            constants::BOUNCE_RANGE * snapshot.empowerment * snapshot.bounce_range_mult;
        let targets = find_next_bounce_targets(
            snapshot.last_hit_position,
            &group.hit_entities,
            &enemies,
            &rods,
            &crystals,
            &ponds,
            bounce_range,
            snapshot.split_count,
            &wall_snapshot,
            &rock_snapshot,
            &tree_snapshot,
        );

        for (target_entity, target_pos) in &targets {
            terrain_damage.write(TerrainDamageMessage {
                position: *target_pos,
                radius: 0.0,
                damage: snapshot.current_damage,
                damage_type: snapshot.damage_type,
            });
            let is_pond = ponds.get(*target_entity).is_ok();
            // Check if this target is a lightning rod
            if let Ok((_, _, mut rod)) = rods.get_mut(*target_entity) {
                // Trigger an immediate lightning strike on the rod
                rod.time_since_strike = f32::MAX;
            } else if !is_pond {
                // Ponds are bounce nodes only; their shock is handled via TerrainDamageMessage.
                let target_killed;
                if let Ok((_, _, team, mut health, mut temp_hp, has_spell_shield)) =
                    enemies.get_mut(*target_entity)
                {
                    apply_spell_damage_with_team(
                        &mut commands,
                        *target_entity,
                        &mut health,
                        temp_hp.as_deref_mut(),
                        snapshot.current_damage,
                        snapshot.damage_type,
                        has_spell_shield,
                        caster_team,
                        *team,
                    );
                    target_killed = health.current <= 0.0;
                } else {
                    target_killed = false;
                }

                // Track talent progress
                if let Some(ref mut progress) = talent_progress {
                    progress.increment(Spell::ChainLightning, 1);
                }

                // Apply on-hit talent effects
                apply_chain_lightning_on_hit(
                    &mut commands,
                    *target_entity,
                    *target_pos,
                    snapshot.last_hit_position,
                    snapshot.static_charge,
                    snapshot.magnetic_pull,
                    Some(&mut slow_query),
                );

                // Chain Reaction: kills explode for AoE and spawn sub-chain
                if snapshot.chain_reaction && target_killed && snapshot.bounces_remaining > 1 {
                    // AoE damage to nearby enemies
                    let aoe_damage =
                        snapshot.current_damage * constants::CHAIN_REACTION_AOE_DAMAGE_MULT;
                    let mut aoe_targets: Vec<(Entity, Vec3)> = Vec::new();
                    for (e, t, _, _, _, _) in enemies.iter() {
                        if e != *target_entity && !group.hit_entities.contains(&e) {
                            let dist = crate::game::units::wizard::spells::utils::xz_distance(
                                *target_pos,
                                t.translation,
                            );
                            if dist <= constants::CHAIN_REACTION_AOE_RADIUS {
                                aoe_targets.push((e, t.translation));
                            }
                        }
                    }
                    for (aoe_entity, _) in &aoe_targets {
                        if let Ok((_, _, team, mut health, mut temp_hp, has_spell_shield)) =
                            enemies.get_mut(*aoe_entity)
                        {
                            apply_spell_damage_with_team(
                                &mut commands,
                                *aoe_entity,
                                &mut health,
                                temp_hp.as_deref_mut(),
                                aoe_damage,
                                snapshot.damage_type,
                                has_spell_shield,
                                caster_team,
                                *team,
                            );
                        }
                    }

                    // Spawn sub-chain from corpse position with half remaining bounces
                    let sub_bounces =
                        snapshot.bounces_remaining / constants::CHAIN_REACTION_BOUNCE_DIVISOR;
                    if sub_bounces > 0 {
                        spawn_child_bolt(&mut commands, &snapshot, sub_bounces, *target_pos);
                    }
                }
            }

            // Add to shared hit list
            group.hit_entities.insert(*target_entity);

            // Spawn arc visual
            spawn_arc(
                &mut commands,
                &visual_assets,
                snapshot.last_hit_position,
                *target_pos,
                snapshot.split_depth + 1,
                snapshot.empowerment,
            );

            // Spawn child bolt if more bounces remain
            if snapshot.bounces_remaining > 1 {
                spawn_child_bolt(
                    &mut commands,
                    &snapshot,
                    snapshot.bounces_remaining - 1,
                    *target_pos,
                );
            }
        }
    }
}

/// Applies on-hit talent effects (Static Charge slow, Magnetic Pull knockback)
/// to a chain lightning target. Used by both initial cast and bounce processing.
/// When `slow_query` is provided, it updates existing slow modifiers; otherwise it
/// always inserts a new one (used for the initial cast where the entity is fresh).
pub(super) fn apply_chain_lightning_on_hit(
    commands: &mut Commands,
    target_entity: Entity,
    target_pos: Vec3,
    pull_origin: Vec3,
    static_charge: bool,
    magnetic_pull: bool,
    slow_query: Option<&mut Query<&mut SlowMovementModifier>>,
) {
    if static_charge {
        let mut applied = false;
        if let Some(slow_q) = slow_query
            && let Ok(mut slow) = slow_q.get_mut(target_entity)
        {
            slow.apply(
                constants::STATIC_CHARGE_SLOW,
                constants::STATIC_CHARGE_DURATION,
            );
            applied = true;
        }
        if !applied {
            commands
                .entity(target_entity)
                .insert(SlowMovementModifier::new(
                    constants::STATIC_CHARGE_SLOW,
                    constants::STATIC_CHARGE_DURATION,
                ));
        }
    }

    if magnetic_pull {
        let pull_dir = (pull_origin - target_pos).normalize_or_zero();
        commands.entity(target_entity).insert(Knockback::new(
            pull_dir,
            constants::MAGNETIC_PULL_SPEED,
            constants::MAGNETIC_PULL_DURATION,
        ));
    }
}

/// Snapshot of bolt data for deferred processing.
struct ChainLightningBoltSnapshot {
    group_entity: Entity,
    current_damage: f32,
    damage_type: DamageType,
    bounces_remaining: u32,
    last_hit_position: Vec3,
    empowerment: f32,
    split_depth: u32,
    split_count: usize,
    damage_falloff: f32,
    static_charge: bool,
    magnetic_pull: bool,
    chain_reaction: bool,
    bounce_range_mult: f32,
}

/// Spawns a child `ChainLightningBolt` from a snapshot, inheriting all talent fields.
fn spawn_child_bolt(
    commands: &mut Commands,
    snapshot: &ChainLightningBoltSnapshot,
    bounces_remaining: u32,
    target_pos: Vec3,
) {
    commands.spawn((
        ChainLightningBolt {
            group_entity: snapshot.group_entity,
            current_damage: snapshot.current_damage * snapshot.damage_falloff,
            damage_type: snapshot.damage_type,
            bounces_remaining,
            last_hit_position: target_pos,
            bounce_delay_timer: constants::BOUNCE_DELAY * snapshot.empowerment,
            empowerment: snapshot.empowerment,
            split_depth: snapshot.split_depth + 1,
            split_count: snapshot.split_count,
            damage_falloff: snapshot.damage_falloff,
            static_charge: snapshot.static_charge,
            magnetic_pull: snapshot.magnetic_pull,
            chain_reaction: snapshot.chain_reaction,
            bounce_range_mult: snapshot.bounce_range_mult,
        },
        OnGameplayScreen,
    ));
}

/// Finds up to `max_targets` enemies, lightning rods, arcane crystals, or ponds within bounce
/// range that haven't been hit yet. Targets all living units (defenders, attackers, and undead),
/// lightning rods, arcane crystals, and ponds — but excludes corpses.
/// Filters out targets blocked by WallOfStone line of sight.
#[allow(clippy::too_many_arguments)]
#[allow(clippy::type_complexity)]
fn find_next_bounce_targets(
    origin: Vec3,
    hit_entities: &HashSet<Entity>,
    enemies: &Query<
        (
            Entity,
            &Transform,
            &Team,
            &mut Health,
            Option<&mut TemporaryHitPoints>,
            Has<SpellShield>,
        ),
        (
            Without<Corpse>,
            Without<crate::game::multiplayer::components::GhostEntity>,
            Without<Wizard>,
            Without<crate::game::pathfinding::StagingAttacker>,
        ),
    >,
    rods: &Query<(Entity, &Transform, &mut LightningRod)>,
    crystals: &Query<(Entity, &Transform), With<ArcaneCrystal>>,
    ponds: &Query<(Entity, &Pond)>,
    bounce_range: f32,
    max_targets: usize,
    walls: &[&WallOfStone],
    rocks: &[&crate::game::terrain::boulder::components::Boulder],
    trees: &[&crate::game::terrain::tree::components::Tree],
) -> Vec<(Entity, Vec3)> {
    use crate::game::terrain::boulder::components::Boulder;
    use crate::game::terrain::tree::components::Tree;

    use crate::game::units::wizard::spells::utils::xz_distance;

    let los_blocked = |from: Vec3, to: Vec3| -> bool {
        WallOfStone::any_blocks_los(walls, from, to)
            || Boulder::any_blocks_los(rocks, from, to)
            || Tree::any_blocks_los(trees, from, to)
    };

    let mut candidates: Vec<(Entity, Vec3, f32)> = Vec::new();
    let try_push = |entity: Entity, pos: Vec3, candidates: &mut Vec<(Entity, Vec3, f32)>| {
        if hit_entities.contains(&entity) {
            return;
        }
        let distance = xz_distance(origin, pos);
        if distance <= bounce_range && !los_blocked(origin, pos) {
            candidates.push((entity, pos, distance));
        }
    };

    // No team filter — spell damages ALL units indiscriminately (except staging attackers, who are excluded).
    for (entity, transform, _, _, _, _) in enemies.iter() {
        try_push(entity, transform.translation, &mut candidates);
    }
    for (entity, transform, _) in rods.iter() {
        try_push(entity, transform.translation, &mut candidates);
    }
    for (entity, transform) in crystals.iter() {
        try_push(entity, transform.translation, &mut candidates);
    }
    for (entity, pond) in ponds.iter() {
        try_push(entity, pond.center, &mut candidates);
    }

    // Sort by distance (closest first)
    candidates.sort_by(|a, b| a.2.partial_cmp(&b.2).unwrap_or(Ordering::Equal));

    // Take up to max_targets
    candidates
        .into_iter()
        .take(max_targets)
        .map(|(entity, pos, _)| (entity, pos))
        .collect()
}

/// Cleans up chain lightning groups that have no remaining bolts.
pub fn cleanup_chain_lightning_groups(
    mut commands: Commands,
    groups: Query<Entity, With<ChainLightningGroup>>,
    bolts: Query<&ChainLightningBolt>,
) {
    for group_entity in &groups {
        let has_bolts = bolts.iter().any(|bolt| bolt.group_entity == group_entity);
        if !has_bolts {
            commands.entity(group_entity).try_despawn();
        }
    }
}
