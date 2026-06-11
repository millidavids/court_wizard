use bevy::prelude::*;
use rand::Rng;

use super::super::components::*;
use super::super::constants::*;
use super::super::resources::DarkMageAssets;
use crate::game::components::{Acceleration, Billboard, OnGameplayScreen, Velocity};
use crate::game::constants::*;
use crate::game::pathfinding::{FlowFieldInfluence, FlowFieldVelocity};
use crate::game::units::boss::components::Boss;
use crate::game::units::components::{
    AnimationOverride, AttackTiming, DamageMultiplier, Effectiveness, FacingDirection,
    FlockingModifier, FlockingVelocity, Health, Hitbox, MovementSpeed, TargetingVelocity, Team,
    Teleportable, WalkingAnimation,
};

/// Spawns the Dark Mage at a tunnel spawn point (walks in like other bosses).
pub fn spawn_dark_mage(rng: &mut impl Rng, mut commands: Commands, assets: Res<DarkMageAssets>) {
    let (spawn_x, spawn_z) = attacker_spawn_position(0, 0.0);
    let (final_x, final_z) = crate::game::units::random_position_in_cell(rng, spawn_x, spawn_z);

    let hitbox = Hitbox::new(DARK_MAGE_RADIUS, DARK_MAGE_HITBOX_HEIGHT);
    // Sprite quad is centered at its origin; lift it half its height plus the
    // hover offset so the bottom of the sprite hovers above the ground.
    let base_y = DARK_MAGE_SPRITE_HEIGHT / 2.0 + DARK_MAGE_FLOAT_BASE_OFFSET;

    // Initial velocity toward castle (approach phase)
    let to_center = Vec3::new(
        WIZARD_POSITION.x - final_x,
        0.0,
        WIZARD_POSITION.z - final_z,
    );
    let initial_velocity = to_center.normalize_or_zero() * DARK_MAGE_APPROACH_SPEED;

    commands
        .spawn((
            // Rendering
            Mesh3d(assets.mesh.clone()),
            MeshMaterial3d(assets.floating_material.clone()),
            Transform::from_xyz(final_x, base_y, final_z),
            // Physics
            Velocity {
                x: initial_velocity.x,
                z: initial_velocity.z,
                ..default()
            },
            Acceleration::new(),
            // Core
            hitbox,
            Health::new(DARK_MAGE_HEALTH),
            MovementSpeed(DARK_MAGE_APPROACH_SPEED),
            AttackTiming::new(),
            Effectiveness::new(),
            Team::Attackers,
            Boss,
            DarkMage,
        ))
        .insert((
            DarkMageState::Approaching,
            DarkMageSpellCooldowns::new(
                METEOR_COOLDOWN * 0.3,    // First meteor comes relatively fast
                LIGHTNING_COOLDOWN * 0.1, // Lightning comes first
                PLAGUE_COOLDOWN * 0.5,    // Plague a bit later
            ),
            DarkMageSpellQueue::new(),
            DarkMageTeleportTimer::new(TELEPORT_COOLDOWN),
            DarkMageEnrage::new(),
            DamageMultiplier(DARK_MAGE_DAMAGE_MULTIPLIER),
            crate::game::units::components::MeleeDamageReduction {
                multiplier: DARK_MAGE_MELEE_DAMAGE_REDUCTION,
            },
            // Movement systems (used during approach phase)
            TargetingVelocity::default(),
            FlowFieldVelocity::default(),
            FlowFieldInfluence::Attacker,
            FlockingVelocity::default(),
            FlockingModifier::new(0.0, 0.0, 0.0),
            Teleportable,
            Billboard,
            OnGameplayScreen,
        ))
        .insert((
            WalkingAnimation {
                columns: DARK_MAGE_SHEET_COLUMNS,
                frame_uv: DARK_MAGE_FRAME_UV,
                direction_rows: DARK_MAGE_DIRECTION_ROWS,
                ..Default::default()
            },
            FacingDirection::Forward,
            AnimationOverride,
            DarkMageFloatBase { base_y },
        ));
}
