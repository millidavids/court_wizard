use std::cmp::Ordering;

use bevy::prelude::*;

use super::super::super::components::{
    CastingState, LocalWizard, Mana, PrimedSpell, Spell, SpellCaster, Wizard, WizardInput,
};
use super::components::*;
use super::constants;
use crate::config::GameConfig;
use crate::game::input::MouseButtonState;
use crate::game::input::messages::MouseLeftReleased;
use crate::game::units::boss::components::Boss;
use crate::game::pathfinding::FlowFieldVelocity;
use crate::game::units::components::{
    AttackTiming, Corpse, FlockingVelocity, Health, Hitbox, MindControlled, TargetingVelocity,
    Team, TemporaryHitPoints, apply_damage_to_unit,
};
use crate::game::units::wizard::spells::audio::{self, SpellSfxAssets};
use crate::game::units::wizard::spells::vfx;
use crate::game::constants::SPELL_ORIGIN;
use crate::game::units::wizard::spells::utils::{
    SpellCircleIndicator, clamp_cursor_to_spell_range, get_cursor_world_position,
    ground_projected_range, spawn_circle_indicator,
};
use crate::game::units::wizard::spells::visual_assets::SpellVisualAssets;
use crate::game::crt_effect::CorrectedCursorPosition;
use crate::game::units::wizard::talents::resources::{ActiveTalents, BattleTalentProgress};

/// Computes talent parameters from active talent selections.
pub(crate) fn compute_talent_params(
    active_talents: Option<&ActiveTalents>,
) -> MindControlTalentParams {
    let mut params = MindControlTalentParams::default();

    let Some(talents) = active_talents else {
        return params;
    };

    let t1 = talents.get_selection(Spell::MindControl, 0);
    let t2 = talents.get_selection(Spell::MindControl, 1);
    let t3 = talents.get_selection(Spell::MindControl, 2);

    // Tier 1: Numeric modifiers
    match t1 {
        Some(0) => params.duration_mult = constants::IRON_WILL_DURATION_MULT,
        Some(1) => params.damage_multiplier = constants::DEEP_DOMINATION_DAMAGE_MULT,
        Some(2) => params.cast_time_mult = constants::QUICK_SUBJUGATION_CAST_MULT,
        _ => {}
    }

    // Tier 2: Behavioral flags
    match t2 {
        Some(0) => params.puppet_master = true,
        Some(1) => params.traitors_mark = true,
        Some(2) => params.amnesia = true,
        _ => {}
    }

    // Tier 3: Transformative flags
    match t3 {
        Some(0) => params.dominate = true,
        Some(1) => params.mass_hysteria = true,
        Some(2) => params.sleeper_agent = true,
        _ => {}
    }

    params
}

/// Tracked highlight target — stored as system-local state so we have a single
/// source of truth that doesn't depend on deferred command timing.
/// Stores the entity, its original material handle (to restore), and the cloned
/// tinted handle (to remove from the asset store on cleanup).
#[derive(Default)]
pub(super) struct HighlightState {
    target: Option<HighlightedUnit>,
}

struct HighlightedUnit {
    entity: Entity,
    /// The entity's original shared material handle — restored on un-highlight.
    original_handle: Handle<StandardMaterial>,
    /// The cloned tinted material we created — removed from assets on cleanup.
    tinted_handle: Handle<StandardMaterial>,
}

