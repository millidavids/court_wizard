use std::cmp::Ordering;
use std::collections::HashSet;

use bevy::prelude::*;
use super::super::super::components::{
    CastingState, LocalWizard, Mana, PrimedSpell, Spell, SpellCaster, WizardInput,
};
use super::components::*;
use super::constants;
use crate::config::GameConfig;
use crate::game::constants::{UNIT_HEALTH, UNIT_MOVEMENT_SPEED};
use crate::game::input::messages::MouseLeftReleased;
use crate::game::units::components::{
    Corpse, Effectiveness, Health, PermanentCorpse, PoisonedModifier, Team, TemporaryHitPoints,
};
use crate::game::units::constants::{
    POISON_DURATION, POISON_EFFECTIVENESS_CAP, POISON_EFFECTIVENESS_PER_STACK,
};
use crate::game::units::infantry::styles::UNDEAD_SPRITE_TINT;
use crate::game::units::undead::resources::UndeadAssets;
use crate::game::units::wizard::spells::audio::{self, SpellSfxAssets};
use crate::game::units::wizard::spells::fireball::components::FireballExplosion;
use crate::game::units::wizard::spells::utils::{
    SpellCircleIndicator, get_cursor_world_position, spawn_circle_indicator,
};
use crate::game::units::wizard::spells::vfx;
use crate::game::units::wizard::spells::visual_assets::SpellVisualAssets;
use crate::game::units::DamageType;
use crate::game::units::wizard::talents::resources::{ActiveTalents, BattleTalentProgress};
use crate::game::crt_effect::CorrectedCursorPosition;

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
    mut wizard_query: Query<(Entity, &mut CastingState, &mut Mana, &PrimedSpell), With<LocalWizard>>,
    camera_query: Query<(&Camera, &GlobalTransform), With<Camera3d>>,
    corrected_cursor: Res<CorrectedCursorPosition>,
    caster_query: Query<&SpellCaster>,
    mut indicator_query: Query<&mut SpellCircleIndicator>,
    corpse_query: Query<(Entity, &Transform), (With<Corpse>, Without<PermanentCorpse>)>,
    undead_assets: Res<UndeadAssets>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    sfx: Res<SpellSfxAssets>,
    game_config: Res<GameConfig>,
    talents_and_progress: (Option<Res<ActiveTalents>>, Option<ResMut<BattleTalentProgress>>),
) {
    let (active_talents, mut talent_progress) = talents_and_progress;
    let released = mouse_left_released.read().next().is_some();
    let cursor_pos = get_cursor_world_position(&camera_query, &corrected_cursor);
    let input = WizardInput {
        just_pressed: true,
        pressed: true,
        just_released: released,
        cursor_pos,
    };

    let Ok((wizard_entity, mut casting_state, mut mana, primed_spell)) =
        wizard_query.single_mut()
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
        if let Ok(caster) = caster_query.get(wizard_entity) {
            if let Some(indicator_entity) = caster.indicator_entity {
                commands.entity(indicator_entity).try_despawn();
            }
            commands.entity(wizard_entity).remove::<SpellCaster>();
        }
        commands.entity(wizard_entity).remove::<CorpseMagnetActive>();
    }

    // Manage indicator based on casting state
    let mana_cost = constants::MANA_COST_PER_CORPSE * talent_params.mana_cost_mult;
    match *casting_state {
        CastingState::Resting => {
            if caster_query.get(wizard_entity).is_err()
                && mana.can_afford(mana_cost)
                && let Some(pos) = cursor_pos
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
            if let Some(pos) = cursor_pos
                && let Ok(caster) = caster_query.get(wizard_entity)
                && let Some(indicator_entity) = caster.indicator_entity
                && let Ok(mut indicator) = indicator_query.get_mut(indicator_entity)
            {
                indicator.position = pos;
            }
        }
        CastingState::Channeling { .. } => {
            if let Some(pos) = cursor_pos
                && let Ok(caster) = caster_query.get(wizard_entity)
                && let Some(indicator_entity) = caster.indicator_entity
                && let Ok(mut indicator) = indicator_query.get_mut(indicator_entity)
            {
                indicator.position = pos;
            }
        }
    }

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
        &mut materials,
        &sfx,
        &game_config,
        &talent_params,
        talent_progress.as_deref_mut(),
    );

    if completed {
        vfx::systems::spawn_school_flare(&mut commands, &visual_assets, vfx::systems::SpellSchool::Dark, time.elapsed_secs());
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
    materials: &mut Assets<StandardMaterial>,
    sfx: &SpellSfxAssets,
    game_config: &GameConfig,
    talent_params: &RaiseTheDeadTalentParams,
    mut talent_progress: Option<&mut BattleTalentProgress>,
) -> bool {
    // Check for release event
    if input.just_released {
        casting_state.cancel();
        commands.entity(wizard_entity).remove::<CorpseMagnetActive>();
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
                        resurrect_nearest_corpse(
                            commands,
                            cursor_pos,
                            corpse_query,
                            undead_assets,
                            materials,
                            primed_spell.empowerment,
                            talent_params,
                            talent_progress.as_deref_mut(),
                        );
                    }
                    casting_state.reset_channel_interval();
                } else {
                    casting_state.cancel();
                    commands.entity(wizard_entity).remove::<CorpseMagnetActive>();
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
                        resurrect_nearest_corpse(
                            commands,
                            cursor_pos,
                            corpse_query,
                            undead_assets,
                            materials,
                            primed_spell.empowerment,
                            talent_params,
                            talent_progress.as_deref_mut(),
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
fn find_nearest_corpse(
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
fn raise_corpse_as_undead(
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

// === Talent Systems ===

/// Tier 2: Plague Bearer — raised undead with PlagueBearerAura damage nearby enemies.
pub fn tick_plague_bearer_aura(
    time: Res<Time>,
    mut commands: Commands,
    mut aura_query: Query<
        (&Transform, &mut PlagueBearerAura),
        (With<RaisedUndead>, Without<Corpse>),
    >,
    mut targets: Query<
        (
            Entity,
            &Transform,
            &Team,
            &mut Health,
            Option<&mut TemporaryHitPoints>,
            Option<&mut PoisonedModifier>,
        ),
        Without<Corpse>,
    >,
    assets: Res<SpellVisualAssets>,
) {
    let delta = time.delta_secs();
    let t = time.elapsed_secs();

    for (aura_transform, mut aura) in &mut aura_query {
        // Spawn plague smoke VFX
        aura.smoke_spawn_timer += delta;
        if aura.smoke_spawn_timer >= vfx::constants::PLAGUE_SMOKE_SPAWN_INTERVAL {
            aura.smoke_spawn_timer -= vfx::constants::PLAGUE_SMOKE_SPAWN_INTERVAL;
            vfx::systems::spawn_plague_smoke_puffs(
                &mut commands,
                &assets,
                aura_transform.translation,
                aura.radius,
                vfx::constants::PLAGUE_SMOKE_COUNT_PER_SPAWN,
                t,
            );
        }

        aura.tick_accumulator += delta;
        if aura.tick_accumulator < aura.tick_interval {
            continue;
        }

        let damage = aura.dps * aura.tick_accumulator;
        aura.tick_accumulator = 0.0;

        for (entity, target_transform, team, mut health, temp_hp, poison) in &mut targets {
            // Only damage living enemies (attackers)
            if *team != Team::Attackers {
                continue;
            }

            let dist = aura_transform
                .translation
                .distance(target_transform.translation);
            if dist <= aura.radius {
                crate::game::units::components::apply_damage_to_unit(
                    &mut health,
                    temp_hp.map(|t| t.into_inner()),
                    damage,
                );

                // Apply poison tint (turns units green via the persistent effect visual system)
                if let Some(mut existing_poison) = poison {
                    existing_poison.stack(
                        POISON_EFFECTIVENESS_PER_STACK,
                        POISON_DURATION,
                        POISON_EFFECTIVENESS_CAP,
                    );
                } else {
                    commands.entity(entity).insert(PoisonedModifier::new(
                        POISON_EFFECTIVENESS_PER_STACK,
                        POISON_DURATION,
                    ));
                }
                // Don't insert Corpse here — let convert_dead_to_corpses handle
                // the full death pipeline (material swap, lay flat, etc.)
            }
        }
    }
}

/// Tier 2: Corpse Magnet — during channeling, pull corpses toward cursor position.
pub fn pull_corpses_to_cursor(
    time: Res<Time>,
    wizard_query: Query<&CorpseMagnetActive>,
    camera_query: Query<(&Camera, &GlobalTransform), With<Camera3d>>,
    corrected_cursor: Res<CorrectedCursorPosition>,
    mut corpse_query: Query<&mut Transform, (With<Corpse>, Without<PermanentCorpse>)>,
) {
    let Ok(magnet) = wizard_query.single() else {
        return;
    };

    let cursor_pos = get_cursor_world_position(&camera_query, &corrected_cursor);
    let Some(target) = cursor_pos else {
        return;
    };

    let delta = time.delta_secs();

    for mut transform in &mut corpse_query {
        let dist = target.distance(transform.translation);
        if dist > magnet.pull_radius || dist < 1.0 {
            continue;
        }

        let direction = (target - transform.translation).normalize_or_zero();
        let move_amount = (magnet.pull_speed * delta).min(dist);
        transform.translation += direction * move_amount;
    }
}

/// Tier 3: Undead Detonation — when a raised undead with UndeadDetonation dies, explode.
pub fn handle_undead_detonation(
    time: Res<Time>,
    mut commands: Commands,
    dead_query: Query<
        (Entity, &UndeadDetonation, &Transform),
        (With<RaisedUndead>, Added<Corpse>),
    >,
    mut targets: Query<
        (&Transform, &Team, &mut Health, Option<&mut TemporaryHitPoints>),
        Without<Corpse>,
    >,
    assets: Res<SpellVisualAssets>,
    sfx: Res<SpellSfxAssets>,
    game_config: Res<GameConfig>,
) {
    let _t = time.elapsed_secs();

    for (dead_entity, detonation, transform) in &dead_query {
        let origin = transform.translation;

        // Despawn the exploding undead — no corpse left behind
        commands.entity(dead_entity).try_despawn();

        // Spawn fireball explosion VFX
        let mut explosion = FireballExplosion::new(
            origin,
            detonation.radius,
            0.0, // No ongoing damage — we apply it manually below
            DamageType::Fire,
            1.0,
        );
        explosion.skip_growth = false;

        commands.spawn((
            Mesh3d(assets.cross_plane_sphere.clone()),
            MeshMaterial3d(assets.fireball_explosion.clone()),
            Transform::from_translation(origin).with_scale(Vec3::splat(0.1)),
            explosion,
            crate::game::components::OnGameplayScreen,
        ));

        // Sparks, smoke, and heat shimmer are spawned by update_explosions on first frame

        audio::play_impact_sfx(
            &mut commands,
            &sfx.fireball_impact,
            origin,
            &game_config,
            &sfx,
        );

        // Apply detonation damage
        for (target_transform, team, mut health, temp_hp) in &mut targets {
            if *team != Team::Attackers {
                continue;
            }

            let dist = origin.distance(target_transform.translation);
            if dist <= detonation.radius {
                crate::game::units::components::apply_damage_to_unit(
                    &mut health,
                    temp_hp.map(|t| t.into_inner()),
                    detonation.damage,
                );
                // Don't insert Corpse here — let convert_dead_to_corpses handle
                // the full death pipeline (material swap, lay flat, etc.)
            }
        }
    }
}

/// Tier 3: Perpetual Unrest — dead enemies near PerpetualUnrest undead are auto-raised.
pub fn handle_perpetual_unrest(
    mut commands: Commands,
    mut raised_this_frame: Local<HashSet<Entity>>,
    new_corpses: Query<
        (Entity, &Transform, &Team),
        (With<Corpse>, Without<PermanentCorpse>, Without<RaisedUndead>),
    >,
    unrest_undead: Query<
        (&Transform, &PerpetualUnrest),
        (With<RaisedUndead>, Without<Corpse>),
    >,
    undead_assets: Res<UndeadAssets>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    active_talents: Option<Res<ActiveTalents>>,
    mut talent_progress: Option<ResMut<BattleTalentProgress>>,
) {
    raised_this_frame.clear();
    let talent_params = compute_talent_params(active_talents.as_deref());

    for (corpse_entity, corpse_transform, team) in &new_corpses {
        if *team != Team::Attackers {
            continue;
        }

        let should_raise = unrest_undead.iter().any(|(undead_transform, unrest)| {
            undead_transform
                .translation
                .distance(corpse_transform.translation)
                <= unrest.raise_radius
        });

        if should_raise && raised_this_frame.insert(corpse_entity) {
            let mut hp_mult = 1.0;
            if talent_params.empowered_undead {
                hp_mult *= constants::EMPOWERED_UNDEAD_HP_MULT;
            }

            raise_corpse_as_undead(
                &mut commands,
                corpse_entity,
                corpse_transform.translation,
                UNIT_HEALTH * hp_mult,
                UNIT_MOVEMENT_SPEED * 0.5,
                &talent_params,
                1.0,
                &undead_assets,
                &mut materials,
                talent_progress.as_deref_mut(),
            );
        }
    }
}

/// Tier 3: Revenant Lord — passively resurrects nearby corpses as undead minions.
pub fn tick_revenant_raise(
    time: Res<Time>,
    mut commands: Commands,
    mut revenant_query: Query<
        (&Transform, &mut RevenantLord),
        (With<RaisedUndead>, Without<Corpse>),
    >,
    corpse_query: Query<
        (Entity, &Transform),
        (With<Corpse>, Without<PermanentCorpse>),
    >,
    undead_assets: Res<UndeadAssets>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    active_talents: Option<Res<ActiveTalents>>,
    mut talent_progress: Option<ResMut<BattleTalentProgress>>,
) {
    let delta = time.delta_secs();
    let talent_params = compute_talent_params(active_talents.as_deref());

    // Strip revenant_lord so passively raised minions don't become Revenants
    let mut minion_params = talent_params.clone();
    minion_params.revenant_lord = false;

    for (rev_transform, mut revenant) in &mut revenant_query {
        revenant.raise_timer += delta;
        if revenant.raise_timer < revenant.raise_interval {
            continue;
        }
        revenant.raise_timer -= revenant.raise_interval;

        if let Some((corpse_entity, position)) =
            find_nearest_corpse(&corpse_query, rev_transform.translation, revenant.raise_radius)
        {
            raise_corpse_as_undead(
                &mut commands,
                corpse_entity,
                position,
                UNIT_HEALTH,
                UNIT_MOVEMENT_SPEED * 0.5,
                &minion_params,
                1.0,
                &undead_assets,
                &mut materials,
                talent_progress.as_deref_mut(),
            );
        }
    }
}
