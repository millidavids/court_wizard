use super::super::super::super::components::{
    CastingState, LocalWizard, Mana, PrimedSpell, Spell, SpellCaster, WizardInput,
};
use super::super::components::TelekinesisIndicator;
use super::drop_ops::{convert_drop_to_flying, execute_storm_pickup, find_nearest_drop};
use super::talents::{TelekinesisConfig, compute_telekinesis_config};
use super::vfx_systems::{apply_harvest_damage, spawn_indicator, spawn_shockwave};
use crate::config::GameConfig;
use crate::game::crt_effect::CorrectedCursorPosition;
use crate::game::drops::components::{FlyingToWizard, IngredientDrop};
use crate::game::input::MouseButtonState;
use crate::game::input::messages::MouseLeftReleased;
use crate::game::units::components::{Health, Team, TemporaryHitPoints};
use crate::game::units::wizard::spells::audio::{self, SpellSfxAssets};
use crate::game::units::wizard::spells::utils::LocalSpellOrigin;
use crate::game::units::wizard::spells::utils::{
    TargetAssistWorldPos, apply_target_assist, build_wizard_input, cleanup_spell_caster,
    handle_spell_release,
};
use crate::game::units::wizard::spells::vfx;
use crate::game::units::wizard::spells::visual_assets::SpellVisualAssets;
use crate::game::units::wizard::talents::resources::{ActiveTalents, BattleTalentProgress};
use bevy::prelude::*;

/// Local wizard Telekinesis casting -- reads mouse input.
#[allow(clippy::too_many_arguments)]
pub(crate) fn handle_telekinesis_casting(
    time: Res<Time>,
    mut mouse_state: ResMut<MouseButtonState>,
    mut mouse_left_released: MessageReader<MouseLeftReleased>,
    mut commands: Commands,
    visual_assets: Res<SpellVisualAssets>,
    mut wizard_query: Query<
        (Entity, &mut CastingState, &mut Mana, &PrimedSpell),
        With<LocalWizard>,
    >,
    camera_query: Query<(&Camera, &GlobalTransform), With<Camera3d>>,
    cursor_resources: (
        Res<CorrectedCursorPosition>,
        Res<TargetAssistWorldPos>,
        Res<LocalSpellOrigin>,
    ),
    caster_query: Query<&SpellCaster>,
    drops_query: Query<(Entity, &Transform, &IngredientDrop), Without<FlyingToWizard>>,
    indicator_query: Query<&TelekinesisIndicator>,
    sfx: Res<SpellSfxAssets>,
    game_config: Res<GameConfig>,
    // Bundled to stay under Bevy's 16-param limit. `mp_session` is checked
    // first to bail out in MP — telekinesis pulls IngredientDrop entities
    // which only exist in SP (no MP drop sync).
    talent_and_mp: (
        Option<Res<ActiveTalents>>,
        Option<ResMut<BattleTalentProgress>>,
        Option<Res<crate::networking::session::MultiplayerSession>>,
    ),
    mut enemies_query: Query<
        (
            Entity,
            &Transform,
            &Team,
            &mut Health,
            Option<&mut TemporaryHitPoints>,
        ),
        (
            Without<IngredientDrop>,
            Without<crate::game::pathfinding::StagingAttacker>,
        ),
    >,
) {
    let (corrected_cursor, target_assist, local_origin) = cursor_resources;
    let (active_talents, mut talent_progress, mp_session) = talent_and_mp;
    let mut input = build_wizard_input(&mut mouse_left_released, &camera_query, &corrected_cursor);
    apply_target_assist(&mut input, &target_assist);

    let Ok((wizard_entity, mut casting_state, mut mana, primed_spell)) = wizard_query.single_mut()
    else {
        return;
    };
    if primed_spell.spell != Spell::Telekinesis {
        return;
    }

    // No-op in multiplayer — there are no IngredientDrop entities to pull
    // (drops are SP-only and not synced over the wire). Return before
    // touching mana or spawning the indicator so the action bar press is
    // visually obvious as "nothing happened" rather than burning mana.
    if mp_session.is_some() {
        return;
    }

    let config = compute_telekinesis_config(active_talents.as_deref());

    // Spawn indicator on Resting -> Casting transition
    if matches!(*casting_state, CastingState::Resting)
        && caster_query.get(wizard_entity).is_err()
        && mana.can_afford(config.mana_cost)
        && let Some(cursor_world_pos) = input.cursor_pos
        && let Some((drop_entity, drop_transform, _drop)) =
            find_nearest_drop(&cursor_world_pos, &drops_query, config.pickup_radius)
    {
        // Telekinesis has infinite range — no distance check needed
        let indicator_entity = spawn_indicator(
            &mut commands,
            &visual_assets,
            drop_transform.translation,
            drop_entity,
        );
        commands
            .entity(wizard_entity)
            .insert(SpellCaster::with_indicator(indicator_entity));
    }

    let completed = telekinesis_casting_logic(
        &input,
        &time,
        wizard_entity,
        &mut casting_state,
        &mut mana,
        primed_spell,
        &caster_query,
        &drops_query,
        &indicator_query,
        &mut commands,
        &sfx,
        &game_config,
        &config,
        &mut enemies_query,
        &visual_assets,
    );

    if completed {
        vfx::systems::spawn_school_flare(
            &mut commands,
            &visual_assets,
            local_origin.0,
            vfx::systems::SpellSchool::Force,
            time.elapsed_secs(),
        );
        mouse_state.left_consumed = true;
        if let Some(ref mut progress) = talent_progress {
            progress.increment(Spell::Telekinesis, 1);
        }
    }
}

