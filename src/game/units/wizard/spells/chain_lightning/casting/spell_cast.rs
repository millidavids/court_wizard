//! Chain lightning casting input handling and core cast logic.

use super::super::chain::apply_chain_lightning_on_hit;
use super::arc_spawn::spawn_arc;
use super::talent_config::{ChainLightningTalentConfig, compute_chain_lightning_talent_config};
use std::cmp::Ordering;
use std::collections::HashSet;

use super::super::super::super::components::{
    CastingState, LocalWizard, Mana, PrimedSpell, Spell, WizardInput,
};
use super::super::components::*;
use super::super::constants;
use crate::config::GameConfig;
use crate::game::components::OnGameplayScreen;
use crate::game::crt_effect::CorrectedCursorPosition;
use crate::game::input::MouseButtonState;
use crate::game::input::messages::MouseLeftReleased;
use crate::game::units::components::{
    Corpse, Health, Team, TemporaryHitPoints, apply_spell_damage,
};
use crate::game::units::king::components::SpellShield;
use crate::game::units::wizard::components::Wizard;
use crate::game::units::wizard::spells::arcane_crystal::components::ArcaneCrystal;
use crate::game::units::wizard::spells::audio::{self, SpellSfxAssets};
use crate::game::units::wizard::spells::lightning_rod::LightningRod;
use crate::game::units::wizard::spells::utils::LocalSpellOrigin;
use crate::game::units::wizard::spells::utils::{
    TargetAssistWorldPos, apply_target_assist, build_wizard_input,
};
use crate::game::units::wizard::spells::vfx;
use crate::game::units::wizard::spells::visual_assets::SpellVisualAssets;
use crate::game::units::wizard::talents::resources::{ActiveTalents, BattleTalentProgress};
use crate::networking::snapshot::SpellSoundId;
use bevy::prelude::*;

/// Local wizard chain lightning casting — reads mouse input.
#[allow(clippy::too_many_arguments)]
pub fn handle_chain_lightning_casting(
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
    enemies_query: Query<
        (Entity, &Transform, &Team),
        (
            Without<Corpse>,
            Without<Wizard>,
            Without<crate::game::pathfinding::StagingAttacker>,
        ),
    >,
    rods_query: Query<(Entity, &Transform, &mut LightningRod)>,
    crystals_query: Query<(Entity, &Transform), With<ArcaneCrystal>>,
    mut health_query: Query<(
        &mut Health,
        Option<&mut TemporaryHitPoints>,
        Has<SpellShield>,
    )>,
    sfx: Res<SpellSfxAssets>,
    game_config: Res<GameConfig>,
    active_talents: Option<Res<ActiveTalents>>,
    (mut talent_progress, mut screen_flash, mut pending_cast_events): (
        Option<ResMut<BattleTalentProgress>>,
        MessageWriter<crate::game::crt_effect::ScreenFlashMessage>,
        ResMut<crate::game::multiplayer::spell_sync::PendingCastEvents>,
    ),
) {
    let (corrected_cursor, target_assist, local_origin) = cursor_resources;
    let mut input = build_wizard_input(&mut mouse_left_released, &camera_query, &corrected_cursor);
    apply_target_assist(&mut input, &target_assist);

    let Ok((wizard_entity, mut casting_state, mut mana, primed_spell)) = wizard_query.single_mut()
    else {
        return;
    };
    if primed_spell.spell != Spell::ChainLightning {
        return;
    }

    let talent_config = compute_chain_lightning_talent_config(active_talents.as_deref());

    let completed = chain_lightning_casting_logic(
        &input,
        &time,
        wizard_entity,
        &mut casting_state,
        &mut mana,
        primed_spell,
        &mut commands,
        &visual_assets,
        &enemies_query,
        &rods_query,
        &crystals_query,
        &mut health_query,
        &talent_config,
        talent_progress.as_deref_mut(),
        local_origin.0,
    );

    if completed {
        screen_flash.write(crate::game::crt_effect::ScreenFlashMessage {
            color: [0.8, 0.9, 1.0],
            duration: 0.1,
            intensity: 0.02,
        });

        vfx::systems::spawn_school_flare_synced(
            &mut commands,
            &visual_assets,
            &mut pending_cast_events,
            local_origin.0,
            vfx::systems::SpellSchool::Lightning,
            time.elapsed_secs(),
        );
        audio::play_sfx_synced(
            &mut commands,
            &mut pending_cast_events,
            SpellSoundId::ChainLightningCast,
            local_origin.0,
            &game_config,
            &sfx,
        );
        mouse_state.left_consumed = true;
    }
}

