use std::cmp::Ordering;
use std::collections::HashSet;

use bevy::prelude::*;

use super::super::super::components::{
    CastingState, LocalWizard, Mana, PrimedSpell, Spell, WizardInput,
};
use super::components::{
    ActiveMarkOfDeath, DeathsLedgerBurst, ExecutionerTriggered, MarkTalentFlags,
    MarkVisualIndicator,
};
use super::constants;
use crate::config::GameConfig;
use crate::game::components::OnGameplayScreen;
use crate::game::crt_effect::CorrectedCursorPosition;
use crate::game::input::MouseButtonState;
use crate::game::input::messages::MouseLeftReleased;
use crate::game::units::DamageType;
use crate::game::units::components::{
    Corpse, Health, MarkedForDeathModifier, TargetingVelocity, Team, TemporaryHitPoints,
    apply_spell_damage,
};
use crate::game::units::king::components::SpellShield;
use crate::game::units::wizard::components::Wizard;
use crate::game::units::wizard::spells::audio::{self, SpellSfxAssets};
use crate::game::units::wizard::spells::utils::LocalSpellOrigin;
use crate::game::units::wizard::spells::utils::{
    TargetAssistWorldPos, apply_target_assist, build_wizard_input,
};
use crate::game::units::wizard::spells::vfx;
use crate::game::units::wizard::spells::vfx::constants::UPWARD_ROTATION;
use crate::game::units::wizard::spells::visual_assets::SpellVisualAssets;
use crate::game::units::wizard::talents::resources::ActiveTalents;
use crate::networking::snapshot::SpellSoundId;

/// Computes the mark indicator pulse scale factor based on elapsed time.
fn mark_pulse_scale(elapsed_secs: f32) -> f32 {
    1.0 + (elapsed_secs * constants::MARK_INDICATOR_PULSE_SPEED * std::f32::consts::TAU).sin()
        * constants::MARK_INDICATOR_PULSE_AMPLITUDE
}

/// Local wizard mark of death casting — reads mouse input.
#[allow(clippy::too_many_arguments)]
pub fn handle_mark_of_death_casting(
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
    corrected_cursor: Res<CorrectedCursorPosition>,
    enemies_query: Query<(Entity, &Transform, &Team), Without<Corpse>>,
    existing_marks: Query<Entity, With<ActiveMarkOfDeath>>,
    audio_ctx: (Res<SpellSfxAssets>, Res<GameConfig>),
    active_talents: Option<Res<ActiveTalents>>,
    mut talent_progress: Option<
        ResMut<crate::game::units::wizard::talents::resources::BattleTalentProgress>,
    >,
    target_assist: Res<TargetAssistWorldPos>,
    local_origin: Res<LocalSpellOrigin>,
    mut pending_cast_events: ResMut<crate::game::multiplayer::spell_sync::PendingCastEvents>,
) {
    let (sfx, game_config) = &audio_ctx;
    let mut input = build_wizard_input(&mut mouse_left_released, &camera_query, &corrected_cursor);
    apply_target_assist(&mut input, &target_assist);
    let cursor_pos = input.cursor_pos;

    let Ok((_wizard_entity, mut casting_state, mut mana, primed_spell)) = wizard_query.single_mut()
    else {
        return;
    };
    if primed_spell.spell != Spell::MarkOfDeath {
        return;
    }

    let completed = mark_of_death_casting_logic(
        &input,
        &time,
        &mut casting_state,
        &mut mana,
        primed_spell,
        &mut commands,
        &enemies_query,
        &existing_marks,
        active_talents.as_deref(),
        &mut talent_progress,
    );

    if completed {
        vfx::systems::spawn_school_flare_synced(
            &mut commands,
            &visual_assets,
            &mut pending_cast_events,
            local_origin.0,
            vfx::systems::SpellSchool::Dark,
            time.elapsed_secs(),
        );
        if let Some(pos) = cursor_pos {
            audio::play_sfx_synced(
                &mut commands,
                &mut pending_cast_events,
                SpellSoundId::MarkOfDeathCast,
                pos,
                game_config,
                sfx,
            );
        }
        mouse_state.left_consumed = true;
    }
}

