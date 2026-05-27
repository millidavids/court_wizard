//! Raise the Dead casting and corpse-raising logic.

use std::cmp::Ordering;

use super::super::super::components::{
    CastingState, LocalWizard, Mana, PrimedSpell, Spell, SpellCaster, WizardInput,
};
use super::components::*;
use super::constants;
use crate::config::GameConfig;
use crate::game::constants::{UNIT_HEALTH, UNIT_MOVEMENT_SPEED};
use crate::game::units::wizard::spells::utils::LocalSpellOrigin;
use crate::game::crt_effect::CorrectedCursorPosition;
use crate::game::input::messages::MouseLeftReleased;
use crate::game::units::components::{Corpse, Effectiveness, PermanentCorpse, Team};
use crate::game::units::infantry::constants::UNDEAD_SPRITE_TINT;
use crate::game::units::undead::resources::UndeadAssets;
use crate::game::units::wizard::spells::audio::{self, SpellSfxAssets};
use crate::game::units::wizard::spells::utils::{
    SpellCircleIndicator, TargetAssistWorldPos, apply_target_assist, build_wizard_input,
    cleanup_spell_caster, spawn_circle_indicator, update_indicator_position,
};
use crate::game::units::wizard::spells::vfx;
use crate::game::units::wizard::spells::visual_assets::SpellVisualAssets;
use crate::game::units::wizard::talents::resources::{ActiveTalents, BattleTalentProgress};
use bevy::prelude::*;

/// Computes talent parameters from active talent selections.
pub(crate) fn compute_talent_params(
    active_talents: Option<&ActiveTalents>,
) -> RaiseTheDeadTalentParams {
    let mut params = RaiseTheDeadTalentParams::default();
    let Some(talents) = active_talents else {
        return params;
    };

    let t1 = talents.get_selection(Spell::RaiseTheDead, 0);
    let t2 = talents.get_selection(Spell::RaiseTheDead, 1);
    let t3 = talents.get_selection(Spell::RaiseTheDead, 2);

    match t1 {
        Some(0) => params.radius_mult = constants::MASS_GRAVES_RADIUS_MULT,
        Some(1) => params.skip_ramp = true,
        Some(2) => params.mana_cost_mult = constants::EFFICIENT_NECROMANCY_MANA_MULT,
        _ => {}
    }

    match t2 {
        Some(0) => params.empowered_undead = true,
        Some(1) => params.plague_bearer = true,
        Some(2) => params.corpse_magnet = true,
        _ => {}
    }

    match t3 {
        Some(0) => params.revenant_lord = true,
        Some(1) => params.undead_detonation = true,
        Some(2) => params.perpetual_unrest = true,
        _ => {}
    }

    params
}

/// Applies talent-specific components to a newly raised undead entity.
///
/// Shared between `resurrect_nearest_corpse` (direct casting) and
/// `handle_perpetual_unrest` (chain-raising on kill).
fn apply_talent_components(
    entity_cmds: &mut EntityCommands,
    talent_params: &RaiseTheDeadTalentParams,
    empowerment: f32,
) {
    // Compute damage bonus from empowerment + talents
    let mut damage_bonus = if empowerment > 1.0 { 0.25 } else { 0.0 };
    if talent_params.empowered_undead {
        damage_bonus += constants::EMPOWERED_UNDEAD_DAMAGE_MULT - 1.0;
    }
    if talent_params.revenant_lord {
        damage_bonus += constants::REVENANT_DAMAGE_MULT - 1.0;
    }
    if damage_bonus > 0.0 {
        let mut effectiveness = Effectiveness::new();
        effectiveness.spell_bonus = damage_bonus;
        entity_cmds.insert(effectiveness);
    }

    if talent_params.plague_bearer {
        entity_cmds.insert(PlagueBearerAura::new(
            constants::PLAGUE_BEARER_DPS,
            constants::PLAGUE_BEARER_RADIUS,
            constants::PLAGUE_BEARER_TICK_INTERVAL,
        ));
    }
    if talent_params.undead_detonation {
        entity_cmds.insert(UndeadDetonation {
            damage: constants::UNDEAD_DETONATION_DAMAGE,
            radius: constants::UNDEAD_DETONATION_RADIUS,
        });
    }
    if talent_params.perpetual_unrest {
        entity_cmds.insert(PerpetualUnrest {
            raise_radius: constants::PERPETUAL_UNREST_RADIUS,
        });
    }
    if talent_params.revenant_lord {
        entity_cmds.insert(RevenantLord {
            raise_radius: constants::REVENANT_RAISE_RADIUS,
            raise_interval: constants::REVENANT_RAISE_INTERVAL,
            raise_timer: 0.0,
        });
    }
}

