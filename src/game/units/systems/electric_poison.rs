use bevy::prelude::*;
use rand::Rng;

use super::super::components::{
    Airborne, Corpse, Effectiveness, FALL_DAMAGE_SCALE, Health, PendingDamageEffect,
    PoisonedModifier, Shocked, SickenedModifier, SmellyModifier, Team, TemporaryHitPoints,
    apply_damage_to_unit,
};
use super::super::constants::{
    ELECTRIC_ARC_COLOR, ELECTRIC_ARC_DAMAGE, ELECTRIC_ARC_LIFETIME, ELECTRIC_ARC_MAX_TARGETS,
    ELECTRIC_ARC_RANGE, ELECTRIC_ARC_WIDTH, POISON_TICK_INTERVAL, SICKENED_DURATION,
    SICKENED_THRESHOLD, SMELLY_DURATION,
};
use crate::game::units::wizard::archetypes::meteorologist::components::{
    ChargedModifier, WetModifier,
};
use crate::game::units::wizard::archetypes::meteorologist::constants::CHARGED_EXTRA_ARC_TARGETS;

/// Visual marker for electric arc effects (auto-despawns after lifetime).
#[derive(Component)]
pub struct ElectricArcVisual {
    pub lifetime: f32,
    pub time_alive: f32,
}

/// Ticks Shocked on affected units, rolls for arcs, and spawns arc visuals.
///
/// Arc damage is applied directly to health without inserting `PendingDamageEffect`,
/// so it does **not** propagate the electric debuff to arc targets.
#[allow(clippy::too_many_arguments)]
pub fn update_shocked(
    mut commands: Commands,
    time: Res<Time>,
    mut game_rng: ResMut<crate::game::seeded_rng::resources::GameRng>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut charge_query: Query<
        (
            Entity,
            &mut Shocked,
            &Transform,
            &Team,
            Has<ChargedModifier>,
        ),
        (
            Without<Corpse>,
            // Ghosts: host owns Shocked → CRDT propagates damage; skip locally.
            Without<crate::game::multiplayer::components::GhostEntity>,
        ),
    >,
    target_query: Query<
        (Entity, &Transform, &Team),
        (
            Without<Corpse>,
            // Electric arcs are spell damage — they must not jump to
            // spell-immune staging attackers.
            Without<crate::game::pathfinding::StagingAttacker>,
        ),
    >,
    mut health_query: Query<
        (
            &mut Health,
            Option<&mut TemporaryHitPoints>,
            Has<WetModifier>,
        ),
        Without<Corpse>,
    >,
) {
    use crate::game::components::OnGameplayScreen;

    let delta = time.delta_secs();

    // Collect arc events to process after iteration (avoids borrow conflicts)
    let mut arc_events: Vec<(Vec3, Entity, Vec3)> = Vec::new();

    for (entity, mut charge, transform, _team, has_charged) in charge_query.iter_mut() {
        let expired = charge.update(delta);
        if expired {
            commands.entity(entity).remove::<Shocked>();
            continue;
        }

        if !charge.can_arc() {
            continue;
        }

        // Roll for arc
        let roll: f32 = game_rng.0.random();
        if roll >= charge.arc_chance {
            continue;
        }

        // Find nearby arc targets (any non-corpse unit)
        let source_pos = transform.translation;
        let mut targets: Vec<(Entity, Vec3, f32)> = target_query
            .iter()
            .filter(|(target_entity, _, _)| *target_entity != entity)
            .filter_map(|(target_entity, target_transform, _)| {
                let dist = Vec3::new(
                    source_pos.x - target_transform.translation.x,
                    0.0,
                    source_pos.z - target_transform.translation.z,
                )
                .length();
                if dist <= ELECTRIC_ARC_RANGE {
                    Some((target_entity, target_transform.translation, dist))
                } else {
                    None
                }
            })
            .collect();

        if targets.is_empty() {
            continue;
        }

        // Sort by distance and take up to max targets
        // Storm synergy: charged units arc to extra targets
        let max_targets = if has_charged {
            ELECTRIC_ARC_MAX_TARGETS + CHARGED_EXTRA_ARC_TARGETS
        } else {
            ELECTRIC_ARC_MAX_TARGETS
        };
        targets.sort_by(|a, b| a.2.partial_cmp(&b.2).unwrap_or(std::cmp::Ordering::Equal));
        targets.truncate(max_targets);

        charge.reset_arc_cooldown();

        for (target_entity, target_pos, _) in &targets {
            arc_events.push((source_pos, *target_entity, *target_pos));
        }
    }

    if arc_events.is_empty() {
        return;
    }

    // Process arc events: deal damage and spawn visuals
    let arc_material = materials.add(StandardMaterial {
        base_color: ELECTRIC_ARC_COLOR,
        unlit: true,
        ..default()
    });
    let arc_mesh = meshes.add(Rectangle::new(ELECTRIC_ARC_WIDTH, ELECTRIC_ARC_WIDTH));

    for (source_pos, target_entity, target_pos) in arc_events {
        // Apply arc damage directly (bypasses PendingDamageEffect so it does NOT
        // propagate the Shocked debuff to arc targets).
        // Wet units take extra electric arc damage.
        if let Ok((mut health, mut temp_hp, is_wet)) = health_query.get_mut(target_entity) {
            let damage = if is_wet {
                ELECTRIC_ARC_DAMAGE
                    * crate::game::terrain::pond::constants::WET_ELECTRIC_DAMAGE_MULTIPLIER
            } else {
                ELECTRIC_ARC_DAMAGE
            };
            apply_damage_to_unit(&mut health, temp_hp.as_deref_mut(), damage);
        }

        // Spawn arc visual (simple straight line between source and target)
        let midpoint = (source_pos + target_pos) / 2.0;
        let direction = (target_pos - source_pos).normalize_or_zero();
        let length = source_pos.distance(target_pos);
        let rotation = Quat::from_rotation_arc(Vec3::Y, direction);

        commands.spawn((
            ElectricArcVisual {
                lifetime: ELECTRIC_ARC_LIFETIME,
                time_alive: 0.0,
            },
            Mesh3d(arc_mesh.clone()),
            MeshMaterial3d(arc_material.clone()),
            Transform::from_translation(midpoint)
                .with_rotation(rotation)
                .with_scale(Vec3::new(1.0, length / ELECTRIC_ARC_WIDTH, 1.0)),
            OnGameplayScreen,
        ));
    }
}

