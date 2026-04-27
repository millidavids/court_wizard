use bevy::prelude::*;
use rand::Rng;

use super::components::Aerialist;
use super::constants::*;
use super::resources::AerialistAssets;
use crate::game::components::{Acceleration, Billboard, OnGameplayScreen, Velocity};
use crate::game::constants::{
    ARCHER_SPAWN_DEPTH_OFFSET, ATTACKER_HITBOX_HEIGHT, attacker_spawn_position,
};
use crate::game::pathfinding::{FlowFieldInfluence, FlowFieldVelocity, StagingAttacker, WaveGroup};
use crate::game::plugin::GlobalAttackCycle;
use crate::game::units::components::{
    AttackTiming, BanishedModifier, Corpse, Effectiveness, FacingDirection, FlockingVelocity,
    Flying, FrozenSolidModifier, HasteModifier, Health, Hitbox, MindControlled, MovementSpeed,
    PolymorphedModifier, RootedModifier, SickenedModifier, SleepModifier, Sleepwalking,
    SlowMovementModifier, TargetingVelocity, Team, Teleportable, WalkingAnimation,
};
use crate::game::units::random_position_in_cell;

/// Aerialist targeting: find nearest ground enemy and set targeting velocity.
/// Flying units ignore wall LOS since they fly above obstacles.
#[allow(clippy::type_complexity)]
pub fn update_aerialist_targeting(
    mut aerialists: Query<
        (Entity, &Transform, &Team, &mut TargetingVelocity),
        (With<Aerialist>, Without<Corpse>, Without<MindControlled>),
    >,
    all_units: Query<
        (Entity, &Transform, &Team),
        (
            Without<Corpse>,
            Without<BanishedModifier>,
            Without<Flying>,
            Without<StagingAttacker>,
        ),
    >,
) {
    // Collect snapshot of ground unit positions (excludes flying and staging)
    let unit_snapshot: Vec<_> = all_units
        .iter()
        .map(|(entity, transform, team)| (entity, transform.translation, *team))
        .collect();

    for (entity, transform, team, mut targeting_velocity) in &mut aerialists {
        let nearest_enemy = unit_snapshot
            .iter()
            .filter(|(other_entity, _, other_team)| {
                *other_entity != entity && team.is_enemy(other_team)
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

        if let Some(&(_, target_pos, _enemy_team)) = nearest_enemy {
            let direction = (target_pos - transform.translation).normalize_or_zero();
            targeting_velocity.velocity = Vec3::new(direction.x, 0.0, direction.z);

            let dx = transform.translation.x - target_pos.x;
            let dz = transform.translation.z - target_pos.z;
            let distance = (dx * dx + dz * dz).sqrt();
            targeting_velocity.distance_to_target = distance;

            // Flying units are never in melee — they attack from above.
            // Keeping them out of InMelee ensures archers can always target them.
        } else {
            targeting_velocity.velocity = Vec3::ZERO;
            targeting_velocity.distance_to_target = f32::MAX;
        }
    }
}

/// Aerialist combat: direct damage to ground enemies within very short range.
#[allow(clippy::type_complexity)]
pub fn aerialist_combat(
    mut commands: Commands,
    mut game_rng: ResMut<crate::game::seeded_rng::resources::GameRng>,
    attack_cycle: Res<GlobalAttackCycle>,
    archer_assets: Res<crate::game::units::archer::resources::ArcherAssets>,
    mut aerialists: Query<
        (
            &Transform,
            &Team,
            &mut AttackTiming,
            Has<SleepModifier>,
            Option<&BanishedModifier>,
            Option<&FrozenSolidModifier>,
            Option<&crate::game::units::components::Stunned>,
        ),
        (With<Aerialist>, Without<Corpse>),
    >,
    targets: Query<(Entity, &Transform, &Team), (Without<Corpse>, Without<Flying>)>,
) {
    let current_time = attack_cycle.current_time;
    let last_time = (current_time - crate::game::constants::APPROX_FRAME_TIME).max(0.0);

    for (
        aerialist_transform,
        aerialist_team,
        mut attack_timing,
        is_sleeping,
        banished,
        frozen,
        stunned,
    ) in &mut aerialists
    {
        if is_sleeping || banished.is_some() || frozen.is_some() || stunned.is_some() {
            continue;
        }

        if !attack_timing.can_attack(current_time, last_time) {
            continue;
        }

        // Find nearest enemy within attack range (full 3D distance — accounts for fly height)
        let nearest = targets
            .iter()
            .filter(|(_, _, team)| aerialist_team.is_enemy(team))
            .filter_map(|(entity, target_transform, _)| {
                let distance = aerialist_transform
                    .translation
                    .distance(target_transform.translation);
                if distance <= AERIALIST_ATTACK_RANGE {
                    Some((entity, target_transform.translation, distance))
                } else {
                    None
                }
            })
            .min_by(|a, b| a.2.partial_cmp(&b.2).unwrap_or(std::cmp::Ordering::Equal));

        if let Some((_target_entity, target_pos, _)) = nearest {
            attack_timing.last_attack_time = Some(current_time);

            // Spawn an arrow projectile toward the target
            crate::game::units::archer::systems::spawn_arrow(
                &mut game_rng.0,
                &mut commands,
                &archer_assets,
                aerialist_transform.translation,
                target_pos,
                *aerialist_team,
            );
        }
    }
}

/// Aerialist movement: momentum-based flying with wide sweeping arcs.
///
/// Unlike ground units, aerialists never stop moving. They maintain a minimum
/// speed and gradually steer toward their desired direction, creating long
/// swooping flyover paths. They attack mid-flight.
#[allow(clippy::type_complexity)]
pub fn aerialist_movement(
    time: Res<Time>,
    mut game_rng: ResMut<crate::game::seeded_rng::resources::GameRng>,
    mut aerialist_units: Query<
        (
            &mut Velocity,
            &MovementSpeed,
            &TargetingVelocity,
            &FlowFieldVelocity,
            (
                Option<&RootedModifier>,
                Option<&HasteModifier>,
                Option<&SlowMovementModifier>,
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
                &Team,
                Has<StagingAttacker>,
                Has<WaveGroup>,
            ),
        ),
        With<Aerialist>,
    >,
) {
    let delta = time.delta_secs();
    if delta <= 0.0 {
        return;
    }

    for (
        mut velocity,
        movement_speed,
        targeting_velocity,
        flow_field_velocity,
        (rooted, haste_modifier, slow_modifier),
        (
            sleeping,
            sleepwalking,
            banished,
            polymorphed,
            sickened,
            frozen,
            stunned,
            petrified,
            team,
            has_staging,
            has_wave_group,
        ),
    ) in &mut aerialist_units
    {
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
            velocity.x = angle.cos() * AERIALIST_MIN_SPEED;
            velocity.z = angle.sin() * AERIALIST_MIN_SPEED;
            continue;
        }

        // Calculate desired direction from flow field + targeting blend
        let is_staging =
            crate::game::units::systems::is_staging_attacker(team, has_staging, has_wave_group);
        let desired_dir = if is_staging {
            // While staging, follow the flow field directly
            flow_field_velocity.velocity
        } else {
            // Blend flow field and targeting based on distance to target
            let dist = targeting_velocity.distance_to_target;
            let flow_weight = if dist > 500.0 {
                0.7
            } else if dist > 200.0 {
                0.5
            } else {
                0.2
            };
            let target_weight = 1.0 - flow_weight;
            flow_field_velocity.velocity * flow_weight + targeting_velocity.velocity * target_weight
        };

        // Calculate target speed with modifiers
        let mut speed = movement_speed.0 * crate::game::constants::GLOBAL_SPEED_MULTIPLIER;
        if let Some(slow) = slow_modifier {
            speed *= slow.modifier;
        }
        if let Some(haste) = haste_modifier {
            speed *= 1.0 + haste.modifier;
        }
        speed = speed.max(AERIALIST_MIN_SPEED);

        // Current velocity direction
        let current_vel = Vec3::new(velocity.x, 0.0, velocity.z);
        let current_speed = current_vel.length();

        // If we have no velocity yet, pick a direction
        if current_speed < 1.0 {
            let dir = if desired_dir.length_squared() > 0.001 {
                desired_dir.normalize()
            } else {
                // Random initial direction
                let angle = game_rng.0.random::<f32>() * std::f32::consts::TAU;
                Vec3::new(angle.cos(), 0.0, angle.sin())
            };
            velocity.x = dir.x * speed;
            velocity.z = dir.z * speed;
            continue;
        }

        let current_dir = current_vel / current_speed;

        // Gradually rotate current direction toward desired direction
        let target_dir = if desired_dir.length_squared() > 0.001 {
            desired_dir.normalize()
        } else {
            current_dir
        };

        // Calculate angle between current and desired direction
        let dot = current_dir.dot(target_dir).clamp(-1.0, 1.0);
        let angle_between = dot.acos();

        // Limit turn rate per frame
        let max_turn = AERIALIST_TURN_RATE * delta;
        let new_dir = if angle_between <= max_turn || angle_between < 0.001 {
            target_dir
        } else {
            // Slerp-like interpolation on XZ plane
            let cross_y = current_dir.x * target_dir.z - current_dir.z * target_dir.x;
            let turn_sign = if cross_y >= 0.0 { 1.0 } else { -1.0 };
            let cos_turn = max_turn.cos();
            let sin_turn = max_turn.sin() * turn_sign;
            Vec3::new(
                current_dir.x * cos_turn - current_dir.z * sin_turn,
                0.0,
                current_dir.x * sin_turn + current_dir.z * cos_turn,
            )
            .normalize_or_zero()
        };

        // Smoothly adjust speed toward target speed
        let new_speed = current_speed + (speed - current_speed) * (3.0 * delta).min(1.0);
        let final_speed = new_speed.max(AERIALIST_MIN_SPEED);

        velocity.x = new_dir.x * final_speed;
        velocity.z = new_dir.z * final_speed;
    }
}

/// Clamps aerialist Y position to fly height after all movement is applied.
pub fn clamp_aerialist_height(
    mut aerialists: Query<&mut Transform, (With<Aerialist>, Without<Corpse>)>,
) {
    for mut transform in &mut aerialists {
        transform.translation.y = AERIALIST_FLY_HEIGHT;
    }
}

/// Spawns a single attacker aerialist unit at a specific index.
pub(in crate::game) fn spawn_single_attacker_aerialist(
    rng: &mut impl Rng,
    commands: &mut Commands,
    aerialist_assets: &AerialistAssets,
    materials: &mut Assets<StandardMaterial>,
    unit_index: u32,
    _level: u32,
) -> Entity {
    // Spawn with archers (same depth offset)
    let (spawn_x, spawn_z) = attacker_spawn_position(unit_index, ARCHER_SPAWN_DEPTH_OFFSET);
    let (final_x, final_z) = random_position_in_cell(rng, spawn_x, spawn_z);

    let hitbox = Hitbox::new(AERIALIST_RADIUS, ATTACKER_HITBOX_HEIGHT);

    // Flying animation: 2 frames per direction (not the default 9-frame walk cycle)
    use crate::game::units::components::{
        SPRITE_DIRECTION_ROWS, SPRITE_SHEET_IMAGE_HEIGHT, sprite_frame_uv,
    };
    let anim = WalkingAnimation {
        current_frame: 0,
        elapsed: rng.random::<f32>() * 0.125, // stagger start
        columns: AERIALIST_FLYING_FRAMES,
        frame_uv: sprite_frame_uv(SPRITE_SHEET_IMAGE_HEIGHT),
        direction_rows: SPRITE_DIRECTION_ROWS,
    };
    let tint = Color::WHITE;
    let material = crate::game::units::systems::create_sprite_material(
        materials,
        aerialist_assets.sprite_texture.clone(),
        tint,
        anim.frame_uv,
        anim.uv_offset(FacingDirection::default()),
    );

    commands
        .spawn((
            Mesh3d(aerialist_assets.sprite_mesh.clone()),
            MeshMaterial3d(material),
            Transform::from_xyz(final_x, AERIALIST_FLY_HEIGHT, final_z),
            Velocity::default(),
            Acceleration::new(),
            hitbox,
            Health::new(crate::game::constants::UNIT_HEALTH * 1.2),
            MovementSpeed(AERIALIST_MOVEMENT_SPEED),
            AttackTiming::new(),
            Effectiveness::new(),
            Team::Attackers,
            Aerialist,
            Flying,
        ))
        .insert((
            anim,
            FacingDirection::default(),
            TargetingVelocity::default(),
            FlockingVelocity::default(),
            FlowFieldVelocity::default(),
            FlowFieldInfluence::Attacker,
            Teleportable,
            Billboard,
            OnGameplayScreen,
        ))
        .id()
}
