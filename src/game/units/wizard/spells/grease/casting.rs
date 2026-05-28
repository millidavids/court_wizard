//! Grease casting and slip effect.

use super::super::super::components::{
    CastingState, LocalWizard, Mana, PrimedSpell, Spell, SpellCaster, Wizard, WizardInput,
};
use super::components::{
    GreaseIgnited, GreaseOilSlickDebuff, GreaseRegenerating, GreaseTalentParams, GreaseZone,
    GreaseZonePresenceTracker,
};
use super::constants;
use super::ignite::spawn_grease_zone;
use crate::config::GameConfig;
use crate::game::crt_effect::CorrectedCursorPosition;
use crate::game::game_mode::components::ActiveToggles;
use crate::game::input::MouseButtonState;
use crate::game::input::messages::MouseLeftReleased;
use crate::game::pathfinding::{OBSTACLE_BUFFER, ObstacleChanged, ObstacleShape, ObstacleType};
use crate::game::units::components::{Corpse, Health, RootedModifier, SlowMovementModifier};
use crate::game::units::wizard::spells::audio::{self, SpellSfxAssets};
use crate::game::units::wizard::spells::utils::LocalSpellOrigin;
use crate::game::units::wizard::spells::utils::{
    SpellCircleIndicator, TargetAssistWorldPos, apply_target_assist, build_wizard_input,
    cleanup_spell_caster, handle_spell_release, try_start_cast_with_indicator,
    update_indicator_position, xz_distance,
};
use crate::game::units::wizard::spells::vfx;
use crate::game::units::wizard::spells::visual_assets::SpellVisualAssets;
use crate::game::units::wizard::talents::resources::{ActiveTalents, BattleTalentProgress};
use bevy::prelude::*;
use rand::Rng;

/// Helper to write an obstacle event for a grease zone.
pub(super) fn write_grease_obstacle(
    origin: Vec3,
    radius: f32,
    obstacle_type: ObstacleType,
    events: &mut MessageWriter<ObstacleChanged>,
) {
    let origin_2d = Vec2::new(origin.x, origin.z);
    let buffered_radius = radius + OBSTACLE_BUFFER;
    events.write(ObstacleChanged {
        bounds: Rect::from_center_size(origin_2d, Vec2::splat(buffered_radius * 2.0)),
        obstacle_type,
        shape: Some(ObstacleShape::circle(origin_2d, buffered_radius)),
        rebuild: false,
    });
}

/// Computes talent parameters from active talent selections.
pub(crate) fn compute_talent_params(active_talents: Option<&ActiveTalents>) -> GreaseTalentParams {
    let mut params = GreaseTalentParams::default();

    let Some(talents) = active_talents else {
        return params;
    };

    let t1 = talents.get_selection(Spell::Grease, 0);
    let t2 = talents.get_selection(Spell::Grease, 1);
    let t3 = talents.get_selection(Spell::Grease, 2);

    // Tier 1: Numeric modifiers
    match t1 {
        Some(0) => {
            // Extra Slippery
            params.slow_mult = constants::EXTRA_SLIPPERY_SLOW_MULT;
        }
        Some(1) => {
            // Wider Slick
            params.radius_mult = constants::WIDER_SLICK_RADIUS_MULT;
        }
        Some(2) => {
            // Volatile Mixture
            params.burn_damage_mult = constants::VOLATILE_MIXTURE_BURN_MULT;
        }
        _ => {}
    }

    // Tier 2: Behavioral flags
    match t2 {
        Some(0) => params.slip_and_fall = true,
        Some(1) => params.oil_slick = true,
        Some(2) => params.lingering_flames = true,
        _ => {}
    }

    // Tier 3: Transformative flags
    match t3 {
        Some(0) => params.chain_combustion = true,
        Some(1) => params.grease_geyser = true,
        Some(2) => params.endless_oil = true,
        _ => {}
    }

    params
}

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

