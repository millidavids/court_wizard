use crate::game::units::components::Corpse;
use bevy::prelude::*;
use rand::Rng;

use super::super::components::*;
use super::super::constants::*;
use super::super::resources::RayAssets;
use crate::config::GameConfig;
use crate::game::components::{Acceleration, Billboard, OnGameplayScreen, Velocity};
use crate::game::constants::*;
use crate::game::pathfinding::{FlowFieldInfluence, FlowFieldVelocity};
use crate::game::units::boss::components::Boss;
use crate::game::units::boss::utils::{EYE_FRAME_UV, EYE_PULSE_FRAME_DURATION, EYE_SHEET_COLUMNS};
use crate::game::units::components::Flying;
use crate::game::units::components::{
    AttackTiming, DamageMultiplier, Effectiveness, FlockingModifier, FlockingVelocity, Health,
    Hitbox, MovementSpeed, PulsingAnimation, TargetingVelocity, Team, Teleportable,
};

/// Attenuated volume for Ray's sound effects — slight falloff from wizard/camera position.
pub(crate) fn ray_sfx_volume(effect_pos: Vec3, game_config: &GameConfig) -> f32 {
    const RAY_SFX_BASE_SCALE: f32 = 0.6;
    const RAY_SFX_MAX_DIST: f32 = 8000.0;
    let distance = effect_pos.distance(crate::game::units::wizard::spells::audio::audio_origin());
    let linear = (1.0 - distance / RAY_SFX_MAX_DIST).clamp(0.0, 1.0);
    game_config.effective_sfx_volume() * linear * RAY_SFX_BASE_SCALE
}

pub fn spawn_ray(rng: &mut impl Rng, mut commands: Commands, assets: Res<RayAssets>) {
    let (spawn_x, spawn_z) = attacker_spawn_position(0, 0.0);
    let (final_x, final_z) = crate::game::units::random_position_in_cell(rng, spawn_x, spawn_z);

    let hitbox = Hitbox::new(RAY_BODY_RADIUS, RAY_BODY_HITBOX_HEIGHT);
    let spawn_y = RAY_BODY_HOVER_HEIGHT + hitbox.height / 2.0;

    let to_center = Vec3::new(
        WIZARD_POSITION.x - final_x,
        0.0,
        WIZARD_POSITION.z - final_z,
    );
    let initial_velocity = to_center.normalize_or_zero() * RAY_APPROACH_SPEED;

    commands
        .spawn((
            Mesh3d(assets.body_mesh.clone()),
            MeshMaterial3d(assets.body_material.clone()),
            Transform::from_xyz(final_x, spawn_y, final_z),
            Velocity {
                x: initial_velocity.x,
                z: initial_velocity.z,
                ..default()
            },
            Acceleration::new(),
            hitbox,
            Health::new(RAY_HEALTH),
            MovementSpeed(RAY_APPROACH_SPEED),
            AttackTiming::new(),
            Effectiveness::new(),
            Team::Attackers,
            Boss,
            Ray,
        ))
        .insert((
            RayState::Approaching,
            RayEyeState::new(),
            DamageMultiplier(RAY_DAMAGE_MULTIPLIER),
            crate::game::units::components::MeleeDamageReduction {
                multiplier: RAY_MELEE_DAMAGE_REDUCTION,
            },
            TargetingVelocity::default(),
            FlowFieldVelocity::default(),
            FlowFieldInfluence::Attacker,
            FlockingVelocity::default(),
            FlockingModifier::new(0.0, 0.0, 0.0),
            Teleportable,
            Billboard,
            OnGameplayScreen,
            PulsingAnimation::new_staggered(
                EYE_SHEET_COLUMNS,
                EYE_FRAME_UV,
                EYE_PULSE_FRAME_DURATION,
                rng,
            ),
        ))
        .insert((
            RayDisintegrationSweep {
                beam_entity: None,
                glow_entity: None,
                sfx_entity: None,
                tip_position: Vec2::ZERO,
                tip_velocity: Vec2::ZERO,
                cooldown: 0.0,
            },
            RayPetrificationSweep {
                beam_entity: None,
                glow_entity: None,
                cooldown: 3.0,
            },
            RayFearSweep {
                beam_entity: None,
                glow_entity: None,
                fear_cooldown: 0.0,
            },
            RayTeleportSweep { cooldown: 5.0 },
            RayMindControlSweep {
                beam_entity: None,
                glow_entity: None,
                cooldown: 4.0,
            },
        ));

    let angle_step = std::f32::consts::TAU / 6.0;
    for (i, &eye_type) in RayEyeType::ALL.iter().enumerate() {
        let angle = angle_step * i as f32;
        let offset_x = angle.cos() * RAY_EYE_WANDER_RADIUS * 0.5;
        let offset_z = angle.sin() * RAY_EYE_WANDER_RADIUS * 0.5;

        let eye_x = final_x + offset_x;
        let eye_z = final_z + offset_z;
        let eye_y = RAY_EYE_FLOAT_HEIGHT;

        commands.spawn((
            Mesh3d(assets.eye_sprite_mesh.clone()),
            MeshMaterial3d(assets.eye_materials[eye_type.index()].clone()),
            Transform::from_xyz(eye_x, eye_y, eye_z),
            Hitbox::new(RAY_EYE_RADIUS, RAY_EYE_HITBOX_HEIGHT),
            Health::new(RAY_EYE_HEALTH),
            // Eyes carry no Boss marker but must not be one-shot by the
            // percent-max-health Finger of Death execute — they take chip
            // damage like the boss body instead.
            crate::game::units::damage::FingerOfDeathResistant,
            Effectiveness::new(),
            AttackTiming::new(),
            Team::Attackers,
            RayEye {
                eye_type,
                heading: Vec2::new(angle.cos(), angle.sin()),
            },
            PulsingAnimation::new_staggered(
                EYE_SHEET_COLUMNS,
                EYE_FRAME_UV,
                EYE_PULSE_FRAME_DURATION,
                rng,
            ),
            Flying,
            Billboard,
            OnGameplayScreen,
        ));
    }
}

#[allow(clippy::too_many_arguments)]
pub fn ray_movement(
    time: Res<Time>,
    mut bosses: Query<
        (
            &Transform,
            &mut Velocity,
            &mut Acceleration,
            &MovementSpeed,
            &TargetingVelocity,
            &FlockingVelocity,
            &FlowFieldVelocity,
            &mut RayState,
        ),
        (With<Ray>, Without<Corpse>),
    >,
) {
    for (
        transform,
        mut velocity,
        mut acceleration,
        movement_speed,
        targeting_velocity,
        flocking_velocity,
        flow_field_velocity,
        mut state,
    ) in &mut bosses
    {
        if matches!(*state, RayState::Approaching) {
            crate::game::units::systems::calculate_weighted_movement(
                &time,
                &mut velocity,
                &mut acceleration,
                movement_speed.0,
                targeting_velocity,
                flocking_velocity,
                flow_field_velocity,
                false,
                None,
                None,
                None,
                None,
                None,
                None,
            );
            if transform.translation.x <= RAY_APPROACH_TARGET_X {
                *state = RayState::Idle;
            }
            continue;
        }

        crate::game::units::systems::calculate_weighted_movement(
            &time,
            &mut velocity,
            &mut acceleration,
            RAY_HOVER_SPEED,
            targeting_velocity,
            flocking_velocity,
            flow_field_velocity,
            false,
            None,
            None,
            None,
            None,
            None,
            None,
        );
    }
}
