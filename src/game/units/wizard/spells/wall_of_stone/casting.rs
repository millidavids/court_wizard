//! Wall of stone casting and spawn.

use super::super::super::components::{
    CastingState, LocalWizard, Mana, PrimedSpell, Spell, Wizard, WizardInput,
};
use super::components::{
    LivingStoneTracker, WallHealth, WallOfStone, WallOfStoneCaster, WallOfStonePreview,
    WallOfStoneTalentParams, WallRising, WallTalents,
};
use super::constants::*;
use crate::config::GameConfig;
use crate::game::components::OnGameplayScreen;
use crate::game::units::wizard::spells::utils::LocalSpellOrigin;
use crate::game::crt_effect::CorrectedCursorPosition;
use crate::game::input::MouseButtonState;
use crate::game::input::messages::MouseLeftReleased;
use crate::game::multiplayer::components::NetworkedSpellEffect;
use crate::game::pathfinding::{ObstacleChanged, ObstacleShape, ObstacleType};
use crate::game::units::wizard::spells::audio::{self, SpellSfxAssets};
use crate::game::units::wizard::spells::utils::{
    TargetAssistWorldPos, apply_target_assist, build_wizard_input, clamp_to_spell_range,
};
use crate::game::units::wizard::spells::vfx;
use crate::game::units::wizard::spells::visual_assets::SpellVisualAssets;
use crate::game::units::wizard::talents::resources::{ActiveTalents, BattleTalentProgress};
use crate::networking::snapshot::SpellEffectKind;
use bevy::prelude::*;

/// Computes talent parameters from active talent selections.
pub(crate) fn compute_talent_params(
    active_talents: Option<&ActiveTalents>,
) -> WallOfStoneTalentParams {
    let mut params = WallOfStoneTalentParams::default();

    let Some(talents) = active_talents else {
        return params;
    };

    let t1 = talents.get_selection(Spell::WallOfStone, 0);
    let t2 = talents.get_selection(Spell::WallOfStone, 1);
    let t3 = talents.get_selection(Spell::WallOfStone, 2);

    // Tier 1: Numeric modifiers
    match t1 {
        Some(0) => {
            // Quarry Master
            params.mana_mult = QUARRY_MASTER_MANA_MULT;
            params.max_length_mult = QUARRY_MASTER_LENGTH_MULT;
        }
        Some(1) => {
            // Reinforced Stone
            params.health_mult = REINFORCED_STONE_HEALTH_MULT;
            params.width_mult = REINFORCED_STONE_WIDTH_MULT;
        }
        Some(2) => {
            // Quick Foundations
            params.quick_foundations = true;
            params.mana_mult = QUICK_FOUNDATIONS_MANA_MULT;
        }
        _ => {}
    }

    // Tier 2: Behavioral flags
    match t2 {
        Some(0) => params.jagged_stone = true,
        Some(1) => params.permafrost_aura = true,
        Some(2) => params.living_stone = true,
        _ => {}
    }

    // Tier 3: Transformative flags
    match t3 {
        Some(0) => params.collapsing_wall = true,
        Some(1) => {
            params.terraformer = true;
        }
        Some(2) => {
            params.maze_architect = true;
            params.mana_mult *= MAZE_ARCHITECT_MANA_MULT;
        }
        _ => {}
    }

    params
}

/// Result from spell casting logic, used to communicate state back to the wrapper.
struct CastResult {
    /// Whether the spell completed (wall was placed).
    completed: bool,
    /// Whether preview should be despawned (release with too-short drag or no mana).
    despawn_preview: bool,
    /// Obstacle bounds for network sync (set when completed=true).
    obstacle_bounds: Option<[f32; 4]>,
    /// Center position of the placed wall (for sound effects).
    wall_center: Option<Vec3>,
}