/// Core mark of death casting logic.
#[allow(clippy::too_many_arguments)]
fn mark_of_death_casting_logic(
    input: &WizardInput,
    time: &Time,
    casting_state: &mut CastingState,
    mana: &mut Mana,
    primed_spell: &PrimedSpell,
    commands: &mut Commands,
    enemies_query: &Query<(Entity, &Transform, &Team), Without<Corpse>>,
    existing_marks: &Query<Entity, With<ActiveMarkOfDeath>>,
    active_talents: Option<&ActiveTalents>,
    talent_progress: &mut Option<
        ResMut<crate::game::units::wizard::talents::resources::BattleTalentProgress>,
    >,
) -> bool {
    // Check for release event
    if input.just_released {
        casting_state.cancel();
        return false;
    }

    let mut completed = false;

    match *casting_state {
        CastingState::Resting => {
            if (input.just_pressed || input.pressed) && mana.can_afford(constants::MANA_COST) {
                casting_state.start_cast();
            }
        }
        CastingState::Casting { .. } => {
            casting_state.advance(time.delta_secs());
            if casting_state.is_complete(primed_spell.cast_time) {
                if mana.consume(constants::MANA_COST) {
                    if let Some(cursor_pos) = input.cursor_pos {
                        // Read talent selections
                        let t1 =
                            active_talents.and_then(|t| t.get_selection(Spell::MarkOfDeath, 0));
                        let t2 =
                            active_talents.and_then(|t| t.get_selection(Spell::MarkOfDeath, 1));
                        let t3 =
                            active_talents.and_then(|t| t.get_selection(Spell::MarkOfDeath, 2));

                        // Tier 1: compute amplification and duration
                        let mut amplification = constants::DAMAGE_AMPLIFICATION;
                        let mut duration = constants::MARK_DURATION;

                        match t1 {
                            Some(0) => amplification = constants::DEEP_MARK_AMPLIFICATION,
                            Some(1) => duration = constants::LINGERING_CURSE_DURATION,
                            _ => {}
                        }

                        // Tier 3-0: Mass Marking overrides amplification
                        let mass_marking = t3 == Some(0);
                        if mass_marking {
                            amplification = constants::MASS_MARKING_AMPLIFICATION;
                        }

                        // Apply empowerment
                        amplification *= primed_spell.empowerment;
                        duration *= primed_spell.empowerment;

                        // Build talent flags
                        let flags = MarkTalentFlags {
                            amplification,
                            swift_hex_refund: if t1 == Some(2) {
                                constants::MANA_COST * constants::SWIFT_HEX_REFUND_PERCENT
                            } else {
                                0.0
                            },
                            spreading_blight: t2 == Some(0),
                            executioner_brand: t2 == Some(1),
                            focal_point: t2 == Some(2),
                            deaths_ledger: t3 == Some(1),
                            doom: t3 == Some(2),
                        };

                        // Remove any existing marks (unless Doom marks which can't be removed)
                        for old_mark_entity in existing_marks.iter() {
                            commands
                                .entity(old_mark_entity)
                                .remove::<MarkedForDeathModifier>()
                                .remove::<ActiveMarkOfDeath>()
                                .remove::<MarkTalentFlags>()
                                .remove::<ExecutionerTriggered>();
                        }

                        let mut marked_count = 0u32;

                        if mass_marking {
                            // Mass Marking: mark all enemies in radius
                            for (entity, transform, team) in enemies_query.iter() {
                                if *team != Team::Attackers && *team != Team::Undead {
                                    continue;
                                }
                                let dist = crate::game::units::wizard::spells::utils::xz_distance(
                                    transform.translation,
                                    cursor_pos,
                                );
                                if dist <= constants::MASS_MARKING_RADIUS {
                                    commands.entity(entity).insert((
                                        MarkedForDeathModifier::new(amplification, duration),
                                        ActiveMarkOfDeath,
                                        flags.clone(),
                                    ));
                                    marked_count += 1;
                                }
                            }
                        } else {
                            // Single target: find nearest enemy to cursor
                            if let Some((target_entity, _)) = enemies_query
                                .iter()
                                .filter(|(_, _, team)| {
                                    **team == Team::Attackers || **team == Team::Undead
                                })
                                .filter_map(|(entity, transform, _)| {
                                    let dist =
                                        crate::game::units::wizard::spells::utils::xz_distance(
                                            transform.translation,
                                            cursor_pos,
                                        );
                                    if dist <= constants::TARGET_SEARCH_RADIUS {
                                        Some((entity, dist))
                                    } else {
                                        None
                                    }
                                })
                                .min_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or(Ordering::Equal))
                            {
                                commands.entity(target_entity).insert((
                                    MarkedForDeathModifier::new(amplification, duration),
                                    ActiveMarkOfDeath,
                                    flags,
                                ));
                                marked_count = 1;
                            }
                        }

                        // Track talent progress
                        if marked_count > 0
                            && let Some(progress) = talent_progress
                        {
                            progress.increment(Spell::MarkOfDeath, marked_count);
                        }
                    }
                    completed = true;
                }
                casting_state.cancel();
            }
        }
        CastingState::Channeling { .. } => {
            casting_state.cancel();
        }
    }

    completed
}