/// Core Telekinesis casting logic -- called by the local casting system.
#[allow(clippy::too_many_arguments)]
fn telekinesis_casting_logic(
    input: &WizardInput,
    time: &Time,
    wizard_entity: Entity,
    casting_state: &mut CastingState,
    mana: &mut Mana,
    primed_spell: &PrimedSpell,
    caster_query: &Query<&SpellCaster>,
    drops_query: &Query<(Entity, &Transform, &IngredientDrop), Without<FlyingToWizard>>,
    indicator_query: &Query<&TelekinesisIndicator>,
    commands: &mut Commands,
    sfx: &SpellSfxAssets,
    game_config: &GameConfig,
    config: &TelekinesisConfig,
    enemies_query: &mut Query<
        (
            Entity,
            &Transform,
            &Team,
            &mut Health,
            Option<&mut TemporaryHitPoints>,
        ),
        (
            Without<IngredientDrop>,
            Without<crate::game::pathfinding::StagingAttacker>,
        ),
    >,
    visual_assets: &SpellVisualAssets,
) -> bool {
    // Check for release event
    if handle_spell_release(input, commands, wizard_entity, casting_state, caster_query) {
        return false;
    }

    let mut completed = false;

    match *casting_state {
        CastingState::Resting => {
            if (input.just_pressed || input.pressed)
                && caster_query.get(wizard_entity).is_err()
                && mana.can_afford(config.mana_cost)
                && let Some(cursor_world_pos) = input.cursor_pos
                && let Some((_drop_entity, _drop_transform, _drop)) =
                    find_nearest_drop(&cursor_world_pos, drops_query, config.pickup_radius)
            {
                // Telekinesis has infinite range — no distance check needed
                casting_state.start_cast();
            }
        }
        CastingState::Casting { .. } => {
            casting_state.advance(time.delta_secs());

            if casting_state.is_complete(config.cast_time.min(primed_spell.cast_time)) {
                if config.is_storm {
                    // Telekinetic Storm: grab ALL drops
                    completed = execute_storm_pickup(
                        mana,
                        config,
                        drops_query,
                        commands,
                        sfx,
                        game_config,
                        enemies_query,
                        visual_assets,
                    );
                } else {
                    // Normal pickup: grab targeted drop
                    let target_drop = caster_query
                        .get(wizard_entity)
                        .ok()
                        .and_then(|caster| caster.indicator_entity)
                        .and_then(|indicator_entity| indicator_query.get(indicator_entity).ok())
                        .map(|indicator| indicator.target_drop);

                    if let Some(drop_entity) = target_drop
                        && mana.consume(config.mana_cost)
                    {
                        if let Ok((_entity, drop_transform, drop_component)) =
                            drops_query.get(drop_entity)
                        {
                            let pickup_pos = drop_transform.translation;
                            convert_drop_to_flying(
                                commands,
                                drop_entity,
                                drop_component.ingredient,
                                pickup_pos,
                            );
                            audio::play_sfx(
                                commands,
                                &sfx.telekinesis_cast,
                                pickup_pos,
                                game_config,
                                sfx,
                            );

                            // T2: Harvest — damage nearby enemies
                            if config.has_harvest {
                                apply_harvest_damage(
                                    commands,
                                    pickup_pos,
                                    visual_assets,
                                    enemies_query,
                                );
                            }

                            // T3: Psychic Shockwave — spawn expanding ring from pickup
                            if config.has_shockwave {
                                spawn_shockwave(commands, visual_assets, pickup_pos);
                            }
                        }
                        completed = true;
                    }
                }

                // Cleanup indicator and caster
                cleanup_spell_caster(commands, wizard_entity, caster_query);
                casting_state.cancel();
            }
        }
        CastingState::Channeling { .. } => {
            // Telekinesis doesn't channel
            cleanup_spell_caster(commands, wizard_entity, caster_query);
            casting_state.cancel();
        }
    }

    completed
}
