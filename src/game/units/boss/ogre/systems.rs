use bevy::prelude::*;

use super::components::*;
use super::constants::*;
use super::resources::OgreAssets;
use crate::game::cauldron::components::CauldronSpeedModifier;
use crate::game::components::{Acceleration, Billboard, OnGameplayScreen, Velocity};
use crate::game::constants::*;
use crate::game::pathfinding::{FlowFieldInfluence, FlowFieldVelocity};
use crate::game::units::boss::components::Boss;
use crate::game::units::components::{
    AttackTiming, BanishedModifier, CommanderAuraSpeedModifier, Corpse, DamageMultiplier,
    Effectiveness, EliteSpeedBonus, FlockingModifier, FlockingVelocity, HasteModifier, Health,
    Hitbox, InMelee, Knockback, MovementSpeed, OriginalMaterial, PolymorphedModifier,
    FrozenSolidModifier, RootedModifier, RoughTerrainModifier, SickenedModifier, SleepModifier,
    SlowMovementModifier,
    TargetingVelocity, Team, Teleportable, TemporaryHitPoints, apply_damage_to_unit,
};
use crate::game::units::random_position_in_cell;

/// Spawns the ogre at the center of the attacker grid.
pub fn spawn_ogre(mut commands: Commands, ogre_assets: Res<OgreAssets>) {
    // Spawn at center of attacker grid (row 0, col ~3)
    let (spawn_x, spawn_z) = calculate_grid_cell_position(0, 3);
    let (final_x, final_z) = random_position_in_cell(spawn_x, spawn_z);

    let hitbox = Hitbox::new(OGRE_RADIUS, OGRE_HITBOX_HEIGHT);
    let spawn_y = hitbox.height / 2.0 + (OGRE_ELLIPSE_DEPTH / 2.0) + 1.0;

    // Initial velocity toward castle
    let to_center = Vec3::new(
        WIZARD_POSITION.x - final_x,
        0.0,
        WIZARD_POSITION.z - final_z,
    );
    let initial_velocity = to_center.normalize_or_zero() * OGRE_MOVEMENT_SPEED;

    commands
        .spawn((
            // Rendering
            Mesh3d(ogre_assets.mesh.clone()),
            MeshMaterial3d(ogre_assets.material_phase0.clone()),
            Transform::from_xyz(final_x, spawn_y, final_z),
            // Physics
            Velocity {
                x: initial_velocity.x,
                z: initial_velocity.z,
            },
            Acceleration::new(),
            // Core
            hitbox,
            Health::new(OGRE_HEALTH),
            MovementSpeed(OGRE_MOVEMENT_SPEED),
            AttackTiming::new(),
            Effectiveness::new(),
            Team::Attackers,
            Boss,
        ))
        .insert((
            OgreEnrageState::new(),
            OgreAttackCooldown::new(),
            // Movement systems
            TargetingVelocity::default(),
            FlowFieldVelocity::default(),
            FlowFieldInfluence::Attacker,
            DamageMultiplier(OGRE_DAMAGE_MULTIPLIER),
            FlockingVelocity::default(),
            FlockingModifier::new(0.0, 0.0, 0.0),
            CommanderAuraSpeedModifier(0.0),
            RoughTerrainModifier(0.0),
            Teleportable,
            Billboard,
            OnGameplayScreen,
        ));
}

/// Updates ogre targeting velocity toward nearest enemy.
pub fn update_ogre_targeting(
    mut commands: Commands,
    mut bosses: Query<(Entity, &Transform, &Team, &mut TargetingVelocity), With<Boss>>,
    all_units: Query<(Entity, &Transform, &Team), (Without<Boss>, Without<Corpse>)>,
) {
    let unit_snapshot: Vec<_> = all_units
        .iter()
        .map(|(entity, transform, team)| (entity, transform.translation, *team))
        .collect();

    for (entity, boss_transform, boss_team, mut targeting) in &mut bosses {
        crate::game::units::systems::update_melee_unit_targeting(
            &unit_snapshot,
            entity,
            boss_transform,
            *boss_team,
            &mut targeting,
            &mut commands,
            None,
        );
    }
}