/// Local wizard Raise The Dead casting — reads mouse input.
#[allow(clippy::too_many_arguments)]
pub fn handle_raise_the_dead_casting(
    time: Res<Time>,
    mut mouse_left_released: MessageReader<MouseLeftReleased>,
    mut commands: Commands,
    visual_assets: Res<SpellVisualAssets>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut wizard_query: Query<
        (Entity, &mut CastingState, &mut Mana, &PrimedSpell),
        With<LocalWizard>,
    >,
    camera_query: Query<(&Camera, &GlobalTransform), With<Camera3d>>,
    cursor_resources: (Res<CorrectedCursorPosition>, Res<TargetAssistWorldPos>, Res<LocalSpellOrigin>),
    // Bundled to free a slot for the MP-context tuple — see `mp_ctx` below.
    cast_ctx: (
        Query<&SpellCaster>,
        Query<&mut SpellCircleIndicator>,
    ),
    corpse_query: Query<(Entity, &Transform), (With<Corpse>, Without<PermanentCorpse>)>,
    undead_assets: Res<UndeadAssets>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    sfx: Res<SpellSfxAssets>,
    game_config: Res<GameConfig>,
    talents_and_progress: (
        Option<Res<ActiveTalents>>,
        Option<ResMut<BattleTalentProgress>>,
    ),
    // Multiplayer context for the guest-side path: when the guest casts
    // Raise The Dead we must NOT spawn an undead unit locally (the host
    // owns authoritative unit existence). Instead we look up the target
    // corpse's NetworkEntityId and ship a `RaiseCorpse` message.
    mp_ctx: (
        Option<Res<crate::networking::session::MultiplayerSession>>,
        Option<ResMut<crate::networking::resources::NetworkConnection>>,
        Query<&crate::networking::entity_map::NetworkEntityId, With<Corpse>>,
    ),
) {
    let (active_talents, mut talent_progress) = talents_and_progress;
    let (corrected_cursor, target_assist, local_origin) = cursor_resources;
    let (caster_query, mut indicator_query) = cast_ctx;
    let (mp_session, mut connection, corpse_ids) = mp_ctx;
    let mut input = build_wizard_input(&mut mouse_left_released, &camera_query, &corrected_cursor);
    apply_target_assist(&mut input, &target_assist);

    let Ok((wizard_entity, mut casting_state, mut mana, primed_spell)) = wizard_query.single_mut()
    else {
        return;
    };
    if primed_spell.spell != Spell::RaiseTheDead {
        return;
    }

    let talent_params = compute_talent_params(active_talents.as_deref());

    let indicator_radius =
        constants::RESURRECTION_RADIUS * primed_spell.empowerment * talent_params.radius_mult;

    // Handle release -- clean up indicator, SpellCaster, and CorpseMagnetActive
    if input.just_released {
        cleanup_spell_caster(&mut commands, wizard_entity, &caster_query);
        commands
            .entity(wizard_entity)
            .remove::<CorpseMagnetActive>();
    }

    // Manage indicator based on casting state
    let mana_cost = constants::MANA_COST_PER_CORPSE * talent_params.mana_cost_mult;
    match *casting_state {
        CastingState::Resting => {
            if caster_query.get(wizard_entity).is_err()
                && mana.can_afford(mana_cost)
                && let Some(pos) = input.cursor_pos
            {
                let circle_entity = spawn_circle_indicator(
                    &mut commands,
                    &mut meshes,
                    visual_assets.raise_the_dead_indicator.clone(),
                    pos,
                    indicator_radius,
                )
                .id();
                commands
                    .entity(wizard_entity)
                    .insert(SpellCaster::with_indicator(circle_entity));
            }
        }
        CastingState::Casting { .. } => {
            if let Some(pos) = input.cursor_pos {
                update_indicator_position(wizard_entity, pos, &caster_query, &mut indicator_query);
            }
        }
        CastingState::Channeling { .. } => {
            if let Some(pos) = input.cursor_pos {
                update_indicator_position(wizard_entity, pos, &caster_query, &mut indicator_query);
            }
        }
    }

    use crate::networking::resources::PeerRole;
    let is_guest = mp_session
        .as_deref()
        .is_some_and(|s| s.role == PeerRole::Guest);

    let completed = raise_the_dead_casting_logic(
        &input,
        &time,
        &mut casting_state,
        &mut mana,
        primed_spell,
        &mut commands,
        wizard_entity,
        &corpse_query,
        &undead_assets,
        &visual_assets,
        &mut materials,
        &sfx,
        &game_config,
        &talent_params,
        talent_progress.as_deref_mut(),
        is_guest,
        connection.as_deref_mut(),
        &corpse_ids,
    );

    if completed {
        vfx::systems::spawn_school_flare(
            &mut commands,
            &visual_assets,
            local_origin.0,
            vfx::systems::SpellSchool::Dark,
            time.elapsed_secs(),
        );
    }
}

