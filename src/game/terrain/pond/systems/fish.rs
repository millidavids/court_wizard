use bevy::prelude::*;

use super::super::components::*;
use super::super::constants::*;
use crate::game::units::components::{Corpse, Health, TemporaryHitPoints, apply_damage_to_unit};
use crate::game::units::wizard::archetypes::meteorologist::components::WetModifier;
use crate::game::units::wizard::spells::visual_assets::SpellVisualAssets;

/// Ticks shocked ponds. Each arc cooldown, arcs lightning from the pond center to up to
/// `POND_SHOCK_MAX_TARGETS` nearby non-corpse units within `POND_SHOCK_ARC_RADIUS`.
/// Damage bypasses `PendingDamageEffect`, so arcs don't propagate the shock condition.
pub fn tick_pond_shocked(
    mut commands: Commands,
    time: Res<Time>,
    visual_assets: Res<SpellVisualAssets>,
    mut ponds: Query<(Entity, &Pond, &mut PondShocked)>,
    target_query: Query<(Entity, &Transform), (With<Health>, Without<Corpse>)>,
    mut health_query: Query<
        (
            &mut Health,
            Option<&mut TemporaryHitPoints>,
            Has<WetModifier>,
        ),
        Without<Corpse>,
    >,
) {
    let delta = time.delta_secs();

    for (entity, pond, mut shock) in &mut ponds {
        shock.time_remaining -= delta;
        shock.arc_cooldown = (shock.arc_cooldown - delta).max(0.0);

        if shock.time_remaining <= 0.0 {
            commands.entity(entity).remove::<PondShocked>();
            continue;
        }

        if shock.arc_cooldown > 0.0 {
            continue;
        }

        // Find nearby targets within the arc radius
        let radius_sq = POND_SHOCK_ARC_RADIUS * POND_SHOCK_ARC_RADIUS;
        let mut targets: Vec<(Entity, Vec3, f32)> = target_query
            .iter()
            .filter_map(|(target_entity, target_transform)| {
                let dx = pond.center.x - target_transform.translation.x;
                let dz = pond.center.z - target_transform.translation.z;
                let dist_sq = dx * dx + dz * dz;
                if dist_sq <= radius_sq {
                    Some((target_entity, target_transform.translation, dist_sq))
                } else {
                    None
                }
            })
            .collect();

        if targets.is_empty() {
            continue;
        }

        targets.sort_by(|a, b| a.2.partial_cmp(&b.2).unwrap_or(std::cmp::Ordering::Equal));
        targets.truncate(POND_SHOCK_MAX_TARGETS);

        shock.arc_cooldown = POND_SHOCK_ARC_COOLDOWN;

        let source_pos = Vec3::new(pond.center.x, POND_SURFACE_Y + 2.0, pond.center.z);

        for (target_entity, target_pos, _) in &targets {
            if let Ok((mut health, mut temp_hp, is_wet)) = health_query.get_mut(*target_entity) {
                let damage = if is_wet {
                    POND_SHOCK_ARC_DAMAGE * WET_ELECTRIC_DAMAGE_MULTIPLIER
                } else {
                    POND_SHOCK_ARC_DAMAGE
                };
                apply_damage_to_unit(&mut health, temp_hp.as_deref_mut(), damage);
            }

            crate::game::units::wizard::spells::chain_lightning::systems::spawn_arc(
                &mut commands,
                &visual_assets,
                source_pos,
                *target_pos,
                1,
                1.0,
            );
        }
    }
}
