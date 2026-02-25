use bevy::prelude::*;

use super::components::*;
use super::constants::*;
use super::resources::DispellerAssets;
use crate::game::cauldron::components::CauldronSpeedModifier;
use crate::game::components::{Acceleration, Billboard, OnGameplayScreen, Velocity};
use crate::game::constants::{
    calculate_defender_grid_position, calculate_grid_cell_position, calculate_spawn_cells,
    calculate_total_archers, calculate_total_infantry, cells_needed, distribute_units_to_cells, *,
};
use crate::game::multiplayer::components::NetworkedSpellEffect;
use crate::game::pathfinding::{FlowFieldInfluence, FlowFieldVelocity, OBSTACLE_BUFFER};
use crate::game::pathfinding::{ObstacleChanged, ObstacleType};
use crate::game::units::components::{
    AttackTiming, BanishedModifier, CommanderAuraSpeedModifier, Corpse, Effectiveness,
    EliteSpeedBonus, FlockingVelocity, FrostSlowModifier, GreaseSlipModifier, HasteModifier,
    Health, Hitbox, MesmerizedModifier, MovementSpeed, PolymorphedModifier, RootedModifier,
    RoughTerrainModifier, SleepModifier, SpikeGrowthSlowModifier, TargetingVelocity, Team,
    Teleportable, TemporaryHitPoints, apply_damage_to_unit,
};
use crate::game::units::infantry::components::DefendersActivated;
use crate::game::units::random_position_in_cell;
use crate::game::units::wizard::spells::grease::components::GreaseZone;
use crate::game::units::wizard::spells::meteor_fall::components::MeteorGroundFire;
use crate::game::units::wizard::spells::spike_growth::components::SpikeGrowthZone;
use crate::game::units::wizard::spells::wall_of_fire::components::WallOfFireEffect;
use crate::game::units::wizard::spells::wall_of_stone::components::WallOfStone;
use crate::networking::snapshot::SpellEffectKind;

/// Returns true if the spell effect kind is dispellable.
fn is_dispellable(kind: SpellEffectKind) -> bool {
    !matches!(
        kind,
        SpellEffectKind::FireballExplosion
            | SpellEffectKind::MeteorExplosion
            | SpellEffectKind::IceExplosion
            | SpellEffectKind::HealingPlumeZone
    )
}

