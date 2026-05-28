//! Squall casting and storm setup.

use bevy::prelude::*;

use super::components::{SquallStorm, SquallStormRing, SquallTalentParams};
use super::constants::*;
use crate::game::components::{ConcentrationSpell, OnGameplayScreen};
use crate::game::crt_effect::CorrectedCursorPosition;
use crate::game::input::MouseButtonState;
use crate::game::input::messages::MouseLeftReleased;
use crate::game::units::components::{FrostAccumulation, SlowMovementModifier};
use crate::game::units::wizard::components::{
    CastingState, LocalWizard, Mana, PrimedSpell, Spell, SpellCaster, Wizard, WizardInput,
};
use crate::game::units::wizard::spells::utils::LocalSpellOrigin;
use crate::game::units::wizard::spells::utils::{
    RETICLE_Y, SpellCircleIndicator, TargetAssistWorldPos, apply_target_assist, build_wizard_input,
    clamp_to_spell_range_ground, cleanup_spell_caster, handle_spell_release, make_reticle_mesh,
    try_start_cast_with_indicator, update_indicator_position,
};
use crate::game::units::wizard::spells::vfx;
use crate::game::units::wizard::spells::visual_assets::SpellVisualAssets;
use crate::game::units::wizard::talents::resources::ActiveTalents;

/// Applies or inserts a [`SlowMovementModifier`] on an entity.
pub(super) fn apply_or_insert_slow(
    commands: &mut Commands,
    entity: Entity,
    existing: Option<Mut<SlowMovementModifier>>,
    modifier: f32,
    duration: f32,
) {
    if let Some(mut slow) = existing {
        slow.apply(modifier, duration);
    } else {
        commands
            .entity(entity)
            .insert(SlowMovementModifier::new(modifier, duration));
    }
}

/// Adds frost accumulation to an entity from an ice hit.
pub(super) fn apply_frost_accumulation(
    commands: &mut Commands,
    entity: Entity,
    existing: Option<Mut<FrostAccumulation>>,
    amount: f32,
) {
    if let Some(mut frost) = existing {
        frost.add_frost(amount, FROST_DECAY_DELAY);
    } else {
        commands
            .entity(entity)
            .insert(FrostAccumulation::new(amount, FROST_DECAY_DELAY));
    }
}

/// Despawns all storm ring reticles.
pub(super) fn despawn_storm_rings(
    commands: &mut Commands,
    rings: &Query<Entity, With<SquallStormRing>>,
) {
    for ring in rings.iter() {
        commands.entity(ring).try_despawn();
    }
}

/// Computes talent parameters from active talent selections.
pub(crate) fn compute_talent_params(active_talents: Option<&ActiveTalents>) -> SquallTalentParams {
    let mut params = SquallTalentParams::default();

    let Some(talents) = active_talents else {
        return params;
    };

    let t1 = talents.get_selection(Spell::Squall, 0);
    let t2 = talents.get_selection(Spell::Squall, 1);
    let t3 = talents.get_selection(Spell::Squall, 2);

    // Tier 1: Numeric modifiers
    match t1 {
        Some(0) => {
            // Bitter Cold: +30% damage
            params.damage_mult = BITTER_COLD_DAMAGE_MULT;
        }
        Some(1) => {
            // Howling Winds: +30% radius
            params.radius_mult = HOWLING_WINDS_RADIUS_MULT;
        }
        Some(2) => {
            // Freezing Rain: faster spawn, less damage per shard
            params.spawn_rate_mult = FREEZING_RAIN_SPAWN_MULT;
            params.damage_mult = FREEZING_RAIN_DAMAGE_MULT;
        }
        _ => {}
    }

    // Tier 2: Behavioral flags
    match t2 {
        Some(0) => params.permafrost = true,
        Some(1) => params.hailstones = true,
        Some(2) => params.sleet_storm = true,
        _ => {}
    }

    // Tier 3: Transformative flags
    match t3 {
        Some(0) => params.absolute_zero = true,
        Some(1) => params.blizzard = true,
        Some(2) => params.ice_age = true,
        _ => {}
    }

    params
}

/// Local wizard squall casting -- reads mouse input.
#[allow(clippy::too_many_arguments)]
pub(super) fn handle_squall_casting(
    time: Res<Time>,
    mut mouse_state: ResMut<MouseButtonState>,
    mut mouse_left_released: MessageReader<MouseLeftReleased>,
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    visual_assets: Res<SpellVisualAssets>,
    mut wizard_query: Query<
        (Entity, &Wizard, &mut CastingState, &mut Mana, &PrimedSpell),
        With<LocalWizard>,
    >,
    camera_query: Query<(&Camera, &GlobalTransform), With<Camera3d>>,
    caster_query: Query<&SpellCaster>,
    mut indicator_query: Query<&mut SpellCircleIndicator>,
    existing_storms: Query<Entity, With<SquallStorm>>,
    existing_rings: Query<Entity, With<SquallStormRing>>,
    active_talents: Option<Res<ActiveTalents>>,
    mut cursor_ctx: (
        Res<CorrectedCursorPosition>,
        Res<TargetAssistWorldPos>,
        Res<LocalSpellOrigin>,
        ResMut<crate::game::multiplayer::spell_sync::PendingCastEvents>,
    ),
) {
    let (ref corrected_cursor, ref target_assist, ref local_origin, ref mut pending_cast_events) =
        cursor_ctx;
    let mut input = build_wizard_input(&mut mouse_left_released, &camera_query, corrected_cursor);
    apply_target_assist(&mut input, target_assist);

    let Ok((wizard_entity, wizard, mut casting_state, mut mana, primed_spell)) =
        wizard_query.single_mut()
    else {
        return;
    };
    if primed_spell.spell != Spell::Squall {
        return;
    }

    let talent_params = compute_talent_params(active_talents.as_deref());

    let completed = squall_casting_logic(
        &input,
        &time,
        wizard_entity,
        wizard,
        &mut casting_state,
        &mut mana,
        primed_spell,
        &caster_query,
        &mut indicator_query,
        &existing_storms,
        &existing_rings,
        &mut commands,
        &mut meshes,
        &visual_assets,
        &talent_params,
        local_origin.0,
    );

    if completed {
        vfx::systems::spawn_school_flare_synced(
            &mut commands,
            &visual_assets,
            pending_cast_events,
            local_origin.0,
            vfx::systems::SpellSchool::Force,
            time.elapsed_secs(),
        );
        mouse_state.left_consumed = true;
    }
}

