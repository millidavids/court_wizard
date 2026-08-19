use super::super::casting::write_grease_obstacle;
use super::super::components::{GreaseTalentParams, GreaseZone};
use super::super::constants;
use crate::game::components::OnGameplayScreen;
use crate::game::multiplayer::components::NetworkedSpellEffect;
use crate::game::pathfinding::{ObstacleChanged, ObstacleType};
use crate::game::units::wizard::spells::visual_assets::SpellVisualAssets;
use crate::networking::snapshot::SpellEffectKind;
use bevy::prelude::*;

#[allow(clippy::too_many_arguments)]
pub(crate) fn spawn_grease_zone(
    commands: &mut Commands,
    assets: &SpellVisualAssets,
    materials: &mut ResMut<Assets<StandardMaterial>>,
    position: Vec3,
    radius: f32,
    empowerment: f32,
    obstacle_events: &mut MessageWriter<ObstacleChanged>,
    talent_params: GreaseTalentParams,
    scorched_mult: f32,
) -> Entity {
    let duration = constants::ZONE_DURATION * empowerment * scorched_mult;
    let slow_mod = constants::SLOW_MODIFIER * talent_params.slow_mult;
    let slow_dur = constants::SLOW_DURATION * empowerment;

    // Notify pathfinding about slow terrain
    write_grease_obstacle(
        Vec3::new(position.x, 0.0, position.z),
        radius,
        ObstacleType::SlowTerrain(3.0),
        obstacle_events,
    );

    let mut base_mat = materials
        .get(&assets.grease_zone)
        .cloned()
        .unwrap_or_default();
    // Use Mask so the grease renders in the opaque phase (before transparent unit sprites).
    // This writes to the depth buffer at Y=2, ensuring all units above it render on top.
    base_mat.alpha_mode = bevy::material::AlphaMode::Mask(0.01);
    let instance_material = materials.add(base_mat);

    commands
        .spawn((
            Mesh3d(assets.unit_circle.clone()),
            MeshMaterial3d(instance_material),
            Transform::from_translation(Vec3::new(
                position.x,
                constants::CIRCLE_Y_POSITION,
                position.z,
            ))
            .with_rotation(Quat::from_rotation_x(-std::f32::consts::FRAC_PI_2))
            .with_scale(Vec3::splat(0.01)),
            GreaseZone::new(
                Vec3::new(position.x, 0.0, position.z),
                radius,
                slow_mod,
                slow_dur,
                constants::TICK_INTERVAL,
                duration,
                constants::IGNITE_DAMAGE,
                constants::IGNITE_BURN_DAMAGE,
                constants::IGNITE_BURN_TICK,
                empowerment,
                talent_params,
            ),
            NetworkedSpellEffect {
                kind: SpellEffectKind::GreaseZone,
            },
            OnGameplayScreen,
        ))
        .id()
}
