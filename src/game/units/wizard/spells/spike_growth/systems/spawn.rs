use super::super::components::{SpikeGrowthTalentParams, SpikeGrowthZone};
use super::super::constants;
use crate::game::components::OnGameplayScreen;
use crate::game::multiplayer::components::NetworkedSpellEffect;
use crate::game::pathfinding::{OBSTACLE_BUFFER, ObstacleChanged, ObstacleShape, ObstacleType};
use crate::game::units::wizard::spells::utils::UniqueHitTracker;
use crate::game::units::wizard::spells::visual_assets::SpellVisualAssets;
use crate::networking::snapshot::SpellEffectKind;
use bevy::prelude::*;

/// Spawns a single spike growth zone with talent parameters.
#[allow(clippy::too_many_arguments)]
pub(crate) fn spawn_spike_growth_zone(
    commands: &mut Commands,
    assets: &SpellVisualAssets,
    position: Vec3,
    radius: f32,
    empowerment: f32,
    obstacle_events: &mut MessageWriter<ObstacleChanged>,
    talent_params: &SpikeGrowthTalentParams,
    scorched_mult: f32,
) {
    let duration = constants::ZONE_DURATION * empowerment * scorched_mult;
    let damage = constants::DAMAGE_PER_TICK * empowerment * talent_params.damage_mult;
    let slow_mod = constants::SLOW_MODIFIER * empowerment;
    let slow_dur = constants::SLOW_DURATION * empowerment;

    // Notify pathfinding about hazard zone
    let origin_2d = Vec2::new(position.x, position.z);
    let buffered_radius = radius + OBSTACLE_BUFFER;
    let hazard_cost = if talent_params.thorn_maze {
        15.0 * constants::THORN_MAZE_HAZARD_MULT
    } else {
        15.0
    };
    obstacle_events.write(ObstacleChanged {
        bounds: Rect::from_center_size(origin_2d, Vec2::splat(buffered_radius * 2.0)),
        obstacle_type: ObstacleType::Hazard(hazard_cost),
        shape: Some(ObstacleShape::circle(origin_2d, buffered_radius)),
        rebuild: false,
    });

    // Use pre-loaded spike storm material from visual assets
    let spike_storm_material = if talent_params.spike_storm {
        Some(assets.spike_storm_projectile.clone())
    } else {
        None
    };

    let mut zone = SpikeGrowthZone::new(
        Vec3::new(position.x, 0.0, position.z),
        radius,
        damage,
        constants::TICK_INTERVAL,
        slow_mod,
        slow_dur,
        duration,
        *talent_params,
    );
    zone.spike_storm_material = spike_storm_material;

    commands.spawn((
        Transform::from_translation(Vec3::new(
            position.x,
            constants::CIRCLE_Y_POSITION,
            position.z,
        )),
        zone,
        UniqueHitTracker::default(),
        NetworkedSpellEffect {
            kind: SpellEffectKind::SpikeGrowthZone,
        },
        OnGameplayScreen,
    ));
}

/// Nature's Minefield: spawns 3 smaller zones in a triangle pattern.
#[allow(clippy::too_many_arguments)]
pub(crate) fn spawn_minefield_zones(
    commands: &mut Commands,
    assets: &SpellVisualAssets,
    position: Vec3,
    base_radius: f32,
    empowerment: f32,
    obstacle_events: &mut MessageWriter<ObstacleChanged>,
    talent_params: &SpikeGrowthTalentParams,
    scorched_mult: f32,
) {
    let sub_radius = base_radius * constants::MINEFIELD_RADIUS_MULT;
    let spread = base_radius * constants::MINEFIELD_SPREAD_FRACTION;

    // Triangle offsets: 120° apart
    let offsets = [
        Vec3::new(0.0, 0.0, -spread),
        Vec3::new(spread * 0.866, 0.0, spread * 0.5),
        Vec3::new(-spread * 0.866, 0.0, spread * 0.5),
    ];

    for offset in &offsets {
        let sub_position = position + *offset;
        spawn_spike_growth_zone(
            commands,
            assets,
            sub_position,
            sub_radius,
            empowerment,
            obstacle_events,
            talent_params,
            scorched_mult,
        );
    }
}