/// Local wizard Wall of Stone casting — reads mouse input, manages preview.
#[allow(clippy::too_many_arguments)]
pub fn handle_wall_of_stone_casting(
    time: Res<Time>,
    mut mouse_left_released: MessageReader<MouseLeftReleased>,
    mut mouse_state: ResMut<MouseButtonState>,
    mut commands: Commands,
    mut materials: ResMut<Assets<StandardMaterial>>,
    visual_assets: Res<SpellVisualAssets>,
    mut wizard_query: Query<
        (Entity, &Wizard, &mut CastingState, &mut Mana, &PrimedSpell),
        With<LocalWizard>,
    >,
    camera_query: Query<(&Camera, &GlobalTransform), With<Camera3d>>,
    corrected_cursor: Res<CorrectedCursorPosition>,
    mut caster_query: Query<&mut WallOfStoneCaster>,
    mut preview_query: Query<&mut Transform, (With<WallOfStonePreview>, Without<Wizard>)>,
    mut obstacle_events: MessageWriter<ObstacleChanged>,
    mut connection: Option<ResMut<crate::networking::resources::NetworkConnection>>,
    target_assist: Res<TargetAssistWorldPos>,
    (sfx, game_config, active_talents, mut talent_progress): (
        Res<SpellSfxAssets>,
        Res<GameConfig>,
        Option<Res<ActiveTalents>>,
        Option<ResMut<BattleTalentProgress>>,
    ),
    local_origin: Res<LocalSpellOrigin>,
) {
    let mut input = build_wizard_input(&mut mouse_left_released, &camera_query, &corrected_cursor);
    apply_target_assist(&mut input, &target_assist);

    let Ok((wizard_entity, wizard, mut casting_state, mut mana, primed_spell)) =
        wizard_query.single_mut()
    else {
        return;
    };
    if primed_spell.spell != Spell::WallOfStone {
        return;
    }

    let mut caster = if let Ok(c) = caster_query.get_mut(wizard_entity) {
        c
    } else {
        commands
            .entity(wizard_entity)
            .insert(WallOfStoneCaster::new());
        return;
    };

    let clamped_pos = input
        .cursor_pos
        .map(|pos| clamp_to_spell_range(pos, local_origin.0, wizard.spell_range));

    let talent_params = compute_talent_params(active_talents.as_deref());

    let cast_result = wall_of_stone_casting_logic(
        &input,
        clamped_pos,
        &mut casting_state,
        &mut mana,
        primed_spell,
        &mut caster,
        &mut commands,
        &visual_assets,
        &mut obstacle_events,
        &talent_params,
    );

    // Send wall placement over network so the other client updates pathfinding
    if cast_result.completed
        && let Some(bounds) = cast_result.obstacle_bounds
        && let Some(ref mut conn) = connection
    {
        conn.outgoing_messages
            .push(crate::networking::protocol::NetworkMessage::WallPlaced {
                bounds,
                placed: true,
            });
    }

    // Local-only: manage preview

    // Handle preview spawning on cast start
    if caster.anchor.is_some()
        && caster.preview_entity.is_none()
        && let Some(pos) = clamped_pos
    {
        let preview_entity = commands
            .spawn((
                Mesh3d(visual_assets.unit_cuboid.clone()),
                MeshMaterial3d(materials.add(StandardMaterial {
                    base_color: WALL_PREVIEW_COLOR,
                    alpha_mode: AlphaMode::Blend,
                    unlit: true,
                    cull_mode: None,
                    ..default()
                })),
                Transform::from_xyz(pos.x, WALL_HEIGHT / 2.0, pos.z).with_scale(Vec3::new(
                    0.0,
                    WALL_HEIGHT,
                    WALL_WIDTH,
                )),
                WallOfStonePreview,
                OnGameplayScreen,
            ))
            .id();

        caster.preview_entity = Some(preview_entity);
    }

    // Update preview during casting
    if matches!(*casting_state, CastingState::Casting { .. })
        && let Some(anchor) = caster.anchor
        && let Some(preview_entity) = caster.preview_entity
        && let Ok(mut preview_transform) = preview_query.get_mut(preview_entity)
        && let Some(pos) = clamped_pos
    {
        let diff = Vec3::new(pos.x - anchor.x, 0.0, pos.z - anchor.z);
        let max_length = MAX_WALL_LENGTH * talent_params.max_length_mult;
        let length = diff.length().min(max_length);

        if length > 0.1 {
            let forward = diff.normalize();
            let center = anchor + forward * (length / 2.0);
            let rotation = Quat::from_rotation_arc(Vec3::X, forward);

            preview_transform.translation = Vec3::new(center.x, WALL_HEIGHT / 2.0, center.z);
            preview_transform.rotation = rotation;
            preview_transform.scale = Vec3::new(length, WALL_HEIGHT, WALL_WIDTH);
        }
    }

    // Despawn preview on completion or cancel
    if cast_result.completed || cast_result.despawn_preview {
        if let Some(preview_entity) = caster.preview_entity {
            commands.entity(preview_entity).try_despawn();
        }
        caster.preview_entity = None;
    }

    if cast_result.completed {
        vfx::systems::spawn_school_flare(
            &mut commands,
            &visual_assets,
            local_origin.0,
            vfx::systems::SpellSchool::Force,
            time.elapsed_secs(),
        );
        // Track talent progress (count walls placed, not casts)
        let walls_placed: u32 = if talent_params.quick_foundations {
            2
        } else {
            1
        };
        if let Some(ref mut progress) = talent_progress {
            progress.increment(Spell::WallOfStone, walls_placed);
        }

        if let Some(center) = cast_result.wall_center {
            audio::play_sfx(
                &mut commands,
                &sfx.wall_of_stone_cast,
                center,
                &game_config,
                &sfx,
            );
        }
        mouse_state.left_consumed = true;
    }
}

/// Core Wall of Stone casting logic.
#[allow(clippy::too_many_arguments)]
fn wall_of_stone_casting_logic(
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

                // Walls are temporary by default; Terraformer makes them permanent
                let (permanent, duration) = if talent_params.terraformer {
                    (true, f32::MAX)
                } else {
                    (false, DEFAULT_WALL_DURATION)
                };

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