/// Local wizard mind control casting — hold-to-cast with dynamic target highlighting.
#[allow(clippy::too_many_arguments)]
pub(super) fn handle_mind_control_casting(
    time: Res<Time>,
    mut mouse_state: ResMut<MouseButtonState>,
    mut mouse_left_released: MessageReader<MouseLeftReleased>,
    mut commands: Commands,
    mut assets: (ResMut<Assets<StandardMaterial>>, ResMut<Assets<Mesh>>),
    mut wizard_query: Query<
        (
            Entity,
            &Wizard,
            &mut CastingState,
            &mut Mana,
            &PrimedSpell,
            Option<&MindControlCooldown>,
        ),
        With<LocalWizard>,
    >,
    camera_query: Query<(&Camera, &GlobalTransform), With<Camera3d>>,
    corrected_cursor: Res<CorrectedCursorPosition>,
    enemies_query: Query<
        (Entity, &Transform, &Team, &MeshMaterial3d<StandardMaterial>),
        (Without<Corpse>, Without<MindControlled>, Without<Boss>, Without<MassHysteriaTarget>),
    >,
    existing_controlled: Query<&MindControlled>,
    existing_dominated: Query<Entity, With<DominatedUnit>>,
    caster_query: Query<&SpellCaster>,
    mut indicator_query: Query<&mut SpellCircleIndicator>,
    mut highlight: Local<HighlightState>,
    loaded_assets: (Res<SpellSfxAssets>, Res<SpellVisualAssets>, Res<GameConfig>),
    talent_resources: (Option<Res<ActiveTalents>>, Option<ResMut<BattleTalentProgress>>),
) {
    let (ref mut materials, ref mut meshes) = assets;
    let (ref sfx, ref visual_assets, ref game_config) = loaded_assets;
    let (active_talents, mut talent_progress) = talent_resources;
    let released = mouse_left_released.read().next().is_some();
    let raw_cursor_pos = get_cursor_world_position(&camera_query, &corrected_cursor);
    // cursor_pos is set after we know spell_range; use raw_cursor_pos for now
    let input = WizardInput {
        just_pressed: true,
        pressed: true,
        just_released: released,
        cursor_pos: raw_cursor_pos,
    };

    let Ok((wizard_entity, wizard, mut casting_state, mut mana, primed_spell, cooldown)) =
        wizard_query.single_mut()
    else {
        return;
    };
    let spell_range = ground_projected_range(wizard.spell_range, SPELL_ORIGIN.y);
    let cursor_pos = clamp_cursor_to_spell_range(raw_cursor_pos, wizard.spell_range, 0.0);
    if primed_spell.spell != Spell::MindControl {
        return;
    }

    let talent_params = compute_talent_params(active_talents.as_deref());
    let controlled_count = existing_controlled.iter().count() as u32;

    // Determine max controlled based on talents
    let max_controlled = if talent_params.dominate {
        // Dominate: only 1 permanent unit at a time
        if existing_dominated.iter().next().is_some() {
            0 // Can't dominate another while one exists
        } else {
            1
        }
    } else if talent_params.puppet_master {
        constants::PUPPET_MASTER_MAX
    } else {
        constants::MAX_CONTROLLED
    };

    // Mass Hysteria doesn't use the normal MC flow
    let mana_cost = if talent_params.mass_hysteria {
        constants::MANA_COST * constants::MASS_HYSTERIA_MANA_MULT
    } else {
        constants::MANA_COST
    };

    let cast_time = primed_spell.cast_time * talent_params.cast_time_mult;

    // On release → cancel cast, remove highlight, and despawn indicator
    if input.just_released {
        if let Ok(caster) = caster_query.get(wizard_entity) {
            if let Some(indicator_entity) = caster.indicator_entity {
                commands.entity(indicator_entity).try_despawn();
            }
            commands.entity(wizard_entity).remove::<SpellCaster>();
        }
        casting_state.cancel();
        clear_highlight(&mut commands, materials, &mut highlight);
        return;
    }

    match *casting_state {
        CastingState::Resting => {
            let can_cast = if talent_params.mass_hysteria {
                // Mass Hysteria doesn't check controlled count
                mana.can_afford(mana_cost)
                    && !cooldown.is_some_and(|cd| cd.remaining > 0.0)
            } else {
                mana.can_afford(mana_cost)
                    && !cooldown.is_some_and(|cd| cd.remaining > 0.0)
                    && controlled_count < max_controlled
            };

            if (input.just_pressed || input.pressed) && can_cast {
                if talent_params.mass_hysteria {
                    if let Some(pos) = cursor_pos {
                        let circle_entity = spawn_circle_indicator(
                            &mut commands,
                            meshes,
                            visual_assets.mind_control_indicator.clone(),
                            pos,
                            constants::MASS_HYSTERIA_RADIUS,
                        )
                        .id();
                        commands
                            .entity(wizard_entity)
                            .insert(SpellCaster::with_indicator(circle_entity));
                    }
                }
                casting_state.start_cast();
            }
        }
        CastingState::Casting { .. } => {
            casting_state.advance(time.delta_secs());

            if talent_params.mass_hysteria {
                // Update indicator position to follow cursor
                if let Some(pos) = cursor_pos
                    && let Ok(caster) = caster_query.get(wizard_entity)
                    && let Some(indicator_entity) = caster.indicator_entity
                    && let Ok(mut indicator) = indicator_query.get_mut(indicator_entity)
                {
                    indicator.position = pos;
                }
            } else {
                // Find nearest enemy to cursor each frame and update highlight
                let nearest = find_nearest_enemy(&enemies_query, cursor_pos, spell_range);
                update_highlight(
                    &mut commands,
                    materials,
                    &enemies_query,
                    &mut highlight,
                    nearest,
                );
            }

            if casting_state.is_complete(cast_time) {
                if mana.consume(mana_cost) {
                    vfx::systems::spawn_school_flare(&mut commands, &visual_assets, vfx::systems::SpellSchool::Dark, time.elapsed_secs());
                    if talent_params.mass_hysteria {
                        // Mass Hysteria: apply chaos to all enemies in radius
                        if let Some(pos) = cursor_pos {
                            let mut count = 0u32;
                            for (entity, transform, team, _) in &enemies_query {
                                if !matches!(*team, Team::Attackers | Team::Undead) {
                                    continue;
                                }
                                if transform.translation.distance(pos) <= constants::MASS_HYSTERIA_RADIUS {
                                    commands.entity(entity).insert(MassHysteriaTarget {
                                        time_remaining: constants::MASS_HYSTERIA_DURATION
                                            * talent_params.duration_mult,
                                    });
                                    count += 1;
                                }
                            }
                            if count > 0 {
                                audio::play_sfx(
                                    &mut commands,
                                    &sfx.mind_control_cast,
                                    pos,
                                    &game_config,
                                    &sfx,
                                );
                                if let Some(ref mut progress) = talent_progress {
                                    progress.increment(Spell::MindControl, count);
                                }
                            }
                        }
                    } else if let Some(ref highlighted) = highlight.target {
                        // Normal single-target MC
                        if let Ok((_, target_transform, _, _)) = enemies_query.get(highlighted.entity) {
                            audio::play_sfx(
                                &mut commands,
                                &sfx.mind_control_cast,
                                target_transform.translation,
                                &game_config,
                                &sfx,
                            );
                        }

                        let wear_off = if talent_params.dominate {
                            f32::MAX // Permanent
                        } else {
                            constants::EFFECT_WEAR_OFF_DURATION * talent_params.duration_mult
                        };

                        let mut entity_cmds = commands.entity(highlighted.entity);
                        entity_cmds.insert(MindControlled {
                            time_elapsed: 0.0,
                            wear_off_duration: wear_off,
                            original_spawn_pos: None,
                            damage_multiplier: talent_params.damage_multiplier,
                        });

                        // Insert talent-specific behavioral components
                        if talent_params.traitors_mark {
                            entity_cmds.insert(TraitorsMarkAura);
                        }
                        if talent_params.amnesia {
                            entity_cmds.insert(AmnesiaOnExpiry);
                        }
                        if talent_params.dominate {
                            entity_cmds.insert(DominatedUnit);
                        }
                        if talent_params.sleeper_agent {
                            entity_cmds.insert(SleeperAgentPending);
                        }

                        commands.entity(wizard_entity).insert(MindControlCooldown {
                            remaining: constants::COOLDOWN,
                        });

                        mouse_state.left_consumed = true;

                        if let Some(ref mut progress) = talent_progress {
                            progress.increment(Spell::MindControl, 1);
                        }
                    }
                }

                if let Ok(caster) = caster_query.get(wizard_entity)
                    && let Some(indicator_entity) = caster.indicator_entity
                {
                    commands.entity(indicator_entity).try_despawn();
                }
                commands.entity(wizard_entity).remove::<SpellCaster>();
                clear_highlight(&mut commands, materials, &mut highlight);
                casting_state.cancel();
            }
        }
        CastingState::Channeling { .. } => {
            if let Ok(caster) = caster_query.get(wizard_entity) {
                if let Some(indicator_entity) = caster.indicator_entity {
                    commands.entity(indicator_entity).try_despawn();
                }
                commands.entity(wizard_entity).remove::<SpellCaster>();
            }
            casting_state.cancel();
            clear_highlight(&mut commands, materials, &mut highlight);
        }
    }
}