/// Ogre melee combat system — runs on its own cooldown timer (not the global attack cycle).
/// Finds nearest enemy in melee range, deals flat damage, and applies a tumbling
/// knockback effect to all nearby enemies.
pub fn ogre_combat(
    time: Res<Time>,
    mut commands: Commands,
    mut bosses: Query<
        (Entity, &Transform, &Hitbox, &Team, &mut OgreAttackCooldown),
        (With<Boss>, Without<Corpse>),
    >,
    mut targets: Query<
        (
            Entity,
            &Transform,
            &Hitbox,
            &Team,
            &mut Health,
            Option<&mut TemporaryHitPoints>,
        ),
        (Without<Boss>, Without<Corpse>),
    >,
) {
    let delta = time.delta_secs();

    for (boss_entity, boss_transform, boss_hitbox, boss_team, mut attack_cooldown) in &mut bosses {
        attack_cooldown.tick(delta);

        if !attack_cooldown.is_ready() {
            continue;
        }

        // Find nearest enemy in melee range
        let boss_pos = boss_transform.translation;
        let mut has_target = false;

        // First pass: check if any enemy is in melee range
        for (entity, target_transform, target_hitbox, team, _, _) in &targets {
            if entity == boss_entity {
                continue;
            }
            if !boss_team.is_enemy(team) {
                continue;
            }

            let dx = boss_pos.x - target_transform.translation.x;
            let dz = boss_pos.z - target_transform.translation.z;
            let distance = (dx * dx + dz * dz).sqrt();
            let attack_range =
                (boss_hitbox.radius + target_hitbox.radius) * ATTACK_RANGE_MULTIPLIER;
            if distance <= attack_range {
                has_target = true;
                break;
            }
        }

        if !has_target {
            continue;
        }

        // Reset cooldown — ogre attacked
        attack_cooldown.reset(OGRE_ATTACK_COOLDOWN);

        // Second pass: apply damage and knockback to all enemies within ogre melee reach
        for (entity, target_transform, target_hitbox, team, mut health, mut temp_hp) in &mut targets
        {
            if entity == boss_entity {
                continue;
            }
            let is_enemy = boss_team.is_enemy(team);
            if !is_enemy {
                continue;
            }

            let target_pos = target_transform.translation;
            let dx = target_pos.x - boss_pos.x;
            let dz = target_pos.z - boss_pos.z;
            let distance = (dx * dx + dz * dz).sqrt();

            // Hit all enemies within boss radius + their hitbox (melee reach)
            if distance > boss_hitbox.radius + target_hitbox.radius {
                continue;
            }

            // Apply damage
            apply_damage_to_unit(&mut health, temp_hp.as_deref_mut(), OGRE_ATTACK_DAMAGE);

            // Apply tumbling knockback (decays over time)
            let direction = if distance > 0.1 {
                Vec3::new(dx, 0.0, dz)
            } else {
                Vec3::X
            };
            commands.entity(entity).insert(Knockback::new(
                direction,
                OGRE_MELEE_KNOCKBACK_SPEED,
                OGRE_MELEE_KNOCKBACK_DURATION,
            ));
        }
    }
}

