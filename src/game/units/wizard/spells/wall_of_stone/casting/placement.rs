use super::super::super::super::components::{CastingState, Mana, PrimedSpell, WizardInput};
use super::super::components::{
    LivingStoneTracker, WallHealth, WallOfStone, WallOfStoneCaster, WallOfStoneTalentParams,
    WallRising, WallTalents,
};
use super::super::constants::*;
use crate::game::components::OnGameplayScreen;
use crate::game::multiplayer::components::NetworkedSpellEffect;
use crate::game::pathfinding::{ObstacleChanged, ObstacleShape, ObstacleType};
use crate::game::units::wizard::spells::visual_assets::SpellVisualAssets;
use crate::networking::snapshot::SpellEffectKind;
use bevy::prelude::*;

/// Result from spell casting logic, used to communicate state back to the wrapper.
pub(crate) struct CastResult {
    /// Whether the spell completed (wall was placed).
    pub(crate) completed: bool,
    /// Whether preview should be despawned (release with too-short drag or no mana).
    pub(crate) despawn_preview: bool,
    /// Obstacle bounds for network sync (set when completed=true).
    pub(crate) obstacle_bounds: Option<[f32; 4]>,
    /// Center position of the placed wall (for sound effects).
    pub(crate) wall_center: Option<Vec3>,
}

/// Core Wall of Stone casting logic.
#[allow(clippy::too_many_arguments)]
pub(crate) fn wall_of_stone_casting_logic(
    input: &WizardInput,
    clamped_pos: Option<Vec3>,
    casting_state: &mut CastingState,
    mana: &mut Mana,
    primed_spell: &PrimedSpell,
    caster: &mut WallOfStoneCaster,
    commands: &mut Commands,
    assets: &SpellVisualAssets,
    obstacle_events: &mut MessageWriter<ObstacleChanged>,
    talent_params: &WallOfStoneTalentParams,
) -> CastResult {
    let mut result = CastResult {
        completed: false,
        despawn_preview: false,
        obstacle_bounds: None,
        wall_center: None,
    };

    let Some(clamped_pos) = clamped_pos else {
        return result;
    };

    let mana_cost = MANA_COST * talent_params.mana_mult;
    let max_length = MAX_WALL_LENGTH * talent_params.max_length_mult;
    let wall_count = if talent_params.quick_foundations {
        2u32
    } else {
        1
    };
    let total_mana_cost = mana_cost * wall_count as f32;

    // Handle release — place wall or cancel
    if input.just_released {
        if let Some(anchor) = caster.anchor {
            let diff = Vec3::new(clamped_pos.x - anchor.x, 0.0, clamped_pos.z - anchor.z);
            let length = diff.length();

            if length >= MIN_WALL_LENGTH && mana.can_afford(total_mana_cost) {
                let clamped_length = length.min(max_length);
                let forward = diff.normalize();
                let right = Vec3::new(-forward.z, 0.0, forward.x);

                mana.consume(total_mana_cost);

                // Apply empowerment scaling
                let scale = primed_spell.empowerment;
                let wall_width = WALL_WIDTH * talent_params.width_mult * scale;
                let wall_height = WALL_HEIGHT * scale;
                let wall_health = WALL_HEALTH * talent_params.health_mult;

                // Walls now last the entire level (swept when the level ends).
                // Terraformer additionally makes them permanent — saved and rebuilt
                // on the next level. Both are still dispellable.
                let permanent = talent_params.terraformer;
                let duration = f32::MAX;

                // Quick Foundations: split into two walls end-to-end
                let segment_length = clamped_length / wall_count as f32;

                for i in 0..wall_count {
                    let segment_start = anchor + forward * (segment_length * i as f32);
                    let center = segment_start + forward * (segment_length / 2.0);
                    let rotation = Quat::from_rotation_arc(Vec3::X, forward);

                    let wall = WallOfStone {
                        center,
                        half_length: segment_length / 2.0,
                        half_width: wall_width / 2.0,
                        forward,
                        right,
                        height: wall_height,
                        time_alive: 0.0,
                        duration,
                        sinking: false,
                        empowerment: primed_spell.empowerment,
                        permanent,
                    };

                    let obs_bounds = wall.obstacle_bounds();

                    // Start the transform underground so `animate_rising_walls`
                    // can drive the y up from below the floor on its first
                    // tick (eased=0 → y=-wall.height). Spawning at the final
                    // y=wall_height/2 produced a one-frame full-height flash
                    // before the animator yanked it down.
                    let mut entity_commands = commands.spawn((
                        Mesh3d(assets.unit_cuboid.clone()),
                        MeshMaterial3d(assets.wall_of_stone.clone()),
                        Transform::from_xyz(center.x, -wall_height / 2.0, center.z)
                            .with_rotation(rotation)
                            .with_scale(Vec3::new(segment_length, wall_height, wall_width)),
                        wall,
                        WallHealth::new(wall_health),
                        WallTalents(talent_params.clone()),
                        NetworkedSpellEffect {
                            kind: SpellEffectKind::WallOfStone,
                        },
                        OnGameplayScreen,
                    ));

                    // Wall rises from the ground
                    entity_commands.insert(WallRising::new(WALL_RISE_DURATION));

                    // Add Living Stone tracker if talent is active
                    if talent_params.living_stone {
                        entity_commands.insert(LivingStoneTracker::new());
                    }

                    obstacle_events.write(ObstacleChanged {
                        bounds: Rect::new(
                            obs_bounds[0],
                            obs_bounds[1],
                            obs_bounds[2],
                            obs_bounds[3],
                        ),
                        obstacle_type: ObstacleType::Blocked,
                        shape: Some(ObstacleShape::obb_from_center(
                            center,
                            forward,
                            segment_length / 2.0,
                            wall_width / 2.0,
                        )),
                        rebuild: false,
                    });

                    // Use the last wall's bounds for network sync
                    result.obstacle_bounds = Some(obs_bounds);
                    result.wall_center = Some(center);
                }

                result.completed = true;
            } else {
                // Too short or can't afford — signal preview despawn
                result.despawn_preview = true;
            }

            caster.anchor = None;
            casting_state.cancel();
        }
        return result;
    }

    match *casting_state {
        CastingState::Resting => {
            if (input.just_pressed || input.pressed) && mana.can_afford(total_mana_cost) {
                caster.anchor = Some(clamped_pos);
                casting_state.start_cast();
            }
        }
        CastingState::Casting { .. } => {
            // Preview update is handled by the local wrapper only
        }
        _ => {}
    }

    result
}