/// Core squall casting logic.
#[allow(clippy::too_many_arguments)]
fn squall_casting_logic(
    input: &WizardInput,
    time: &Time,
    wizard_entity: Entity,
    wizard: &Wizard,
    casting_state: &mut CastingState,
    mana: &mut Mana,
    primed_spell: &PrimedSpell,
    caster_query: &Query<&SpellCaster>,
    indicator_query: &mut Query<&mut SpellCircleIndicator>,
    existing_storms: &Query<Entity, With<SquallStorm>>,
    existing_rings: &Query<Entity, With<SquallStormRing>>,
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    assets: &SpellVisualAssets,
    talent_params: &SquallTalentParams,
    local_origin: Vec3,
) -> bool {
    let mut completed = false;

    // Check for release event - cancel cast
    if handle_spell_release(input, commands, wizard_entity, casting_state, caster_query) {
        return false;
    }

    // Get cursor world position and clamp to wizard's spell range
    let Some(mut cursor_world_pos) = input.cursor_pos else {
        return false;
    };

    let wizard_pos = local_origin;
    let scale = primed_spell.empowerment;
    let storm_radius = STORM_RADIUS * scale * talent_params.radius_mult;

    cursor_world_pos = clamp_to_spell_range_ground(
        cursor_world_pos,
        wizard_pos,
        wizard.spell_range,
        storm_radius,
    );

    // If Absolute Zero storm is already active, don't start a new cast
    if talent_params.absolute_zero && !existing_storms.is_empty() {
        return false;
    }

    // Handle casting based on state
    match *casting_state {
        CastingState::Resting => {
            if input.just_pressed || input.pressed {
                // Absolute Zero: no initial mana cost
                let effective_mana_cost = if talent_params.absolute_zero {
                    0.0
                } else {
                    MANA_COST
                };
                try_start_cast_with_indicator(
                    commands,
                    meshes,
                    assets.squall_indicator.clone(),
                    wizard_entity,
                    casting_state,
                    mana,
                    effective_mana_cost,
                    cursor_world_pos,
                    storm_radius,
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
                // Absolute Zero: no mana cost on cast completion
                let mana_ok = talent_params.absolute_zero || mana.consume(MANA_COST);
                if mana_ok {
                    // Despawn any existing storms and their rings (only one storm at a time)
                    for existing_storm in existing_storms.iter() {
                        commands.entity(existing_storm).try_despawn();
                    }
                    despawn_storm_rings(commands, existing_rings);

                    if let Ok(caster) = caster_query.get(wizard_entity)
                        && let Some(indicator_entity) = caster.indicator_entity
                    {
                        if let Ok(indicator) = indicator_query.get(indicator_entity) {
                            let mut storm_entity = commands.spawn((
                                SquallStorm::new(
                                    indicator.position,
                                    storm_radius,
                                    primed_spell.empowerment,
                                    *talent_params,
                                ),
                                // Tag for MP visual sync — the host's storm
                                // entity ships to the guest, which spawns a
                                // local ghost SquallStorm so the reticle
                                // mist + Sleet Storm evasion debuff systems
                                // (gated on `Without<GhostSpellEffect>` for
                                // gameplay-only) have something to attach
                                // visuals to on the remote peer.
                                crate::game::multiplayer::components::NetworkedSpellEffect {
                                    kind: crate::networking::snapshot::SpellEffectKind::SquallStorm,
                                },
                                OnGameplayScreen,
                            ));

                            // Absolute Zero: channeled, no concentration marker
                            if !talent_params.absolute_zero {
                                storm_entity.insert(ConcentrationSpell {
                                    spell_name: "Squall",
                                    mana_cost: MANA_COST,
                                });
                            }

                            // Spawn persistent annulus ring reticle for the storm
                            let ring_mesh = meshes.add(make_reticle_mesh(storm_radius));
                            commands.spawn((
                                Mesh3d(ring_mesh),
                                MeshMaterial3d(assets.squall_indicator.clone()),
                                Transform::from_translation(Vec3::new(
                                    indicator.position.x,
                                    RETICLE_Y,
                                    indicator.position.z,
                                ))
                                .with_rotation(Quat::from_rotation_x(-std::f32::consts::FRAC_PI_2))
                                .with_scale(Vec3::splat(storm_radius)),
                                SquallStormRing { time_alive: 0.0 },
                                OnGameplayScreen,
                            ));
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