/// Updates dispeller targeting — seeks nearest dispellable spell effect, or falls back to enemy targeting.
pub fn update_dispeller_targeting(
    defenders_activated: Res<DefendersActivated>,
    mut commands: Commands,
    mut dispellers: Query<
        (Entity, &Transform, &Team, &mut TargetingVelocity),
        (With<Dispeller>, Without<Corpse>),
    >,
    spell_effects: Query<(Entity, &Transform, &NetworkedSpellEffect)>,
    all_units: Query<(Entity, &Transform, &Team), Without<Corpse>>,
) {
    // Collect dispellable spell effect positions
    let spell_targets: Vec<(Entity, Vec3)> = spell_effects
        .iter()
        .filter(|(_, _, nse)| is_dispellable(nse.kind))
        .map(|(entity, transform, _)| (entity, transform.translation))
        .collect();

    // Collect unit snapshot for enemy targeting fallback
    let unit_snapshot: Vec<(Entity, Vec3, Team)> = all_units
        .iter()
        .map(|(entity, transform, team)| (entity, transform.translation, *team))
        .collect();

    for (entity, transform, team, mut targeting_velocity) in &mut dispellers {
        // Skip inactive defender dispellers
        if *team == Team::Defenders && !defenders_activated.active {
            *targeting_velocity = TargetingVelocity::default();
            continue;
        }

        // Priority 1: Nearest dispellable spell effect
        if !spell_targets.is_empty() {
            let nearest_spell = spell_targets.iter().min_by(|a, b| {
                let dist_a = (transform.translation.x - a.1.x).powi(2)
                    + (transform.translation.z - a.1.z).powi(2);
                let dist_b = (transform.translation.x - b.1.x).powi(2)
                    + (transform.translation.z - b.1.z).powi(2);
                dist_a
                    .partial_cmp(&dist_b)
                    .unwrap_or(std::cmp::Ordering::Equal)
            });

            if let Some(&(_, target_pos)) = nearest_spell {
                let diff = target_pos - transform.translation;
                let distance = (diff.x.powi(2) + diff.z.powi(2)).sqrt();
                targeting_velocity.distance_to_target = distance;

                if distance <= DISPEL_RANGE {
                    // In dispel range — stop moving
                    targeting_velocity.velocity = Vec3::ZERO;
                } else {
                    // Move toward spell effect
                    let direction = diff.normalize_or_zero();
                    targeting_velocity.velocity = Vec3::new(direction.x, 0.0, direction.z);
                }

                // Dispellers don't engage in melee — remove InMelee
                commands
                    .entity(entity)
                    .remove::<crate::game::units::components::InMelee>();
                continue;
            }
        }

        // Priority 2: Fall back to ranged enemy targeting (like archers)
        let nearest_enemy = unit_snapshot
            .iter()
            .filter(|(other_entity, _, other_team)| {
                *other_entity != entity
                    && match (*team, other_team) {
                        (Team::Undead, Team::Undead) => false,
                        (Team::Undead, _) => true,
                        (_, Team::Undead) => true,
                        _ => *other_team != *team,
                    }
            })
            .min_by(|a, b| {
                let dist_a = (transform.translation.x - a.1.x).powi(2)
                    + (transform.translation.z - a.1.z).powi(2);
                let dist_b = (transform.translation.x - b.1.x).powi(2)
                    + (transform.translation.z - b.1.z).powi(2);
                dist_a
                    .partial_cmp(&dist_b)
                    .unwrap_or(std::cmp::Ordering::Equal)
            });

        if let Some(&(_, target_pos, _)) = nearest_enemy {
            let diff = target_pos - transform.translation;
            let distance = (diff.x.powi(2) + diff.z.powi(2)).sqrt();
            targeting_velocity.distance_to_target = distance;

            if distance <= ATTACK_RANGE {
                // In attack range — stop and shoot
                targeting_velocity.velocity = Vec3::ZERO;
            } else {
                // Move toward enemy
                let direction = diff.normalize_or_zero();
                targeting_velocity.velocity = Vec3::new(direction.x, 0.0, direction.z);
            }
        } else {
            targeting_velocity.velocity = Vec3::ZERO;
            targeting_velocity.distance_to_target = f32::MAX;
        }

        commands
            .entity(entity)
            .remove::<crate::game::units::components::InMelee>();
    }
}