/// Finds the nearest enemy to the cursor within TARGET_SEARCH_RADIUS and spell range.
/// Bosses are excluded from mind control targeting.
fn find_nearest_enemy(
    enemies_query: &Query<
        (Entity, &Transform, &Team, &MeshMaterial3d<StandardMaterial>),
        (Without<Corpse>, Without<MindControlled>, Without<Boss>, Without<MassHysteriaTarget>),
    >,
    cursor_pos: Option<Vec3>,
    spell_range: f32,
) -> Option<Entity> {
    let wizard_pos = SPELL_ORIGIN;
    cursor_pos.and_then(|pos| {
        enemies_query
            .iter()
            .filter(|(_, _, team, _)| **team == Team::Attackers || **team == Team::Undead)
            .filter(|(_, transform, _, _)| {
                let dx = transform.translation.x - wizard_pos.x;
                let dz = transform.translation.z - wizard_pos.z;
                (dx * dx + dz * dz).sqrt() <= spell_range
            })
            .filter_map(|(entity, transform, _, _)| {
                let dist = transform.translation.distance(pos);
                if dist <= constants::TARGET_SEARCH_RADIUS {
                    Some((entity, dist))
                } else {
                    None
                }
            })
            .min_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or(Ordering::Equal))
            .map(|(entity, _)| entity)
    })
}