/// Doom talent: increase damage amplification over time for doom-marked targets.
pub fn tick_doom_marks(
    time: Res<Time>,
    mut marks: Query<(&mut MarkedForDeathModifier, &MarkTalentFlags)>,
) {
    let dt = time.delta_secs();
    for (mut mark, flags) in &mut marks {
        if flags.doom {
            mark.damage_amplification += constants::DOOM_AMP_PER_SECOND * dt;
            // Doom marks never expire — reset timer to keep them alive
            if mark.time_remaining < 1.0 {
                mark.time_remaining = 1.0;
            }
        }
    }
}

/// Executioner's Brand: deal burst damage when marked target falls below 30% HP.
pub fn executioner_brand_check(
    mut commands: Commands,
    mut targets: Query<
        (
            Entity,
            &mut Health,
            Option<&mut TemporaryHitPoints>,
            &MarkTalentFlags,
            Has<SpellShield>,
        ),
        (
            With<MarkedForDeathModifier>,
            Without<ExecutionerTriggered>,
            Without<Corpse>,
        ),
    >,
) {
    for (entity, mut health, mut temp_hp, flags, has_spell_shield) in &mut targets {
        if !flags.executioner_brand {
            continue;
        }
        if health.current <= health.max * constants::EXECUTIONER_HP_THRESHOLD
            && health.current > 0.0
        {
            apply_spell_damage(
                &mut commands,
                entity,
                &mut health,
                temp_hp.as_deref_mut(),
                constants::EXECUTIONER_BURST_DAMAGE,
                DamageType::Necrotic,
                has_spell_shield,
            );
            commands.entity(entity).insert(ExecutionerTriggered);
        }
    }
}

/// Handles all death-triggered talent effects for marked corpses.
/// Runs when any MarkTalentFlags exists — checks for Corpse to detect death.
/// Processes spreading blight, swift hex refund, and death's ledger, then cleans up.
#[allow(clippy::too_many_arguments)]
pub fn handle_marked_corpses(
    mut commands: Commands,
    dead_marked: Query<(Entity, &Health, &MarkTalentFlags, &Transform), With<Corpse>>,
    alive_enemies: Query<
        (Entity, &Transform, &Team),
        (Without<Corpse>, Without<MarkedForDeathModifier>),
    >,
    mut wizard: Query<&mut Mana, With<Wizard>>,
    visual_assets: Res<SpellVisualAssets>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    for (entity, health, flags, transform) in &dead_marked {
        // Swift Hex: refund mana on death
        if flags.swift_hex_refund > 0.0
            && let Ok(mut mana) = wizard.single_mut()
        {
            mana.current = (mana.current + flags.swift_hex_refund).min(mana.max);
        }

        // Spreading Blight: jump mark to nearest unmarked enemy
        if flags.spreading_blight {
            let nearest = alive_enemies
                .iter()
                .filter(|(_, _, team)| **team == Team::Attackers || **team == Team::Undead)
                .min_by(|a, b| {
                    let dist_a = a.1.translation.distance_squared(transform.translation);
                    let dist_b = b.1.translation.distance_squared(transform.translation);
                    dist_a.partial_cmp(&dist_b).unwrap_or(Ordering::Equal)
                });

            if let Some((target_entity, _, _)) = nearest {
                let new_duration =
                    constants::MARK_DURATION * constants::SPREADING_BLIGHT_DURATION_PERCENT;
                commands.entity(target_entity).insert((
                    MarkedForDeathModifier::new(flags.amplification, new_duration),
                    ActiveMarkOfDeath,
                    flags.clone(),
                ));
            }
        }

        // Death's Ledger: explode proportional to max HP
        if flags.deaths_ledger {
            let explosion_damage = health.max * constants::DEATHS_LEDGER_DAMAGE_PER_MAX_HP;
            spawn_deaths_ledger_explosion(
                &mut commands,
                transform.translation,
                explosion_damage,
                &visual_assets,
                &mut materials,
            );
        }

        // Clean up mark components from corpse
        commands
            .entity(entity)
            .remove::<ActiveMarkOfDeath>()
            .remove::<MarkTalentFlags>()
            .remove::<ExecutionerTriggered>();
    }
}

