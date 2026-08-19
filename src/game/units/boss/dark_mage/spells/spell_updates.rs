use bevy::prelude::*;

use super::super::components::*;
use super::super::constants::*;
use crate::game::pathfinding::messages::{ObstacleChanged, ObstacleShape, ObstacleType};
use crate::game::units::components::{
    BanishedModifier, Corpse, Health, Hitbox, Team, TemporaryHitPoints, apply_spell_damage,
};
use crate::game::units::damage::DamageType;
use crate::game::units::king::components::SpellShield;
use crate::game::units::wizard::components::Wizard;
use crate::game::units::wizard::spells::visual_assets::{
    FireExplosionSphereMaterial, explosion_fade_opacity,
};

/// Grows the explosion sphere to full radius over `METEOR_EXPLOSION_GROWTH_TIME`,
/// fades its opacity over the last 40% of lifetime (matches `meteor_fall`), and
/// applies one-shot damage to defender units within radius on the first frame.
#[allow(clippy::type_complexity)]
pub fn update_meteor_explosions(
    time: Res<Time>,
    mut commands: Commands,
    mut sphere_materials: ResMut<Assets<FireExplosionSphereMaterial>>,
    mut explosions: Query<
        (
            Entity,
            &mut DarkMageMeteorExplosion,
            &mut Transform,
            &MeshMaterial3d<FireExplosionSphereMaterial>,
        ),
        Without<DarkMage>,
    >,
    mut targets: Query<
        (
            Entity,
            &Transform,
            &Hitbox,
            &Team,
            &mut Health,
            Option<&mut TemporaryHitPoints>,
            Has<SpellShield>,
        ),
        (
            Without<Corpse>,
            Without<BanishedModifier>,
            Without<DarkMageMeteorExplosion>,
            Without<Wizard>,
        ),
    >,
) {
    let delta = time.delta_secs();

    for (entity, mut explosion, mut transform, mat_handle) in &mut explosions {
        explosion.time_alive += delta;

        let progress = (explosion.time_alive / METEOR_EXPLOSION_DURATION).clamp(0.0, 1.0);

        let growth_t = (explosion.time_alive / METEOR_EXPLOSION_GROWTH_TIME).min(1.0);
        if growth_t < 1.0 {
            transform.scale = Vec3::splat(explosion.radius * growth_t);
        }

        if let Some(mut mat) = sphere_materials.get_mut(mat_handle) {
            mat.opacity = explosion_fade_opacity(progress);
        }

        if !explosion.damage_applied {
            explosion.damage_applied = true;
            let center = transform.translation;

            for (
                target_entity,
                target_transform,
                target_hitbox,
                team,
                mut health,
                temp_hp,
                has_shield,
            ) in &mut targets
            {
                if *team != Team::Defenders {
                    continue;
                }

                let dx = target_transform.translation.x - center.x;
                let dz = target_transform.translation.z - center.z;
                let dist = (dx * dx + dz * dz).sqrt();

                if dist <= explosion.radius + target_hitbox.radius {
                    apply_spell_damage(
                        &mut commands,
                        target_entity,
                        &mut health,
                        temp_hp.map(|t| t.into_inner()),
                        explosion.damage,
                        DamageType::Fire,
                        has_shield,
                    );
                }
            }
        }

        if explosion.time_alive >= METEOR_EXPLOSION_DURATION {
            commands.entity(entity).try_despawn();
        }
    }
}

/// Updates lightning strike visuals and applies one-time corridor damage.
#[allow(clippy::type_complexity)]
pub fn update_lightning_strikes(
    time: Res<Time>,
    mut commands: Commands,
    mut strikes: Query<(Entity, &Transform, &mut DarkMageLightningStrike), Without<DarkMage>>,
    mut targets: Query<
        (
            Entity,
            &Transform,
            &Hitbox,
            &Team,
            &mut Health,
            Option<&mut TemporaryHitPoints>,
            Has<SpellShield>,
        ),
        (
            Without<Corpse>,
            Without<BanishedModifier>,
            Without<DarkMageLightningStrike>,
            Without<Wizard>,
        ),
    >,
) {
    let delta = time.delta_secs();

    for (entity, strike_transform, mut strike) in &mut strikes {
        strike.lifetime -= delta;

        // Apply one-time damage in the corridor
        if !strike.damage_applied {
            strike.damage_applied = true;
            let center = strike_transform.translation;
            let dir = strike.direction;
            let perp = Vec3::new(-dir.z, 0.0, dir.x);

            for (
                target_entity,
                target_transform,
                target_hitbox,
                team,
                mut health,
                temp_hp,
                has_shield,
            ) in &mut targets
            {
                if *team != Team::Defenders {
                    continue;
                }

                let to_target = target_transform.translation - center;
                let along = to_target.dot(dir);
                let across = to_target.dot(perp).abs();

                if along.abs() <= strike.half_length + target_hitbox.radius
                    && across <= strike.half_width + target_hitbox.radius
                {
                    apply_spell_damage(
                        &mut commands,
                        target_entity,
                        &mut health,
                        temp_hp.map(|t| t.into_inner()),
                        strike.damage,
                        DamageType::Electric,
                        has_shield,
                    );
                }
            }
        }

        if strike.lifetime <= 0.0 {
            commands.entity(entity).try_despawn();
        }
    }
}

