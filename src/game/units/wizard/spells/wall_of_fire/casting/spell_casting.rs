//! Wall of Fire casting system — state machine, preview management, and wall placement.

use super::super::super::super::components::{
    CastingState, LocalWizard, Mana, PrimedSpell, Spell, Wizard,
};
use super::super::components::{WallOfFireCaster, WallOfFireEffect, WallOfFirePreview};
use super::super::constants;
use super::super::constants::*;
use super::placement::{
    compute_talent_params, spawn_wall_vfx, wall_obstacle_bounds, wall_transform,
};
use super::state_machine::wall_of_fire_casting_logic;
use crate::config::GameConfig;
use crate::game::components::OnGameplayScreen;
use crate::game::crt_effect::CorrectedCursorPosition;
use crate::game::game_mode::components::ActiveToggles;
use crate::game::input::MouseButtonState;
use crate::game::input::messages::MouseLeftReleased;
use crate::game::multiplayer::components::NetworkedSpellEffect;
use crate::game::pathfinding::{OBSTACLE_BUFFER, ObstacleChanged, ObstacleShape, ObstacleType};
use crate::game::units::DamageType;
use crate::game::units::wizard::spells::audio::SpellSfxAssets;
use crate::game::units::wizard::spells::utils::LocalSpellOrigin;
use crate::game::units::wizard::spells::utils::{
    TargetAssistWorldPos, UniqueHitTracker, apply_target_assist, build_wizard_input,
    clamp_to_spell_range,
};
use crate::game::units::wizard::spells::vfx;
use crate::game::units::wizard::spells::visual_assets::SpellVisualAssets;
use crate::game::units::wizard::talents::resources::ActiveTalents;
use crate::networking::snapshot::SpellEffectKind;
use bevy::prelude::*;