/// Updates the highlight to point at a new target (or clears if None).
/// Clones the material for the highlighted entity so only it gets tinted.
fn update_highlight(
    commands: &mut Commands,
    materials: &mut ResMut<Assets<StandardMaterial>>,
    enemies_query: &Query<
        (Entity, &Transform, &Team, &MeshMaterial3d<StandardMaterial>),
        (Without<Corpse>, Without<MindControlled>, Without<Boss>, Without<MassHysteriaTarget>),
    >,
    highlight: &mut HighlightState,
    nearest: Option<Entity>,
) {
    let current_entity = highlight.target.as_ref().map(|h| h.entity);

    // If the target hasn't changed, nothing to do
    if nearest == current_entity {
        return;
    }

    // Restore old target's original material
    clear_highlight(commands, materials, highlight);

    // Clone + tint new target's material
    if let Some(target_entity) = nearest
        && let Ok((_, _, _, material_handle)) = enemies_query.get(target_entity)
    {
        let original_handle = material_handle.0.clone();
        if let Some(original_mat) = materials.get(&original_handle) {
            let mut tinted_mat = original_mat.clone();
            let base_linear = tinted_mat.base_color.to_linear();
            let highlight_linear = constants::HIGHLIGHT_COLOR.to_linear();
            let blended = base_linear.mix(&highlight_linear, 0.6);
            tinted_mat.base_color = Color::from(blended);
            let tinted_handle = materials.add(tinted_mat);

            // Swap the entity's material to the tinted clone
            commands
                .entity(target_entity)
                .insert(MeshMaterial3d(tinted_handle.clone()));

            highlight.target = Some(HighlightedUnit {
                entity: target_entity,
                original_handle,
                tinted_handle,
            });
        }
    }
}

/// Restores the highlighted entity's original material and cleans up the tinted clone.
fn clear_highlight(
    commands: &mut Commands,
    materials: &mut ResMut<Assets<StandardMaterial>>,
    highlight: &mut HighlightState,
) {
    if let Some(highlighted) = highlight.target.take() {
        // Restore the entity's original shared material
        commands
            .entity(highlighted.entity)
            .insert(MeshMaterial3d(highlighted.original_handle));

        // Remove the tinted clone from assets
        materials.remove(&highlighted.tinted_handle);
    }
}

/// Ticks the mind control cooldown timer.
pub(super) fn tick_mind_control_cooldown(
    time: Res<Time>,
    mut commands: Commands,
    mut query: Query<(Entity, &mut MindControlCooldown)>,
) {
    let delta = time.delta_secs();

    for (entity, mut cd) in &mut query {
        cd.remaining -= delta;
        if cd.remaining <= 0.0 {
            commands.entity(entity).remove::<MindControlCooldown>();
        }
    }
}

// --- Talent behavior systems ---

