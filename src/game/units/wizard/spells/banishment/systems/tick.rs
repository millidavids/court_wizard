use rand::Rng;

use super::super::components::{DimensionalShunt, Displacement, OneWayTrip, PainfulReturn};
use crate::game::constants::BATTLEFIELD_SIZE;
use crate::game::units::components::{
    BanishedModifier, Health, TemporaryHitPoints, WasBanished, apply_spell_damage,
};
use crate::game::units::damage::DamageType;
use bevy::prelude::*;

/// Ticks banished unit timers and restores them when expired.
/// Handles talent effects on return: Painful Return, Displacement, Dimensional Shunt, One-Way Trip.
#[allow(clippy::type_complexity)]
pub fn tick_banished_units(
    mut commands: Commands,
    time: Res<Time>,
    mut game_rng: ResMut<crate::game::seeded_rng::resources::GameRng>,
    mut banished: Query<(
        Entity,
        &mut BanishedModifier,
        &mut Transform,
        &mut Health,
        Option<&mut TemporaryHitPoints>,
        Option<&PainfulReturn>,
        Option<&Displacement>,
        Option<&DimensionalShunt>,
        Option<&OneWayTrip>,
    )>,
) {
    let delta = time.delta_secs();
    for (
        entity,
        mut modifier,
        mut transform,
        mut health,
        mut temp_hp,
        painful_return,
        displacement,
        dimensional_shunt,
        one_way_trip,
    ) in &mut banished
    {
        if !modifier.update(delta) {
            continue;
        }

        // One-Way Trip: unit doesn't return, just dies (stays hidden until corpse conversion)
        if one_way_trip.is_some() {
            health.current = 0.0;
            commands
                .entity(entity)
                .remove::<BanishedModifier>()
                .remove::<OneWayTrip>()
                .insert(WasBanished);
            continue;
        }

        // Dimensional Shunt: set HP to fraction of max
        if let Some(shunt) = dimensional_shunt {
            let target_hp = health.max * shunt.hp_fraction;
            if health.current > target_hp {
                health.current = target_hp;
            }
        }

        // Painful Return: deal damage on return
        if let Some(painful) = painful_return {
            apply_spell_damage(
                &mut commands,
                entity,
                &mut health,
                temp_hp.as_deref_mut(),
                painful.damage,
                DamageType::Force,
                false,
            );
        }

        // Displacement: randomize return position, clamped to battlefield
        if let Some(displace) = displacement {
            let half = BATTLEFIELD_SIZE / 2.0;
            let angle = game_rng.0.random::<f32>() * std::f32::consts::TAU;
            let dist = displace.radius * 0.5 + game_rng.0.random::<f32>() * displace.radius * 0.5;
            transform.translation.x =
                (transform.translation.x + angle.cos() * dist).clamp(-half, half);
            transform.translation.z =
                (transform.translation.z + angle.sin() * dist).clamp(-half, half);
        }

        // Clean up talent components and restore visibility
        commands
            .entity(entity)
            .remove::<BanishedModifier>()
            .remove::<PainfulReturn>()
            .remove::<Displacement>()
            .remove::<DimensionalShunt>()
            .remove::<OneWayTrip>()
            .insert(Visibility::Visible)
            .insert(WasBanished);
    }
}
