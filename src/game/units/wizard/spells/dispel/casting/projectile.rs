use super::super::components::*;
use super::super::constants;
use super::input::insert_talent_markers;
use crate::game::components::OnGameplayScreen;
use crate::game::units::wizard::spells::visual_assets::SpellVisualAssets;
use bevy::prelude::*;

// ===== Projectile + Impact Systems =====

/// Moves dispel projectiles. Detonates on ground impact (y<=0) or lifetime expiry.
/// Transfers talent markers from projectile to impact entity.
#[allow(clippy::type_complexity)]
pub fn move_dispel_projectiles(
    mut commands: Commands,
    time: Res<Time>,
    visual_assets: Res<SpellVisualAssets>,
    mut projectiles: Query<(
        Entity,
        &mut Transform,
        &mut DispelProjectile,
        Has<BroadSpectrum>,
        Has<ManaDrain>,
        Has<ExplosiveNullification>,
        Has<SpellReflection>,
        Has<NullZoneOnImpact>,
        Has<WizardCastDispel>,
    )>,
) {
    let delta = time.delta_secs();
    for (
        entity,
        mut transform,
        mut projectile,
        has_broad_spectrum,
        has_mana_drain,
        has_explosive,
        has_reflection,
        has_null_zone,
        has_wizard_cast,
    ) in &mut projectiles
    {
        // Move projectile
        transform.translation += projectile.velocity * delta;
        projectile.lifetime -= delta;

        // Detonate when hitting the battlefield (y<=0) or lifetime expired
        let hit_ground = transform.translation.y <= 0.0;
        if hit_ground || projectile.lifetime <= 0.0 {
            // Impact position slightly above ground so cross-plane sphere is visible
            let impact_pos = Vec3::new(transform.translation.x, 5.0, transform.translation.z);

            let mut impact_entity = commands.spawn((
                Mesh3d(visual_assets.explosion_sphere.clone()),
                MeshMaterial3d(visual_assets.guardian_aura_sphere.clone()),
                Transform::from_translation(impact_pos).with_scale(Vec3::ZERO),
                DispelImpact {
                    time_alive: 0.0,
                    duration: constants::IMPACT_DURATION,
                    expand_speed: projectile.expand_speed,
                },
                crate::game::multiplayer::components::NetworkedSpellEffect {
                    kind: crate::networking::snapshot::SpellEffectKind::DispelImpact,
                },
                OnGameplayScreen,
            ));

            // Transfer talent markers from projectile to impact
            let params = DispelTalentParams {
                broad_spectrum: has_broad_spectrum,
                mana_drain: has_mana_drain,
                explosive_nullification: has_explosive,
                spell_reflection: has_reflection,
                null_zone: has_null_zone,
                ..Default::default()
            };
            insert_talent_markers(&mut impact_entity, &params);
            if has_wizard_cast {
                impact_entity.insert(WizardCastDispel);
            }

            commands.entity(entity).try_despawn();
        }
    }
}