/// Ticks poison timer on affected units and checks for sickened threshold.
///
/// When poison expires naturally, the modifier is removed and effectiveness restored.
/// When accumulated poison reaches the threshold, poison is removed and replaced
/// with sickened (stun), and a UnitSickenedMessage is sent for achievement tracking.
pub fn update_poisoned(
    mut commands: Commands,
    time: Res<Time>,
    mut query: Query<
        (Entity, &mut PoisonedModifier, &mut Effectiveness),
        Without<crate::game::multiplayer::components::GhostEntity>,
    >,
    mut sickened_events: MessageWriter<crate::game::achievements::messages::UnitSickenedMessage>,
) {
    let delta = time.delta_secs();

    for (entity, mut poison, mut effectiveness) in query.iter_mut() {
        // Check sickened threshold first
        if poison.is_sickened(SICKENED_THRESHOLD) {
            // Remove all penalty applied to spell_bonus
            effectiveness.spell_bonus -= poison.applied_to_spell_bonus;
            commands.entity(entity).remove::<PoisonedModifier>();
            commands
                .entity(entity)
                .insert(SickenedModifier::new(SICKENED_DURATION));
            sickened_events.write(crate::game::achievements::messages::UnitSickenedMessage);
            continue;
        }

        let expired = poison.update(delta);

        if expired {
            // Remove all penalty applied to spell_bonus
            effectiveness.spell_bonus -= poison.applied_to_spell_bonus;
            commands.entity(entity).remove::<PoisonedModifier>();
        } else {
            // Apply effectiveness penalty via tick timer
            poison.tick_timer += delta;
            if poison.tick_timer >= POISON_TICK_INTERVAL {
                poison.tick_timer = 0.0;
                effectiveness.spell_bonus += poison.effectiveness_penalty;
                poison.applied_to_spell_bonus += poison.effectiveness_penalty;
            }
        }
    }
}

/// Ticks sickened timer and replaces with smelly when expired.
pub fn update_sickened(
    mut commands: Commands,
    time: Res<Time>,
    mut query: Query<
        (Entity, &mut SickenedModifier),
        Without<crate::game::multiplayer::components::GhostEntity>,
    >,
) {
    let delta = time.delta_secs();

    for (entity, mut sickened) in query.iter_mut() {
        if sickened.update(delta) {
            commands.entity(entity).remove::<SickenedModifier>();
            commands
                .entity(entity)
                .insert(SmellyModifier::new(SMELLY_DURATION));
        }
    }
}

/// Updates and despawns electric arc visuals after their lifetime expires.
pub fn update_electric_arc_visuals(
    mut commands: Commands,
    time: Res<Time>,
    mut query: Query<(Entity, &mut ElectricArcVisual)>,
) {
    let delta = time.delta_secs();

    for (entity, mut arc) in query.iter_mut() {
        arc.time_alive += delta;
        if arc.time_alive >= arc.lifetime {
            commands.entity(entity).try_despawn();
        }
    }
}

/// Updates airborne units: applies gravity, offsets Y visually, and deals
/// velocity-based fall damage on landing. Any system can make a unit airborne
/// by inserting the `Airborne` component with a launch velocity and gravity.
pub fn update_airborne_units(
    mut commands: Commands,
    time: Res<Time>,
    mut units: Query<
        (
            Entity,
            &mut Transform,
            &mut Airborne,
            &mut Health,
            Option<&mut TemporaryHitPoints>,
        ),
        // Defense-in-depth chokepoint: a staging unit must never be airborne
        // (all launch sources are staging-filtered), but if a future spell
        // forgets its filter this keeps fall damage off spell-immune units.
        Without<crate::game::pathfinding::StagingAttacker>,
    >,
) {
    let delta = time.delta_secs();
    for (entity, mut transform, mut airborne, mut health, mut temp_hp) in &mut units {
        // Apply gravity
        airborne.vertical_velocity -= airborne.gravity * delta;
        airborne.height += airborne.vertical_velocity * delta;

        if airborne.height <= 0.0 {
            // Landed — apply velocity-based fall damage and restore position
            airborne.height = 0.0;
            transform.translation.y = airborne.base_y;

            let impact_velocity = airborne.vertical_velocity.abs();
            let fall_damage = impact_velocity * FALL_DAMAGE_SCALE;
            if fall_damage > 0.0 {
                apply_damage_to_unit(&mut health, temp_hp.as_deref_mut(), fall_damage);
                commands.entity(entity).insert(PendingDamageEffect {
                    damage_type: airborne.damage_type,
                    damage: fall_damage,
                    source_team: None,
                });
            }
            commands.entity(entity).remove::<Airborne>();
        } else {
            // Offset the visual Y position during flight
            transform.translation.y = airborne.base_y + airborne.height;
        }
    }
}