/// Updates persistent plague clouds: ticks damage and despawns when expired.
#[allow(clippy::type_complexity)]
pub fn update_plague_clouds(
    time: Res<Time>,
    mut commands: Commands,
    mut obstacle_events: MessageWriter<ObstacleChanged>,
    mut clouds: Query<(Entity, &Transform, &mut DarkMagePlagueCloud), Without<DarkMage>>,
    mut targets: Query<
        (
            Entity,
            &Transform,
            &Hitbox,
            &Team,
            &mut Health,
            Option<&mut TemporaryHitPoints>,
            Has<SpellShield>,
        ),
        (
            Without<Corpse>,
            Without<BanishedModifier>,
            Without<DarkMagePlagueCloud>,
            Without<Wizard>,
        ),
    >,
) {
    let delta = time.delta_secs();

    for (entity, cloud_transform, mut cloud) in &mut clouds {
        cloud.lifetime -= delta;
        cloud.tick_timer -= delta;

        if cloud.tick_timer <= 0.0 {
            cloud.tick_timer += PLAGUE_TICK_INTERVAL;

            let center = cloud_transform.translation;

            for (
                target_entity,
                target_transform,
                target_hitbox,
                team,
                mut health,
                temp_hp,
                has_shield,
            ) in &mut targets
            {
                if *team != Team::Defenders {
                    continue;
                }

                let dx = target_transform.translation.x - center.x;
                let dz = target_transform.translation.z - center.z;
                let dist = (dx * dx + dz * dz).sqrt();

                if dist <= cloud.radius + target_hitbox.radius {
                    apply_spell_damage(
                        &mut commands,
                        target_entity,
                        &mut health,
                        temp_hp.map(|t| t.into_inner()),
                        cloud.damage,
                        DamageType::Poison,
                        has_shield,
                    );
                }
            }
        }

        if cloud.lifetime <= 0.0 {
            // Remove hazard from flow field
            let center_xz = Vec2::new(cloud_transform.translation.x, cloud_transform.translation.z);
            obstacle_events.write(ObstacleChanged {
                bounds: Rect::from_center_size(center_xz, Vec2::splat(cloud.radius * 2.0)),
                obstacle_type: ObstacleType::Removed,
                shape: Some(ObstacleShape::circle(center_xz, cloud.radius)),
                rebuild: false,
            });
            commands.entity(entity).try_despawn();
        }
    }
}

/// Monitors health thresholds and updates enrage state. Lower phases shorten
/// spell cooldowns; the gameplay tell is the casting frequency itself.
pub fn dark_mage_enrage(mut bosses: Query<(&Health, &mut DarkMageEnrage), With<DarkMage>>) {
    for (health, mut enrage) in &mut bosses {
        let hp_ratio = health.current / health.max;

        let new_phase = if hp_ratio <= ENRAGE_PHASE_3_THRESHOLD {
            3
        } else if hp_ratio <= ENRAGE_PHASE_2_THRESHOLD {
            2
        } else if hp_ratio <= ENRAGE_PHASE_1_THRESHOLD {
            1
        } else {
            0
        };

        if new_phase != enrage.phase {
            enrage.phase = new_phase;
            enrage.cooldown_mult = match new_phase {
                1 => ENRAGE_1_COOLDOWN_MULT,
                2 => ENRAGE_2_COOLDOWN_MULT,
                3 => ENRAGE_3_COOLDOWN_MULT,
                _ => 1.0,
            };
        }
    }
}
