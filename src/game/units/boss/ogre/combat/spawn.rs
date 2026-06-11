use crate::game::units::components::Health;
use bevy::prelude::*;
use rand::Rng;

use super::super::components::*;
use super::super::constants::*;
use super::super::resources::OgreAssets;
use crate::game::components::{Acceleration, Billboard, OnGameplayScreen, Velocity};
use crate::game::constants::*;
use crate::game::pathfinding::{FlowFieldInfluence, FlowFieldVelocity};
use crate::game::units::boss::components::Boss;
use crate::game::units::brute::components::RockThrowCooldown;
use crate::game::units::components::{
    AttackTiming, CommanderAuraSpeedModifier, DamageMultiplier, Effectiveness, FlockingModifier,
    FlockingVelocity, Hitbox, MovementSpeed, RoughTerrainModifier, TargetingVelocity, Team,
    Teleportable,
};
use crate::game::units::components::{FacingDirection, WalkingAnimation};
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