/// Traitor's Mark aura: apply/remove Demoralized on enemies near MC'd units with the aura.
/// Also cleans up lingering Demoralized when no aura sources remain.
pub(super) fn update_traitors_mark_aura(
    mut commands: Commands,
    aura_query: Query<&Transform, (With<MindControlled>, With<TraitorsMarkAura>, Without<Corpse>)>,
    mut enemies: Query<
        (Entity, &Transform, &Team, Has<Demoralized>),
        (Without<Corpse>, Without<MindControlled>),
    >,
) {
    for (entity, transform, team, has_demoralized) in &mut enemies {
        if !matches!(*team, Team::Attackers | Team::Undead) {
            continue;
        }

        // Check if within range of any aura
        let in_aura = aura_query.iter().any(|aura_transform| {
            aura_transform.translation.distance(transform.translation)
                <= constants::TRAITORS_MARK_RADIUS
        });

        if in_aura && !has_demoralized {
            commands.entity(entity).insert(Demoralized {
                damage_amplification: constants::TRAITORS_MARK_DAMAGE_AMP,
            });
        } else if !in_aura && has_demoralized {
            commands.entity(entity).remove::<Demoralized>();
        }
    }
}

/// Shared logic for confused combat: find nearest target in range and attack it.
/// Used by both Mass Hysteria and Amnesia effects.
fn confused_combat_attack(
    attacker_entity: Entity,
    attacker_pos: Vec3,
    attacker_hitbox: &Hitbox,
    current_time: f32,
    timing: &mut AttackTiming,
    potential_targets: &mut Query<
        (Entity, &Transform, &Hitbox, &mut Health, Option<&mut TemporaryHitPoints>),
        Without<Corpse>,
    >,
) {
    let last_time = (current_time - crate::game::constants::APPROX_FRAME_TIME).max(0.0);
    if !timing.can_attack(current_time, last_time) {
        return;
    }

    let mut nearest: Option<(Entity, f32)> = None;

    for (target_entity, target_transform, target_hitbox, _, _) in potential_targets.iter() {
        if target_entity == attacker_entity {
            continue;
        }
        let dist = attacker_pos.distance(target_transform.translation);
        let attack_range =
            (attacker_hitbox.radius + target_hitbox.radius) * constants::CONFUSED_ATTACK_RANGE_MULT;
        if dist <= attack_range && nearest.as_ref().is_none_or(|n| dist < n.1) {
            nearest = Some((target_entity, dist));
        }
    }

    if let Some((target_entity, _)) = nearest
        && let Ok((_, _, _, mut health, mut temp_hp)) = potential_targets.get_mut(target_entity)
    {
        apply_damage_to_unit(
            &mut health,
            temp_hp.as_deref_mut(),
            constants::COMBAT_DAMAGE,
        );
        timing.last_attack_time = Some(current_time);
    }
}

/// Mass Hysteria: ticks down the effect timer and removes the component when expired.
/// Combat is handled by the shared combat system which checks `Has<MassHysteriaTarget>`
/// to allow team-agnostic attacks.
pub(super) fn tick_mass_hysteria(
    time: Res<Time>,
    mut commands: Commands,
    mut hysteria_query: Query<(Entity, &mut MassHysteriaTarget), Without<Corpse>>,
) {
    let delta = time.delta_secs();

    for (entity, mut hysteria) in &mut hysteria_query {
        hysteria.time_remaining -= delta;

        if hysteria.time_remaining <= 0.0 {
            commands.entity(entity).remove::<MassHysteriaTarget>();
        }
    }
}

/// Amnesia effect: confused units attack random nearby targets (friend or foe).
pub(super) fn tick_amnesia_effect(
    time: Res<Time>,
    attack_cycle: Res<crate::game::plugin::GlobalAttackCycle>,
    mut commands: Commands,
    mut amnesia_query: Query<
        (Entity, &Transform, &Hitbox, &mut AmnesiaEffect, &mut AttackTiming),
        Without<Corpse>,
    >,
    mut potential_targets: Query<
        (Entity, &Transform, &Hitbox, &mut Health, Option<&mut TemporaryHitPoints>),
        Without<Corpse>,
    >,
) {
    let delta = time.delta_secs();
    let current_time = attack_cycle.current_time;

    for (entity, transform, hitbox, mut amnesia, mut timing) in &mut amnesia_query {
        amnesia.time_remaining -= delta;

        if amnesia.time_remaining <= 0.0 {
            commands.entity(entity).remove::<AmnesiaEffect>();
            continue;
        }

        confused_combat_attack(
            entity,
            transform.translation,
            hitbox,
            current_time,
            &mut timing,
            &mut potential_targets,
        );
    }
}