/// Applies slow to units inside grease zones, ticks time_alive for non-ignited zones,
/// and handles Slip and Fall / Oil Slick talent effects.
#[allow(clippy::too_many_arguments)]
pub fn apply_grease_slow(
    mut commands: Commands,
    time: Res<Time>,
    mut game_rng: ResMut<crate::game::seeded_rng::resources::GameRng>,
    mut zones: Query<(
        Entity,
        &mut GreaseZone,
        Has<GreaseIgnited>,
        Has<GreaseRegenerating>,
    )>,
    mut targets: Query<
        (
            Entity,
            &Transform,
            Option<&mut SlowMovementModifier>,
            Option<&mut GreaseZonePresenceTracker>,
            Option<&GreaseOilSlickDebuff>,
            Option<&mut Health>,
        ),
        Without<Corpse>,
    >,
    mut talent_progress: Option<ResMut<BattleTalentProgress>>,
) {
    let delta = time.delta_secs();
    let rng = &mut game_rng.0;

    for (zone_entity, mut zone, is_ignited, is_regenerating) in &mut zones {
        // Only track time_alive for non-ignited zones
        // (ignited zones track time_alive in apply_grease_burn instead)
        if !is_ignited {
            zone.time_alive += delta;
        }

        // Don't apply slow while regenerating (not yet slippery)
        if is_regenerating {
            continue;
        }

        zone.time_since_last_tick += delta;
        if zone.time_since_last_tick >= zone.tick_interval {
            zone.time_since_last_tick = 0.0;

            let mut units_slowed: u32 = 0;
            let needs_tracking = zone.talent_params.slip_and_fall || zone.talent_params.oil_slick;

            for (entity, transform, existing_slow, existing_tracker, has_oil_debuff, mut health) in
                &mut targets
            {
                let dist = xz_distance(zone.origin, transform.translation);

                if dist <= zone.radius {
                    // Apply slow
                    if let Some(mut slow) = existing_slow {
                        slow.apply(zone.slow_modifier, zone.slow_duration);
                    } else {
                        let modifier =
                            SlowMovementModifier::new(zone.slow_modifier, zone.slow_duration);
                        commands
                            .entity(entity)
                            .queue_silenced(move |mut e: EntityWorldMut| {
                                e.insert(modifier);
                            });
                    }
                    units_slowed += 1;

                    if needs_tracking {
                        let is_new = existing_tracker
                            .as_ref()
                            .is_none_or(|t| t.zone_entity != zone_entity);

                        if is_new {
                            commands
                                .entity(entity)
                                .insert(GreaseZonePresenceTracker { zone_entity });

                            // Talent: Slip and Fall — stun on zone entry
                            if zone.talent_params.slip_and_fall {
                                let roll: f32 = rng.random_range(0.0..1.0);
                                if roll < constants::SLIP_AND_FALL_CHANCE {
                                    commands.entity(entity).insert(RootedModifier::new(
                                        constants::SLIP_AND_FALL_STUN_DURATION,
                                    ));
                                }
                            }

                            // Talent: Oil Slick — apply vulnerability debuff (once per unit)
                            if zone.talent_params.oil_slick
                                && has_oil_debuff.is_none()
                                && let Some(ref mut health) = health
                            {
                                health.spell_vulnerability += constants::OIL_SLICK_VULNERABILITY;
                                commands.entity(entity).insert(GreaseOilSlickDebuff::new());
                            }
                        }
                    }
                } else if needs_tracking {
                    // Unit is outside the zone — clean up tracker and debuffs
                    if let Some(ref tracker) = existing_tracker
                        && tracker.zone_entity == zone_entity
                    {
                        commands
                            .entity(entity)
                            .remove::<GreaseZonePresenceTracker>();

                        // Remove Oil Slick vulnerability when leaving
                        if let Some(debuff) = has_oil_debuff {
                            if let Some(ref mut health) = health {
                                health.spell_vulnerability =
                                    (health.spell_vulnerability - debuff.vulnerability).max(0.0);
                            }
                            commands.entity(entity).remove::<GreaseOilSlickDebuff>();
                        }
                    }
                }
            }

            // Track talent progress
            if units_slowed > 0
                && let Some(ref mut progress) = talent_progress
            {
                progress.increment(Spell::Grease, units_slowed);
            }
        }
    }
}