/// Dispeller movement system using shared weighted movement.
#[allow(clippy::type_complexity)]
pub fn dispeller_movement(
    time: Res<Time>,
    mut dispeller_units: Query<
        (
            &mut Velocity,
            &mut Acceleration,
            &MovementSpeed,
            &Effectiveness,
            &TargetingVelocity,
            &FlockingVelocity,
            &FlowFieldVelocity,
            Option<&crate::game::units::components::InMelee>,
            Option<&CommanderAuraSpeedModifier>,
            Option<&RoughTerrainModifier>,
            (Option<&FrostSlowModifier>, Option<&SpikeGrowthSlowModifier>),
            (
                Option<&CauldronSpeedModifier>,
                Option<&RootedModifier>,
                Option<&HasteModifier>,
                Option<&EliteSpeedBonus>,
            ),
            (
                Option<&MesmerizedModifier>,
                Option<&SleepModifier>,
                Option<&BanishedModifier>,
                Option<&GreaseSlipModifier>,
                Option<&PolymorphedModifier>,
            ),
            Has<DispelChanneling>,
        ),
        With<Dispeller>,
    >,
) {
    for (
        mut velocity,
        mut acceleration,
        movement_speed,
        effectiveness,
        targeting_velocity,
        flocking_velocity,
        flow_field_velocity,
        in_melee,
        aura_modifier,
        terrain_modifier,
        (frost_modifier, spike_growth_modifier),
        (cauldron_modifier, rooted, haste_modifier, elite_speed),
        (mesmerized, sleeping, banished, grease, polymorphed),
        is_channeling,
    ) in &mut dispeller_units
    {
        // CC'd units cannot move
        if rooted.is_some() || mesmerized.is_some() || sleeping.is_some() || banished.is_some() {
            velocity.x = 0.0;
            velocity.z = 0.0;
            continue;
        }

        // Polymorphed units wander randomly
        if polymorphed.is_some() {
            let angle = (time.elapsed_secs() * 0.5 + velocity.x.to_bits() as f32).sin()
                * std::f32::consts::TAU;
            velocity.x = angle.cos() * 20.0;
            velocity.z = angle.sin() * 20.0;
            continue;
        }

        // Channeling dispellers stand still
        if is_channeling {
            velocity.x = 0.0;
            velocity.z = 0.0;
            continue;
        }

        // Use shared weighted movement function
        crate::game::units::systems::calculate_weighted_movement(
            &time,
            &mut velocity,
            &mut acceleration,
            movement_speed.0,
            effectiveness,
            targeting_velocity,
            flocking_velocity,
            flow_field_velocity,
            in_melee.is_some(),
            aura_modifier.map(|m| m.0),
            terrain_modifier.map(|m| m.0),
            frost_modifier.map(|m| m.modifier),
            spike_growth_modifier.map(|m| m.modifier),
            cauldron_modifier.map(|m| m.0),
            haste_modifier.map(|m| m.modifier),
            elite_speed.map(|e| e.0),
            grease.map(|g| g.modifier),
        );

        // Stop completely when in optimal position (not in melee, not on hazard)
        if in_melee.is_none() && flow_field_velocity.terrain_cost <= 1.0 {
            let targeting_is_zero = targeting_velocity.velocity.length_squared() < 0.01;
            if targeting_is_zero {
                velocity.x = 0.0;
                velocity.z = 0.0;
                acceleration.x = 0.0;
                acceleration.z = 0.0;
            }
        }
    }
}

/// Manages dispel channeling: starts, advances, completes, and interrupts channels.
#[allow(clippy::too_many_arguments)]
pub fn update_dispel_channeling(
    mut commands: Commands,
    time: Res<Time>,
    mut channeling_dispellers: Query<
        (Entity, &Transform, &mut DispelChanneling),
        (With<Dispeller>, Without<Corpse>),
    >,
    non_channeling_dispellers: Query<
        (Entity, &Transform),
        (With<Dispeller>, Without<Corpse>, Without<DispelChanneling>),
    >,
    spell_effects: Query<(Entity, &Transform, &NetworkedSpellEffect)>,
    // Spell-specific queries for obstacle cleanup
    wall_of_stone_query: Query<&WallOfStone>,
    wall_of_fire_query: Query<&WallOfFireEffect>,
    spike_growth_query: Query<&SpikeGrowthZone>,
    grease_query: Query<&GreaseZone>,
    meteor_fire_query: Query<&MeteorGroundFire>,
    mut obstacle_events: MessageWriter<ObstacleChanged>,
) {
    let delta = time.delta_secs();

    // Advance existing channels
    for (entity, transform, mut channeling) in &mut channeling_dispellers {
        // Check if target still exists and is in range
        let target_ok = spell_effects
            .get(channeling.target_entity)
            .map(|(_, target_tf, nse)| {
                let dist = (transform.translation.x - target_tf.translation.x).powi(2)
                    + (transform.translation.z - target_tf.translation.z).powi(2);
                dist.sqrt() <= DISPEL_INTERRUPT_RANGE && is_dispellable(nse.kind)
            })
            .unwrap_or(false);

        if !target_ok {
            // Target gone or out of range — interrupt
            commands.entity(entity).remove::<DispelChanneling>();
            continue;
        }

        channeling.elapsed += delta;

        if channeling.elapsed >= DISPEL_CHANNEL_TIME {
            // Channel complete — despawn the spell effect
            despawn_spell_effect(
                &mut commands,
                channeling.target_entity,
                &wall_of_stone_query,
                &wall_of_fire_query,
                &spike_growth_query,
                &grease_query,
                &meteor_fire_query,
                &mut obstacle_events,
            );
            commands.entity(entity).remove::<DispelChanneling>();
        }
    }

    // Start new channels for non-channeling dispellers near spell effects
    let dispellable_effects: Vec<(Entity, Vec3)> = spell_effects
        .iter()
        .filter(|(_, _, nse)| is_dispellable(nse.kind))
        .map(|(e, tf, _)| (e, tf.translation))
        .collect();

    if dispellable_effects.is_empty() {
        return;
    }

    for (entity, transform) in &non_channeling_dispellers {
        let nearest = dispellable_effects
            .iter()
            .filter_map(|&(spell_entity, spell_pos)| {
                let dist = ((transform.translation.x - spell_pos.x).powi(2)
                    + (transform.translation.z - spell_pos.z).powi(2))
                .sqrt();
                if dist <= DISPEL_RANGE {
                    Some((spell_entity, dist))
                } else {
                    None
                }
            })
            .min_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or(std::cmp::Ordering::Equal));

        if let Some((target_entity, _)) = nearest {
            commands.entity(entity).insert(DispelChanneling {
                target_entity,
                elapsed: 0.0,
            });
        }
    }
}

