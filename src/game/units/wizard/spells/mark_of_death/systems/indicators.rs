use std::collections::HashSet;

use bevy::prelude::*;

use super::super::components::{ActiveMarkOfDeath, MarkVisualIndicator};
use super::super::constants;
use crate::game::components::OnGameplayScreen;
use crate::game::units::components::Corpse;
use crate::game::units::wizard::spells::visual_assets::SpellVisualAssets;

/// Computes the mark indicator pulse scale factor based on elapsed time.
fn mark_pulse_scale(elapsed_secs: f32) -> f32 {
    1.0 + (elapsed_secs * constants::MARK_INDICATOR_PULSE_SPEED * std::f32::consts::TAU).sin()
        * constants::MARK_INDICATOR_PULSE_AMPLITUDE
}

/// Spawns a purple circle indicator above newly marked units that don't have one yet.
pub fn spawn_mark_indicators(
    mut commands: Commands,
    marked_units: Query<(Entity, &Transform), (With<ActiveMarkOfDeath>, Without<Corpse>)>,
    existing_indicators: Query<&MarkVisualIndicator>,
    visual_assets: Res<SpellVisualAssets>,
    time: Res<Time>,
) {
    let tracked: HashSet<Entity> = existing_indicators.iter().map(|i| i.target).collect();

    for (entity, transform) in &marked_units {
        if tracked.contains(&entity) {
            continue;
        }

        let pos = transform.translation;
        let pulse = mark_pulse_scale(time.elapsed_secs());

        commands.spawn((
            MarkVisualIndicator { target: entity },
            Mesh3d(visual_assets.unit_circle.clone()),
            MeshMaterial3d(visual_assets.mark_indicator.clone()),
            Transform::from_translation(Vec3::new(
                pos.x,
                pos.y + constants::MARK_INDICATOR_Y_OFFSET,
                pos.z,
            ))
            .with_rotation(Quat::from_rotation_x(-std::f32::consts::FRAC_PI_2))
            .with_scale(Vec3::splat(constants::MARK_INDICATOR_RADIUS * pulse)),
            OnGameplayScreen,
        ));
    }
}

/// Updates mark indicator positions to follow their target and pulse.
/// Despawns indicators whose target no longer has a mark or is dead.
pub fn update_mark_indicators(
    mut commands: Commands,
    mut indicators: Query<(Entity, &MarkVisualIndicator, &mut Transform)>,
    marked_units: Query<&Transform, (With<ActiveMarkOfDeath>, Without<MarkVisualIndicator>)>,
    time: Res<Time>,
) {
    for (indicator_entity, indicator, mut indicator_transform) in &mut indicators {
        if let Ok(target_transform) = marked_units.get(indicator.target) {
            // Follow target position
            indicator_transform.translation.x = target_transform.translation.x;
            indicator_transform.translation.z = target_transform.translation.z;
            indicator_transform.translation.y =
                target_transform.translation.y + constants::MARK_INDICATOR_Y_OFFSET;

            // Pulse scale
            let pulse = mark_pulse_scale(time.elapsed_secs());
            indicator_transform.scale = Vec3::splat(constants::MARK_INDICATOR_RADIUS * pulse);
        } else {
            // Target lost its mark or died — despawn indicator
            commands.entity(indicator_entity).try_despawn();
        }
    }
}
