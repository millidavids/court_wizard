use super::super::super::super::components::{
    CastingState, LocalWizard, Mana, PrimedSpell, Spell, Wizard,
};
use super::super::components::{WallOfStoneCaster, WallOfStonePreview};
use super::super::constants::*;
use super::placement::wall_of_stone_casting_logic;
use super::talents::compute_talent_params;
use crate::config::GameConfig;
use crate::game::components::OnGameplayScreen;
use crate::game::crt_effect::CorrectedCursorPosition;
use crate::game::input::MouseButtonState;
use crate::game::input::messages::MouseLeftReleased;
use crate::game::pathfinding::ObstacleChanged;
use crate::game::units::wizard::spells::audio::{self, SpellSfxAssets};
use crate::game::units::wizard::spells::messages::announce_area_cast;
use crate::game::units::wizard::spells::utils::LocalSpellOrigin;
use crate::game::units::wizard::spells::utils::{
    TargetAssistWorldPos, apply_target_assist, build_wizard_input, clamp_to_spell_range,
};
use crate::game::units::wizard::spells::vfx;
use crate::game::units::wizard::spells::visual_assets::SpellVisualAssets;
use crate::game::units::wizard::talents::resources::{ActiveTalents, BattleTalentProgress};
use bevy::prelude::*;

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
    (sfx, game_config, active_talents, mut talent_progress, mut pending_cast_events): (
        Res<SpellSfxAssets>,
        Res<GameConfig>,
        Option<Res<ActiveTalents>>,
        Option<ResMut<BattleTalentProgress>>,
        ResMut<crate::game::multiplayer::spell_sync::PendingCastEvents>,
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

    // A wall raised across a crystal anchors and charges it. One message per
    // segment, sized from that segment's own bounds, so Quick Foundations'
    // second wall is not missed and a long wall does not reach crystals it
    // never touched.
    if cast_result.completed {
        for bounds in &cast_result.obstacle_bounds {
            let [min_x, min_z, max_x, max_z] = *bounds;
            let (span_x, span_z) = (max_x - min_x, max_z - min_z);
            // Walk the segment's long axis in steps sized to its short axis,
            // announcing a circle at each. A single circle round the whole AABB
            // would have the wall's *length* as its radius, anchoring crystals
            // half a wall away that the stone never came near.
            let reach = span_x.min(span_z).max(1.0) * 0.5;
            let span = span_x.max(span_z);
            let steps = ((span / reach.max(1.0)).ceil() as usize).clamp(1, 16);
            let along_x = span_x >= span_z;
            for step in 0..=steps {
                let t = step as f32 / steps as f32;
                let point = if along_x {
                    Vec3::new(min_x + span_x * t, 0.0, (min_z + max_z) * 0.5)
                } else {
                    Vec3::new((min_x + max_x) * 0.5, 0.0, min_z + span_z * t)
                };
                announce_area_cast(
                    &mut commands,
                    Spell::WallOfStone,
                    point,
                    reach,
                    primed_spell.empowerment,
                );
            }
        }
    }

    // Send each placed wall segment over the network so the other client updates
    // pathfinding for ALL of them (Quick Foundations places two).
    if cast_result.completed
        && let Some(ref mut conn) = connection
    {
        for bounds in cast_result.obstacle_bounds {
            conn.outgoing_messages
                .push(crate::networking::protocol::NetworkMessage::WallPlaced {
                    bounds,
                    placed: true,
                });
        }
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
        vfx::systems::spawn_school_flare_synced(
            &mut commands,
            &visual_assets,
            &mut pending_cast_events,
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
