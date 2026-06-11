use bevy::prelude::*;
use rand::Rng;

use crate::game::components::{Acceleration, Billboard, OnGameplayScreen, Velocity};
use crate::game::constants::*;
use crate::game::pathfinding::{FlowFieldInfluence, FlowFieldVelocity};
use crate::game::units::components::{
    CommanderAuraSpeedModifier, Effectiveness, FacingDirection, FlockingModifier, FlockingVelocity,
    Health, Hitbox, MovementSpeed, RoughTerrainModifier, TargetingVelocity, Team, Teleportable,
    UnitTypeGlow, WalkingAnimation,
};
use crate::game::units::random_position_in_cell;
use crate::game::units::ranged_bolt::RangedAttackTimer;
use crate::game::units::teleporter::components::{Teleporter, TeleporterState};
use crate::game::units::teleporter::constants::*;
use crate::game::units::teleporter::resources::TeleporterAssets;

pub(crate) fn spawn_single_teleporter(
    rng: &mut impl Rng,
    commands: &mut Commands,
    teleporter_assets: &TeleporterAssets,
    materials: &mut Assets<StandardMaterial>,
) {
    let (spawn_x, spawn_z) = attacker_spawn_position(0, 0.0);
    let (final_x, final_z) = random_position_in_cell(rng, spawn_x, spawn_z);

    let hitbox = Hitbox::new(TELEPORTER_RADIUS, TELEPORTER_HITBOX_HEIGHT);
    let spawn_y = hitbox.height / 2.0 + 1.0;

    let to_center = Vec3::new(
        WIZARD_POSITION.x - final_x,
        0.0,
        WIZARD_POSITION.z - final_z,
    );
    let initial_velocity = to_center.normalize_or_zero() * TELEPORTER_MOVEMENT_SPEED;

    let anim = WalkingAnimation::new_staggered(rng);
    let material = crate::game::units::systems::create_default_sprite_material(
        materials,
        teleporter_assets.sprite_texture.clone(),
        TELEPORTER_SPRITE_TINT,
    );

    commands
        .spawn((
            Mesh3d(teleporter_assets.sprite_mesh.clone()),
            MeshMaterial3d(material),
            Transform::from_xyz(final_x, spawn_y, final_z)
                .with_scale(Vec3::splat(TELEPORTER_SCALE)),
            Velocity {
                x: initial_velocity.x,
                z: initial_velocity.z,
                ..default()
            },
            Acceleration::new(),
            hitbox,
            Health::new(TELEPORTER_HEALTH),
            MovementSpeed(TELEPORTER_MOVEMENT_SPEED),
            Effectiveness::new(),
            Team::Attackers,
            Teleporter,
            TargetingVelocity::default(),
            FlowFieldVelocity::default(),
            FlowFieldInfluence::Attacker,
        ))
        .insert((
            TeleporterState::default(),
            RangedAttackTimer::new(),
            anim,
            FacingDirection::default(),
            FlockingVelocity::default(),
            FlockingModifier::new(1.0, 1.0, 1.0),
            CommanderAuraSpeedModifier(0.0),
            RoughTerrainModifier(0.0),
            Teleportable,
            Billboard,
            OnGameplayScreen,
            UnitTypeGlow {
                color: TELEPORTER_GLOW_COLOR,
            },
        ));
}
