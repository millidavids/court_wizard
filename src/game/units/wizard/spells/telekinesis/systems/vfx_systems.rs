use super::super::components::{HarvestFlash, PsychicShockwave, TelekinesisIndicator};
use super::super::constants;
use crate::game::components::OnGameplayScreen;
use crate::game::drops::components::IngredientDrop;
use crate::game::units::components::{
    Health, Knockback, Team, TemporaryHitPoints, apply_spell_damage,
};
use crate::game::units::damage::DamageType;
use crate::game::units::wizard::spells::visual_assets::SpellVisualAssets;
use bevy::prelude::*;

/// Spawns a visual indicator ring around a targeted drop.
pub(super) fn spawn_indicator(
    commands: &mut Commands,
    assets: &SpellVisualAssets,
    position: Vec3,
    target_drop: Entity,
) -> Entity {
    commands
        .spawn((
            Mesh3d(assets.unit_circle.clone()),
            MeshMaterial3d(assets.telekinesis_indicator.clone()),
            Transform::from_translation(Vec3::new(position.x, constants::INDICATOR_Y, position.z))
                .with_rotation(Quat::from_rotation_x(-std::f32::consts::FRAC_PI_2))
                .with_scale(Vec3::splat(constants::INDICATOR_RADIUS)),
            TelekinesisIndicator::new(target_drop),
            OnGameplayScreen,
        ))
        .id()
}

/// Clones a shared material asset into a per-entity copy for independent alpha fade.
/// Returns true if the clone was performed (first call), false otherwise.
pub(super) fn clone_material_if_needed(
    commands: &mut Commands,
    entity: Entity,
    materials: &mut Assets<StandardMaterial>,
    source_handle: &Handle<StandardMaterial>,
    already_cloned: &mut bool,
) {
    if !*already_cloned {
        *already_cloned = true;
        if let Some(base_mat) = materials.get(source_handle).cloned() {
            let cloned = materials.add(base_mat);
            commands.entity(entity).insert(MeshMaterial3d(cloned));
        }
    }
}

/// T2: Harvest — deals damage to enemies near the pickup point and spawns flash overlays.
pub(super) fn apply_harvest_damage(
    commands: &mut Commands,
    pickup_pos: Vec3,
    visual_assets: &SpellVisualAssets,
    enemies_query: &mut Query<
        (
            Entity,
            &Transform,
            &Team,
            &mut Health,
            Option<&mut TemporaryHitPoints>,
        ),
        Without<IngredientDrop>,
    >,
) {
    let radius_sq = constants::HARVEST_RADIUS * constants::HARVEST_RADIUS;
    for (entity, transform, _team, mut health, temp_hp) in enemies_query.iter_mut() {
        let dx = transform.translation.x - pickup_pos.x;
        let dz = transform.translation.z - pickup_pos.z;
        if dx * dx + dz * dz <= radius_sq {
            apply_spell_damage(
                commands,
                entity,
                &mut health,
                temp_hp.map(|t| t.into_inner()),
                constants::HARVEST_DAMAGE,
                DamageType::Force,
                false,
            );
            // Spawn a light blue circle flash at the enemy's position
            commands.spawn((
                Mesh3d(visual_assets.unit_circle.clone()),
                MeshMaterial3d(visual_assets.harvest_flash_material.clone()),
                Transform::from_translation(Vec3::new(
                    transform.translation.x,
                    constants::HARVEST_FLASH_Y,
                    transform.translation.z,
                ))
                .with_rotation(Quat::from_rotation_x(-std::f32::consts::FRAC_PI_2))
                .with_scale(Vec3::splat(constants::HARVEST_FLASH_SCALE)),
                HarvestFlash {
                    time_remaining: constants::HARVEST_FLASH_DURATION,
                    material_cloned: false,
                },
                OnGameplayScreen,
            ));
        }
    }
}

/// T3: Psychic Shockwave — spawns an expanding torus ring from the ingredient pickup position.
/// Material is cloned per-entity on the first update frame for independent alpha fade.
pub(super) fn spawn_shockwave(
    commands: &mut Commands,
    visual_assets: &SpellVisualAssets,
    pickup_pos: Vec3,
) {
    commands.spawn((
        Mesh3d(visual_assets.shockwave_torus.clone()),
        MeshMaterial3d(visual_assets.shockwave_material.clone()),
        Transform::from_translation(Vec3::new(
            pickup_pos.x,
            constants::SHOCKWAVE_SPAWN_Y,
            pickup_pos.z,
        ))
        .with_scale(Vec3::splat(0.01)),
        PsychicShockwave {
            time_alive: 0.0,
            prev_radius: 0.0,
            origin: Vec3::new(pickup_pos.x, 0.0, pickup_pos.z),
            material_cloned: false,
        },
        OnGameplayScreen,
    ));
}