/// Core Raise The Dead casting logic.
///
/// Handles the full Resting -> Casting -> Channeling state machine.
/// During channeling, resurrects corpses at increasing frequency.
/// With Revenant Lord talent, only one corpse is raised (no channeling phase).
#[allow(clippy::too_many_arguments)]
fn raise_the_dead_casting_logic(
    input: &WizardInput,
    time: &Time,
    casting_state: &mut CastingState,
    mana: &mut Mana,
    primed_spell: &PrimedSpell,
    commands: &mut Commands,
    wizard_entity: Entity,
    corpse_query: &Query<(Entity, &Transform), (With<Corpse>, Without<PermanentCorpse>)>,
    undead_assets: &UndeadAssets,
    visual_assets: &SpellVisualAssets,
    materials: &mut Assets<StandardMaterial>,
    sfx: &SpellSfxAssets,
    game_config: &GameConfig,
    talent_params: &RaiseTheDeadTalentParams,
    mut talent_progress: Option<&mut BattleTalentProgress>,
    is_guest: bool,
    mut connection: Option<&mut crate::networking::resources::NetworkConnection>,
    corpse_ids: &Query<&crate::networking::entity_map::NetworkEntityId, With<Corpse>>,
) -> bool {
    // Check for release event
    if input.just_released {
        casting_state.cancel();
        commands
            .entity(wizard_entity)
            .remove::<CorpseMagnetActive>();
        return false;
    }

    let mana_cost = constants::MANA_COST_PER_CORPSE * talent_params.mana_cost_mult;
    // For Hasty Ritual: skip ramp by using 0 ramp_time
    let ramp_time = if talent_params.skip_ramp {
        0.0
    } else {
        constants::CHANNEL_RAMP_TIME
    };

    match *casting_state {
        CastingState::Channeling { .. } => {
            casting_state.advance_channel(time.delta_secs());

            if casting_state.should_channel(
                constants::INITIAL_CHANNEL_INTERVAL,
                constants::MIN_CHANNEL_INTERVAL,
                ramp_time,
            ) {
                if mana.consume(mana_cost) {
                    if let Some(cursor_pos) = input.cursor_pos {
                        try_raise_or_forward(
                            commands,
                            cursor_pos,
                            corpse_query,
                            undead_assets,
                            materials,
                            primed_spell.empowerment,
                            talent_params,
                            talent_progress.as_deref_mut(),
                            is_guest,
                            connection.as_deref_mut(),
                            corpse_ids,
                        );
                    }
                    casting_state.reset_channel_interval();
                } else {
                    casting_state.cancel();
                    commands
                        .entity(wizard_entity)
                        .remove::<CorpseMagnetActive>();
                }
            }
        }
        CastingState::Casting { .. } => {
            casting_state.advance(time.delta_secs());

            if casting_state.is_complete(primed_spell.cast_time) {
                if mana.consume(mana_cost) {
                    if let Some(cursor_pos) = input.cursor_pos {
                        audio::play_sfx(
                            commands,
                            &sfx.raise_the_dead_cast,
                            cursor_pos,
                            game_config,
                            sfx,
                        );
                        vfx::systems::spawn_aura_bubble(
                            commands,
                            visual_assets,
                            visual_assets.raise_dead_aura_sphere.clone(),
                            cursor_pos,
                            constants::RESURRECTION_RADIUS,
                            2.0,
                        );
                        try_raise_or_forward(
                            commands,
                            cursor_pos,
                            corpse_query,
                            undead_assets,
                            materials,
                            primed_spell.empowerment,
                            talent_params,
                            talent_progress,
                            is_guest,
                            connection.as_deref_mut(),
                            corpse_ids,
                        );
                    }
                    casting_state.start_channeling();

                    // Add Corpse Magnet if talent is active
                    if talent_params.corpse_magnet {
                        commands.entity(wizard_entity).insert(CorpseMagnetActive {
                            pull_radius: constants::CORPSE_MAGNET_RADIUS,
                            pull_speed: constants::CORPSE_MAGNET_PULL_SPEED,
                        });
                    }
                    return true;
                } else {
                    casting_state.cancel();
                }
            }
        }
        CastingState::Resting => {
            if (input.just_pressed || input.pressed) && mana.can_afford(mana_cost) {
                casting_state.start_cast();
            }
        }
    }

    false
}