/// Spawns a Death's Ledger explosion visual at a position.
fn spawn_deaths_ledger_explosion(
    commands: &mut Commands,
    position: Vec3,
    damage: f32,
    visual_assets: &SpellVisualAssets,
    materials: &mut Assets<StandardMaterial>,
) {
    let pulse_material = materials
        .get(&visual_assets.necrotic_pulse)
        .cloned()
        .unwrap_or_default();
    let instance = materials.add(pulse_material);

    commands.spawn((
        DeathsLedgerBurst {
            time_alive: 0.0,
            lifetime: constants::DEATHS_LEDGER_PULSE_LIFETIME,
            max_radius: constants::DEATHS_LEDGER_RADIUS,
            damage,
            damage_applied: false,
        },
        Mesh3d(visual_assets.unit_circle.clone()),
        MeshMaterial3d(instance),
        Transform::from_translation(Vec3::new(position.x, 10.0, position.z))
            .with_rotation(UPWARD_ROTATION)
            .with_scale(Vec3::splat(1.0)),
        OnGameplayScreen,
    ));
}

/// Apply AoE damage from Death's Ledger explosions (one-shot).
pub fn apply_deaths_ledger_damage(
    mut commands: Commands,
    mut explosions: Query<(&Transform, &mut DeathsLedgerBurst)>,
    mut targets: Query<
        (
            Entity,
            &Transform,
            &mut Health,
            Option<&mut TemporaryHitPoints>,
            Has<SpellShield>,
        ),
        (Without<Wizard>, Without<Corpse>),
    >,
) {
    for (explosion_transform, mut burst) in &mut explosions {
        if burst.damage_applied {
            continue;
        }
        burst.damage_applied = true;

        for (entity, target_transform, mut health, mut temp_hp, has_spell_shield) in &mut targets {
            let dx = explosion_transform.translation.x - target_transform.translation.x;
            let dz = explosion_transform.translation.z - target_transform.translation.z;
            let dist = (dx * dx + dz * dz).sqrt();
            if dist <= burst.max_radius {
                apply_spell_damage(
                    &mut commands,
                    entity,
                    &mut health,
                    temp_hp.as_deref_mut(),
                    burst.damage,
                    DamageType::Necrotic,
                    has_spell_shield,
                );
            }
        }
    }
}

/// Update Death's Ledger burst visuals — expand and fade.
pub fn update_deaths_ledger_bursts(
    mut commands: Commands,
    time: Res<Time>,
    mut bursts: Query<(
        Entity,
        &mut DeathsLedgerBurst,
        &mut Transform,
        &MeshMaterial3d<StandardMaterial>,
    )>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let dt = time.delta_secs();

    for (entity, mut burst, mut transform, material_handle) in bursts.iter_mut() {
        burst.time_alive += dt;

        if burst.time_alive >= burst.lifetime {
            commands.entity(entity).try_despawn();
            continue;
        }

        let progress = burst.time_alive / burst.lifetime;
        let scale = burst.max_radius * progress;
        transform.scale = Vec3::splat(scale.max(0.1));

        if let Some(mat) = materials.get_mut(material_handle) {
            let alpha = 1.0 - progress;
            mat.base_color = mat.base_color.with_alpha(alpha * 0.5);
        }
    }
}

