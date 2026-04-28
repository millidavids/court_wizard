use bevy::math::Affine2;
use bevy::prelude::*;
use rand::Rng;

use super::components::{
    ANIMATION_MOVE_THRESHOLD_SQ, AnimationOverride, CORPSE_MATERIAL_VARIANTS, CombatAnimation,
    DyingAnimation, FacingDirection, FacingDwell, FacingHysteresisBoost, PulsingAnimation,
    SPRITE_FRAME_SIZE, SPRITE_SHEET_IMAGE_HEIGHT, SmoothedFacingVelocity, WalkingAnimation,
};
use super::components::{
    Airborne, BanishedModifier, Corpse, Effectiveness, ElectricCharge, FALL_DAMAGE_SCALE,
    FearModifier, FireDoT, FlockingVelocity, FrostAccumulation, FrozenSolidModifier, Health,
    Hitbox, InMelee, MindControlled, OriginalMaterial, PendingDamageEffect, Petrified,
    PoisonedModifier, RemoteElectricEffect, RemoteFireEffect, RemoteFrostEffect, RootedModifier,
    SickenedModifier, SlowMovementModifier, SmellyModifier, Stunned, TargetingVelocity, Team,
    TemporaryHitPoints, TimedModifier, apply_damage_to_unit,
};
use super::constants::{
    BERSERKER_RAGE_EFFECT_COLOR, BERSERKER_RAGE_EFFECT_INTENSITY, ELECTRIC_ARC_COLOR,
    ELECTRIC_ARC_DAMAGE, ELECTRIC_ARC_LIFETIME, ELECTRIC_ARC_MAX_TARGETS, ELECTRIC_ARC_RANGE,
    ELECTRIC_ARC_WIDTH, ELECTRIC_EFFECT_COLOR, ELECTRIC_EFFECT_FLICKER_SPEED,
    ELECTRIC_EFFECT_MAX_INTENSITY, ELECTRIC_EFFECT_MIN_INTENSITY, ELITE_EFFECT_COLOR,
    ELITE_EFFECT_MAX_INTENSITY, ELITE_EFFECT_MIN_INTENSITY, ELITE_EFFECT_PULSE_SPEED,
    FROST_ACCUMULATION_PER_HIT, FROST_EFFECT_COLOR, FROST_EFFECT_MAX_INTENSITY,
    FROST_GENERIC_DECAY_DELAY, MIND_CONTROL_EFFECT_COLOR, MIND_CONTROL_EFFECT_INTENSITY,
    POISON_DURATION, POISON_EFFECT_COLOR, POISON_EFFECT_INTENSITY, POISON_EFFECTIVENESS_CAP,
    POISON_EFFECTIVENESS_PER_STACK, SHIELD_EFFECT_COLOR, SHIELD_EFFECT_MAX_INTENSITY,
    SHIELD_EFFECT_MIN_INTENSITY, SHIELD_EFFECT_PULSE_SPEED, SICKENED_DURATION,
    SICKENED_EFFECT_COLOR, SICKENED_EFFECT_INTENSITY, SICKENED_THRESHOLD, SMELLY_DURATION,
    SMELLY_EFFECT_COLOR, SMELLY_EFFECT_INTENSITY, UNIT_TYPE_GLOW_MAX_INTENSITY,
    UNIT_TYPE_GLOW_MIN_INTENSITY, UNIT_TYPE_GLOW_PULSE_SPEED, WET_EFFECT_COLOR,
    WET_EFFECT_INTENSITY,
};
use super::damage::DamageType;
use super::king::components::SpellShield;
use crate::config::GameConfig;
use crate::config::WizardType;
use crate::game::components::{Acceleration, Billboard, OnGameplayScreen, Velocity};
use crate::game::constants::{
    DEFENDER_HITBOX_HEIGHT, GLOBAL_SPEED_MULTIPLIER, MELEE_SLOWDOWN_DISTANCE,
    MELEE_SLOWDOWN_FACTOR, STEERING_FORCE, VELOCITY_DAMPING,
};
use crate::game::pathfinding::FlowFieldVelocity;
use crate::game::units::components::BerserkerRageModifier;
use crate::game::units::wizard::archetypes::meteorologist::components::{
    BurningPatch, ChargedModifier, ColdModifier, DryModifier, WetModifier,
};
use crate::game::units::wizard::archetypes::meteorologist::constants::{
    BURNING_PATCH_COLOR, BURNING_PATCH_DPS, BURNING_PATCH_LIFETIME, BURNING_PATCH_RADIUS,
    BURNING_PATCH_TICK_INTERVAL, CHARGED_EXTRA_ARC_TARGETS, COLD_FREEZE_DURATION,
    COLD_FROST_SLOW_MULTIPLIER, DRY_BURNING_PATCH_COUNT, DRY_BURNING_PATCH_SCATTER,
    WET_FIRE_DOT_MULTIPLIER,
};
use crate::game::units::wizard::spells::mind_control::components::MassHysteriaTarget;

/// Returns true if the unit is immobilized by any crowd control effect.
/// Centralizes CC checks so new CC types only need updating here.
/// Sleepwalking units (Dreamwalker talent) are NOT immobilized.
#[inline]
#[allow(clippy::too_many_arguments)]
pub fn is_cc_immobilized(
    rooted: Option<&RootedModifier>,
    has_sleep: bool,
    has_sleepwalking: bool,
    banished: Option<&BanishedModifier>,
    sickened: Option<&SickenedModifier>,
    frozen: Option<&FrozenSolidModifier>,
    stunned: Option<&Stunned>,
    petrified: Option<&Petrified>,
) -> bool {
    rooted.is_some()
        || (has_sleep && !has_sleepwalking)
        || banished.is_some()
        || sickened.is_some()
        || frozen.is_some()
        || stunned.is_some()
        || petrified.is_some()
}

/// Returns true if the unit is a staging attacker (not yet activated).
/// Handles the 1-frame deferred command delay: units without WaveGroup yet
/// are also considered staging (they haven't been tagged by `tag_new_attackers`).
#[inline]
pub fn is_staging_attacker(team: &Team, has_staging: bool, has_wave_group: bool) -> bool {
    *team == Team::Attackers && (has_staging || !has_wave_group)
}