/// Core chain lightning casting logic. Returns true if the spell completed.
#[allow(clippy::too_many_arguments)]
fn chain_lightning_casting_logic(
    input: &WizardInput,
    time: &Time,
    _wizard_entity: Entity,
    casting_state: &mut CastingState,
    mana: &mut Mana,
    primed_spell: &PrimedSpell,
    commands: &mut Commands,
    assets: &SpellVisualAssets,
    enemies_query: &Query<
        (Entity, &Transform, &Team),
        (
            Without<Corpse>,
            Without<Wizard>,
            Without<crate::game::pathfinding::StagingAttacker>,
        ),
    >,
    rods_query: &Query<(Entity, &Transform, &mut LightningRod)>,
    crystals_query: &Query<(Entity, &Transform), With<ArcaneCrystal>>,
    health_query: &mut Query<(
        &mut Health,
        Option<&mut TemporaryHitPoints>,
        Has<SpellShield>,
    )>,
    talent_config: &ChainLightningTalentConfig,
    mut talent_progress: Option<&mut BattleTalentProgress>,
    local_origin: Vec3,
) -> bool {
    let mut completed = false;

    // Check for release event
    if input.just_released {
        casting_state.cancel();
        return false;
    }

    let effective_mana_cost = constants::MANA_COST * talent_config.mana_cost_mult;

    match *casting_state {
        CastingState::Channeling { .. } => {
            casting_state.cancel();
        }
        CastingState::Casting { .. } => {
            casting_state.advance(time.delta_secs());

            if casting_state.is_complete(primed_spell.cast_time) {
                if mana.consume(effective_mana_cost)
                    && let Some(cursor_pos) = input.cursor_pos
                {
                    // Storm talent: cast multiple times at different targets
                    let mut already_targeted: Vec<Entity> = Vec::new();

                    for _ in 0..talent_config.thunderstorm_count {
                        // Find enemy, rod, or crystal near cursor (skip already targeted)
                        let target = find_target_near_position_excluding(
                            cursor_pos,
                            enemies_query,
                            rods_query,
                            crystals_query,
                            &already_targeted,
                        );

                        if let Some((target_entity, target_pos)) = target {
                            already_targeted.push(target_entity);

                            let wizard_pos =
                                local_origin + Vec3::new(0.0, constants::SPAWN_HEIGHT_OFFSET, 0.0);

                            // Scale damage by empowerment and talent multiplier
                            let initial_damage = primed_spell.scale(constants::INITIAL_DAMAGE)
                                * talent_config.initial_damage_mult;

                            // Apply initial damage
                            if let Ok((mut health, mut temp_hp, has_spell_shield)) =
                                health_query.get_mut(target_entity)
                            {
                                apply_spell_damage(
                                    commands,
                                    target_entity,
                                    &mut health,
                                    temp_hp.as_deref_mut(),
                                    initial_damage,
                                    constants::DAMAGE_TYPE,
                                    has_spell_shield,
                                );
                            }

                            // Track talent progress
                            if let Some(ref mut progress) = talent_progress {
                                progress.increment(Spell::ChainLightning, 1);
                            }

                            // Apply on-hit talent effects
                            apply_chain_lightning_on_hit(
                                commands,
                                target_entity,
                                target_pos,
                                local_origin,
                                talent_config.static_charge,
                                talent_config.magnetic_pull,
                                None,
                            );

                            // Spawn first arc from wizard to target (depth 0 for initial arc)
                            spawn_arc(
                                commands,
                                assets,
                                wizard_pos,
                                target_pos,
                                0,
                                primed_spell.empowerment,
                            );

                            // Spawn shared hit tracking group
                            let group_entity = commands
                                .spawn((
                                    ChainLightningGroup {
                                        hit_entities: HashSet::from([target_entity]),
                                    },
                                    OnGameplayScreen,
                                ))
                                .id();

                            // Spawn chain lightning bolt to track splitting
                            commands.spawn((
                                ChainLightningBolt {
                                    group_entity,
                                    current_damage: initial_damage * talent_config.damage_falloff,
                                    damage_type: constants::DAMAGE_TYPE,
                                    bounces_remaining: talent_config.max_bounces,
                                    last_hit_position: target_pos,
                                    bounce_delay_timer: primed_spell.scale(constants::BOUNCE_DELAY),
                                    empowerment: primed_spell.empowerment,
                                    split_depth: 0,
                                    split_count: talent_config.split_count,
                                    damage_falloff: talent_config.damage_falloff,
                                    static_charge: talent_config.static_charge,
                                    magnetic_pull: talent_config.magnetic_pull,
                                    chain_reaction: talent_config.chain_reaction,
                                    bounce_range_mult: talent_config.bounce_range_mult,
                                },
                                OnGameplayScreen,
                            ));
                        }
                    }
                    completed = true;
                }

                casting_state.cancel();
            }
        }
        CastingState::Resting => {
            if (input.just_pressed || input.pressed) && mana.can_afford(effective_mana_cost) {
                casting_state.start_cast();
            }
        }
    }

    completed
}