/// Finds the nearest corpse to a position within a given radius.
pub(super) fn find_nearest_corpse(
    corpse_query: &Query<(Entity, &Transform), (With<Corpse>, Without<PermanentCorpse>)>,
    target_pos: Vec3,
    radius: f32,
) -> Option<(Entity, Vec3)> {
    corpse_query
        .iter()
        .filter(|(_, transform)| target_pos.distance(transform.translation) <= radius)
        .min_by(|a, b| {
            let dist_a = target_pos.distance(a.1.translation);
            let dist_b = target_pos.distance(b.1.translation);
            dist_a.partial_cmp(&dist_b).unwrap_or(Ordering::Equal)
        })
        .map(|(entity, transform)| (entity, transform.translation))
}

/// Raises a corpse entity as undead infantry with talent components.
///
/// Shared between direct casting, Perpetual Unrest, and Revenant Lord.
#[allow(clippy::too_many_arguments)]
pub(super) fn raise_corpse_as_undead(
    commands: &mut Commands,
    corpse_entity: Entity,
    position: Vec3,
    health: f32,
    speed: f32,
    talent_params: &RaiseTheDeadTalentParams,
    empowerment: f32,
    undead_assets: &UndeadAssets,
    materials: &mut Assets<StandardMaterial>,
    talent_progress: Option<&mut BattleTalentProgress>,
) {
    crate::game::units::systems::resurrect_corpse_as_infantry(
        commands,
        corpse_entity,
        position,
        Team::Undead,
        health,
        speed,
        UNDEAD_SPRITE_TINT,
        undead_assets.sprite_texture.clone(),
        undead_assets.sprite_mesh.clone(),
        materials,
        Some(undead_assets.death_texture.clone()),
    );

    let mut entity_cmds = commands.entity(corpse_entity);
    entity_cmds.insert(RaisedUndead);
    apply_talent_components(&mut entity_cmds, talent_params, empowerment);

    if let Some(progress) = talent_progress {
        progress.increment(Spell::RaiseTheDead, 1);
    }
}

