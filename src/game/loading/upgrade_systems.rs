//! Systems for applying elite and commander upgrades to units.
//!
//! This module handles the actual transformation of base units into elites
//! or commanders by adding components, swapping materials, and spawning visual effects.

use bevy::prelude::*;

use super::constants::*;
use crate::game::components::OnGameplayScreen;
use crate::game::units::archer::Archer;
use crate::game::units::archer::components::{ArcherMovementTimer, AttackRange};
use crate::game::units::commander::{AuraDamageBuff, AuraSpeedBuff, Commander, TeamFilter};
use crate::game::units::components::Hitbox;
use crate::game::units::components::{Health, MovementSpeed, UnitTypeGlow};
use crate::game::units::constants::{
    COMMANDER_GLOW_COLOR, DISPELLER_GLOW_COLOR, HEALER_GLOW_COLOR, SHIELDER_GLOW_COLOR,
};
use crate::game::units::dispeller::components::Dispeller;
use crate::game::units::dispeller::constants::{
    DISPELLER_HEALTH, DISPELLER_MOVEMENT_SPEED, DISPELLER_RADIUS,
};
use crate::game::units::elite::{
    ELITE_ATTACK_SPEED_BONUS, ELITE_DAMAGE_BONUS, ELITE_HEALTH_BONUS, ELITE_SPEED_BONUS,
    EliteAttackSpeedBonus, EliteDamageBonus, EliteHealthBonus, EliteSpeedBonus,
};
use crate::game::units::healer::components::{Healer, HealerAttackTimer};
use crate::game::units::healer::constants::{HEALER_HEALTH, HEALER_MOVEMENT_SPEED, HEALER_RADIUS};
use crate::game::units::infantry::Infantry;
use crate::game::units::ranged_bolt::RangedAttackTimer;
use crate::game::units::shielder::components::Shielder;
use crate::game::units::shielder::constants::{
    SHIELDER_HEALTH, SHIELDER_MOVEMENT_SPEED, SHIELDER_RADIUS,
};
use crate::game::units::wizard::spells::visual_assets::SpellVisualAssets;

/// Scales a unit's transform and hitbox by a multiplier, adjusting Y position
/// so the enlarged sprite sits correctly on the ground.
fn apply_size_scaling(
    commands: &mut Commands,
    entity: Entity,
    current_transform: &Transform,
    current_hitbox: &Hitbox,
    multiplier: f32,
) {
    let mut new_transform = *current_transform;
    new_transform.scale *= multiplier;

    let new_hitbox = Hitbox::new(
        current_hitbox.radius * multiplier,
        current_hitbox.height * multiplier,
    );

    new_transform.translation.y = new_hitbox.height / 2.0 + 1.0;

    commands.entity(entity).insert(new_transform);
    commands.entity(entity).insert(new_hitbox);
}

/// Queries an entity's current Transform and Hitbox, returning copies if both exist.
/// Used by elite and commander upgrades which need the current size for scaling.
pub(in crate::game) fn get_transform_and_hitbox(
    entity: Entity,
    transform_query: &Query<&Transform>,
    hitbox_query: &Query<&Hitbox>,
) -> Option<(Transform, Hitbox)> {
    let transform = *transform_query.get(entity).ok()?;
    let hitbox = *hitbox_query.get(entity).ok()?;
    Some((transform, hitbox))
}

/// Applies elite upgrade to a unit entity.
///
/// Adds elite components (health, damage, speed bonuses), swaps material
/// to brighter color, and scales up the unit's transform.
pub(in crate::game) fn apply_elite_upgrade(
    commands: &mut Commands,
    entity: Entity,
    current_transform: &Transform,
    current_hitbox: &Hitbox,
) {
    commands.entity(entity).insert((
        EliteHealthBonus(ELITE_HEALTH_BONUS),
        EliteDamageBonus(ELITE_DAMAGE_BONUS),
        EliteSpeedBonus(ELITE_SPEED_BONUS),
        EliteAttackSpeedBonus(ELITE_ATTACK_SPEED_BONUS),
    ));

    apply_size_scaling(
        commands,
        entity,
        current_transform,
        current_hitbox,
        ELITE_SIZE_MULTIPLIER,
    );
}