/// Sleeper Agent: ticks the delay timer and triggers betrayal attack.
pub(super) fn tick_sleeper_agent(
    time: Res<Time>,
    mut commands: Commands,
    mut agent_query: Query<
        (Entity, &Transform, &Hitbox, &Team, &mut SleeperAgentActive),
        Without<Corpse>,
    >,
    mut potential_targets: Query<
        (Entity, &Transform, &Hitbox, &Team, &mut Health, Option<&mut TemporaryHitPoints>),
        (Without<Corpse>, Without<SleeperAgentActive>),
    >,
) {
    let delta = time.delta_secs();

    for (entity, transform, hitbox, team, mut agent) in &mut agent_query {
        agent.delay_remaining -= delta;

        if agent.delay_remaining <= 0.0 {
            // Betrayal! Attack nearest same-team unit with bonus damage
            let pos = transform.translation;
            let mut nearest: Option<(Entity, f32)> = None;

            for (target_entity, target_transform, target_hitbox, target_team, _, _) in
                &potential_targets
            {
                if target_entity == entity || target_team != team {
                    continue;
                }
                let dist = pos.distance(target_transform.translation);
                let attack_range =
                    (hitbox.radius + target_hitbox.radius) * constants::SLEEPER_AGENT_RANGE_MULT;
                if dist <= attack_range && nearest.as_ref().is_none_or(|n| dist < n.1) {
                    nearest = Some((target_entity, dist));
                }
            }

            if let Some((target_entity, _)) = nearest
                && let Ok((_, _, _, _, mut health, mut temp_hp)) =
                    potential_targets.get_mut(target_entity)
            {
                let damage = constants::COMBAT_DAMAGE * agent.damage_multiplier;
                apply_damage_to_unit(&mut health, temp_hp.as_deref_mut(), damage);
            }

            commands.entity(entity).remove::<SleeperAgentActive>();
        }
    }
}

/// Strips all mind-control talent components from an entity.
/// Mass Hysteria targeting: affected units move toward the nearest unit regardless of team.
pub(super) fn update_mass_hysteria_targeting(
    mut hysteria_units: Query<
        (Entity, &Transform, &mut TargetingVelocity, &mut FlowFieldVelocity, &mut FlockingVelocity),
        (With<MassHysteriaTarget>, Without<Corpse>),
    >,
    all_units: Query<
        (Entity, &Transform),
        Without<Corpse>,
    >,
) {
    for (entity, transform, mut targeting, mut flow_field, mut flocking) in &mut hysteria_units {
        let pos = transform.translation;
        let mut nearest: Option<(f32, Vec3)> = None;

        for (other_entity, other_transform) in &all_units {
            if other_entity == entity {
                continue;
            }
            let dx = other_transform.translation.x - pos.x;
            let dz = other_transform.translation.z - pos.z;
            let dist = (dx * dx + dz * dz).sqrt();

            if nearest.as_ref().is_none_or(|(best, _)| dist < *best) {
                nearest = Some((dist, other_transform.translation));
            }
        }

        if let Some((_, target_pos)) = nearest {
            let dir =
                Vec3::new(target_pos.x - pos.x, 0.0, target_pos.z - pos.z).normalize_or_zero();
            targeting.velocity = dir;
        } else {
            targeting.velocity = Vec3::ZERO;
        }

        // Zero out flow field and flocking so targeting fully controls movement
        // Guard to avoid triggering Bevy change detection unnecessarily
        if flow_field.velocity != Vec3::ZERO {
            flow_field.velocity = Vec3::ZERO;
        }
        if flocking.velocity != Vec3::ZERO {
            flocking.velocity = Vec3::ZERO;
        }
    }
}

pub(crate) fn strip_mind_control_talent_components(commands: &mut Commands, entity: Entity) {
    commands
        .entity(entity)
        .remove::<TraitorsMarkAura>()
        .remove::<AmnesiaOnExpiry>()
        .remove::<AmnesiaEffect>()
        .remove::<DominatedUnit>()
        .remove::<SleeperAgentPending>()
        .remove::<SleeperAgentActive>();
}