/// Resurrects the nearest corpse to the target position as undead infantry.
/// Returns true if a corpse was raised.
#[allow(clippy::too_many_arguments)]
fn resurrect_nearest_corpse(
    commands: &mut Commands,
    target_pos: Vec3,
    corpse_query: &Query<(Entity, &Transform), (With<Corpse>, Without<PermanentCorpse>)>,
    undead_assets: &UndeadAssets,
    materials: &mut Assets<StandardMaterial>,
    empowerment: f32,
    talent_params: &RaiseTheDeadTalentParams,
    talent_progress: Option<&mut BattleTalentProgress>,
) -> bool {
    let search_radius = constants::RESURRECTION_RADIUS * empowerment * talent_params.radius_mult;

    let Some((corpse_entity, position)) =
        find_nearest_corpse(corpse_query, target_pos, search_radius)
    else {
        return false;
    };

    // Compute HP with Empowered Undead and Revenant Lord modifiers
    let mut hp_mult = 1.0;
    if talent_params.empowered_undead {
        hp_mult *= constants::EMPOWERED_UNDEAD_HP_MULT;
    }
    if talent_params.revenant_lord {
        hp_mult *= constants::REVENANT_HP_MULT;
    }

    let health = UNIT_HEALTH * empowerment * hp_mult;
    let speed = UNIT_MOVEMENT_SPEED * 0.5 * empowerment;

    raise_corpse_as_undead(
        commands,
        corpse_entity,
        position,
        health,
        speed,
        talent_params,
        empowerment,
        undead_assets,
        materials,
        talent_progress,
    );

    true
}

/// Multiplayer dispatcher for the raise action. On the host (and SP) it just
/// calls `resurrect_nearest_corpse` locally; on the guest it looks up the
/// nearest GHOST-corpse's `NetworkEntityId` and ships a `RaiseCorpse` message
/// — the host then performs the authoritative raise via
/// `receive_raise_corpse_messages` and the new undead propagates back to the
/// guest in the regular unit snapshot. Talent flags are packed into the
/// message's `flags` u32 so Plague Bearer / Perpetual Unrest / Revenant Lord
/// / Undead Detonation get applied host-side.
#[allow(clippy::too_many_arguments)]
fn try_raise_or_forward(
    commands: &mut Commands,
    target_pos: Vec3,
    corpse_query: &Query<(Entity, &Transform), (With<Corpse>, Without<PermanentCorpse>)>,
    undead_assets: &UndeadAssets,
    materials: &mut Assets<StandardMaterial>,
    empowerment: f32,
    talent_params: &RaiseTheDeadTalentParams,
    talent_progress: Option<&mut BattleTalentProgress>,
    is_guest: bool,
    connection: Option<&mut crate::networking::resources::NetworkConnection>,
    corpse_ids: &Query<&crate::networking::entity_map::NetworkEntityId, With<Corpse>>,
) -> bool {
    if !is_guest {
        return resurrect_nearest_corpse(
            commands,
            target_pos,
            corpse_query,
            undead_assets,
            materials,
            empowerment,
            talent_params,
            talent_progress,
        );
    }

    let Some(connection) = connection else {
        return false;
    };
    let search_radius = constants::RESURRECTION_RADIUS * empowerment * talent_params.radius_mult;
    let Some((corpse_entity, _)) = find_nearest_corpse(corpse_query, target_pos, search_radius)
    else {
        return false;
    };
    let Ok(net_id) = corpse_ids.get(corpse_entity) else {
        // Corpse exists locally but has no network ID — likely a
        // late-spawn race; skip this tick and try again on the next.
        return false;
    };

    use crate::networking::protocol::status_flags as sf;
    let mut flags: u32 = 0;
    if talent_params.plague_bearer {
        flags |= sf::RAISE_PLAGUE_BEARER;
    }
    if talent_params.perpetual_unrest {
        flags |= sf::RAISE_PERPETUAL_UNREST;
    }
    if talent_params.revenant_lord {
        flags |= sf::RAISE_REVENANT_LORD;
    }
    if talent_params.undead_detonation {
        flags |= sf::RAISE_UNDEAD_DETONATION;
    }
    if talent_params.empowered_undead {
        flags |= sf::RAISE_EMPOWERED_UNDEAD;
    }

    connection
        .outgoing_messages
        .push(crate::networking::protocol::NetworkMessage::RaiseCorpse {
            target_network_id: net_id.0,
            flags,
            empowerment,
        });
    true
}