/// Ogre movement system using weighted velocities.
/// Feeds enrage speed bonus through the haste parameter.
#[allow(clippy::type_complexity)]
pub fn ogre_movement(
    time: Res<Time>,
    mut bosses: Query<
        (
            &mut Velocity,
            &mut Acceleration,
            &MovementSpeed,
            &Effectiveness,
            &TargetingVelocity,
            &FlockingVelocity,
            &FlowFieldVelocity,
            &OgreEnrageState,
            Option<&InMelee>,
            Option<&CommanderAuraSpeedModifier>,
            Option<&RoughTerrainModifier>,
            Option<&SlowMovementModifier>,
            (
                Option<&CauldronSpeedModifier>,
                Option<&RootedModifier>,
                Option<&HasteModifier>,
                Option<&EliteSpeedBonus>,
            ),
            (
                Option<&SleepModifier>,
                Option<&BanishedModifier>,
                Option<&PolymorphedModifier>,
                Option<&SickenedModifier>,
                Option<&FrozenSolidModifier>,
            ),
        ),
        With<Boss>,
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
        enrage_state,
        in_melee,
        aura_modifier,
        terrain_modifier,
        slow_modifier,
        (cauldron_modifier, rooted, haste_modifier, elite_speed),
        (sleeping, banished, polymorphed, sickened, frozen),
    ) in &mut bosses
    {
        // CC'd units cannot move
        if crate::game::units::systems::is_cc_immobilized(rooted, sleeping, banished, sickened, frozen) {
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

        // Combine haste modifier with enrage speed bonus
        let combined_haste =
            Some(haste_modifier.map(|m| m.modifier).unwrap_or(0.0) + enrage_state.speed_bonus);

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
            slow_modifier.map(|m| m.modifier),
            cauldron_modifier.map(|m| m.0),
            combined_haste,
            elite_speed.map(|e| e.0),
        );
    }
}

/// Updates the ogre's enrage state based on HP thresholds.
/// Swaps material to match the current enrage phase.
#[allow(clippy::type_complexity)]
pub fn update_enrage_state(
    mut bosses: Query<
        (
            &Health,
            &mut OgreEnrageState,
            &mut DamageMultiplier,
            &mut MeshMaterial3d<StandardMaterial>,
            Option<&mut OriginalMaterial>,
        ),
        With<Boss>,
    >,
    ogre_assets: Res<OgreAssets>,
) {
    for (health, mut enrage, mut damage_mult, mut mesh_material, original_material) in &mut bosses {
        let hp_ratio = health.current / health.max;

        let new_phase = if hp_ratio <= ENRAGE_PHASE_3_THRESHOLD {
            3
        } else if hp_ratio <= ENRAGE_PHASE_2_THRESHOLD {
            2
        } else if hp_ratio <= ENRAGE_PHASE_1_THRESHOLD {
            1
        } else {
            0
        };

        if new_phase != enrage.phase {
            enrage.phase = new_phase;

            // Update bonuses
            match new_phase {
                1 => {
                    enrage.speed_bonus = ENRAGE_1_SPEED_BONUS;
                    enrage.damage_bonus = ENRAGE_1_DAMAGE_BONUS;
                }
                2 => {
                    enrage.speed_bonus = ENRAGE_2_SPEED_BONUS;
                    enrage.damage_bonus = ENRAGE_2_DAMAGE_BONUS;
                }
                3 => {
                    enrage.speed_bonus = ENRAGE_3_SPEED_BONUS;
                    enrage.damage_bonus = ENRAGE_3_DAMAGE_BONUS;
                }
                _ => {
                    enrage.speed_bonus = 0.0;
                    enrage.damage_bonus = 0.0;
                }
            }

            // Update damage multiplier (base + enrage bonus)
            damage_mult.0 = OGRE_DAMAGE_MULTIPLIER + enrage.damage_bonus;

            // Pick the phase material
            let phase_material = match new_phase {
                1 => ogre_assets.material_phase1.clone(),
                2 => ogre_assets.material_phase2.clone(),
                3 => ogre_assets.material_phase3.clone(),
                _ => ogre_assets.material_phase0.clone(),
            };

            // If OriginalMaterial is present (spell effect active), update that
            // so the correct enrage color restores when the effect ends.
            if let Some(mut orig) = original_material {
                orig.0 = phase_material;
            } else {
                mesh_material.0 = phase_material;
            }
        }
    }
}
