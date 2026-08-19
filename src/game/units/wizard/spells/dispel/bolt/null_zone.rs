use super::super::components::NullZone;
use super::super::constants;
use super::suppress::{
    collect_dispellable_effects, remove_mind_control_in_radius, suppress_spell_effects_in_radius,
};
use crate::game::multiplayer::components::NetworkedSpellEffect;
use crate::game::pathfinding::ObstacleChanged;
use crate::game::units::components::MindControlled;
use crate::game::units::wizard::spells::grease::components::{GreaseIgnited, GreaseZone};
use crate::game::units::wizard::spells::meteor_fall::components::MeteorGroundFire;
use crate::game::units::wizard::spells::spike_growth::components::SpikeGrowthZone;
use crate::game::units::wizard::spells::wall_of_fire::components::WallOfFireEffect;
use crate::game::units::wizard::spells::wall_of_stone::components::WallOfStone;
use bevy::prelude::*;

/// Ticks Null Zone timers. Despawns expired zones.
/// Active null zones suppress (despawn) spell effects that enter them.
#[allow(clippy::too_many_arguments)]
pub fn update_null_zones(
    mut commands: Commands,
    time: Res<Time>,
    mut zones: Query<(Entity, &mut NullZone, &MeshMaterial3d<StandardMaterial>)>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    spell_effects: Query<(Entity, &Transform, &NetworkedSpellEffect), Without<NullZone>>,
    wall_of_fire_query: Query<&WallOfFireEffect>,
    wall_of_stone_query: Query<&WallOfStone>,
    spike_growth_query: Query<&SpikeGrowthZone>,
    grease_query: Query<(&GreaseZone, Has<GreaseIgnited>)>,
    meteor_fire_query: Query<&MeteorGroundFire>,
    mut obstacle_events: MessageWriter<ObstacleChanged>,
    mind_controlled_query: Query<(Entity, &Transform), (With<MindControlled>, Without<NullZone>)>,
) {
    let delta = time.delta_secs();
    for (zone_entity, mut zone, material_handle) in &mut zones {
        zone.time_remaining -= delta;
        if zone.time_remaining <= 0.0 {
            commands.entity(zone_entity).try_despawn();
            continue;
        }

        // Fade alpha as zone expires
        let life_frac = zone.time_remaining / constants::NULL_ZONE_DURATION;
        let alpha = constants::NULL_ZONE_COLOR.alpha() * life_frac;
        if let Some(mut material) = materials.get_mut(material_handle) {
            material.base_color = constants::NULL_ZONE_COLOR.with_alpha(alpha);
        }

        // Collect dispellable spell effects once for this frame
        let all_dispellable: Vec<_> = collect_dispellable_effects(
            spell_effects
                .iter()
                .map(|(e, tf, nse)| (e, tf.translation, nse.kind)),
        );

        // Suppress spell effects inside the zone
        suppress_spell_effects_in_radius(
            &mut commands,
            zone.origin,
            zone.radius,
            &all_dispellable,
            &wall_of_fire_query,
            &wall_of_stone_query,
            &spike_growth_query,
            &grease_query,
            &meteor_fire_query,
            &mut obstacle_events,
        );

        // Remove mind control from units in zone
        remove_mind_control_in_radius(
            &mut commands,
            zone.origin,
            zone.radius,
            mind_controlled_query
                .iter()
                .map(|(e, tf)| (e, tf.translation)),
        );
    }
}
