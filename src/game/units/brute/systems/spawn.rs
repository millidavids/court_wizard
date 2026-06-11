use bevy::prelude::*;
use rand::Rng;

use super::super::components::*;
use super::super::constants::*;
use crate::game::components::{Acceleration, Billboard, OnGameplayScreen, Velocity};
use crate::game::constants::*;
use crate::game::pathfinding::{FlowFieldInfluence, FlowFieldVelocity};
use crate::game::resources::CurrentLevel;
use crate::game::units::components::{
    AttackTiming, CommanderAuraSpeedModifier, DamageMultiplier, Effectiveness, FlockingModifier,
    FlockingVelocity, Health, Hitbox, MovementSpeed, RoughTerrainModifier, TargetingVelocity, Team,
    Teleportable, UnitTypeGlow,
};
use crate::game::units::infantry::constants::ATTACKER_SPRITE_TINT;
use crate::game::units::infantry::resources::InfantryAssets;
use crate::game::units::random_position_in_cell;

/// Spawns a brute attacker.
/// Brutes spawn in the archer row alongside archers.
pub(in crate::game) fn spawn_brute(
    rng: &mut impl Rng,
    mut commands: Commands,
    infantry_assets: Res<InfantryAssets>,
    materials: &mut Assets<StandardMaterial>,
    _current_level: Res<CurrentLevel>,
) {
    // Brute spawns at the front with infantry
    let (spawn_x, spawn_z) = attacker_spawn_position(0, 0.0);
    let (final_x, final_z) = random_position_in_cell(rng, spawn_x, spawn_z);

    let hitbox = Hitbox::new(BRUTE_RADIUS, BRUTE_HITBOX_HEIGHT);
    let spawn_y = hitbox.height / 2.0 + 1.0;

    let to_center = Vec3::new(
        WIZARD_POSITION.x - final_x,
        0.0,
        WIZARD_POSITION.z - final_z,
    );
    let initial_velocity = to_center.normalize_or_zero() * BRUTE_MOVEMENT_SPEED;

    // Use infantry sprite mesh but scaled larger
    let anim = crate::game::units::components::WalkingAnimation::new_staggered(rng);
    let material = crate::game::units::systems::create_default_sprite_material(
        materials,
        infantry_assets.sprite_texture.clone(),
        ATTACKER_SPRITE_TINT,
    );

    commands
        .spawn((
            // Rendering — infantry sprite, scaled up
            Mesh3d(infantry_assets.sprite_mesh.clone()),
            MeshMaterial3d(material),
            Transform::from_xyz(final_x, spawn_y, final_z).with_scale(Vec3::splat(BRUTE_SCALE)),
            // Physics
            Velocity {
                x: initial_velocity.x,
                z: initial_velocity.z,
                ..default()
            },
            Acceleration::new(),
            // Core
            hitbox,
            Health::new(BRUTE_HEALTH),
            MovementSpeed(BRUTE_MOVEMENT_SPEED),
            AttackTiming::new(),
            Effectiveness::new(),
            Team::Attackers,
            Brute,
            // Movement systems
            TargetingVelocity::default(),
            FlowFieldVelocity::default(),
            FlowFieldInfluence::Attacker,
        ))
        .insert((
            anim,
            crate::game::units::components::FacingDirection::default(),
            // Normal single-target melee damage
            DamageMultiplier(0.0),
            FlockingVelocity::default(),
            FlockingModifier::new(1.0, 1.0, 1.0),
            CommanderAuraSpeedModifier(0.0),
            RoughTerrainModifier(0.0),
            Teleportable,
            Billboard,
            OnGameplayScreen,
            UnitTypeGlow {
                color: crate::game::units::constants::BRUTE_GLOW_COLOR,
            },
            // Rock throw with initial cooldown so brute doesn't throw immediately on spawn
            RockThrowCooldown::new(5.0),
        ));
}