/// Despawns a spell effect entity and cleans up its pathfinding obstacle if applicable.
#[allow(clippy::too_many_arguments)]
fn despawn_spell_effect(
    commands: &mut Commands,
    spell_entity: Entity,
    wall_of_stone_query: &Query<&WallOfStone>,
    wall_of_fire_query: &Query<&WallOfFireEffect>,
    spike_growth_query: &Query<&SpikeGrowthZone>,
    grease_query: &Query<&GreaseZone>,
    meteor_fire_query: &Query<&MeteorGroundFire>,
    obstacle_events: &mut MessageWriter<ObstacleChanged>,
) {
    // Wall of Stone — blocked obstacle
    if let Ok(wall) = wall_of_stone_query.get(spell_entity) {
        let unbuffered_min_x =
            wall.center.x - wall.forward.x * wall.half_length - wall.right.x * wall.half_width;
        let unbuffered_max_x =
            wall.center.x + wall.forward.x * wall.half_length + wall.right.x * wall.half_width;
        let unbuffered_min_z =
            wall.center.z - wall.forward.z * wall.half_length - wall.right.z * wall.half_width;
        let unbuffered_max_z =
            wall.center.z + wall.forward.z * wall.half_length + wall.right.z * wall.half_width;

        let min_x = unbuffered_min_x.min(unbuffered_max_x) - OBSTACLE_BUFFER;
        let max_x = unbuffered_min_x.max(unbuffered_max_x) + OBSTACLE_BUFFER;
        let min_z = unbuffered_min_z.min(unbuffered_max_z) - OBSTACLE_BUFFER;
        let max_z = unbuffered_min_z.max(unbuffered_max_z) + OBSTACLE_BUFFER;

        obstacle_events.write(ObstacleChanged {
            bounds: Rect::new(
                min_x.min(max_x),
                min_z.min(max_z),
                (max_x - min_x).abs(),
                (max_z - min_z).abs(),
            ),
            obstacle_type: ObstacleType::Removed,
        });
    }

    // Wall of Fire — hazard obstacle
    if let Ok(effect) = wall_of_fire_query.get(spell_entity) {
        let a = Vec2::new(effect.start.x, effect.start.z);
        let b = Vec2::new(effect.end.x, effect.end.z);
        let dir = b - a;
        let perp = Vec2::new(-dir.y, dir.x).normalize_or_zero() * effect.half_width;
        let c0 = a + perp;
        let c1 = a - perp;
        let c2 = b + perp;
        let c3 = b - perp;
        let min_x = c0.x.min(c1.x).min(c2.x).min(c3.x) - OBSTACLE_BUFFER;
        let max_x = c0.x.max(c1.x).max(c2.x).max(c3.x) + OBSTACLE_BUFFER;
        let min_y = c0.y.min(c1.y).min(c2.y).min(c3.y) - OBSTACLE_BUFFER;
        let max_y = c0.y.max(c1.y).max(c2.y).max(c3.y) + OBSTACLE_BUFFER;

        obstacle_events.write(ObstacleChanged {
            bounds: Rect::new(min_x, min_y, max_x - min_x, max_y - min_y),
            obstacle_type: ObstacleType::Removed,
        });
    }

    // Spike Growth — hazard obstacle (circular zone)
    if let Ok(zone) = spike_growth_query.get(spell_entity) {
        let origin_2d = Vec2::new(zone.origin.x, zone.origin.z);
        let buffered_radius = zone.radius + OBSTACLE_BUFFER;
        obstacle_events.write(ObstacleChanged {
            bounds: Rect::from_center_size(origin_2d, Vec2::splat(buffered_radius * 2.0)),
            obstacle_type: ObstacleType::Removed,
        });
    }

    // Grease — hazard obstacle when ignited
    if let Ok(zone) = grease_query.get(spell_entity)
        && zone.ignited
    {
        let origin_2d = Vec2::new(zone.origin.x, zone.origin.z);
        let buffered_radius = zone.radius + OBSTACLE_BUFFER;
        obstacle_events.write(ObstacleChanged {
            bounds: Rect::from_center_size(origin_2d, Vec2::splat(buffered_radius * 2.0)),
            obstacle_type: ObstacleType::Removed,
        });
    }

    // Meteor Ground Fire — hazard obstacle
    if let Ok(fire) = meteor_fire_query.get(spell_entity) {
        let origin_2d = Vec2::new(fire.origin.x, fire.origin.z);
        let buffered = fire.radius + OBSTACLE_BUFFER;
        obstacle_events.write(ObstacleChanged {
            bounds: Rect::from_center_size(origin_2d, Vec2::splat(buffered * 2.0)),
            obstacle_type: ObstacleType::Removed,
        });
    }

    commands.entity(spell_entity).despawn();
}