/// Generic targeting system for melee units.
///
/// Finds the nearest enemy using team-based logic and updates targeting velocity.
/// Also manages the InMelee component based on distance to enemy.
///
/// If `retaliation_target` is `Some(entity)`, that entity is also considered a valid
/// target regardless of team (used when a mind-controlled unit attacks an ally).
#[inline]
pub fn update_melee_unit_targeting(
    unit_snapshot: &[(Entity, Vec3, Team)],
    entity: Entity,
    transform: &Transform,
    team: Team,
    targeting_velocity: &mut TargetingVelocity,
    commands: &mut Commands,
    retaliation_target: Option<Entity>,
) {
    // Find nearest enemy using team-based targeting logic
    let nearest_enemy = unit_snapshot
        .iter()
        .filter(|(other_entity, _, other_team)| {
            *other_entity != entity
                && (retaliation_target == Some(*other_entity) || team.is_enemy(other_team))
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

    // Set targeting velocity toward target
    if let Some(&(_, target_pos, enemy_team)) = nearest_enemy {
        let direction = (target_pos - transform.translation).normalize_or_zero();
        targeting_velocity.velocity = Vec3::new(direction.x, 0.0, direction.z);

        // Store distance for formation weighting (XZ plane only)
        let dx = transform.translation.x - target_pos.x;
        let dz = transform.translation.z - target_pos.z;
        let distance = (dx * dx + dz * dz).sqrt();
        targeting_velocity.distance_to_target = distance;

        // Check if enemy is in melee range
        if distance < MELEE_SLOWDOWN_DISTANCE {
            commands.entity(entity).insert(InMelee(enemy_team));
        } else {
            commands.entity(entity).remove::<InMelee>();
        }
    } else {
        // No enemies found, clear targeting
        targeting_velocity.velocity = Vec3::ZERO;
        targeting_velocity.distance_to_target = f32::MAX;
        commands.entity(entity).remove::<InMelee>();
    }
}

/// Generic weighted movement system used by infantry, brute, and other melee units.
///
/// Combines three velocity sources with distance-based weighting:
/// - Flow field: Pathfinding around obstacles
/// - Flocking: Separation from nearby allies
/// - Targeting: Direct movement toward/away from enemies
///
/// This function implements the core movement logic and returns the final steering force.
#[inline]
#[allow(clippy::too_many_arguments)]
pub fn calculate_weighted_movement(
    time: &Time,
    velocity: &mut Velocity,
    acceleration: &mut Acceleration,
    movement_speed: f32,
    targeting_velocity: &TargetingVelocity,
    flocking_velocity: &FlockingVelocity,
    flow_field_velocity: &FlowFieldVelocity,
    in_melee: bool,
    commander_aura_modifier: Option<f32>,
    terrain_modifier: Option<f32>,
    slow_modifier: Option<f32>,
    cauldron_modifier: Option<f32>,
    haste_modifier: Option<f32>,
    elite_speed_modifier: Option<f32>,
) {
    // If the unit has reached its flow field destination and has no target,
    // stop entirely so flocking doesn't push it around between waves.
    let no_target = targeting_velocity.velocity.length_squared() < 0.001;
    if flow_field_velocity.at_destination && no_target {
        velocity.x *= VELOCITY_DAMPING.powf(time.delta_secs() * 60.0);
        velocity.z *= VELOCITY_DAMPING.powf(time.delta_secs() * 60.0);
        return;
    }

    // Use pathfinding distance (accounts for obstacles)
    let distance = flow_field_velocity.pathfinding_distance;

    // Distance-based weighting with interpolation
    // Far: prioritize pathfinding, Medium: balanced, Close: prioritize targeting
    let (mut flow_weight, mut flocking_weight, mut targeting_weight) = if distance > 500.0 {
        (0.7, 0.2, 0.1)
    } else if distance > 200.0 {
        // Interpolate between far and medium
        let t = (500.0 - distance) / 300.0;
        let flow = 0.7 - (0.2 * t);
        let targeting = 0.1 + (0.2 * t);
        (flow, 0.2, targeting)
    } else if distance > 50.0 {
        // Interpolate between medium and close
        let t = (200.0 - distance) / 150.0;
        let flow = 0.5 - (0.3 * t);
        let targeting = 0.3 + (0.3 * t);
        (flow, 0.2, targeting)
    } else {
        // In melee range
        (0.1, 0.1, 0.8)
    };

    // When pathfinding distance is INFINITY, the unit has no valid path to its goal
    // (e.g. completely enclosed by walls). Give targeting full weight so the unit
    // can move toward and attack the nearest wall to break free.
    if distance.is_infinite() && targeting_velocity.velocity.length_squared() > 0.001 {
        flow_weight = 0.0;
        flocking_weight = 0.0;
        targeting_weight = 1.0;
    }

    // On hazardous terrain, boost flow field weight so units follow the rerouted path
    // instead of charging through the hazard toward their target
    if !distance.is_infinite() && flow_field_velocity.terrain_cost > 1.0 {
        flow_weight = 0.8;
        flocking_weight = 0.1;
        targeting_weight = 0.1;
    }

    // When pathfinding distance is much larger than straight-line distance to the
    // target, a wall is likely between the unit and its target. Keep flow field
    // weight high so units navigate around the wall instead of pushing into it.
    // Skip when distance is INFINITY — those units are fully blocked and need
    // targeting weight to attack walls.
    if !distance.is_infinite() && targeting_velocity.velocity.length_squared() > 0.001 {
        let straight_line_distance = targeting_velocity.velocity.length();
        if straight_line_distance > 1.0
            && flow_field_velocity.pathfinding_distance > straight_line_distance * 2.0
        {
            flow_weight = flow_weight.max(0.7);
            targeting_weight = targeting_weight.min(0.1);
            flocking_weight = 1.0 - flow_weight - targeting_weight;
        }
    }

    // Combine three velocity sources with distance-based weighting
    let weighted_direction = (flow_field_velocity.velocity * flow_weight
        + flocking_velocity.velocity * flocking_weight
        + targeting_velocity.velocity * targeting_weight)
        .normalize_or_zero();

    // Calculate speed modifiers
    let aura_percentage = commander_aura_modifier.unwrap_or(0.0);
    let terrain_percentage = terrain_modifier.unwrap_or(0.0);
    let slow_percentage = slow_modifier.unwrap_or(0.0);
    let cauldron_percentage = cauldron_modifier.unwrap_or(0.0);
    let haste_percentage = haste_modifier.unwrap_or(0.0);
    let elite_speed_percentage = elite_speed_modifier.unwrap_or(0.0);
    let total_percentage = aura_percentage
        + terrain_percentage
        + slow_percentage
        + cauldron_percentage
        + haste_percentage
        + elite_speed_percentage;
    let speed_multiplier = (1.0 + total_percentage).max(0.0); // Clamp to prevent negative speed

    // Calculate max speed with modifiers and melee slowdown
    let mut max_speed = movement_speed * GLOBAL_SPEED_MULTIPLIER * speed_multiplier;
    if in_melee {
        max_speed *= MELEE_SLOWDOWN_FACTOR;
    }

    // Calculate steering force with clamping to prevent overshooting
    let desired_velocity = weighted_direction * max_speed;
    let velocity_change_needed = Vec3::new(
        desired_velocity.x - velocity.x,
        0.0,
        desired_velocity.z - velocity.z,
    );

    // Apply steering force, scaled by max_speed so faster units accelerate proportionally.
    // Without this, the fixed STEERING_FORCE creates an equilibrium speed (due to damping)
    // that caps all units at the same effective speed regardless of movement_speed.
    let speed_scale = max_speed / (200.0 * GLOBAL_SPEED_MULTIPLIER);
    let steering = velocity_change_needed.normalize_or_zero()
        * STEERING_FORCE
        * speed_multiplier
        * speed_scale;
    let steering_magnitude = steering.length();
    let max_steering = velocity_change_needed.length() / time.delta_secs();

    let final_steering = if steering_magnitude > max_steering && max_steering > 0.0 {
        steering.normalize() * max_steering
    } else {
        steering
    };

    acceleration.add_force(final_steering);

    // Apply smelly repulsion directly as a strong force, bypassing normal movement weighting
    if flocking_velocity.smelly_repulsion.length_squared() > 0.001 {
        let smelly_force = flocking_velocity.smelly_repulsion * max_speed;
        acceleration.add_force(smelly_force);
    }

    // Store max_speed for velocity clamping in apply_unit_movement
    velocity.max_speed = max_speed;

    // Apply damping to current velocity (allows external forces like black hole gravity)
    // Frame-rate independent: normalize to 60 FPS reference rate
    let damping = VELOCITY_DAMPING.powf(time.delta_secs() * 60.0);
    velocity.x *= damping;
    velocity.z *= damping;
}

/// Generic system that ticks all instances of a `TimedModifier` component and removes expired ones.
pub fn update_timed_modifier<
    T: Component<Mutability = bevy::ecs::component::Mutable> + TimedModifier,
>(
    mut commands: Commands,
    time: Res<Time>,
    mut query: Query<(Entity, &mut T)>,
) {
    let delta = time.delta_secs();
    for (entity, mut modifier) in query.iter_mut() {
        if modifier.tick(delta) {
            commands.entity(entity).remove::<T>();
        }
    }
}

/// Processes `PendingDamageEffect` markers and creates/stacks persistent effects.
///
/// Each frame, reads all PendingDamageEffect components, determines the damage type,
/// and either creates a new persistent effect component or stacks onto an existing one.
#[allow(clippy::too_many_arguments)]
pub fn process_pending_damage_effects(
    mut commands: Commands,
    mut game_rng: ResMut<crate::game::seeded_rng::resources::GameRng>,
    config: Res<GameConfig>,
    pending_query: Query<(
        Entity,
        &PendingDamageEffect,
        &Transform,
        Has<SpellShield>,
        Has<ColdModifier>,
        Has<DryModifier>,
    )>,
    mut fire_query: Query<&mut FireDoT>,
    mut frost_query: Query<&mut FrostAccumulation>,
    mut electric_query: Query<&mut ElectricCharge>,
    mut poison_query: Query<&mut PoisonedModifier>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    // Shared mesh/material for burning patches (Drought fire synergy), created once if needed
    let mut burning_patch_mesh: Option<Handle<Mesh>> = None;
    let mut burning_patch_material: Option<Handle<StandardMaterial>> = None;

    for (entity, pending, transform, has_shield, has_cold, has_dry) in pending_query.iter() {
        if has_shield {
            commands.entity(entity).remove::<PendingDamageEffect>();
            continue;
        }
        // Excremage converts all damage types to Poop
        let effective_type = if config.wizard_type == WizardType::Excremage {
            DamageType::Poop
        } else {
            pending.damage_type
        };
        match effective_type {
            DamageType::Fire => {
                if let Ok(mut fire_dot) = fire_query.get_mut(entity) {
                    fire_dot.stack(pending.damage);
                } else {
                    commands.entity(entity).insert(FireDoT::new(pending.damage));
                }
                // Drought synergy: fire spells create burning ground patches
                if has_dry {
                    let impact_pos = transform.translation;
                    let patch_mesh = burning_patch_mesh
                        .get_or_insert_with(|| meshes.add(Circle::new(BURNING_PATCH_RADIUS)));
                    let patch_material = burning_patch_material.get_or_insert_with(|| {
                        materials.add(StandardMaterial {
                            base_color: BURNING_PATCH_COLOR,
                            unlit: true,
                            alpha_mode: AlphaMode::Blend,
                            ..default()
                        })
                    });
                    for _ in 0..DRY_BURNING_PATCH_COUNT {
                        let offset_x = game_rng
                            .0
                            .random_range(-DRY_BURNING_PATCH_SCATTER..DRY_BURNING_PATCH_SCATTER);
                        let offset_z = game_rng
                            .0
                            .random_range(-DRY_BURNING_PATCH_SCATTER..DRY_BURNING_PATCH_SCATTER);
                        let patch_pos = Vec3::new(
                            impact_pos.x + offset_x,
                            0.5, // Just above ground
                            impact_pos.z + offset_z,
                        );
                        commands.spawn((
                            BurningPatch {
                                lifetime: BURNING_PATCH_LIFETIME,
                                radius: BURNING_PATCH_RADIUS,
                                damage_per_tick: BURNING_PATCH_DPS * BURNING_PATCH_TICK_INTERVAL,
                                tick_timer: BURNING_PATCH_TICK_INTERVAL,
                            },
                            Mesh3d(patch_mesh.clone()),
                            MeshMaterial3d(patch_material.clone()),
                            Transform::from_translation(patch_pos)
                                .with_rotation(Quat::from_rotation_x(-std::f32::consts::FRAC_PI_2)),
                            OnGameplayScreen,
                        ));
                    }
                }
            }
            DamageType::Frost => {
                // Frost damage drives accumulation (progressive slow + freeze)
                let frost_amount = if has_cold {
                    FROST_ACCUMULATION_PER_HIT * COLD_FROST_SLOW_MULTIPLIER
                } else {
                    FROST_ACCUMULATION_PER_HIT
                };

                if let Ok(mut frost) = frost_query.get_mut(entity) {
                    frost.add_frost(frost_amount, FROST_GENERIC_DECAY_DELAY);
                } else {
                    commands.entity(entity).insert(FrostAccumulation::new(
                        frost_amount,
                        FROST_GENERIC_DECAY_DELAY,
                    ));
                }
                // Blizzard synergy: frost + cold = freeze (brief root)
                if has_cold {
                    commands
                        .entity(entity)
                        .insert(RootedModifier::new(COLD_FREEZE_DURATION));
                }
            }
            DamageType::Electric => {
                if let Ok(mut charge) = electric_query.get_mut(entity) {
                    charge.stack(pending.damage);
                } else {
                    commands
                        .entity(entity)
                        .insert(ElectricCharge::new(pending.damage));
                }
            }
            DamageType::Poison => {
                if let Ok(mut poison) = poison_query.get_mut(entity) {
                    poison.stack(
                        POISON_EFFECTIVENESS_PER_STACK,
                        POISON_DURATION,
                        POISON_EFFECTIVENESS_CAP,
                    );
                } else {
                    commands.entity(entity).insert(PoisonedModifier::new(
                        POISON_EFFECTIVENESS_PER_STACK,
                        POISON_DURATION,
                    ));
                }
            }
            DamageType::Poop => {
                commands
                    .entity(entity)
                    .insert(SmellyModifier::new(SMELLY_DURATION));
            }
            // Force, Necrotic, Nature — no persistent effect
            _ => {}
        }

        commands.entity(entity).remove::<PendingDamageEffect>();
    }
}

/// Updates frost accumulation: decays over time, applies proportional slow,
/// and triggers freeze at max level.
pub fn update_frost_accumulation(
    time: Res<Time>,
    mut commands: Commands,
    mut query: Query<(
        Entity,
        &mut FrostAccumulation,
        Option<&mut SlowMovementModifier>,
        Has<FireDoT>,
    )>,
    storms: Query<&crate::game::units::wizard::spells::squall::components::SquallStorm>,
) {
    use crate::game::units::wizard::spells::squall::constants::{
        FROST_DECAY_RATE, FROST_FREEZE_DURATION, FROST_MAX_SLOW, PERMAFROST_FREEZE_DURATION,
    };

    /// Rate at which fire melts frost accumulation (per second).
    const FIRE_MELTS_FROST_RATE: f32 = 0.3;

    let delta = time.delta_secs();
    let has_permafrost = storms.iter().any(|s| s.talent_params.permafrost);
    let freeze_duration = if has_permafrost {
        PERMAFROST_FREEZE_DURATION
    } else {
        FROST_FREEZE_DURATION
    };

    for (entity, mut frost, slow_mod, has_fire) in &mut query {
        // Fire melts frost
        if has_fire {
            frost.level -= FIRE_MELTS_FROST_RATE * delta;
        }

        // Decay logic
        frost.decay_delay -= delta;
        if frost.decay_delay <= 0.0 {
            frost.level -= FROST_DECAY_RATE * delta;
        }

        // Remove if fully thawed
        if frost.level <= 0.0 {
            commands
                .entity(entity)
                .remove::<FrostAccumulation>()
                .remove::<SlowMovementModifier>();
            continue;
        }

        // Freeze at max
        if frost.level >= 1.0 {
            commands
                .entity(entity)
                .insert(FrozenSolidModifier::new(freeze_duration))
                .remove::<FrostAccumulation>()
                .remove::<SlowMovementModifier>();
            continue;
        }

        // Update slow proportionally (refreshed each frame)
        let slow_amount = frost.level * FROST_MAX_SLOW;
        if let Some(mut slow) = slow_mod {
            slow.modifier = slow_amount;
            slow.time_remaining = 0.5; // short refresh window
        } else {
            commands
                .entity(entity)
                .insert(SlowMovementModifier::new(slow_amount, 0.5));
        }
    }
}

/// Ticks FireDoT damage on affected units and removes expired DoTs.
///
/// DoT damage is applied directly to health (does not trigger more DoT).
pub fn update_fire_dot(
    mut commands: Commands,
    time: Res<Time>,
    mut query: Query<(
        Entity,
        &mut FireDoT,
        &mut Health,
        Option<&mut TemporaryHitPoints>,
        Has<SpellShield>,
        Has<WetModifier>,
        Option<&FrostAccumulation>,
    )>,
) {
    /// Rate at which frost quenches fire DPS (DPS reduction per second per frost level).
    const FROST_QUENCHES_FIRE_RATE: f32 = 5.0;

    let delta = time.delta_secs();

    for (entity, mut fire_dot, mut health, temp_hp, has_shield, is_wet, frost) in query.iter_mut() {
        // Frost quenches fire — reduce DPS proportional to frost level
        if let Some(frost) = frost {
            fire_dot.damage_per_tick = (fire_dot.damage_per_tick
                - FROST_QUENCHES_FIRE_RATE * frost.level * delta)
                .max(0.0);
        }

        let (tick_damage, expired) = fire_dot.update(delta);

        if let Some(damage) = tick_damage
            && !has_shield
        {
            // Storm synergy: fire DoT damage is reduced on wet units
            let effective_damage = if is_wet {
                damage * WET_FIRE_DOT_MULTIPLIER
            } else {
                damage
            };
            apply_damage_to_unit(
                &mut health,
                temp_hp.map(|t| t.into_inner()),
                effective_damage,
            );
        }

        if expired {
            commands.entity(entity).remove::<FireDoT>();
        }
    }
}

/// Spawn fire VFX (smoke, sparks, embers) on units with an active FireDoT.
pub fn emit_burning_unit_vfx(
    mut commands: Commands,
    burning_units: Query<(&Transform, &Hitbox), With<FireDoT>>,
    visual_assets: Res<crate::game::units::wizard::spells::visual_assets::SpellVisualAssets>,
    time: Res<Time>,
    mut smoke_timer: Local<f32>,
    mut spark_timer: Local<f32>,
) {
    let delta = time.delta_secs();
    let t = time.elapsed_secs();

    *smoke_timer += delta;
    *spark_timer += delta;

    let emit_smoke = *smoke_timer >= 0.5;
    let emit_sparks = *spark_timer >= 1.5;

    if emit_smoke {
        *smoke_timer -= 0.5;
    }
    if emit_sparks {
        *spark_timer -= 1.5;
    }

    if !emit_smoke && !emit_sparks {
        return;
    }

    for (transform, hitbox) in &burning_units {
        let pos = transform.translation;
        let half_height = hitbox.height * 0.5;
        let radius = hitbox.radius * 0.5;

        if emit_smoke {
            let seed = pos.x * 0.1 + t;
            let y_offset = ((seed * 7.3).sin() * 0.5) * half_height;
            crate::game::units::wizard::spells::vfx::systems::spawn_fire_orange_smoke(
                &mut commands,
                &visual_assets,
                Vec3::new(pos.x, pos.y + y_offset, pos.z),
                radius,
                1,
                t + seed,
            );
        }
        if emit_sparks {
            let seed = pos.z * 0.1 + t;
            let y_offset = ((seed * 11.1).sin() * 0.5) * half_height;
            crate::game::units::wizard::spells::vfx::systems::spawn_fire_sparks(
                &mut commands,
                &visual_assets,
                Vec3::new(pos.x, pos.y + y_offset, pos.z),
                1,
                t + seed,
            );
        }
    }
}

/// Visual marker for electric arc effects (auto-despawns after lifetime).
#[derive(Component)]
pub struct ElectricArcVisual {
    pub lifetime: f32,
    pub time_alive: f32,
}

/// Ticks ElectricCharge on affected units, rolls for arcs, and spawns arc visuals.
///
/// Arc damage is applied directly to health without inserting `PendingDamageEffect`,
/// so it does **not** propagate the electric debuff to arc targets.
#[allow(clippy::too_many_arguments)]
pub fn update_electric_charge(
    mut commands: Commands,
    time: Res<Time>,
    mut game_rng: ResMut<crate::game::seeded_rng::resources::GameRng>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut charge_query: Query<
        (
            Entity,
            &mut ElectricCharge,
            &Transform,
            &Team,
            Has<ChargedModifier>,
        ),
        Without<Corpse>,
    >,
    target_query: Query<(Entity, &Transform, &Team), Without<Corpse>>,
    mut health_query: Query<
        (
            &mut Health,
            Option<&mut TemporaryHitPoints>,
            Has<WetModifier>,
        ),
        Without<Corpse>,
    >,
) {
    let delta = time.delta_secs();

    // Collect arc events to process after iteration (avoids borrow conflicts)
    let mut arc_events: Vec<(Vec3, Entity, Vec3)> = Vec::new();

    for (entity, mut charge, transform, _team, has_charged) in charge_query.iter_mut() {
        let expired = charge.update(delta);
        if expired {
            commands.entity(entity).remove::<ElectricCharge>();
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
        // propagate the ElectricCharge debuff to arc targets).
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
    mut query: Query<(Entity, &mut PoisonedModifier, &mut Effectiveness)>,
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
            if poison.tick_timer >= super::constants::POISON_TICK_INTERVAL {
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
    mut query: Query<(Entity, &mut SickenedModifier)>,
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

/// Blends a pulsing color effect onto a linear color value.
///
/// Uses a sine wave to oscillate intensity between min and max bounds.
fn blend_pulsing_effect(
    result: &mut LinearRgba,
    effect_color: &LinearRgba,
    elapsed: f32,
    pulse_speed: f32,
    min_intensity: f32,
    max_intensity: f32,
) {
    let pulse = (elapsed * pulse_speed).sin() * 0.5 + 0.5;
    let intensity = min_intensity + pulse * (max_intensity - min_intensity);
    *result = result.mix(effect_color, intensity);
}

/// Updates visual tinting on units affected by persistent damage effects.
///
/// Considers both local effects (FireDoT, FrostEffectMarker, ElectricCharge) and
/// remote effect markers (RemoteFireEffect, etc.) from the other multiplayer peer.
///
/// Three-phase logic per entity:
/// 1. Unit has effects but no OriginalMaterial: clone the material, store original
/// 2. Unit has effects and OriginalMaterial: blend effect colors onto cloned material
/// 3. Unit has OriginalMaterial but no effects: restore original, remove OriginalMaterial
#[allow(clippy::type_complexity)]
pub fn update_persistent_effect_visuals(
    mut commands: Commands,
    time: Res<Time>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    query: Query<
        (
            Entity,
            &MeshMaterial3d<StandardMaterial>,
            Option<&FireDoT>,
            Option<&FrostAccumulation>,
            Option<&ElectricCharge>,
            Has<RemoteFireEffect>,
            Has<RemoteFrostEffect>,
            Has<RemoteElectricEffect>,
            Has<MindControlled>,
            Has<MassHysteriaTarget>,
            Option<&OriginalMaterial>,
            (
                Has<PoisonedModifier>,
                Has<SickenedModifier>,
                Has<SmellyModifier>,
                Has<BerserkerRageModifier>,
                Has<super::shielder::components::ShielderDamageReduction>,
                Has<super::elite::EliteHealthBonus>,
                Option<&super::components::UnitTypeGlow>,
                Has<WetModifier>,
                Has<Petrified>,
                Has<FearModifier>,
            ),
        ),
        Or<(
            With<FireDoT>,
            With<FrostAccumulation>,
            With<ElectricCharge>,
            With<RemoteFireEffect>,
            With<RemoteFrostEffect>,
            With<RemoteElectricEffect>,
            With<MindControlled>,
            With<MassHysteriaTarget>,
            With<OriginalMaterial>,
            With<PoisonedModifier>,
            With<SickenedModifier>,
            With<SmellyModifier>,
            With<BerserkerRageModifier>,
            Or<(
                With<super::shielder::components::ShielderDamageReduction>,
                With<super::elite::EliteHealthBonus>,
                With<super::components::UnitTypeGlow>,
                With<WetModifier>,
                With<Petrified>,
                With<FearModifier>,
            )>,
        )>,
    >,
) {
    let elapsed = time.elapsed_secs();

    // Pre-compute linear versions of constant effect colors (avoid per-entity conversion)
    let frost_linear = FROST_EFFECT_COLOR.to_linear();
    let wet_linear = WET_EFFECT_COLOR.to_linear();
    let electric_linear = ELECTRIC_EFFECT_COLOR.to_linear();
    let mc_linear = MIND_CONTROL_EFFECT_COLOR.to_linear();
    let poison_linear = POISON_EFFECT_COLOR.to_linear();
    let sickened_linear = SICKENED_EFFECT_COLOR.to_linear();
    let smelly_linear = SMELLY_EFFECT_COLOR.to_linear();
    let rage_linear = BERSERKER_RAGE_EFFECT_COLOR.to_linear();
    let shield_linear = SHIELD_EFFECT_COLOR.to_linear();
    let elite_linear = ELITE_EFFECT_COLOR.to_linear();

    for (
        entity,
        material_handle,
        fire,
        frost,
        electric,
        remote_fire,
        remote_frost,
        remote_electric,
        has_mind_control,
        has_mass_hysteria,
        original_mat,
        (
            has_poisoned,
            has_sickened,
            has_smelly,
            has_rage,
            has_shield_glow,
            has_elite,
            unit_type_glow,
            has_wet,
            has_petrified,
            has_fear,
        ),
    ) in &query
    {
        let has_fire = fire.is_some() || remote_fire;
        let has_frost = frost.is_some() || remote_frost;
        let has_electric = electric.is_some() || remote_electric;
        let has_mc_visual = has_mind_control || has_mass_hysteria;
        let has_any_effect = has_fire
            || has_frost
            || has_petrified
            || has_fear
            || has_electric
            || has_mc_visual
            || has_poisoned
            || has_sickened
            || has_smelly
            || has_rage
            || has_shield_glow
            || has_elite
            || has_wet
            || unit_type_glow.is_some();

        if has_any_effect && original_mat.is_none() {
            // Phase 1: First effect applied — clone the material and store original
            let current_handle = material_handle.0.clone();
            let Some(current_material) = materials.get(&current_handle) else {
                continue;
            };
            let cloned = current_material.clone();
            let cloned_handle = materials.add(cloned);
            commands
                .entity(entity)
                .queue_silenced(move |mut e: EntityWorldMut| {
                    e.insert((
                        OriginalMaterial(current_handle),
                        MeshMaterial3d(cloned_handle),
                    ));
                });
        } else if has_any_effect {
            // Phase 2: Blend effect colors onto the cloned material
            let Some(original) = original_mat else {
                continue;
            };
            let Some(original_material) = materials.get(&original.0) else {
                continue;
            };
            let base_linear = original_material.base_color.to_linear();

            let mut result_linear = base_linear;

            // Fire tint removed — burning is shown via particle VFX only.

            if has_frost {
                let frost_intensity = frost
                    .map(|f| f.level * FROST_EFFECT_MAX_INTENSITY)
                    .unwrap_or(FROST_EFFECT_MAX_INTENSITY * 0.5);
                result_linear = result_linear.mix(&frost_linear, frost_intensity);
            }

            if has_wet {
                result_linear = result_linear.mix(&wet_linear, WET_EFFECT_INTENSITY);
            }

            if has_electric {
                blend_pulsing_effect(
                    &mut result_linear,
                    &electric_linear,
                    elapsed,
                    ELECTRIC_EFFECT_FLICKER_SPEED,
                    ELECTRIC_EFFECT_MIN_INTENSITY,
                    ELECTRIC_EFFECT_MAX_INTENSITY,
                );
            }

            if has_mc_visual {
                result_linear = result_linear.mix(&mc_linear, MIND_CONTROL_EFFECT_INTENSITY);
            }

            if has_poisoned {
                result_linear = result_linear.mix(&poison_linear, POISON_EFFECT_INTENSITY);
            }

            if has_sickened {
                result_linear = result_linear.mix(&sickened_linear, SICKENED_EFFECT_INTENSITY);
            }

            if has_smelly {
                result_linear = result_linear.mix(&smelly_linear, SMELLY_EFFECT_INTENSITY);
            }

            if has_rage {
                result_linear = result_linear.mix(&rage_linear, BERSERKER_RAGE_EFFECT_INTENSITY);
            }

            if has_shield_glow {
                blend_pulsing_effect(
                    &mut result_linear,
                    &shield_linear,
                    elapsed,
                    SHIELD_EFFECT_PULSE_SPEED,
                    SHIELD_EFFECT_MIN_INTENSITY,
                    SHIELD_EFFECT_MAX_INTENSITY,
                );
            }

            if has_elite {
                blend_pulsing_effect(
                    &mut result_linear,
                    &elite_linear,
                    elapsed,
                    ELITE_EFFECT_PULSE_SPEED,
                    ELITE_EFFECT_MIN_INTENSITY,
                    ELITE_EFFECT_MAX_INTENSITY,
                );
            }

            if let Some(glow) = unit_type_glow {
                let glow_linear = glow.color.to_linear();
                blend_pulsing_effect(
                    &mut result_linear,
                    &glow_linear,
                    elapsed,
                    UNIT_TYPE_GLOW_PULSE_SPEED,
                    UNIT_TYPE_GLOW_MIN_INTENSITY,
                    UNIT_TYPE_GLOW_MAX_INTENSITY,
                );
            }

            if has_fear {
                let purple = LinearRgba::new(0.5, 0.1, 0.6, 1.0);
                result_linear = result_linear.mix(&purple, 0.3);
            }

            if has_petrified {
                let gray = LinearRgba::new(0.4, 0.4, 0.4, 1.0);
                result_linear = result_linear.mix(&gray, 0.85);
            }

            if let Some(cloned_material) = materials.get_mut(material_handle) {
                cloned_material.base_color = Color::from(result_linear);
            }
        } else if let Some(original) = original_mat {
            // Phase 3: All effects expired — restore original material
            let restored = original.0.clone();
            commands
                .entity(entity)
                .queue_silenced(move |mut e: EntityWorldMut| {
                    e.insert(MeshMaterial3d(restored));
                    e.remove::<OriginalMaterial>();
                });
        }
    }
}

// === Sprite animation helpers ===

/// Creates corpse sprite materials, each using a random frame from the sprite sheet.
///
/// Each material shows one frozen animation frame with the corpse tint color applied.
/// These are shared across all deaths of a given unit type/team for efficiency.
pub fn create_corpse_sprite_materials(
    materials: &mut Assets<StandardMaterial>,
    texture: Handle<Image>,
    tint: Color,
) -> [Handle<StandardMaterial>; CORPSE_MATERIAL_VARIANTS] {
    let mut rng = rand::rng();
    let anim = WalkingAnimation::default();
    let rows = (SPRITE_SHEET_IMAGE_HEIGHT / SPRITE_FRAME_SIZE) as usize;
    let total_frames = anim.columns * rows;

    std::array::from_fn(|_| {
        let frame = rng.random_range(0..total_frames);
        let col = frame % anim.columns;
        let row = frame / anim.columns;
        let uv_offset = Vec2::new(col as f32 * anim.frame_uv.x, row as f32 * anim.frame_uv.y);
        let handle =
            create_sprite_material(materials, texture.clone(), tint, anim.frame_uv, uv_offset);
        // Corpses use semi-transparent alpha, so override to Blend
        if let Some(mat) = materials.get_mut(&handle) {
            mat.alpha_mode = AlphaMode::Blend;
        }
        handle
    })
}

/// Creates a per-entity sprite material with the given texture and tint.
pub fn create_sprite_material(
    materials: &mut Assets<StandardMaterial>,
    texture: Handle<Image>,
    tint: Color,
    uv_scale: Vec2,
    uv_offset: Vec2,
) -> Handle<StandardMaterial> {
    materials.add(StandardMaterial {
        base_color_texture: Some(texture),
        base_color: tint,
        alpha_mode: AlphaMode::Mask(0.5),
        unlit: true,
        cull_mode: None,
        uv_transform: Affine2::from_scale_angle_translation(uv_scale, 0.0, uv_offset),
        ..default()
    })
}

/// Creates a sprite material using the default forward-facing UV offset.
///
/// Convenience wrapper used by all unit spawn functions.
pub fn create_default_sprite_material(
    materials: &mut Assets<StandardMaterial>,
    texture: Handle<Image>,
    tint: Color,
) -> Handle<StandardMaterial> {
    let anim = WalkingAnimation::default();
    create_sprite_material(
        materials,
        texture,
        tint,
        anim.frame_uv,
        anim.uv_offset(FacingDirection::default()),
    )
}

/// Picks a corpse material by team from the standard `[defender, attacker, undead]` arrays.
pub fn corpse_material_for_team(
    defender: &[Handle<StandardMaterial>; CORPSE_MATERIAL_VARIANTS],
    attacker: &[Handle<StandardMaterial>; CORPSE_MATERIAL_VARIANTS],
    undead: &[Handle<StandardMaterial>; CORPSE_MATERIAL_VARIANTS],
    team: Team,
    idx: usize,
) -> Handle<StandardMaterial> {
    match team {
        Team::Defenders => defender[idx].clone(),
        Team::Attackers => attacker[idx].clone(),
        Team::Undead => undead[idx].clone(),
    }
}

/// Returns the sprite tint color for a given team (infantry/generic).
pub fn sprite_tint_for_team(team: Team) -> Color {
    use super::infantry::constants::{
        ATTACKER_SPRITE_TINT, DEFENDER_SPRITE_TINT, UNDEAD_SPRITE_TINT,
    };
    match team {
        Team::Defenders => DEFENDER_SPRITE_TINT,
        Team::Attackers => ATTACKER_SPRITE_TINT,
        Team::Undead => UNDEAD_SPRITE_TINT,
    }
}

/// Returns the sprite tint color for archers (lighter attacker tint).
pub fn archer_sprite_tint_for_team(team: Team) -> Color {
    use super::archer::constants::ATTACKER_SPRITE_TINT as ARCHER_ATTACKER_TINT;
    use super::infantry::constants::{DEFENDER_SPRITE_TINT, UNDEAD_SPRITE_TINT};
    match team {
        Team::Defenders => DEFENDER_SPRITE_TINT,
        Team::Attackers => ARCHER_ATTACKER_TINT,
        Team::Undead => UNDEAD_SPRITE_TINT,
    }
}

/// Resurrects a corpse entity as an infantry unit with sprite animation.
///
/// Shared between Raise the Dead, Perpetual Unrest, and Font of Life.
/// The caller should add any extra components (RaisedUndead, PermanentCorpse, etc.) afterward.
///
/// `sprite_texture` and `sprite_mesh` allow overriding the default infantry sprites
/// (e.g. passing undead-specific textures for Team::Undead).
#[allow(clippy::too_many_arguments)]
pub fn resurrect_corpse_as_infantry(
    commands: &mut Commands,
    entity: Entity,
    position: Vec3,
    team: Team,
    health: f32,
    speed: f32,
    tint: Color,
    sprite_texture: Handle<Image>,
    sprite_mesh: Handle<Mesh>,
    materials: &mut Assets<StandardMaterial>,
) {
    use super::components::{
        AttackTiming, Effectiveness, FlockingVelocity, Hitbox, MovementSpeed, RoughTerrain,
        TargetingVelocity, Teleportable,
    };
    use super::infantry::components::Infantry;
    use super::infantry::constants::UNIT_RADIUS;

    let hitbox = Hitbox::new(UNIT_RADIUS, DEFENDER_HITBOX_HEIGHT);
    let spawn_y = hitbox.height / 2.0 + 1.0;
    let upright_transform = Transform::from_xyz(position.x, spawn_y, position.z);

    let anim = WalkingAnimation::default();
    let material = create_default_sprite_material(materials, sprite_texture, tint);

    commands
        .entity(entity)
        .remove::<Corpse>()
        .remove::<RoughTerrain>()
        // Strip all non-infantry unit markers so resurrected units
        // behave purely as infantry (melee only, no special pathing).
        .remove::<super::archer::Archer>()
        .remove::<super::archer::components::ArcherMovementTimer>()
        .remove::<super::assassin::Assassin>()
        .remove::<super::dispeller::components::Dispeller>()
        .remove::<super::ranged_bolt::RangedAttackTimer>()
        .remove::<super::dispeller::components::DispellerDispelCooldown>()
        .remove::<super::shielder::components::Shielder>()
        .remove::<super::shielder::components::ShielderShieldCooldown>()
        .remove::<super::shielder::components::ShielderDamageReduction>()
        .remove::<super::healer::components::Healer>()
        .remove::<super::healer::components::HealerAttackTimer>()
        .remove::<super::commander::components::Commander>()
        .remove::<super::brute::components::Brute>()
        .remove::<super::aerialist::Aerialist>()
        .remove::<super::components::Flying>()
        // Strip staging/wave components so resurrected undead don't
        // trigger wave activation or get treated as staging attackers.
        .remove::<crate::game::pathfinding::StagingAttacker>()
        .remove::<crate::game::pathfinding::WaveGroup>()
        .insert(upright_transform)
        .insert(Mesh3d(sprite_mesh))
        .insert(MeshMaterial3d(material))
        .insert(anim)
        .insert(FacingDirection::default())
        .insert(team)
        .insert(Health::new(health))
        .insert(Velocity::default())
        .insert(Acceleration::new())
        .insert(MovementSpeed(speed))
        .insert(AttackTiming::new())
        .insert(Effectiveness::new())
        .insert(Billboard)
        .insert(hitbox)
        .insert(Teleportable)
        .insert(Infantry)
        .insert(TargetingVelocity::default())
        .insert(FlockingVelocity::default())
        // Ensure resurrected units use the standard attacker flow field
        .insert(crate::game::pathfinding::FlowFieldInfluence::Attacker);
}

/// Advances walking animation frames and updates UV transforms.
/// Skips entities with active combat or dying animations.
pub fn update_walking_animation(
    time: Res<Time>,
    mut anim_query: Query<
        (
            &mut WalkingAnimation,
            &MeshMaterial3d<StandardMaterial>,
            &Velocity,
            &FacingDirection,
        ),
        (
            Without<Corpse>,
            Without<CombatAnimation>,
            Without<DyingAnimation>,
            Without<AnimationOverride>,
        ),
    >,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let delta = time.delta_secs();

    for (mut anim, material_handle, velocity, facing) in &mut anim_query {
        let speed_sq = Vec3::new(velocity.x, 0.0, velocity.z).length_squared();

        // Stationary: reset to frame 0
        if speed_sq < ANIMATION_MOVE_THRESHOLD_SQ {
            if anim.current_frame != 0 {
                anim.current_frame = 0;
                anim.elapsed = 0.0;
                if let Some(mat) = materials.get_mut(material_handle) {
                    mat.uv_transform = anim.uv_transform(*facing);
                }
            }
            continue;
        }

        // Advance animation
        if anim.tick(delta)
            && let Some(mat) = materials.get_mut(material_handle)
        {
            mat.uv_transform = anim.uv_transform(*facing);
        }
    }
}

/// Advances looping in-place pulsing animations and updates UV transforms.
/// Used for non-moving entities (eyes, idle props) that don't need facing direction.
pub fn update_pulsing_animation(
    time: Res<Time>,
    mut anim_query: Query<(&mut PulsingAnimation, &MeshMaterial3d<StandardMaterial>)>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let delta = time.delta_secs();

    for (mut anim, material_handle) in &mut anim_query {
        if anim.tick(delta)
            && let Some(mat) = materials.get_mut(material_handle)
        {
            mat.uv_transform = anim.uv_transform();
        }
    }
}

/// Advances one-shot combat animations (melee attack, ranged shooting).
/// Swaps texture to the combat sheet on start, restores walking sheet when finished.
pub fn update_combat_animation(
    time: Res<Time>,
    mut commands: Commands,
    mut anim_query: Query<
        (
            Entity,
            &mut CombatAnimation,
            &MeshMaterial3d<StandardMaterial>,
            &FacingDirection,
            Option<&WalkingAnimation>,
        ),
        Without<Corpse>,
    >,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let delta = time.delta_secs();

    for (entity, mut anim, material_handle, facing, walking_anim) in &mut anim_query {
        let Some(mat) = materials.get_mut(material_handle) else {
            continue;
        };

        // First frame: swap texture to combat sheet
        if !anim.started {
            anim.started = true;
            mat.base_color_texture = Some(anim.combat_texture.clone());
            mat.uv_transform = anim.uv_transform(*facing);
        }

        // Advance animation
        if anim.tick(delta) {
            if anim.finished() {
                // Restore walking texture and remove component
                mat.base_color_texture = Some(anim.walking_texture.clone());
                // Use the entity's own walking animation UV if available,
                // falling back to the default for standard infantry-sized sheets.
                mat.uv_transform = if let Some(walk) = walking_anim {
                    walk.uv_transform(*facing)
                } else {
                    WalkingAnimation::idle_uv_transform(*facing)
                };
                commands.entity(entity).remove::<CombatAnimation>();
            } else {
                mat.uv_transform = anim.uv_transform(*facing);
            }
        }
    }
}

/// Advances death animations and updates UV transforms.
/// When the animation finishes, inserts `DeathAnimationFinished` marker.
pub fn update_dying_animation(
    time: Res<Time>,
    mut commands: Commands,
    mut anim_query: Query<
        (
            Entity,
            &mut DyingAnimation,
            &MeshMaterial3d<StandardMaterial>,
        ),
        Without<super::components::DeathAnimationFinished>,
    >,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let delta = time.delta_secs();

    for (entity, mut anim, material_handle) in &mut anim_query {
        let Some(mat) = materials.get_mut(material_handle) else {
            continue;
        };

        // First frame: swap texture to death sheet
        if !anim.started {
            anim.started = true;
            mat.base_color_texture = Some(anim.death_texture.clone());
            mat.uv_transform = anim.uv_transform();
        }

        // Advance animation
        if anim.tick(delta) {
            mat.uv_transform = anim.uv_transform();
            if anim.finished() {
                commands
                    .entity(entity)
                    .insert(super::components::DeathAnimationFinished);
            }
        }
    }
}

/// Finalizes dying entities whose death animation has completed.
/// Lays the corpse flat, applies corpse tint, and adds rough terrain.
pub fn finalize_dying_to_corpse(
    mut commands: Commands,
    query: Query<
        (
            Entity,
            &DyingAnimation,
            &Transform,
            &Team,
            &MeshMaterial3d<StandardMaterial>,
        ),
        With<super::components::DeathAnimationFinished>,
    >,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    for (entity, anim, transform, team, material_handle) in &query {
        // Apply corpse tint and switch to Blend for semi-transparent rendering
        if let Some(mat) = materials.get_mut(material_handle) {
            use crate::game::constants::{
                ATTACKER_CORPSE_COLOR, DEFENDER_CORPSE_COLOR, UNDEAD_CORPSE_COLOR,
            };
            mat.base_color = match *team {
                Team::Defenders => DEFENDER_CORPSE_COLOR,
                Team::Attackers => ATTACKER_CORPSE_COLOR,
                Team::Undead => UNDEAD_CORPSE_COLOR,
            };
            mat.uv_transform = anim.last_frame_uv_transform();
            mat.alpha_mode = AlphaMode::Blend;
        }

        let mut entity_commands = commands.entity(entity);
        lay_corpse_flat(&mut entity_commands, transform.translation);
        entity_commands
            .remove::<DyingAnimation>()
            .remove::<super::components::DeathAnimationFinished>();
    }
}

/// Lays a corpse entity flat on the ground. Shared between instant corpse swap
/// and death animation finalization.
pub fn lay_corpse_flat(entity_commands: &mut EntityCommands, position: Vec3) {
    let corpse_transform = Transform::from_xyz(position.x, 1.0, position.z)
        .with_rotation(Quat::from_rotation_x(-std::f32::consts::FRAC_PI_2));

    entity_commands
        .insert(corpse_transform)
        .insert(super::components::RoughTerrain {
            slowdown_factor: 0.4,
        })
        .remove::<crate::game::components::Billboard>();
}

/// Ticks all knockback effects, applying decaying position offsets each frame.
/// Units tumble outward and gradually slow down. Removes the component when expired.
pub fn apply_knockback_effects(
    time: Res<Time>,
    mut commands: Commands,
    mut units: Query<(Entity, &mut Transform, &mut super::components::Knockback)>,
) {
    let delta = time.delta_secs();

    for (entity, mut transform, mut knockback) in &mut units {
        knockback.remaining -= delta;

        if knockback.remaining <= 0.0 {
            commands
                .entity(entity)
                .remove::<super::components::Knockback>();
            continue;
        }

        // Linear decay: full speed at start, zero at end
        let decay = knockback.remaining / knockback.duration;
        let speed = knockback.speed * decay;

        transform.translation.x += knockback.direction_x * speed * delta;
        transform.translation.z += knockback.direction_z * speed * delta;
    }
}

/// Updates airborne units: applies gravity, offsets Y visually, and deals
/// velocity-based fall damage on landing. Any system can make a unit airborne
/// by inserting the `Airborne` component with a launch velocity and gravity.
pub fn update_airborne_units(
    mut commands: Commands,
    time: Res<Time>,
    mut units: Query<(
        Entity,
        &mut Transform,
        &mut Airborne,
        &mut Health,
        Option<&mut TemporaryHitPoints>,
    )>,
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
                });
            }
            commands.entity(entity).remove::<Airborne>();
        } else {
            // Offset the visual Y position during flight
            transform.translation.y = airborne.base_y + airborne.height;
        }
    }
}

/// Updates facing direction based on velocity relative to camera, updating UV row.
pub fn update_facing_direction(
    camera_query: Query<&Transform, With<Camera3d>>,
    time: Res<Time>,
    mut unit_query: Query<
        (
            &Velocity,
            &mut FacingDirection,
            &WalkingAnimation,
            &MeshMaterial3d<StandardMaterial>,
            Option<&FacingHysteresisBoost>,
            Option<&mut FacingDwell>,
            Option<&mut SmoothedFacingVelocity>,
        ),
        Without<Corpse>,
    >,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let Ok(camera_transform) = camera_query.single() else {
        return;
    };

    // Camera forward/right on XZ plane
    let cam_forward = camera_transform.forward();
    let cam_forward_xz = Vec3::new(cam_forward.x, 0.0, cam_forward.z).normalize_or_zero();
    let cam_right = Vec3::new(-cam_forward_xz.z, 0.0, cam_forward_xz.x);
    let dt = time.delta_secs();

    // Base angular buffer past the 45° axis boundary. With `buffer = 8°`,
    // a unit currently on the forward-back axis only switches to left-right
    // when the velocity is more than `45 + 8 = 53°` off-axis, and vice versa.
    // `FacingHysteresisBoost` widens the buffer further for jittery units.
    const BASE_BUFFER_DEG: f32 = 8.0;
    let default_buffer_ratio = (45.0_f32 + BASE_BUFFER_DEG).to_radians().tan();

    for (velocity, mut facing, anim, material_handle, hysteresis_boost, mut dwell, mut smoothed) in
        &mut unit_query
    {
        let raw_vel = Vec3::new(velocity.x, 0.0, velocity.z);

        // Update the smoothed velocity (low-pass filter) every frame, even when
        // the dwell is locking the facing — keeps the trend representative.
        let smoothed_vel = if let Some(s) = smoothed.as_mut() {
            let alpha = if s.time_constant > 0.0 {
                (dt / s.time_constant).clamp(0.0, 1.0)
            } else {
                1.0
            };
            s.velocity = s.velocity.lerp(raw_vel, alpha);
            s.velocity
        } else {
            raw_vel
        };

        // Tick dwell timer regardless of velocity so it expires while the unit
        // is briefly stationary; while non-zero the facing is locked in.
        if let Some(d) = dwell.as_mut() {
            d.time_remaining = (d.time_remaining - dt).max(0.0);
            if d.time_remaining > 0.0 {
                continue;
            }
        }

        if smoothed_vel.length_squared() < ANIMATION_MOVE_THRESHOLD_SQ {
            continue;
        }

        // Project the smoothed velocity onto camera axes.
        let forward_dot = smoothed_vel.dot(cam_forward_xz);
        let right_dot = smoothed_vel.dot(cam_right);
        let abs_fwd = forward_dot.abs();
        let abs_right = right_dot.abs();

        // Buffer ratio = tan(45° + buffer). At buffer=8°, ratio ≈ 1.327: the
        // new axis must dominate by 32.7% before we switch. Precomputed for the
        // default; boosted entities (rare) recompute per-entity.
        let buffer_ratio = match hysteresis_boost {
            Some(boost) if (boost.0 - 1.0).abs() > f32::EPSILON => (45.0_f32
                + BASE_BUFFER_DEG * boost.0)
                .clamp(45.0, 89.0)
                .to_radians()
                .tan(),
            _ => default_buffer_ratio,
        };

        let current_is_forward_back =
            matches!(*facing, FacingDirection::Forward | FacingDirection::Back);
        let on_forward_back = if current_is_forward_back {
            // Stay on FB axis unless |right| is more than buffer_ratio × |fwd|.
            abs_right < abs_fwd * buffer_ratio
        } else {
            // On LR axis: switch to FB only if |fwd| beats |right| by buffer_ratio.
            abs_fwd > abs_right * buffer_ratio
        };

        let new_facing = if on_forward_back {
            if forward_dot >= 0.0 {
                FacingDirection::Forward
            } else {
                FacingDirection::Back
            }
        } else if right_dot >= 0.0 {
            FacingDirection::Right
        } else {
            FacingDirection::Left
        };

        if *facing != new_facing {
            *facing = new_facing;
            if let Some(mat) = materials.get_mut(material_handle) {
                mat.uv_transform = anim.uv_transform(new_facing);
            }
            if let Some(d) = dwell.as_mut() {
                d.time_remaining = d.duration;
            }
        }
    }
}

/// Finds the closest enemy within range of a position. Shared by brute and ogre rock throw.
pub fn find_closest_enemy_in_range(
    origin: Vec3,
    team: &Team,
    range: f32,
    targets: &Query<
        (&Transform, &Team),
        (
            Without<Corpse>,
            Without<super::components::BanishedModifier>,
            Without<crate::game::pathfinding::StagingAttacker>,
        ),
    >,
) -> Option<Vec3> {
    let mut best: Option<(Vec3, f32)> = None;
    for (target_transform, target_team) in targets.iter() {
        if !team.is_enemy(target_team) {
            continue;
        }
        let dx = target_transform.translation.x - origin.x;
        let dz = target_transform.translation.z - origin.z;
        let distance = (dx * dx + dz * dz).sqrt();
        if distance > range {
            continue;
        }
        if best.is_none_or(|(_, d)| distance < d) {
            best = Some((target_transform.translation, distance));
        }
    }
    best.map(|(pos, _)| pos)
}