/// Local wizard Wall of Fire casting — reads mouse input, manages preview.
#[allow(clippy::too_many_arguments)]
pub fn handle_wall_of_fire_casting(
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
    cursor_resources: (
        Res<CorrectedCursorPosition>,
        Res<TargetAssistWorldPos>,
        Res<LocalSpellOrigin>,
    ),
    mut caster_query: Query<&mut WallOfFireCaster>,
    mut preview_query: Query<&mut Transform, (With<WallOfFirePreview>, Without<Wizard>)>,
    mut obstacle_events: MessageWriter<ObstacleChanged>,
    active_talents: Option<Res<ActiveTalents>>,
    active_toggles: Option<Res<ActiveToggles>>,
    mut audio_ctx: (
        Res<SpellSfxAssets>,
        Res<GameConfig>,
        ResMut<crate::game::multiplayer::spell_sync::PendingCastEvents>,
    ),
) {
    let (ref sfx, ref game_config, ref mut pending_cast_events) = audio_ctx;
    let scorched_mult =
        crate::game::game_mode::components::scorched_earth_mult(active_toggles.as_deref());
    let (corrected_cursor, target_assist, local_origin) = cursor_resources;
    let mut input = build_wizard_input(&mut mouse_left_released, &camera_query, &corrected_cursor);
    apply_target_assist(&mut input, &target_assist);

    let Ok((wizard_entity, wizard, mut casting_state, mut mana, primed_spell)) =
        wizard_query.single_mut()
    else {
        return;
    };
    if primed_spell.spell != Spell::WallOfFire {
        return;
    }

    let mut caster = if let Ok(c) = caster_query.get_mut(wizard_entity) {
        c
    } else {
        commands
            .entity(wizard_entity)
            .insert(WallOfFireCaster::new());
        return;
    };

    let talent_params = compute_talent_params(active_talents.as_deref());

    let clamped_pos = input
        .cursor_pos
        .map(|pos| clamp_to_spell_range(pos, local_origin.0, wizard.spell_range));

    let cast_result = wall_of_fire_casting_logic(
        &input,
        clamped_pos,
        &mut casting_state,
        &mut mana,
        primed_spell,
        &mut caster,
        &mut obstacle_events,
        &talent_params,
        scorched_mult,
    );

    // Handle preview spawning on cast start (anchor set, no preview yet)
    if caster.anchor.is_some()
        && caster.preview_entity.is_none()
        && let Some(pos) = clamped_pos
    {
        let preview_entity = commands
            .spawn((
                Mesh3d(visual_assets.unit_cuboid.clone()),
                MeshMaterial3d(materials.add(StandardMaterial {
                    base_color: PREVIEW_COLOR,
                    alpha_mode: AlphaMode::Blend,
                    unlit: true,
                    cull_mode: None,
                    ..default()
                })),
                Transform::from_xyz(pos.x, constants::WALL_RENDER_HEIGHT / 2.0 + 1.0, pos.z)
                    .with_scale(Vec3::new(
                        0.0,
                        constants::WALL_RENDER_HEIGHT,
                        WALL_WIDTH * talent_params.width_mult,
                    )),
                WallOfFirePreview,
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
        let max_len = MAX_WALL_LENGTH * talent_params.max_length_mult;
        let length = diff.length().min(max_len);

        if length > 0.1 {
            let forward = diff.normalize();
            let center = anchor + forward * (length / 2.0);
            let rotation = Quat::from_rotation_arc(Vec3::X, forward);
            let preview_width = WALL_WIDTH * talent_params.width_mult;

            preview_transform.translation = Vec3::new(
                center.x,
                constants::WALL_RENDER_HEIGHT / 2.0 + 1.0,
                center.z,
            );
            preview_transform.rotation = rotation;
            preview_transform.scale =
                Vec3::new(length, constants::WALL_RENDER_HEIGHT, preview_width);
        }
    }

    // On successful placement, convert preview entity to active fire wall
    if let Some(ref info) = cast_result.wall_placed {
        // Build the list of walls to spawn (1 normally, 2 for Twin Walls)
        let (walls, wall_count) = if info.talent_params.twin_walls {
            let wall_dir = (info.wall_end - info.wall_start).normalize_or_zero();
            let perp = Vec3::new(-wall_dir.z, 0.0, wall_dir.x);
            let offset = perp * info.half_width;
            let twin_damage = info.damage * constants::TWIN_WALLS_DAMAGE_MULT;
            (
                [
                    (
                        info.wall_start + offset,
                        info.wall_end + offset,
                        info.half_width,
                        twin_damage,
                    ),
                    (
                        info.wall_start - offset,
                        info.wall_end - offset,
                        info.half_width,
                        twin_damage,
                    ),
                ],
                2,
            )
        } else {
            (
                [
                    (info.wall_start, info.wall_end, info.half_width, info.damage),
                    (Vec3::ZERO, Vec3::ZERO, 0.0, 0.0), // unused
                ],
                1,
            )
        };

        for (i, &(start, end, hw, dmg)) in walls[..wall_count].iter().enumerate() {
            let wall_mat = materials.add(StandardMaterial {
                base_color: Color::NONE,
                alpha_mode: AlphaMode::Blend,
                unlit: true,
                ..default()
            });

            let effect = WallOfFireEffect::new(
                start,
                end,
                hw,
                dmg,
                DamageType::Fire,
                TICK_INTERVAL,
                info.fire_duration,
                info.talent_params.clone(),
            );
            let transform = wall_transform(start, end, hw);
            let net = NetworkedSpellEffect {
                kind: SpellEffectKind::WallOfFire,
            };

            let wall_entity = if i == 0 {
                if let Some(preview_entity) = caster.preview_entity {
                    commands
                        .entity(preview_entity)
                        .remove::<WallOfFirePreview>()
                        .insert((
                            MeshMaterial3d(wall_mat),
                            transform,
                            effect,
                            UniqueHitTracker::default(),
                            net,
                        ));
                    preview_entity
                } else {
                    continue;
                }
            } else {
                commands
                    .spawn((
                        Mesh3d(visual_assets.unit_cuboid.clone()),
                        MeshMaterial3d(wall_mat),
                        transform,
                        effect,
                        UniqueHitTracker::default(),
                        net,
                        OnGameplayScreen,
                    ))
                    .id()
            };

            // Twin Walls repositions the first wall, so re-notify pathfinding for all walls
            if info.talent_params.twin_walls || i > 0 {
                obstacle_events.write(ObstacleChanged {
                    bounds: wall_obstacle_bounds(start, end, hw),
                    obstacle_type: ObstacleType::Hazard(4.5),
                    shape: Some(ObstacleShape::obb_from_wall(
                        start,
                        end,
                        hw + OBSTACLE_BUFFER,
                    )),
                    rebuild: true,
                });
            }

            spawn_wall_vfx(
                &mut commands,
                &visual_assets,
                sfx,
                game_config,
                start,
                end,
                wall_entity,
            );
        }

        caster.preview_entity = None;
    }

    // Despawn preview on failure (too short / can't afford)
    if cast_result.despawn_preview {
        if let Some(preview_entity) = caster.preview_entity {
            commands.entity(preview_entity).try_despawn();
        }
        caster.preview_entity = None;
    }

    if cast_result.completed {
        vfx::systems::spawn_school_flare_synced(
            &mut commands,
            &visual_assets,
            pending_cast_events,
            local_origin.0,
            vfx::systems::SpellSchool::Fire,
            time.elapsed_secs(),
        );
        mouse_state.left_consumed = true;
    }
}