/// Fires weak magic bolts at enemies when no spell effects exist to dispel.
#[allow(clippy::type_complexity)]
pub fn dispeller_ranged_combat(
    mut commands: Commands,
    time: Res<Time>,
    dispeller_assets: Res<DispellerAssets>,
    spell_effects: Query<&NetworkedSpellEffect>,
    mut dispellers: Query<
        (
            Entity,
            &Transform,
            &Team,
            &mut DispellerAttackTimer,
            Option<&MesmerizedModifier>,
            Option<&SleepModifier>,
            Option<&BanishedModifier>,
        ),
        (With<Dispeller>, Without<Corpse>, Without<DispelChanneling>),
    >,
    targets: Query<(Entity, &Transform, &Team), Without<Corpse>>,
) {
    // Only fire bolts when no dispellable spell effects exist
    let has_spell_targets = spell_effects.iter().any(|nse| is_dispellable(nse.kind));
    if has_spell_targets {
        return;
    }

    let delta = time.delta_secs();

    for (
        dispeller_entity,
        dispeller_transform,
        dispeller_team,
        mut attack_timer,
        mesmerized,
        sleeping,
        banished,
    ) in &mut dispellers
    {
        attack_timer.time_since_last_attack += delta;

        // Skip if CC'd
        if mesmerized.is_some() || sleeping.is_some() || banished.is_some() {
            continue;
        }

        // Check cooldown
        if attack_timer.time_since_last_attack < ATTACK_COOLDOWN {
            continue;
        }

        // Find nearest enemy within attack range
        let nearest_enemy = targets
            .iter()
            .filter(|(entity, _, team)| {
                *entity != dispeller_entity
                    && match (*dispeller_team, *team) {
                        (Team::Undead, Team::Undead) => false,
                        (Team::Undead, _) => true,
                        (_, Team::Undead) => true,
                        _ => **team != *dispeller_team,
                    }
            })
            .filter(|(_, transform, _)| {
                let distance = dispeller_transform
                    .translation
                    .distance(transform.translation);
                distance <= ATTACK_RANGE
            })
            .min_by(|a, b| {
                let dist_a = dispeller_transform.translation.distance(a.1.translation);
                let dist_b = dispeller_transform.translation.distance(b.1.translation);
                dist_a.partial_cmp(&dist_b).unwrap()
            });

        if let Some((_, target_transform, _)) = nearest_enemy {
            // Spawn bolt toward target
            let origin = dispeller_transform.translation + Vec3::Y * 10.0;
            let diff = target_transform.translation - origin;
            let direction = Vec3::new(diff.x, 0.0, diff.z).normalize_or_zero();

            commands.spawn((
                Mesh3d(dispeller_assets.bolt_mesh.clone()),
                MeshMaterial3d(dispeller_assets.bolt_material.clone()),
                Transform::from_translation(origin),
                DispellerBolt {
                    velocity: direction * BOLT_SPEED,
                    damage: BOLT_DAMAGE,
                    source_team: *dispeller_team,
                    lifetime: BOLT_LIFETIME,
                },
                Billboard,
                OnGameplayScreen,
            ));

            attack_timer.time_since_last_attack = 0.0;
        }
    }
}

