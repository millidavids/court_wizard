//! Shared ranged-bolt attack support for magic units (dispeller, teleporter).

use bevy::prelude::*;

use crate::game::components::{Billboard, OnGameplayScreen};
use crate::game::units::components::{
    BanishedModifier, Corpse, Health, Hitbox, Team, TemporaryHitPoints, apply_damage_to_unit,
};
use crate::game::units::shielder::components::ShielderDamageReduction;
use crate::game::units::shielder::constants::SHIELDER_DAMAGE_REDUCTION;

#[derive(Component)]
pub(in crate::game) struct RangedAttackTimer {
    pub time_since_last_attack: f32,
}

impl RangedAttackTimer {
    pub const fn new() -> Self {
        Self {
            time_since_last_attack: 999.0,
        }
    }
}

#[derive(Component)]
pub(in crate::game) struct MagicBolt {
    pub velocity: Vec3,
    pub damage: f32,
    pub source_team: Team,
    pub lifetime: f32,
    pub radius: f32,
}

#[allow(clippy::too_many_arguments)]
pub(in crate::game) fn spawn_magic_bolt(
    commands: &mut Commands,
    mesh: &Handle<Mesh>,
    material: &Handle<StandardMaterial>,
    origin: Vec3,
    direction: Vec3,
    speed: f32,
    damage: f32,
    radius: f32,
    lifetime: f32,
    source_team: Team,
) {
    commands.spawn((
        Mesh3d(mesh.clone()),
        MeshMaterial3d(material.clone()),
        Transform::from_translation(origin),
        MagicBolt {
            velocity: direction * speed,
            damage,
            source_team,
            lifetime,
            radius,
        },
        Billboard,
        OnGameplayScreen,
    ));
}

pub(in crate::game) fn move_magic_bolts(
    mut commands: Commands,
    time: Res<Time>,
    mut bolts: Query<(Entity, &mut Transform, &mut MagicBolt)>,
) {
    let delta = time.delta_secs();
    for (entity, mut transform, mut bolt) in &mut bolts {
        transform.translation += bolt.velocity * delta;
        bolt.lifetime -= delta;
        if bolt.lifetime <= 0.0 {
            commands.entity(entity).try_despawn();
        }
    }
}

#[allow(clippy::type_complexity)]
pub(in crate::game) fn check_magic_bolt_collisions(
    mut commands: Commands,
    bolts: Query<(Entity, &Transform, &MagicBolt)>,
    mut targets: Query<
        (
            &Transform,
            &Hitbox,
            &Team,
            &mut Health,
            Option<&mut TemporaryHitPoints>,
            Has<ShielderDamageReduction>,
        ),
        (Without<Corpse>, Without<BanishedModifier>),
    >,
) {
    for (bolt_entity, bolt_transform, bolt) in &bolts {
        let bolt_pos = bolt_transform.translation;

        for (target_transform, hitbox, team, mut health, mut temp_hp, has_shielder_reduction) in
            &mut targets
        {
            if !bolt.source_team.is_enemy(team) {
                continue;
            }

            if bolt_pos.distance(target_transform.translation) < hitbox.radius + bolt.radius {
                let mut damage = bolt.damage;
                if has_shielder_reduction {
                    damage *= SHIELDER_DAMAGE_REDUCTION;
                }
                apply_damage_to_unit(&mut health, temp_hp.as_deref_mut(), damage);
                commands.entity(bolt_entity).try_despawn();
                break;
            }
        }
    }
}
