//! Systems for applying elite and commander upgrades to units.
//!
//! This module handles the actual transformation of base units into elites
//! or commanders by adding components, swapping materials, and spawning visual effects.

use bevy::prelude::*;

use super::constants::*;
use super::spawn_queue::UnitType;
use crate::game::components::OnGameplayScreen;
use crate::game::units::archer::Archer;
use crate::game::units::archer::components::{ArcherMovementTimer, AttackRange};
use crate::game::units::commander::{AuraDamageBuff, AuraSpeedBuff, Commander, TeamFilter};
use crate::game::units::components::Hitbox;
use crate::game::units::components::{Health, MovementSpeed};
use crate::game::units::dispeller::DispellerAssets;
use crate::game::units::dispeller::components::{Dispeller, DispellerAttackTimer};
use crate::game::units::dispeller::constants::{
    DISPELLER_HEALTH, DISPELLER_MOVEMENT_SPEED, DISPELLER_RADIUS,
};
use crate::game::units::elite::{
    ELITE_DAMAGE_BONUS, ELITE_HEALTH_BONUS, ELITE_SPEED_BONUS, EliteDamageBonus, EliteHealthBonus,
    EliteSpeedBonus,
};
use crate::game::units::healer::HealerAssets;
use crate::game::units::healer::components::{Healer, HealerAttackTimer};
use crate::game::units::healer::constants::{HEALER_HEALTH, HEALER_MOVEMENT_SPEED, HEALER_RADIUS};

/// Applies elite upgrade to a unit entity.
///
/// Adds elite components (health, damage, speed bonuses), swaps material
/// to brighter color, and scales up the unit's transform.
pub(super) fn apply_elite_upgrade(
    commands: &mut Commands,
    entity: Entity,
    unit_type: UnitType,
    materials: &mut Assets<StandardMaterial>,
    current_transform: &Transform,
    current_hitbox: &Hitbox,
) {
    // Add elite components
    commands.entity(entity).insert((
        EliteHealthBonus(ELITE_HEALTH_BONUS),
        EliteDamageBonus(ELITE_DAMAGE_BONUS),
        EliteSpeedBonus(ELITE_SPEED_BONUS),
    ));

    // Swap material to elite color
    let elite_color = match unit_type {
        UnitType::Infantry => ELITE_INFANTRY_COLOR,
        UnitType::Archer => ELITE_ARCHER_COLOR,
    };

    let elite_material = materials.add(StandardMaterial {
        base_color: elite_color,
        unlit: true,
        ..default()
    });

    commands
        .entity(entity)
        .insert(MeshMaterial3d(elite_material));

    // Scale up the unit (preserving position)
    let mut new_transform = *current_transform;
    new_transform.scale *= ELITE_SIZE_MULTIPLIER;
    commands.entity(entity).insert(new_transform);

    // Scale hitbox to match new size
    let new_hitbox = Hitbox::new(
        current_hitbox.radius * ELITE_SIZE_MULTIPLIER,
        current_hitbox.height * ELITE_SIZE_MULTIPLIER,
    );
    commands.entity(entity).insert(new_hitbox);
}

/// Applies commander upgrade to a unit entity.
///
/// Adds commander components (aura provider with buffs), swaps material
/// to gold color, scales up the unit, and spawns a visible aura ring on the ground.
pub(super) fn apply_commander_upgrade(
    commands: &mut Commands,
    entity: Entity,
    unit_type: UnitType,
    materials: &mut Assets<StandardMaterial>,
    meshes: &mut Assets<Mesh>,
    current_transform: &Transform,
    current_hitbox: &Hitbox,
) {
    // Add commander components
    commands.entity(entity).insert((
        Commander {
            aura_radius: ATTACKER_COMMANDER_AURA_RADIUS,
            team_filter: TeamFilter::Attackers,
            visual_color: ATTACKER_COMMANDER_AURA_COLOR,
        },
        AuraDamageBuff(ATTACKER_COMMANDER_DAMAGE_BUFF),
        AuraSpeedBuff(ATTACKER_COMMANDER_SPEED_BUFF),
    ));

    // Swap material to commander color
    let commander_color = match unit_type {
        UnitType::Infantry => COMMANDER_INFANTRY_COLOR,
        UnitType::Archer => COMMANDER_ARCHER_COLOR,
    };

    let commander_material = materials.add(StandardMaterial {
        base_color: commander_color,
        unlit: true,
        ..default()
    });

    commands
        .entity(entity)
        .insert(MeshMaterial3d(commander_material));

    // Scale up the unit (preserving position)
    let mut new_transform = *current_transform;
    new_transform.scale *= COMMANDER_SIZE_MULTIPLIER;
    commands.entity(entity).insert(new_transform);

    // Scale hitbox to match new size
    let new_hitbox = Hitbox::new(
        current_hitbox.radius * COMMANDER_SIZE_MULTIPLIER,
        current_hitbox.height * COMMANDER_SIZE_MULTIPLIER,
    );
    commands.entity(entity).insert(new_hitbox);

    // Spawn visual aura ring on ground
    spawn_aura_ring(commands, entity, current_transform, materials, meshes);
}