/// Updates telekinesis indicator visuals during casting.
pub(crate) fn update_telekinesis_indicator(
    time: Res<Time>,
    mut indicators: Query<(&mut TelekinesisIndicator, &mut Transform)>,
    drops: Query<&Transform, (With<IngredientDrop>, Without<TelekinesisIndicator>)>,
) {
    for (mut indicator, mut transform) in indicators.iter_mut() {
        indicator.time_alive += time.delta_secs();

        // Follow the drop's position
        if let Ok(drop_transform) = drops.get(indicator.target_drop) {
            transform.translation.x = drop_transform.translation.x;
            transform.translation.y = constants::INDICATOR_Y;
            transform.translation.z = drop_transform.translation.z;
        }

        // Pulse animation (unit-sized mesh scaled by radius)
        let pulse = indicator.pulse_scale();
        transform.scale = Vec3::splat(constants::INDICATOR_RADIUS * pulse);
    }
}

/// Updates harvest flash overlay entities — clones material on first frame, fades alpha, despawns.
pub(crate) fn update_harvest_flash(
    time: Res<Time>,
    mut commands: Commands,
    mut materials: ResMut<Assets<StandardMaterial>>,
    visual_assets: Res<SpellVisualAssets>,
    mut query: Query<(Entity, &mut HarvestFlash, &MeshMaterial3d<StandardMaterial>)>,
) {
    let delta = time.delta_secs();

    for (entity, mut flash, material_handle) in &mut query {
        clone_material_if_needed(
            &mut commands,
            entity,
            &mut materials,
            &visual_assets.harvest_flash_material,
            &mut flash.material_cloned,
        );

        flash.time_remaining -= delta;

        if flash.time_remaining <= 0.0 {
            commands.entity(entity).try_despawn();
            continue;
        }

        // Fade alpha over the flash duration
        let alpha =
            (flash.time_remaining / constants::HARVEST_FLASH_DURATION).clamp(0.0, 1.0) * 0.7;
        if let Some(mat) = materials.get_mut(material_handle) {
            mat.base_color = constants::HARVEST_FLASH_COLOR.with_alpha(alpha);
        }
    }
}

/// Updates expanding psychic shockwave torus rings.
///
/// Expands the ring, applies knockback to enemies as the ring passes over them,
/// fades alpha, and despawns when complete.
pub(crate) fn update_psychic_shockwave(
    time: Res<Time>,
    mut commands: Commands,
    mut materials: ResMut<Assets<StandardMaterial>>,
    visual_assets: Res<SpellVisualAssets>,
    mut shockwaves: Query<(
        Entity,
        &mut PsychicShockwave,
        &mut Transform,
        &MeshMaterial3d<StandardMaterial>,
    )>,
    enemies: Query<
        (Entity, &Transform, &Team),
        (Without<PsychicShockwave>, Without<IngredientDrop>),
    >,
) {
    let delta = time.delta_secs();

    for (entity, mut shockwave, mut transform, material_handle) in &mut shockwaves {
        clone_material_if_needed(
            &mut commands,
            entity,
            &mut materials,
            &visual_assets.shockwave_material,
            &mut shockwave.material_cloned,
        );

        shockwave.time_alive += delta;

        if shockwave.time_alive >= constants::SHOCKWAVE_EXPAND_DURATION {
            commands.entity(entity).try_despawn();
            continue;
        }

        let progress = shockwave.time_alive / constants::SHOCKWAVE_EXPAND_DURATION;
        let current_radius = constants::SHOCKWAVE_MAX_RADIUS * progress;

        // Scale the torus to current radius
        transform.scale = Vec3::splat(current_radius.max(0.1));

        // Ring collision: knockback enemies between prev_radius and current_radius
        let prev_r_sq = shockwave.prev_radius * shockwave.prev_radius;
        let curr_r_sq = current_radius * current_radius;
        let origin = shockwave.origin;

        for (enemy_entity, enemy_transform, _team) in &enemies {
            let diff = enemy_transform.translation - origin;
            let dist_sq = diff.x * diff.x + diff.z * diff.z;

            if dist_sq > prev_r_sq && dist_sq <= curr_r_sq && dist_sq > 0.001 {
                let direction = Vec3::new(diff.x, 0.0, diff.z);
                commands.entity(enemy_entity).insert(Knockback::new(
                    direction,
                    constants::SHOCKWAVE_KNOCKBACK_SPEED,
                    constants::SHOCKWAVE_KNOCKBACK_DURATION,
                ));
            }
        }

        shockwave.prev_radius = current_radius;

        // Fade alpha as the ring expands
        if let Some(mat) = materials.get_mut(material_handle) {
            let alpha = (1.0 - progress) * 0.6;
            mat.base_color = constants::SHOCKWAVE_COLOR.with_alpha(alpha);
        }
    }
}