/// Moves dispeller bolts and despawns expired ones.
pub fn move_dispeller_bolts(
    mut commands: Commands,
    time: Res<Time>,
    mut bolts: Query<(Entity, &mut Transform, &mut DispellerBolt)>,
) {
    let delta = time.delta_secs();
    for (entity, mut transform, mut bolt) in &mut bolts {
        transform.translation += bolt.velocity * delta;
        bolt.lifetime -= delta;
        if bolt.lifetime <= 0.0 {
            commands.entity(entity).despawn();
        }
    }
}

/// Checks bolt collisions with enemy units.
pub fn check_bolt_collisions(
    mut commands: Commands,
    bolts: Query<(Entity, &Transform, &DispellerBolt)>,
    mut targets: Query<
        (
            &Transform,
            &Hitbox,
            &Team,
            &mut Health,
            Option<&mut TemporaryHitPoints>,
        ),
        Without<Corpse>,
    >,
) {
    #[allow(clippy::significant_drop_in_scrutinee)]
    for (bolt_entity, bolt_transform, bolt) in &bolts {
        let bolt_pos = bolt_transform.translation;

        for (target_transform, hitbox, team, mut health, mut temp_hp) in &mut targets {
            // Skip same team
            if *team == bolt.source_team {
                continue;
            }

            // Check if enemy
            let is_enemy = match (bolt.source_team, *team) {
                (Team::Undead, Team::Undead) => false,
                (Team::Undead, _) => true,
                (_, Team::Undead) => true,
                _ => *team != bolt.source_team,
            };

            if !is_enemy {
                continue;
            }

            // Check collision
            let distance = bolt_pos.distance(target_transform.translation);
            if distance < hitbox.radius + BOLT_RADIUS {
                apply_damage_to_unit(&mut health, temp_hp.as_deref_mut(), bolt.damage);
                commands.entity(bolt_entity).despawn();
                break;
            }
        }
    }
}