/// Finds the closest enemy, lightning rod, or arcane crystal near the given position
/// within TARGETING_RADIUS, excluding specified entities.
/// Note: position should be at Y=0 (battlefield plane). Uses XZ distance for targeting.
fn find_target_near_position_excluding(
    position: Vec3,
    enemies: &Query<
        (Entity, &Transform, &Team),
        (
            Without<Corpse>,
            Without<Wizard>,
            Without<crate::game::pathfinding::StagingAttacker>,
        ),
    >,
    rods: &Query<(Entity, &Transform, &mut LightningRod)>,
    crystals: &Query<(Entity, &Transform), With<ArcaneCrystal>>,
    exclude: &[Entity],
) -> Option<(Entity, Vec3)> {
    let target_pos_2d = Vec3::new(position.x, 0.0, position.z);

    let unit_candidates = enemies
        .iter()
        .map(|(entity, transform, _)| (entity, transform.translation));

    let rod_candidates = rods
        .iter()
        .map(|(entity, transform, _)| (entity, transform.translation));

    let crystal_candidates = crystals
        .iter()
        .map(|(entity, transform)| (entity, transform.translation));

    unit_candidates
        .chain(rod_candidates)
        .chain(crystal_candidates)
        .filter(|(entity, _)| !exclude.contains(entity))
        .filter(|(_, pos)| {
            let pos_2d = Vec3::new(pos.x, 0.0, pos.z);
            target_pos_2d.distance(pos_2d) <= constants::TARGETING_RADIUS
        })
        .min_by(|a, b| {
            let a_pos_2d = Vec3::new(a.1.x, 0.0, a.1.z);
            let b_pos_2d = Vec3::new(b.1.x, 0.0, b.1.z);
            let dist_a = target_pos_2d.distance(a_pos_2d);
            let dist_b = target_pos_2d.distance(b_pos_2d);
            dist_a.partial_cmp(&dist_b).unwrap_or(Ordering::Equal)
        })
}
