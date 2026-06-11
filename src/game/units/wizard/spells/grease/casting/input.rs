use super::super::super::super::components::{
    CastingState, LocalWizard, Mana, PrimedSpell, Spell, SpellCaster, Wizard, WizardInput,
};
use super::super::components::GreaseTalentParams;
use super::super::constants;
use super::super::ignite::spawn_grease_zone;
use super::talents::compute_talent_params;
use crate::config::GameConfig;
use crate::game::crt_effect::CorrectedCursorPosition;
use crate::game::game_mode::components::ActiveToggles;
use crate::game::input::MouseButtonState;
use crate::game::input::messages::MouseLeftReleased;
use crate::game::pathfinding::ObstacleChanged;
use crate::game::units::wizard::spells::audio::{self, SpellSfxAssets};
use crate::game::units::wizard::spells::utils::LocalSpellOrigin;
use crate::game::units::wizard::spells::utils::{
    SpellCircleIndicator, TargetAssistWorldPos, apply_target_assist, build_wizard_input,
    cleanup_spell_caster, handle_spell_release, try_start_cast_with_indicator,
    update_indicator_position,
};
use crate::game::units::wizard::spells::vfx;
use crate::game::units::wizard::spells::visual_assets::SpellVisualAssets;
use crate::game::units::wizard::talents::resources::{ActiveTalents, BattleTalentProgress};
use bevy::prelude::*;

/// Local wizard grease casting -- reads mouse input.
#[allow(clippy::too_many_arguments)]
pub fn handle_grease_casting(
    time: Res<Time>,
    mut mouse_state: ResMut<MouseButtonState>,
    mut mouse_left_released: MessageReader<MouseLeftReleased>,
    mut commands: Commands,
    visual_assets: Res<SpellVisualAssets>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
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
    caster_query: Query<&SpellCaster>,
    mut indicator_query: Query<&mut SpellCircleIndicator>,
    mut obstacle_events: MessageWriter<ObstacleChanged>,
    sfx: Res<SpellSfxAssets>,
    game_config: Res<GameConfig>,
    talent_resources: (
        Option<Res<ActiveTalents>>,
        Option<ResMut<BattleTalentProgress>>,
        Option<Res<ActiveToggles>>,
        ResMut<crate::game::multiplayer::spell_sync::PendingCastEvents>,
    ),
) {
    let (active_talents, _talent_progress, active_toggles, mut pending_cast_events) =
        talent_resources;
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
    if primed_spell.spell != Spell::Grease {
        return;
    }

    let talent_params = compute_talent_params(active_talents.as_deref());

    let completed = grease_casting_logic(
        &input,
        &time,
        wizard_entity,
        wizard,
        &mut casting_state,
        &mut mana,
        primed_spell,
        &caster_query,
        &mut indicator_query,
        &mut commands,
        &visual_assets,
        &mut meshes,
        &mut materials,
        &mut obstacle_events,
        &sfx,
        &game_config,
        &talent_params,
        scorched_mult,
        local_origin.0,
    );

    if completed {
        vfx::systems::spawn_school_flare_synced(
            &mut commands,
            &visual_assets,
            &mut pending_cast_events,
            local_origin.0,
            vfx::systems::SpellSchool::Transmutation,
            time.elapsed_secs(),
        );
        mouse_state.left_consumed = true;
    }
}

/// Core grease casting logic. Returns true if the spell completed.
#[allow(clippy::too_many_arguments)]
fn grease_casting_logic(
    input: &WizardInput,
    time: &Time,
    wizard_entity: Entity,
    wizard: &Wizard,
    casting_state: &mut CastingState,
    mana: &mut Mana,
    primed_spell: &PrimedSpell,
    caster_query: &Query<&SpellCaster>,
    indicator_query: &mut Query<&mut SpellCircleIndicator>,
    commands: &mut Commands,
    assets: &SpellVisualAssets,
    meshes: &mut Assets<Mesh>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
    obstacle_events: &mut MessageWriter<ObstacleChanged>,
    sfx: &SpellSfxAssets,
    game_config: &GameConfig,
    talent_params: &GreaseTalentParams,
    scorched_mult: f32,
    local_origin: Vec3,
) -> bool {
    let mut completed = false;

    if handle_spell_release(input, commands, wizard_entity, casting_state, caster_query) {
        return false;
    }

    let Some(mut cursor_world_pos) = input.cursor_pos else {
        return false;
    };

    let wizard_pos = local_origin;
    let wizard_height = wizard_pos.y;
    let max_ground_radius = if wizard_height < wizard.spell_range {
        (wizard.spell_range * wizard.spell_range - wizard_height * wizard_height).sqrt()
    } else {
        0.0
    };
    let scale = primed_spell.empowerment;
    let circle_radius = constants::CIRCLE_RADIUS * scale * talent_params.radius_mult;
    let max_center_distance = (max_ground_radius - circle_radius).max(0.0);
    let direction = cursor_world_pos - wizard_pos;
    let distance = (direction.x * direction.x + direction.z * direction.z).sqrt();
    if distance > max_center_distance && distance > 0.001 {
        let normalized_direction = direction / distance;
        cursor_world_pos = wizard_pos + normalized_direction * max_center_distance;
    }

    match *casting_state {
        CastingState::Resting => {
            if input.just_pressed || input.pressed {
                try_start_cast_with_indicator(
                    commands,
                    meshes,
                    assets.grease_indicator.clone(),
                    wizard_entity,
                    casting_state,
                    mana,
                    constants::MANA_COST,
                    cursor_world_pos,
                    circle_radius,
                    caster_query,
                );
            }
        }
        CastingState::Casting { .. } => {
            casting_state.advance(time.delta_secs());
            update_indicator_position(
                wizard_entity,
                cursor_world_pos,
                caster_query,
                indicator_query,
            );
            if casting_state.is_complete(primed_spell.cast_time) {
                if mana.consume(constants::MANA_COST) {
                    if let Ok(caster) = caster_query.get(wizard_entity)
                        && let Some(indicator_entity) = caster.indicator_entity
                    {
                        if let Ok(indicator) = indicator_query.get(indicator_entity) {
                            let radius = constants::CIRCLE_RADIUS
                                * primed_spell.empowerment
                                * talent_params.radius_mult;
                            audio::play_sfx(
                                commands,
                                &sfx.grease_cast,
                                indicator.position,
                                game_config,
                                sfx,
                            );
                            spawn_grease_zone(
                                commands,
                                assets,
                                materials,
                                indicator.position,
                                radius,
                                primed_spell.empowerment,
                                obstacle_events,
                                *talent_params,
                                scorched_mult,
                            );
                        }
                        commands.entity(indicator_entity).try_despawn();
                    }
                    commands.entity(wizard_entity).remove::<SpellCaster>();
                    casting_state.cancel();
                    completed = true;
                } else {
                    cleanup_spell_caster(commands, wizard_entity, caster_query);
                    casting_state.cancel();
                }
            }
        }
        CastingState::Channeling { .. } => {
            cleanup_spell_caster(commands, wizard_entity, caster_query);
            casting_state.cancel();
        }
    }

    completed
}