/// Spawns a single defender dispeller unit at a specific index.
pub(in crate::game) fn spawn_single_defender_dispeller(
    commands: &mut Commands,
    dispeller_assets: &DispellerAssets,
    unit_index: u32,
) {
    // Calculate position: dispellers go in the row behind archers
    let infantry_cells = cells_needed(INITIAL_DEFENDER_COUNT);
    let infantry_rows = infantry_cells.div_ceil(DEFENDER_GRID_COLS);
    let last_infantry_row = DEFENDER_GRID_ROWS.saturating_sub(infantry_rows);
    let archer_row = last_infantry_row.saturating_sub(1);
    let dispeller_row = archer_row.saturating_sub(1);

    let dispeller_cells_needed = cells_needed(INITIAL_DISPELLER_DEFENDER_COUNT);
    let units_per_cell = distribute_units_to_cells(INITIAL_DISPELLER_DEFENDER_COUNT);

    let mut units_counted = 0;
    for cell_idx in 0..dispeller_cells_needed.min(DEFENDER_GRID_COLS) {
        let units_in_this_cell = units_per_cell[cell_idx as usize];
        if unit_index < units_counted + units_in_this_cell {
            let (spawn_x, spawn_z) = calculate_defender_grid_position(dispeller_row, cell_idx);
            let (final_x, final_z) = random_position_in_cell(spawn_x, spawn_z);

            let hitbox = Hitbox::new(DISPELLER_RADIUS, DEFENDER_HITBOX_HEIGHT);
            let spawn_y = hitbox.height / 2.0 + 1.0;

            commands
                .spawn((
                    Mesh3d(dispeller_assets.mesh.clone()),
                    MeshMaterial3d(dispeller_assets.defender_material.clone()),
                    Transform::from_xyz(final_x, spawn_y, final_z),
                    Velocity::default(),
                    Acceleration::new(),
                    hitbox,
                    Health::new(DISPELLER_HEALTH),
                    MovementSpeed(DISPELLER_MOVEMENT_SPEED),
                    AttackTiming::new(),
                    Effectiveness::new(),
                    Team::Defenders,
                    Dispeller,
                ))
                .insert((
                    DispellerAttackTimer::new(),
                    TargetingVelocity::default(),
                    FlockingVelocity::default(),
                    FlowFieldVelocity::default(),
                    FlowFieldInfluence::Defender {
                        spawn_pos: Vec2::new(spawn_x, spawn_z),
                    },
                    Teleportable,
                    Billboard,
                    OnGameplayScreen,
                ));
            return;
        }
        units_counted += units_in_this_cell;
    }
}

/// Spawns a single attacker dispeller unit at a specific index.
pub(in crate::game) fn spawn_single_attacker_dispeller(
    commands: &mut Commands,
    dispeller_assets: &DispellerAssets,
    unit_index: u32,
    level: u32,
) {
    let total_dispellers = calculate_attacker_dispellers(level);
    let total_infantry = calculate_total_infantry(level);
    let total_archers = calculate_total_archers(level);
    let infantry_cells_needed = cells_needed(total_infantry);
    let archer_cells_needed = cells_needed(total_archers);

    // Dispellers go in the row behind archers
    let (_, archer_cells) = calculate_spawn_cells(infantry_cells_needed, archer_cells_needed);
    let last_archer_row = archer_cells.iter().map(|&(r, _)| r).max().unwrap_or(0);
    let dispeller_row = last_archer_row + 1;

    let col_fill_order: [u32; 6] = [2, 3, 1, 4, 0, 5];
    let units_per_cell = distribute_units_to_cells(total_dispellers);

    let mut units_counted = 0;
    for (cell_idx, &col) in col_fill_order.iter().enumerate() {
        if cell_idx >= units_per_cell.len() {
            break;
        }
        let units_in_this_cell = units_per_cell[cell_idx];
        if unit_index < units_counted + units_in_this_cell {
            let (spawn_x, spawn_z) = calculate_grid_cell_position(dispeller_row, col);
            let (final_x, final_z) = random_position_in_cell(spawn_x, spawn_z);

            let hitbox = Hitbox::new(DISPELLER_RADIUS, ATTACKER_HITBOX_HEIGHT);
            let spawn_y = hitbox.height / 2.0 + 1.0;

            commands
                .spawn((
                    Mesh3d(dispeller_assets.mesh.clone()),
                    MeshMaterial3d(dispeller_assets.attacker_material.clone()),
                    Transform::from_xyz(final_x, spawn_y, final_z),
                    Velocity::default(),
                    Acceleration::new(),
                    hitbox,
                    Health::new(DISPELLER_HEALTH),
                    MovementSpeed(DISPELLER_MOVEMENT_SPEED),
                    AttackTiming::new(),
                    Effectiveness::new(),
                    Team::Attackers,
                    Dispeller,
                ))
                .insert((
                    DispellerAttackTimer::new(),
                    TargetingVelocity::default(),
                    FlockingVelocity::default(),
                    FlowFieldVelocity::default(),
                    FlowFieldInfluence::Attacker,
                    Teleportable,
                    Billboard,
                    OnGameplayScreen,
                ));
            return;
        }
        units_counted += units_in_this_cell;
    }
}