/// Spawns a visible aura ring on the ground beneath a commander.
///
/// Creates a flat ring mesh with the aura radius, positioned at ground level,
/// and parents it to the commander entity so it follows the unit.
fn spawn_aura_ring(
    commands: &mut Commands,
    parent_entity: Entity,
    parent_transform: &Transform,
    materials: &mut Assets<StandardMaterial>,
    meshes: &mut Assets<Mesh>,
) {
    // Create filled circle mesh
    let ring_mesh = meshes.add(Circle::new(ATTACKER_COMMANDER_AURA_RADIUS));

    // Create semi-transparent red material
    let ring_material = materials.add(StandardMaterial {
        base_color: ATTACKER_COMMANDER_AURA_COLOR,
        alpha_mode: AlphaMode::Blend,
        unlit: true,
        ..default()
    });

    // Position ring at ground level relative to parent
    // Since the ring is parented to the commander, use local coordinates (0, y_offset, 0)
    // y_offset brings the ring down from the parent's position to ground level
    let y_offset = 5.0 - parent_transform.translation.y;
    let ring_position = Vec3::new(0.0, y_offset, 0.0);

    // Spawn ring and parent it to the commander
    let ring_entity = commands
        .spawn((
            Mesh3d(ring_mesh),
            MeshMaterial3d(ring_material),
            Transform::from_translation(ring_position)
                .with_rotation(Quat::from_rotation_x(-std::f32::consts::FRAC_PI_2)),
            OnGameplayScreen,
        ))
        .id();

    // Parent the ring to the commander so it follows them
    commands.entity(parent_entity).add_child(ring_entity);
}

/// Applies dispeller upgrade to an archer entity.
///
/// Converts an attacker archer into a dispeller by swapping components:
/// removes archer-specific components, adds dispeller components, and
/// updates mesh/material/stats to match dispeller configuration.
pub(super) fn apply_dispeller_upgrade(
    commands: &mut Commands,
    entity: Entity,
    dispeller_assets: &DispellerAssets,
) {
    // Remove archer-specific components
    commands
        .entity(entity)
        .remove::<(Archer, AttackRange, ArcherMovementTimer)>();

    // Add dispeller components
    commands
        .entity(entity)
        .insert((Dispeller, DispellerAttackTimer::new()));

    // Swap mesh and material to dispeller visuals
    commands.entity(entity).insert((
        Mesh3d(dispeller_assets.mesh.clone()),
        MeshMaterial3d(dispeller_assets.attacker_material.clone()),
    ));

    // Update stats to dispeller values
    commands.entity(entity).insert((
        Health::new(DISPELLER_HEALTH),
        MovementSpeed(DISPELLER_MOVEMENT_SPEED),
        Hitbox::new(
            DISPELLER_RADIUS,
            crate::game::constants::ATTACKER_HITBOX_HEIGHT,
        ),
    ));
}

/// Applies healer upgrade to an archer entity.
///
/// Converts an attacker archer into a healer by swapping components:
/// removes archer-specific components, adds healer components, and
/// updates mesh/material/stats to match healer configuration.
pub(super) fn apply_healer_upgrade(
    commands: &mut Commands,
    entity: Entity,
    healer_assets: &HealerAssets,
) {
    // Remove archer-specific components
    commands
        .entity(entity)
        .remove::<(Archer, AttackRange, ArcherMovementTimer)>();

    // Add healer components
    commands
        .entity(entity)
        .insert((Healer, HealerAttackTimer::new()));

    // Swap mesh and material to healer visuals
    commands.entity(entity).insert((
        Mesh3d(healer_assets.mesh.clone()),
        MeshMaterial3d(healer_assets.attacker_material.clone()),
    ));

    // Update stats to healer values
    commands.entity(entity).insert((
        Health::new(HEALER_HEALTH),
        MovementSpeed(HEALER_MOVEMENT_SPEED),
        Hitbox::new(
            HEALER_RADIUS,
            crate::game::constants::ATTACKER_HITBOX_HEIGHT,
        ),
    ));
}
