//! Ogre spawn, facing, targeting, combat, movement, enrage.

use super::charge::ogre_combat_animation;

use bevy::prelude::*;
use rand::Rng;

use super::components::*;
use super::constants::*;
use super::resources::OgreAssets;
use crate::game::cauldron::components::CauldronSpeedModifier;
use crate::game::components::{Acceleration, Billboard, OnGameplayScreen, Velocity};
use crate::game::constants::*;
use crate::game::pathfinding::{FlowFieldInfluence, FlowFieldVelocity, StagingAttacker};
use crate::game::units::boss::components::Boss;
use crate::game::units::boss::lich::Lich;
use crate::game::units::brute::components::RockThrowCooldown;
use crate::game::units::components::{
    AttackTiming, BanishedModifier, CommanderAuraSpeedModifier, Corpse, DamageMultiplier,
    Effectiveness, EliteSpeedBonus, FlockingModifier, FlockingVelocity, FrozenSolidModifier,
    HasteModifier, Health, Hitbox, InMelee, Knockback, MovementSpeed, OriginalMaterial,
    PolymorphedModifier, RootedModifier, RoughTerrainModifier, SickenedModifier, SleepModifier,
    Sleepwalking, SlowMovementModifier, TargetingVelocity, Team, Teleportable, TemporaryHitPoints,
    apply_damage_to_unit,
};
use crate::game::units::components::{CombatAnimation, FacingDirection, WalkingAnimation};
use crate::game::units::random_position_in_cell;

