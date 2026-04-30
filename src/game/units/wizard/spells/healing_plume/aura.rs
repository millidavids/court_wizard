//! Healing plume passive systems: font of life, rain zones, field medic.

use super::components::{
    CleansingPlumeZone, FieldMedicConverted, FieldMedicOriginalType, FontOfLifePending,
    FontOfLifeZone, HealingPlumeTalentParams, HealingPlumeZone, HealingRainZone, OverflowZone,
    TriagePulseZone,
};
use super::constants;
use crate::game::components::OnGameplayScreen;
use crate::game::crt_effect::CorrectedCursorPosition;
use crate::game::multiplayer::components::NetworkedSpellEffect;
use crate::game::units::components::{Corpse, Team};
use crate::game::units::wizard::spells::utils::{get_cursor_world_position, xz_distance};
use crate::game::units::wizard::spells::visual_assets::SpellVisualAssets;
use crate::networking::snapshot::SpellEffectKind;
use bevy::prelude::*;

/// Computes talent parameters from active talent selections.
pub fn font_of_life_detect_deaths(
    mut commands: Commands,
    mut zones: Query<(Entity, &HealingPlumeZone, &mut FontOfLifeZone)>,
    new_corpses: Query<(Entity, &Transform), (With<Corpse>, Without<FontOfLifePending>)>,
) {
    for (_zone_entity, zone, mut font) in &mut zones {
        for (corpse_entity, transform) in &new_corpses {
            // Skip already-resurrected entities
            if font.resurrected.contains(&corpse_entity) {
                continue;
            }

            let distance = xz_distance(zone.origin, transform.translation);

            if distance <= zone.radius {
                font.resurrected.insert(corpse_entity);
                commands.entity(corpse_entity).insert(FontOfLifePending {
                    time_remaining: constants::FONT_OF_LIFE_RESURRECT_DELAY,
                });
            }
        }
    }
}

/// Tier 3: Font of Life — processes pending resurrections.
#[allow(clippy::too_many_arguments)]
pub fn font_of_life_resurrect(
    time: Res<Time>,
    mut commands: Commands,
    mut pending: Query<(Entity, &Transform, &mut FontOfLifePending), With<Corpse>>,
    infantry_assets: Res<crate::game::units::infantry::resources::InfantryAssets>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let delta = time.delta_secs();

    for (entity, transform, mut pending_res) in &mut pending {
        pending_res.time_remaining -= delta;

        if pending_res.time_remaining <= 0.0 {
            let max_hp = crate::game::constants::UNIT_HEALTH;
            let resurrect_hp = max_hp * constants::FONT_OF_LIFE_RESURRECT_HP_PERCENT;

            crate::game::units::systems::resurrect_corpse_as_infantry(
                &mut commands,
                entity,
                transform.translation,
                Team::Defenders,
                resurrect_hp,
                constants::FONT_OF_LIFE_RESURRECT_SPEED,
                Color::srgba(0.3, 0.9, 0.3, 1.0), // Green tint for resurrected
                infantry_assets.sprite_texture.clone(),
                infantry_assets.sprite_mesh.clone(),
                &mut materials,
                None,
            );

            commands.entity(entity).remove::<FontOfLifePending>();
        }
    }
}

/// Tier 3: Healing Rain — moves the zone toward the wizard's cursor each frame.
pub fn move_healing_rain_zones(
    time: Res<Time>,
    camera_query: Query<(&Camera, &GlobalTransform), With<Camera3d>>,
    corrected_cursor: Res<CorrectedCursorPosition>,
    mut zones: Query<(&mut HealingPlumeZone, &mut Transform), With<HealingRainZone>>,
) {
    let Some(cursor_pos) = get_cursor_world_position(&camera_query, &corrected_cursor) else {
        return;
    };
    let target = Vec3::new(cursor_pos.x, 0.0, cursor_pos.z);
    let delta = time.delta_secs();

    for (mut zone, mut transform) in &mut zones {
        let direction = target - zone.origin;
        let dist = direction.length();
        if dist < 1.0 {
            continue;
        }

        let move_amount = (constants::HEALING_RAIN_MOVE_SPEED * delta).min(dist);
        let offset = direction.normalize() * move_amount;
        zone.origin += offset;
        transform.translation.x = zone.origin.x;
        transform.translation.z = zone.origin.z;
    }
}

