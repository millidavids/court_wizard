//! Necrotic explosion: spawn AoE visual and apply damage.

use super::super::components::*;
use super::super::constants;
use crate::game::components::OnGameplayScreen;
use crate::game::units::components::{
    Health, Team, TemporaryHitPoints, apply_spell_damage_with_team,
};
use crate::game::units::damage::DamageType;
use crate::game::units::king::components::SpellShield;
use crate::game::units::wizard::components::Wizard;
use crate::game::units::wizard::spells::utils::local_player_team;
use crate::game::units::wizard::spells::vfx::constants::UPWARD_ROTATION;
use crate::game::units::wizard::spells::visual_assets::SpellVisualAssets;
use crate::networking::session::MultiplayerSession;
use bevy::prelude::*;

/// Spawns a necrotic explosion AoE visual and applies damage at a position.
pub(crate) fn spawn_necrotic_explosion(
    commands: &mut Commands,
    position: Vec3,
    damage: f32,
    visual_assets: &SpellVisualAssets,
    materials: &mut Assets<StandardMaterial>,
) {
    let pulse_material = materials
        .get(&visual_assets.necrotic_pulse)
        .cloned()
        .unwrap_or_default();
    let instance = materials.add(pulse_material);

    commands.spawn((
        NecroticExplosionBurst {
            time_alive: 0.0,
            lifetime: constants::NECROTIC_EXPLOSION_PULSE_LIFETIME,
            max_radius: constants::NECROTIC_EXPLOSION_RADIUS,
            damage,
            damage_applied: false,
        },
        Mesh3d(visual_assets.unit_circle.clone()),
        MeshMaterial3d(instance),
        Transform::from_translation(Vec3::new(
            position.x,
            constants::PULSE_Y_POSITION,
            position.z,
        ))
        .with_rotation(UPWARD_ROTATION)
        .with_scale(Vec3::splat(1.0)),
        OnGameplayScreen,
    ));
}

/// Applies necrotic explosion AoE damage to enemies near explosion positions (one-shot).
pub fn apply_necrotic_explosion_damage(
    mut commands: Commands,
    mut explosions: Query<(&Transform, &mut NecroticExplosionBurst)>,
    mut targets: Query<
        (
            Entity,
            &Transform,
            &mut Health,
            Option<&mut TemporaryHitPoints>,
            Has<SpellShield>,
            &Team,
        ),
        Without<Wizard>,
    >,
    session: Option<Res<MultiplayerSession>>,
) {
    let caster_team = local_player_team(session.as_deref());

    // Collect positions and damage of explosions that haven't applied damage yet
    let mut pending: Vec<(Vec3, f32)> = Vec::new();
    for (transform, mut burst) in explosions.iter_mut() {
        if !burst.damage_applied {
            burst.damage_applied = true;
            pending.push((transform.translation, burst.damage));
        }
    }

    for (explosion_pos, explosion_damage) in &pending {
        for (entity, transform, mut health, mut temp_hp, has_spell_shield, team) in
            targets.iter_mut()
        {
            let dist = crate::game::units::wizard::spells::utils::xz_distance(
                transform.translation,
                *explosion_pos,
            );
            if dist <= constants::NECROTIC_EXPLOSION_RADIUS {
                apply_spell_damage_with_team(
                    &mut commands,
                    entity,
                    &mut health,
                    temp_hp.as_deref_mut(),
                    *explosion_damage,
                    DamageType::Necrotic,
                    has_spell_shield,
                    caster_team,
                    *team,
                );
            }
        }
    }
}