/// Spawns the ogre at one of the tunnel spawn points.
pub fn spawn_ogre(
    rng: &mut impl Rng,
    mut commands: Commands,
    ogre_assets: Res<OgreAssets>,
    materials: &mut Assets<StandardMaterial>,
) {
    let (spawn_x, spawn_z) = attacker_spawn_position(0, 0.0);
    let (final_x, final_z) = random_position_in_cell(rng, spawn_x, spawn_z);

    let hitbox = Hitbox::new(OGRE_RADIUS, OGRE_HITBOX_HEIGHT);
    let spawn_y = OGRE_SPRITE_HEIGHT / 2.0 - OGRE_SPRITE_Y_OFFSET;

    // Initial velocity toward castle
    let to_center = Vec3::new(
        WIZARD_POSITION.x - final_x,
        0.0,
        WIZARD_POSITION.z - final_z,
    );
    let initial_velocity = to_center.normalize_or_zero() * OGRE_MOVEMENT_SPEED;

    let anim = WalkingAnimation {
        current_frame: 0,
        elapsed: rng.random::<f32>() * 0.125,
        columns: OGRE_SPRITE_COLUMNS,
        frame_uv: OGRE_FRAME_UV,
        direction_rows: OGRE_WALKING_DIRECTION_ROWS,
    };
    let material = crate::game::units::systems::create_sprite_material(
        materials,
        ogre_assets.walking_texture.clone(),
        OGRE_COLOR,
        OGRE_FRAME_UV,
        anim.uv_offset(FacingDirection::default()),
    );

    commands
        .spawn((
            // Rendering
            Mesh3d(ogre_assets.sprite_mesh.clone()),
            MeshMaterial3d(material),
            Transform::from_xyz(final_x, spawn_y, final_z),
            // Physics
            Velocity {
                x: initial_velocity.x,
                z: initial_velocity.z,
                ..default()
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
            OgreChargeState::Idle {
                cooldown: OGRE_CHARGE_COOLDOWN,
            },
            crate::game::units::components::MeleeDamageReduction {
                multiplier: OGRE_MELEE_DAMAGE_REDUCTION,
            },
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
        ))
        .insert((
            anim,
            FacingDirection::default(),
            crate::game::units::components::MeleeRangeBonus(OGRE_MELEE_RANGE_BONUS),
        ))
        .insert(RockThrowCooldown::new(8.0));
}

/// Overrides the ogre's facing direction to strongly prefer forward/backward.
/// Runs after the shared `update_facing_direction` to correct left/right
/// picks when the ogre is moving at a slight angle.
///
/// Filters on `With<OgreEnrageState>` (an ogre-only marker) — `With<Boss>`
/// would also match hags / dark mage / ray, which have their own facing logic
/// and would otherwise have their hysteresis-buffered facing clobbered each
/// frame by this raw-velocity override.
pub fn update_ogre_facing(
    camera_query: Query<&Transform, (With<Camera3d>, Without<Boss>)>,
    mut bosses: Query<
        (
            &Velocity,
            &mut FacingDirection,
            &WalkingAnimation,
            &MeshMaterial3d<StandardMaterial>,
        ),
        (
            With<OgreEnrageState>,
            Without<Corpse>,
            Without<CombatAnimation>,
        ),
    >,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let Ok(camera_transform) = camera_query.single() else {
        return;
    };
    let cam_forward = camera_transform.forward();
    let cam_forward_xz = Vec3::new(cam_forward.x, 0.0, cam_forward.z).normalize_or_zero();
    let cam_right = Vec3::new(-cam_forward_xz.z, 0.0, cam_forward_xz.x);

    for (velocity, mut facing, anim, material_handle) in &mut bosses {
        let vel_xz = Vec3::new(velocity.x, 0.0, velocity.z);
        if vel_xz.length_squared() < crate::game::units::components::ANIMATION_MOVE_THRESHOLD_SQ {
            continue;
        }

        let forward_dot = vel_xz.dot(cam_forward_xz);
        let right_dot = vel_xz.dot(cam_right);

        // Strong forward/back bias: only use left/right if the lateral component
        // is more than 3x the forward component
        let new_facing = if right_dot.abs() > forward_dot.abs() * 3.0 {
            if right_dot > 0.0 {
                FacingDirection::Right
            } else {
                FacingDirection::Left
            }
        } else if forward_dot < 0.0 {
            FacingDirection::Back
        } else {
            FacingDirection::Forward
        };

        if *facing != new_facing {
            *facing = new_facing;
            if let Some(mat) = materials.get_mut(material_handle) {
                mat.uv_transform = anim.uv_transform(new_facing);
            }
        }
    }
}

/// Updates ogre targeting velocity toward nearest enemy.
pub fn update_ogre_targeting(
    mut commands: Commands,
    mut bosses: Query<
        (Entity, &Transform, &Team, &mut TargetingVelocity),
        (With<Boss>, Without<Lich>),
    >,
    all_units: Query<
        (Entity, &Transform, &Team),
        (
            Without<Boss>,
            Without<Corpse>,
            Without<BanishedModifier>,
            Without<StagingAttacker>,
        ),
    >,
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
#[allow(clippy::type_complexity)]
pub fn ogre_combat(
    time: Res<Time>,
    mut commands: Commands,
    ogre_assets: Res<OgreAssets>,
    game_config: Res<crate::config::GameConfig>,
    mut bosses: Query<
        (
            Entity,
            &Transform,
            &Hitbox,
            &Team,
            &mut OgreAttackCooldown,
            &OgreChargeState,
        ),
        (
            With<Boss>,
            Without<Corpse>,
            Without<CombatAnimation>,
            Without<OgreThrowWindup>,
        ),
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
        (Without<Boss>, Without<Corpse>, Without<BanishedModifier>),
    >,
) {
    let delta = time.delta_secs();

    for (boss_entity, boss_transform, boss_hitbox, boss_team, mut attack_cooldown, charge_state) in
        &mut bosses
    {
        // Skip normal melee attacks during charge
        if charge_state.is_movement_locked() {
            continue;
        }

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

        // Play swing sound effect
        crate::game::units::wizard::spells::audio::play_sfx_scaled(
            &mut commands,
            &ogre_assets.swing_sfx,
            boss_pos,
            &game_config,
            1.0,
        );

        // Trigger attack animation
        commands.entity(boss_entity).insert(ogre_combat_animation(
            OGRE_ATTACKING_DIRECTION_ROWS,
            ogre_assets.attacking_texture.clone(),
            ogre_assets.walking_texture.clone(),
        ));

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
            &TargetingVelocity,
            &FlockingVelocity,
            &FlowFieldVelocity,
            &OgreEnrageState,
            &OgreChargeState,
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
                Has<SleepModifier>,
                Has<Sleepwalking>,
                Option<&BanishedModifier>,
                Option<&PolymorphedModifier>,
                Option<&SickenedModifier>,
                Option<&FrozenSolidModifier>,
                Option<&crate::game::units::components::Stunned>,
                Option<&crate::game::units::components::Petrified>,
            ),
        ),
        With<Boss>,
    >,
) {
    for (
        mut velocity,
        mut acceleration,
        movement_speed,
        targeting_velocity,
        flocking_velocity,
        flow_field_velocity,
        enrage_state,
        charge_state,
        in_melee,
        aura_modifier,
        terrain_modifier,
        slow_modifier,
        (cauldron_modifier, rooted, haste_modifier, elite_speed),
        (sleeping, sleepwalking, banished, polymorphed, sickened, frozen, stunned, petrified),
    ) in &mut bosses
    {
        // Freeze normal movement during charge phases; also zero acceleration
        // so external forces (e.g. black hole gravity) don't drift the ogre off course
        if charge_state.is_movement_locked() {
            velocity.x = 0.0;
            velocity.z = 0.0;
            acceleration.x = 0.0;
            acceleration.z = 0.0;
            continue;
        }

        // CC'd units cannot move
        if crate::game::units::systems::is_cc_immobilized(
            rooted,
            sleeping,
            sleepwalking,
            banished,
            sickened,
            frozen,
            stunned,
            petrified,
        ) {
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
/// Modifies the sprite material's base_color to match the current enrage phase.
#[allow(clippy::type_complexity)]
pub fn update_enrage_state(
    mut bosses: Query<
        (
            &Health,
            &mut OgreEnrageState,
            &mut DamageMultiplier,
            &MeshMaterial3d<StandardMaterial>,
            Option<&OriginalMaterial>,
        ),
        With<Boss>,
    >,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    for (health, mut enrage, mut damage_mult, mesh_material, original_material) in &mut bosses {
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

            let phase_tint = enrage_phase_tint(new_phase);

            // Update base_color on the per-entity sprite material.
            // If OriginalMaterial is present (spell effect active), update that
            // so the correct enrage tint restores when the effect ends.
            if let Some(orig) = original_material {
                if let Some(orig_mat) = materials.get_mut(&orig.0) {
                    orig_mat.base_color = phase_tint;
                }
            } else if let Some(mat) = materials.get_mut(&mesh_material.0) {
                mat.base_color = phase_tint;
            }
        }
    }
}

/// Returns the sprite tint color for a given enrage phase.
pub(super) fn enrage_phase_tint(phase: u8) -> Color {
    match phase {
        1 => OGRE_ENRAGE_1_COLOR,
        2 => OGRE_ENRAGE_2_COLOR,
        3 => OGRE_ENRAGE_3_COLOR,
        _ => OGRE_COLOR,
    }
}