/// Focal Point: redirect defender targeting toward marked focal-point targets.
pub fn focal_point_retarget(
    marked_targets: Query<
        (Entity, &Transform, &MarkTalentFlags),
        (With<MarkedForDeathModifier>, Without<Corpse>),
    >,
    mut defenders: Query<
        (&Transform, &mut TargetingVelocity, &Team),
        (Without<Corpse>, Without<Wizard>),
    >,
) {
    // Find the focal point target (if any)
    let focal_target = marked_targets
        .iter()
        .find(|(_, _, flags)| flags.focal_point);

    let Some((_, target_transform, _)) = focal_target else {
        return;
    };

    let target_pos = target_transform.translation;

    // Override defender targeting velocity toward the focal point target
    for (defender_transform, mut targeting, team) in &mut defenders {
        if *team != Team::Defenders {
            continue;
        }
        let direction = (target_pos - defender_transform.translation).normalize_or_zero();
        targeting.velocity = Vec3::new(direction.x, 0.0, direction.z);
        let dx = defender_transform.translation.x - target_pos.x;
        let dz = defender_transform.translation.z - target_pos.z;
        targeting.distance_to_target = (dx * dx + dz * dz).sqrt();
    }
}

/// Spawns a purple circle indicator above newly marked units that don't have one yet.
pub fn spawn_mark_indicators(
    mut commands: Commands,
    marked_units: Query<(Entity, &Transform), (With<ActiveMarkOfDeath>, Without<Corpse>)>,
    existing_indicators: Query<&MarkVisualIndicator>,
    visual_assets: Res<SpellVisualAssets>,
    time: Res<Time>,
) {
    let tracked: HashSet<Entity> = existing_indicators.iter().map(|i| i.target).collect();

    for (entity, transform) in &marked_units {
        if tracked.contains(&entity) {
            continue;
        }

        let pos = transform.translation;
        let pulse = mark_pulse_scale(time.elapsed_secs());

        commands.spawn((
            MarkVisualIndicator { target: entity },
            Mesh3d(visual_assets.unit_circle.clone()),
            MeshMaterial3d(visual_assets.mark_indicator.clone()),
            Transform::from_translation(Vec3::new(
                pos.x,
                pos.y + constants::MARK_INDICATOR_Y_OFFSET,
                pos.z,
            ))
            .with_rotation(Quat::from_rotation_x(-std::f32::consts::FRAC_PI_2))
            .with_scale(Vec3::splat(constants::MARK_INDICATOR_RADIUS * pulse)),
            OnGameplayScreen,
        ));
    }
}

/// Updates mark indicator positions to follow their target and pulse.
/// Despawns indicators whose target no longer has a mark or is dead.
pub fn update_mark_indicators(
    mut commands: Commands,
    mut indicators: Query<(Entity, &MarkVisualIndicator, &mut Transform)>,
    marked_units: Query<&Transform, (With<ActiveMarkOfDeath>, Without<MarkVisualIndicator>)>,
    time: Res<Time>,
) {
    for (indicator_entity, indicator, mut indicator_transform) in &mut indicators {
        if let Ok(target_transform) = marked_units.get(indicator.target) {
            // Follow target position
            indicator_transform.translation.x = target_transform.translation.x;
            indicator_transform.translation.z = target_transform.translation.z;
            indicator_transform.translation.y =
                target_transform.translation.y + constants::MARK_INDICATOR_Y_OFFSET;

            // Pulse scale
            let pulse = mark_pulse_scale(time.elapsed_secs());
            indicator_transform.scale = Vec3::splat(constants::MARK_INDICATOR_RADIUS * pulse);
        } else {
            // Target lost its mark or died — despawn indicator
            commands.entity(indicator_entity).try_despawn();
        }
    }
}