/// Tier 3: Field Medic — attempts to convert the nearest defender in the zone to a healer.
pub(super) fn try_convert_field_medic(
    commands: &mut Commands,
    zone_entity: Entity,
    position: Vec3,
    radius: f32,
    defenders_query: &Query<
        (
            Entity,
            &Transform,
            &Team,
            &MeshMaterial3d<StandardMaterial>,
            Has<crate::game::units::infantry::components::Infantry>,
            Has<crate::game::units::archer::components::Archer>,
        ),
        (
            Without<Corpse>,
            Without<crate::game::units::healer::components::Healer>,
        ),
    >,
    materials: &mut ResMut<Assets<StandardMaterial>>,
) {
    let mut best: Option<(
        Entity,
        f32,
        FieldMedicOriginalType,
        Handle<StandardMaterial>,
    )> = None;

    for (entity, transform, team, material_handle, is_infantry, is_archer) in defenders_query.iter()
    {
        if *team != Team::Defenders {
            continue;
        }

        let original_type = if is_archer {
            FieldMedicOriginalType::Archer
        } else if is_infantry {
            FieldMedicOriginalType::Infantry
        } else {
            continue;
        };

        let distance = xz_distance(position, transform.translation);

        if distance <= radius && best.as_ref().is_none_or(|(_, d, _, _)| distance < *d) {
            best = Some((entity, distance, original_type, material_handle.0.clone()));
        }
    }

    if let Some((entity, _, original_type, original_material)) = best {
        // Remove original unit marker immediately
        match original_type {
            FieldMedicOriginalType::Infantry => {
                commands
                    .entity(entity)
                    .remove::<crate::game::units::infantry::components::Infantry>();
            }
            FieldMedicOriginalType::Archer => {
                commands
                    .entity(entity)
                    .remove::<crate::game::units::archer::components::Archer>();
            }
        }

        // Create green-tinted material for the converted healer
        let (r, g, b, a) = constants::FIELD_MEDIC_COLOR;
        let green_material = materials.add(StandardMaterial {
            base_color: Color::srgba(r, g, b, a),
            ..Default::default()
        });

        commands.entity(entity).insert((
            crate::game::units::healer::components::Healer,
            crate::game::units::healer::components::HealerAttackTimer::new(),
            MeshMaterial3d(green_material),
            FieldMedicConverted {
                zone_entity,
                original_type,
                original_material,
            },
        ));
    }
}

/// Tier 3: Field Medic — reverts converted healers when the zone expires.
pub fn field_medic_cleanup(
    mut commands: Commands,
    zones: Query<Entity, With<HealingPlumeZone>>,
    converted_units: Query<(Entity, &FieldMedicConverted)>,
) {
    for (entity, converted) in &converted_units {
        // If the zone no longer exists, revert the conversion
        if zones.get(converted.zone_entity).is_err() {
            commands
                .entity(entity)
                .remove::<crate::game::units::healer::components::Healer>()
                .remove::<crate::game::units::healer::components::HealerAttackTimer>()
                .remove::<FieldMedicConverted>()
                .insert(MeshMaterial3d(converted.original_material.clone()));

            // Restore original unit type marker
            match converted.original_type {
                FieldMedicOriginalType::Infantry => {
                    commands
                        .entity(entity)
                        .insert(crate::game::units::infantry::components::Infantry);
                }
                FieldMedicOriginalType::Archer => {
                    commands
                        .entity(entity)
                        .insert(crate::game::units::archer::components::Archer);
                }
            }
        }
    }
}

/// Fades healing plume zone visual over the last few seconds.
pub fn fade_healing_plume_zone(mut zones: Query<(&HealingPlumeZone, &mut Transform)>) {
    const GROW_DURATION: f32 = 1.5;

    for (zone, mut transform) in &mut zones {
        // Slow growth over the first portion of lifetime
        let grow_t = (zone.time_alive / GROW_DURATION).min(1.0);
        // Ease-out curve for smooth deceleration
        let eased = 1.0 - (1.0 - grow_t) * (1.0 - grow_t);
        let scale = zone.radius * eased;

        // Fade out at end by shrinking
        let remaining = zone.duration - zone.time_alive;
        let fade = if remaining < constants::FADE_DURATION {
            (remaining / constants::FADE_DURATION).max(0.0)
        } else {
            1.0
        };

        transform.scale = Vec3::splat(scale * fade);
    }
}

/// Despawns expired healing plume zones.
pub fn cleanup_healing_plume_zone(
    mut commands: Commands,
    zones: Query<(Entity, &HealingPlumeZone)>,
) {
    for (entity, zone) in &zones {
        if zone.time_alive >= zone.duration {
            commands.entity(entity).try_despawn();
        }
    }
}

pub(crate) fn spawn_healing_plume_zone(
    commands: &mut Commands,
    assets: &SpellVisualAssets,
    position: Vec3,
    radius: f32,
    empowerment: f32,
    talent_params: &HealingPlumeTalentParams,
    scorched_mult: f32,
) -> Entity {
    let duration =
        constants::ZONE_DURATION * empowerment * talent_params.duration_mult * scorched_mult;
    let mut heal = constants::HEAL_PER_TICK * empowerment * talent_params.heal_mult;
    if talent_params.healing_rain {
        heal *= constants::HEALING_RAIN_HEAL_MULT;
    }

    let zone_entity = commands
        .spawn((
            Mesh3d(assets.explosion_sphere.clone()),
            MeshMaterial3d(assets.healing_aura_sphere.clone()),
            Transform::from_translation(Vec3::new(position.x, 0.0, position.z))
                .with_scale(Vec3::splat(0.1)),
            HealingPlumeZone::new(
                Vec3::new(position.x, 0.0, position.z),
                radius,
                heal,
                constants::TICK_INTERVAL,
                duration,
            ),
            NetworkedSpellEffect {
                kind: SpellEffectKind::HealingPlumeZone,
            },
            OnGameplayScreen,
        ))
        .id();

    // Add talent-specific zone components
    if talent_params.cleansing_plume {
        commands
            .entity(zone_entity)
            .insert(CleansingPlumeZone::new());
    }
    if talent_params.overflow {
        commands.entity(zone_entity).insert(OverflowZone);
    }
    if talent_params.triage_pulse {
        commands.entity(zone_entity).insert(TriagePulseZone);
    }
    if talent_params.font_of_life {
        commands.entity(zone_entity).insert(FontOfLifeZone::new());
    }
    if talent_params.healing_rain {
        commands.entity(zone_entity).insert(HealingRainZone);
    }

    zone_entity
}