/// Applies commander upgrade to a unit entity.
///
/// Adds commander components (aura provider with buffs), scales up the unit,
/// adds orange glow, and spawns a visible aura ring on the ground.
pub(in crate::game) fn apply_commander_upgrade(
    commands: &mut Commands,
    entity: Entity,
    _materials: &mut Assets<StandardMaterial>,
    _meshes: &mut Assets<Mesh>,
    spell_assets: &SpellVisualAssets,
    current_transform: &Transform,
    current_hitbox: &Hitbox,
) {
    // Add commander components + orange glow
    commands.entity(entity).insert((
        Commander {
            aura_radius: ATTACKER_COMMANDER_AURA_RADIUS,
            team_filter: TeamFilter::Attackers,
        },
        AuraDamageBuff(ATTACKER_COMMANDER_DAMAGE_BUFF),
        AuraSpeedBuff(ATTACKER_COMMANDER_SPEED_BUFF),
        UnitTypeGlow {
            color: COMMANDER_GLOW_COLOR,
        },
    ));

    apply_size_scaling(
        commands,
        entity,
        current_transform,
        current_hitbox,
        COMMANDER_SIZE_MULTIPLIER,
    );

    // Spawn visual aura sphere around commander
    let aura_entity = commands
        .spawn((
            Mesh3d(spell_assets.explosion_sphere.clone()),
            MeshMaterial3d(spell_assets.commander_aura_sphere.clone()),
            Transform::from_xyz(0.0, 0.0, 0.0)
                .with_scale(Vec3::splat(ATTACKER_COMMANDER_AURA_RADIUS)),
            OnGameplayScreen,
        ))
        .id();
    commands.entity(entity).add_child(aura_entity);
}

/// Applies dispeller upgrade to an archer entity.
///
/// Converts an attacker archer into a dispeller by swapping components:
/// removes archer-specific components, adds dispeller components and white glow,
/// swaps to dispeller sprite sheet, and updates stats to match dispeller configuration.
pub(in crate::game) fn apply_dispeller_upgrade(
    commands: &mut Commands,
    entity: Entity,
    dispeller_assets: &crate::game::units::dispeller::resources::DispellerAssets,
    materials: &mut Assets<StandardMaterial>,
) {
    // Remove archer-specific components
    commands
        .entity(entity)
        .remove::<(Archer, AttackRange, ArcherMovementTimer)>();

    // Add dispeller components + white glow
    commands.entity(entity).insert((
        Dispeller,
        RangedAttackTimer::new(),
        UnitTypeGlow {
            color: DISPELLER_GLOW_COLOR,
        },
    ));

    // Swap to dispeller sprite mesh and material
    let material = crate::game::units::systems::create_default_sprite_material(
        materials,
        dispeller_assets.sprite_texture.clone(),
        crate::game::units::dispeller::constants::DISPELLER_SPRITE_TINT,
    );
    commands.entity(entity).insert((
        Mesh3d(dispeller_assets.sprite_mesh.clone()),
        MeshMaterial3d(material),
        crate::game::units::components::WalkingAnimation::default(),
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
/// removes archer-specific components, adds healer components and green glow,
/// swaps to healer sprite sheet, and updates stats to match healer configuration.
pub(in crate::game) fn apply_healer_upgrade(
    commands: &mut Commands,
    entity: Entity,
    healer_assets: &crate::game::units::healer::resources::HealerAssets,
    materials: &mut Assets<StandardMaterial>,
) {
    // Remove archer-specific components
    commands
        .entity(entity)
        .remove::<(Archer, AttackRange, ArcherMovementTimer)>();

    // Add healer components + green glow
    commands.entity(entity).insert((
        Healer,
        HealerAttackTimer::new(),
        UnitTypeGlow {
            color: HEALER_GLOW_COLOR,
        },
    ));

    // Swap to healer sprite mesh and material
    let material = crate::game::units::systems::create_default_sprite_material(
        materials,
        healer_assets.sprite_texture.clone(),
        crate::game::units::healer::constants::HEALER_SPRITE_TINT,
    );
    commands.entity(entity).insert((
        Mesh3d(healer_assets.sprite_mesh.clone()),
        MeshMaterial3d(material),
        crate::game::units::components::WalkingAnimation::default(),
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

/// Applies shielder upgrade to an infantry entity.
///
/// Converts an attacker infantry into a shielder by swapping components:
/// removes infantry-specific components, adds shielder components and purple glow,
/// swaps to shielder sprite sheet, and updates stats to match shielder configuration.
pub(in crate::game) fn apply_shielder_upgrade(
    commands: &mut Commands,
    entity: Entity,
    shielder_assets: &crate::game::units::shielder::resources::ShielderAssets,
    materials: &mut Assets<StandardMaterial>,
) {
    // Remove infantry-specific components
    commands.entity(entity).remove::<Infantry>();

    // Add shielder components + purple glow
    commands.entity(entity).insert((
        Shielder,
        UnitTypeGlow {
            color: SHIELDER_GLOW_COLOR,
        },
    ));

    // Swap to shielder sprite mesh and material
    let material = crate::game::units::systems::create_default_sprite_material(
        materials,
        shielder_assets.sprite_texture.clone(),
        crate::game::units::shielder::constants::SHIELDER_SPRITE_TINT,
    );
    commands.entity(entity).insert((
        Mesh3d(shielder_assets.sprite_mesh.clone()),
        MeshMaterial3d(material),
        crate::game::units::components::WalkingAnimation::default(),
    ));

    // Update stats to shielder values
    commands.entity(entity).insert((
        Health::new(SHIELDER_HEALTH),
        MovementSpeed(SHIELDER_MOVEMENT_SPEED),
        Hitbox::new(
            SHIELDER_RADIUS,
            crate::game::constants::ATTACKER_HITBOX_HEIGHT,
        ),
    ));
}
